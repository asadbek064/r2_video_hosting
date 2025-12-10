-- Remove analytics functionality
-- Drops view_count column and views table if they exist

-- Drop views table (if it exists from analytics migration)
DROP TABLE IF EXISTS views;

-- Drop indexes (if they exist)
DROP INDEX IF EXISTS idx_views_video_id;
DROP INDEX IF EXISTS idx_views_created_at;

-- Note: We intentionally leave the view_count column in videos table
-- Dropping columns in SQLite requires recreating the table, which is risky
-- The column is harmless (8 bytes per row) and simplifies migration
