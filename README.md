# r2_video_hosting
A self-hosted video streaming platform with HLS encoding, Cloudflare R2 storage, and a modern admin dashboard.

> **Note:** This project was created for learning Rust and for hosting **first-party video content** such as marketing pages, documentation, onboarding, and product demos. It is intended for **self-hosted website video delivery**, not as a public video hosting or social video platform. It allows me to host marketing videos for my projects without relying on YouTube, which can be blocked in certain regions, is heavy with tracking and ads, and may remove content arbitrarily.


> **PS:** You can swap the video player for any other player the core feature of this project is generating HLS (`.m3u8`) video streams.


## Features

- **Video Upload & Transcoding**: Upload videos with automatic HLS transcoding at multiple resolutions (4K, 1440p, 1080p, 720p, 480p, 360p).
- **Cloudflare R2 Integration**: Store video segments and thumbnails on R2 for fast, scalable, and cost-efficient delivery.
- **Hardware & Software Encoding**: Supports NVIDIA (h264_nvenc), AMD/Intel VAAPI (h264_vaapi), Intel QuickSync (h264_qsv), or CPU-based encoding (libx264).
- **Subtitle Handling**: Extract and display ASS/SSA/SRT subtitles from MKV files using libass rendering.
- **Chapter Support**: Read and present video chapters from container metadata.
- **Embedded Font Extraction**: Extract fonts from MKV containers for accurate subtitle rendering.
- **Large File Uploads**: Supports chunked uploads with progress monitoring.
- **Admin Dashboard**: Modern Next.js web interface for managing videos, uploads, and analytics.
- **Background Processing**: Queue-based video encoding with concurrency limits for optimized performance.

## Supported Video Formats

The platform accepts videos in the following container formats:

| Format | Extensions | Common Use Cases |
|--------|------------|------------------|
| **MP4** | `.mp4`, `.m4v` | Most common format, universal compatibility |
| **QuickTime** | `.mov` | macOS/iOS recordings, professional video |
| **Matroska** | `.mkv` | High-quality video with subtitles/chapters |
| **AVI** | `.avi` | Legacy Windows format |
| **WebM** | `.webm` | Web-optimized format |
| **Flash Video** | `.flv` | Streaming video |
| **Windows Media** | `.wmv` | Windows-native format |
| **MPEG** | `.mpg`, `.mpeg` | DVD and broadcast video |
| **MPEG-TS** | `.ts`, `.mts`, `.m2ts` | Broadcast and camera recordings |
| **3GPP** | `.3gp`, `.3g2` | Mobile phone video |

### Input Video Codecs Supported

FFmpeg can decode virtually any video codec, including:
- H.264/AVC, H.265/HEVC, H.266/VVC
- VP8, VP9, AV1
- MPEG-1, MPEG-2, MPEG-4
- ProRes, DNxHD, DivX, Xvid

### Output Format

All videos are transcoded to:
- **Video Codec**: H.264 (High Profile, Level 4.1)
- **Audio Codec**: AAC (128kbps, stereo)
- **Container**: HLS (HTTP Live Streaming)
- **Delivery**: MPEG-TS segments + M3U8 playlists

**Note:** Videos with metadata/timecode streams (common in professional cameras and screen recordings) are automatically handled - only the video and audio streams are encoded while metadata is safely ignored.

## Tech Stack

### Backend (Rust)
- **Axum** – Web server framework
- **SQLx** – SQLite for storing video metadata
- **AWS SDK** – R2/S3-compatible storage
- **ClickHouse** – Analytics and view tracking
- **FFmpeg** – Video processing, transcoding, and metadata extraction

### Frontend (Next.js)
- **Next.js 16** with App Router
- **Tailwind CSS 4** + DaisyUI 5
- **TypeScript**

## Prerequisites

- Cloudflare R2 bucket
- Rust (2024 edition)
- FFmpeg with encoding support
- Bun (for web UI)

## Configuration

Copy `config.yml.example` to `config.yml` and configure:

