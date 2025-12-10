-- Add is_public column (defaults to private for security)
ALTER TABLE videos ADD COLUMN is_public INTEGER NOT NULL DEFAULT 0;

-- Create index for filtering public videos
CREATE INDEX idx_videos_is_public ON videos(is_public);
