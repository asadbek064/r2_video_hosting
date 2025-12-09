# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

r2_video_hosting is a self-hosted video streaming platform with HLS encoding, Cloudflare R2 storage, and real-time analytics. It consists of:

- **Rust backend** (`src/`): Axum web server handling video processing, storage, and analytics
- **Next.js admin UI** (`admin-webui/`): React 19 + Next.js 16 dashboard for video management

## Development Setup

### Prerequisites

- Rust (2024 edition)
- FFmpeg with encoding support (libx264 minimum, NVIDIA/AMD hardware encoders optional)
- Bun (for frontend)
- Cloudflare R2 bucket (or S3-compatible storage)
- ClickHouse (optional, for analytics; see setup below)

### ClickHouse Setup

ClickHouse is optional but required for view analytics. Start it with Docker:

```bash
# Start ClickHouse without password authentication
docker run -d -p 8123:8123 -p 9000:9000 --name clickhouse \
  -e CLICKHOUSE_DB=default \
  -e CLICKHOUSE_DEFAULT_ACCESS_MANAGEMENT=1 \
  --ulimit nofile=262144:262144 \
  clickhouse/clickhouse-server

# If it crashes, restart with:
docker start clickhouse
```

### Configuration

Copy `config.yml.example` to `config.yml` and fill in actual values (no environment variable substitution - use literal values):

```yaml
server:
  secret_key: "your-secret-key"
  admin_password: "your-password"
  max_concurrent_encodes: 1  # Limit FFmpeg jobs

r2:
  endpoint: "https://<account-id>.r2.cloudflarestorage.com"
  bucket: "your-bucket"
  access_key_id: "your-key"
  secret_access_key: "your-secret"
  public_base_url: "https://your-cdn.r2.dev"

video:
  encoder: "libx264"  # or h264_nvenc, h264_vaapi, h264_qsv

clickhouse:
  url: "http://localhost:8123"
  user: "default"
  password: ""  # Empty for Docker setup above
```

### Running

```bash
# Backend (from repo root)
cargo build --release
./target/release/r2_video_hosting  # Requires config.yml

# Frontend dev server (from admin-webui/)
cd admin-webui
bun install
bun run dev  # Runs on :3001, proxies API to :3000

# Production frontend build
bun run build  # Outputs to ../webui/ for backend to serve
```

In production, the backend serves the static frontend at `/admin-webui`.

## Architecture

### Data Flow

1. **Upload**: Chunked upload API → temp storage → FFmpeg HLS encoding → parallel R2 upload
2. **Metadata**: SQLite stores video/subtitle/chapter data; ClickHouse stores view analytics
3. **Playback**: `/player/{id}` serves ArtPlayer with dynamic plugin loading based on video features
4. **Analytics**: Real-time viewer tracking via SSE, historical data from ClickHouse

### Core State (AppState)

Located in `src/types.rs`:

- `s3`: AWS SDK S3 client for R2 operations
- `db_pool`: SQLite connection pool
- `clickhouse`: ClickHouse client (operations use safe wrappers with timeouts)
- `progress`: Shared HashMap tracking upload/encoding progress
- `active_viewers`: Real-time viewer tracking per video
- `ffmpeg_semaphore`: Limits concurrent encodes (configured via `max_concurrent_encodes`)
- `chunked_uploads`: Tracks in-progress chunked uploads

### Module Structure

```
src/
├── main.rs              # Route definitions, middleware, server setup
├── config.rs            # Config loading from config.yml
├── types.rs             # All shared types, DTOs, AppState
├── database.rs          # SQLite operations via sqlx
├── clickhouse.rs        # Analytics with safe wrappers (timeouts/retries)
├── video.rs             # FFmpeg operations, metadata extraction
├── storage.rs           # R2 upload/download operations
└── handlers/
    ├── mod.rs           # Re-exports
    ├── upload.rs        # Upload, chunking, queue management
    ├── video.rs         # List, update, delete videos
    ├── analytics.rs     # View tracking, realtime/history endpoints
    ├── player.rs        # HLS serving, player page
    ├── content.rs       # Subtitles, attachments, chapters
    └── common.rs        # Shared utilities
```

### Video Processing Pipeline

