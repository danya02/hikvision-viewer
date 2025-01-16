-- Add migration script here
-- Create a table that maps clip UUIDs to thumbnail image IDs.
CREATE TABLE IF NOT EXISTS clip_thumbnail_images (
    clip_uuid TEXT NOT NULL REFERENCES server_clips (local_file_uuid),

    -- The kind of thumbnail it is. There can only be one thumbnail of a given type per clip.
    thumbnail_type INTEGER NOT NULL,
    PRIMARY KEY (clip_uuid, thumbnail_type)
);