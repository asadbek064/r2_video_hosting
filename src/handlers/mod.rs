pub mod common;
pub mod content;
pub mod player;
pub mod upload;
pub mod video;

// Re-export specific handlers if needed by main.rs
#[allow(unused)]
pub use common::{get_config_info, internal_err, minify_js};
pub use content::{
    get_attachment_file, get_jassub_worker, get_libbitsub_worker, get_subtitle_file,
    get_video_attachments, get_video_audio_tracks, get_video_chapters, get_video_subtitles,
};
pub use player::{get_hls_file, get_player};

#[allow(unused)]
pub use upload::{
    CancelQueueResponse, CleanupResponse, cancel_queue, finalize_chunked_upload, get_progress, list_queues,
    upload_chunk, upload_video, cleanup_uploads,
};
pub use video::{delete_videos, list_videos, update_video, update_video_visibility};