```yaml
server:
  host: "0.0.0.0"
  port: 3000
  secret_key: "your-secret-key"
  admin_password: "your-admin-password"
  max_concurrent_encodes: 1
  max_concurrent_uploads: 30

r2:
  endpoint: "https://<accountid>.r2.cloudflarestorage.com"
  bucket: "your-bucket"
  access_key_id: "your-access-key"
  secret_access_key: "your-secret-key"
  public_base_url: "https://your-domain.com/"

video:
  encoder: "libx264"  # or h264_nvenc, h264_vaapi, h264_qsv
```

## Installation

### Backend

```bash

# Configure your yml file
mv example.config.yml config.yml

# Go to cloudflare.com
# Click from left sidebar [Storage & databases] -> [R2 object storage] -> [Overview]
# Click [+ Create bucket]
# Enter Bucket Name | Location toggle [Automatic] | Default Storage Class toggle [Standard]
# Click Create Bucket

# Fill out r2 section in config.yml
# Go to [R2 Object Storage] -> [bucket name] -> click Settings
# copy [Public Development URL] -> [public_base_url]
# copy [S3 API] -> [endpoint]
# Enable CORS on the bucket click [CORS Policy] paste and save
# [{"AllowedOrigins":["*"],"AllowedMethods":["GET","HEAD"],"AllowedHeaders":["Range","Content-Type","Accept"],"ExposeHeaders":["Content-Length","Content-Range","Accept-Ranges"],"MaxAgeSeconds":86400}]

# Go to https://dash.cloudflare.com/profile/api-tokens -> Click [Create Token]
# copy [Access Key ID] -> [access_key_id]
# copy [Secret Access Key] -> [secret_access_key]



# Build the Rust backend
cargo build --release

# Run the server
./target/release/r2_video_hosting

```

### Web UI

```bash
cd admin-webui

# Install dependencies
bun install

# Development
bun run dev

# Production build
bun run build
```

## API Endpoints

### Public
- `GET /player/{id}` - Embedded video player with libass subtitle rendering
- `GET /hls/{id}/{file}` - HLS segments and playlists
- `GET /api/videos/{id}/subtitles` - List available subtitles
- `GET /api/videos/{id}/subtitles/{track}` - Get subtitle file
- `GET /api/videos/{id}/attachments` - List font attachments
- `GET /api/videos/{id}/chapters` - Get video chapters
- `GET /api/progress/{upload_id}` - Upload/encoding progress

### Protected (requires Bearer token)
- `POST /api/upload` - Upload video file
- `POST /api/upload/chunk` - Chunked upload
- `POST /api/upload/finalize` - Finalize chunked upload
- `GET /api/videos` - List videos with pagination/filtering
- `PUT /api/videos/{id}` - Update video metadata
- `DELETE /api/videos` - Delete videos
- `GET /api/queues` - List processing queue
- `DELETE /api/queues/{id}` - Cancel queued item

## Database

SQLite is used for video metadata with migrations in `migrations/`:
- Videos table with FTS5 search
- Subtitles and attachments metadata
- Chapters table

## NOTES / TODO

- [ ] Test all of the Input Video Codec (some are producing unplayable playback either with Shaka Player or via FFMPEG out)
- [x] Implement bulk deletion for multiple large videos instead of deleting files individually, as current method is slow.
- [x] Add proper rate-limiting protection to the admin web UI.
- [x] Test Cloudflare R2 CDN playback of media thoroughly.
- [x] Implement improved compression techniques and optimize bitrate calculation for efficiency and quality.
- [x] Remove analytics and prep for prod
- [x] Fix FFmpeg encoding to handle videos with timecode/data streams (explicit stream mapping with `-map 0:v:0`)
- [x] Add proper content-type headers for HLS playlists and segments
- [x] Implement video format validation on upload (file extension only)
## License

[Apache License 2.0](./LICENSE)


## Is this legal?

> _"This project can deliver **HLS video** backed by **Cloudflare R2**. According to Cloudflare’s public Terms update, customers may serve video and large files through the CDN when the content is hosted on Cloudflare services such as R2."_

- [Update Terms Explanation](https://blog.cloudflare.com/updated-tos)
- [CF Terms](https://www.cloudflare.com/terms/)

---
_Created by Asadbek Karimov  | [contact@asadk.dev](mailto:contact@asadk.dev) | [asadk.dev](https://asadk.dev)_