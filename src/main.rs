mod config;
mod database;
mod handlers;
mod rate_limit;
mod storage;
mod types;
mod video;

use anyhow::{Context, Result};
use aws_sdk_s3::{Client as S3Client, config::Region};
use axum::extract::{ConnectInfo, DefaultBodyLimit};
use axum::{
    Router,
    extract::{Request, State},
    http::{Method, StatusCode, header},
    middleware::{self, Next},
    response::Redirect,
    response::Response,
    routing::{delete, get, post, put},
};
use config::Config;
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::sync::RwLock;
use tower_http::cors::{CorsLayer};
use tower_http::services::{ServeDir, ServeFile};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use types::AppState;

async fn auth_middleware(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let ip = addr.ip();

    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok());

    let expected_auth = format!("Bearer {}", state.config.server.admin_password);

    match auth_header {
        Some(auth) if auth == expected_auth => {
            // Success - reset rate limit for this IP
            state.auth_rate_limiter.reset(ip).await;
            Ok(next.run(req).await)
        }
        _ => {
            // Failed auth - check rate limit
            if let Err(_retry_after) = state.auth_rate_limiter.check_and_increment(ip).await {
                // Rate limited
                return Err(StatusCode::TOO_MANY_REQUESTS);
            }
            Err(StatusCode::UNAUTHORIZED)
        }
    }
}

async fn check_auth() -> Result<(), StatusCode> {
    Ok(())
}

async fn root_redirect(State(state): State<AppState>) -> Redirect {
    state
        .config
        .server
        .root_redirect_url
        .as_deref()
        .map(Redirect::permanent)
        .unwrap_or_else(|| Redirect::permanent("https://asadk.dev/"))
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,r2_video_hosting=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Config::load("config.yml").await?;

    let s3_config = aws_sdk_s3::config::Builder::new()
        .endpoint_url(&config.r2.endpoint)
        .region(Region::new("auto"))
        .credentials_provider(aws_sdk_s3::config::Credentials::new(
            &config.r2.access_key_id,
            &config.r2.secret_access_key,
            None,
            None,
            "r2",
        ))
        .build();
    let s3 = S3Client::from_conf(s3_config);

    let database_url = "sqlite://videos.db";
    let db_pool = database::initialize_database(database_url).await?;

    let progress = Arc::new(RwLock::new(HashMap::new()));

    let ffmpeg_semaphore = Arc::new(tokio::sync::Semaphore::new(
        config.server.max_concurrent_encodes,
    ));

    let auth_rate_limiter = rate_limit::AuthRateLimiter::new();

    // Spawn cleanup task for rate limiter
    let limiter_clone = auth_rate_limiter.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(300)).await; // Every 5 min
            limiter_clone.cleanup_old_entries().await;
        }
    });

    let host = config.server.host.clone();
    let port = config.server.port;

    let state = AppState {
        config,
        s3,
        db_pool,
        progress: progress.clone(),
        ffmpeg_semaphore,
        chunked_uploads: Arc::new(RwLock::new(HashMap::new())),
        auth_rate_limiter,
    };

    let public_routes = Router::new()
        .route("/videos/{id}/subtitles", get(handlers::get_video_subtitles))
        .route(
            "/videos/{id}/subtitles/{track_with_ext}",
            get(handlers::get_subtitle_file),
        )
        .route(
            "/videos/{id}/attachments",
            get(handlers::get_video_attachments),
        )
        .route(
            "/videos/{id}/attachments/{filename}",
            get(handlers::get_attachment_file),
        )
        .route("/videos/{id}/chapters", get(handlers::get_video_chapters))
        .route(
            "/videos/{id}/audio-tracks",
            get(handlers::get_video_audio_tracks),
        )
        .route("/progress/{upload_id}", get(handlers::get_progress));

    let protected_routes = Router::new()
        .route("/upload", post(handlers::upload_video))
        .route("/upload/chunk", post(handlers::upload_chunk))
        .route("/upload/finalize", post(handlers::finalize_chunked_upload))
        .route("/videos", get(handlers::list_videos))
        .route("/videos", delete(handlers::delete_videos))
        .route("/videos/{id}", put(handlers::update_video))
        .route("/videos/{id}/visibility", put(handlers::update_video_visibility))
        .route("/queues", get(handlers::list_queues))
        .route("/queues/{id}", delete(handlers::cancel_queue))
        .route("/queues/cleanup", post(handlers::cleanup_uploads))
        .route("/auth/check", get(check_auth))
        .route("/config", get(handlers::get_config_info))
        //.route("/purge", delete(handlers::purge_bucket))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    let api_routes = Router::new().merge(public_routes).merge(protected_routes);

    let app = Router::new()
        .nest("/api", api_routes)
        .route("/hls/{id}/{*file}", get(handlers::get_hls_file))
        .route("/player/{id}", get(handlers::get_player))
        .route("/jassub/{filename}", get(handlers::get_jassub_worker))
        .route("/libbitsub/{filename}", get(handlers::get_libbitsub_worker))
        .nest_service(
            "/admin-webui",
            ServeDir::new("webui")
                .append_index_html_on_directories(false)
                .fallback(ServeFile::new("webui/index.html")),
        )
        .route("/", get(root_redirect))
        // e.g. 1 GB body limit
        .layer(DefaultBodyLimit::max(1024 * 1024 * 1024))
        .layer(tower_http::trace::TraceLayer::new_for_http())
        // CORS layer - same-origin only for production security
        .layer(
            CorsLayer::new()
                .allow_origin(tower_http::cors::AllowOrigin::mirror_request())
                .allow_methods([
                    Method::GET,
                    Method::POST,
                    Method::PUT,
                    Method::DELETE,
                    Method::OPTIONS,
                ])
                .allow_headers([
                    header::AUTHORIZATION,
                    header::CONTENT_TYPE,
                    header::ACCEPT,
                    header::HeaderName::from_static("x-upload-id"),
                ])
                .allow_credentials(true),
        )
        .with_state(state);

    let addr: SocketAddr = format!("{}:{}", host, port).parse().unwrap();
    info!("listening on {}", addr);

    axum::serve(
        tokio::net::TcpListener::bind(addr).await?,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .context("server error")?;
    Ok(())
}