1. **Upload**: `upload_video` or chunked (`upload_chunk` → `finalize_chunked_upload`)
2. **Encoding**: `encode_to_hls` in `video.rs` generates multi-resolution HLS
   - Resolutions: 1080p, 720p, 480p, 360p (adaptive based on source)
   - Bitrates calculated via BPP formula in `VideoVariant::calculate_bitrate`
3. **Storage**: `upload_hls_to_r2` in `storage.rs` does concurrent segment upload
4. **Metadata**: Extract and save subtitles/attachments/chapters/audio tracks
5. **Progress**: Real-time updates via `progress` HashMap, queryable at `/api/progress/{id}`

### Frontend Architecture

- **Auth**: `AuthWrapper.tsx` checks `/api/auth/check` with Bearer token from localStorage
- **Routes**: `/` (main page), `/videos/{id}` (detail), `/analytics` (dashboard)
- **State**: `UploadContext` tracks upload queue for progress display
- **Styling**: DaisyUI 5 + Tailwind 4, wrapped in custom components (`Button.tsx`, `Input.tsx`)
- **Dev config**: `basePath` removed in dev mode for rewrites to work; set to `/admin-webui` for production

## API Routes

### Protected (require `Authorization: Bearer {admin_password}`)

- `POST /api/upload` - Single file upload
- `POST /api/upload/chunk` - Chunked upload
- `POST /api/upload/finalize` - Finalize chunked upload
- `GET /api/videos` - List with pagination/search
- `PUT /api/videos/{id}` - Update metadata
- `DELETE /api/videos` - Batch delete
- `GET /api/queues` - Processing queue status
- `DELETE /api/queues/{id}` - Cancel queued job
- `POST /api/queues/cleanup` - Clean stale uploads

### Public

- `GET /player/{id}` - Embedded player page
- `GET /hls/{id}/{file}` - HLS segments/playlists
- `POST /api/videos/{id}/heartbeat` - Viewer heartbeat
- `POST /api/videos/{id}/view` - Track view
- `GET /api/videos/{id}/subtitles` - List subtitles
- `GET /api/videos/{id}/subtitles/{track}` - Get subtitle file
- `GET /api/videos/{id}/attachments` - List font attachments
- `GET /api/videos/{id}/chapters` - Get chapters
- `GET /api/videos/{id}/audio-tracks` - List audio tracks
- `GET /api/analytics/realtime` - SSE stream
- `GET /api/analytics/history` - Historical views
- `GET /api/progress/{upload_id}` - Upload progress

## Database Schema

SQLite migrations in `migrations/` (auto-run on startup):

- **videos**: Core table with FTS5 search on name/tags
- **subtitles**: Tracks with language, codec, storage keys (supports VobSub .idx)
- **attachments**: Font files from MKV
- **chapters**: Timeline markers
- **audio_tracks**: Multi-audio support

ClickHouse (via `clickhouse.rs`):
- **views**: video_id, ip_address, user_agent, created_at

## Common Development Tasks

### Adding a New API Endpoint

1. Add handler in `src/handlers/{module}.rs`
2. Export from `src/handlers/mod.rs`
3. Register route in `src/main.rs` (public or protected router)
4. Add request/response types to `src/types.rs` if needed

### Adding a Database Field

1. Create migration: `migrations/{timestamp}_{description}.sql`
2. Update queries in `src/database.rs`
3. Update corresponding DTO in `src/types.rs`

### Adding a Frontend Page

1. Create `admin-webui/app/{route}/page.tsx`
2. Add navigation in `Navbar.tsx`
3. Use `useUpload()` hook if showing upload progress

### Video Encoder Configuration

Supported encoders in `config.yml`:
- `libx264` (CPU, universal)
- `h264_nvenc` (NVIDIA GPU)
- `h264_vaapi` (AMD/Intel on Linux)
- `h264_qsv` (Intel QuickSync)

FFmpeg commands are built in `video.rs:encode_to_hls()`.

## Key Implementation Notes

- **ClickHouse operations**: Always use `*_safe` wrappers (e.g., `insert_view_safe`) which include timeouts and won't crash the app if ClickHouse is down
- **Concurrency**: FFmpeg jobs are limited by semaphore; R2 uploads use tokio concurrent streams
- **Progress tracking**: Stored in `AppState.progress`, cleaned up automatically on completion/failure
- **Chunked uploads**: Temporary files stored in OS temp dir, cleaned up after 24h of inactivity
- **Next.js config**: Uses `basePath: '/admin-webui'` in production only; dev mode has no basePath to enable rewrites
