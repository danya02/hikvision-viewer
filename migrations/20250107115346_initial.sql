-- Add migration script here

-- Table containing clips which were ever seen from the NVR, and whether they were ever downloaded or not.
CREATE TABLE IF NOT EXISTS server_clips (
    media_url TEXT PRIMARY KEY NOT NULL,
    -- The unix time of when this clip started. Used to determine where to begin searching for new ones.
    start_unix_time INTEGER NOT NULL,
    -- The UUID of the downloaded file. Only set if we ever downloaded it. If the file with this UUID is missing, that means we downloaded it, then deleted it.
    local_file_uuid TEXT UNIQUE
);

CREATE INDEX server_clips_start_time ON server_clips (start_unix_time);
CREATE INDEX server_clips_uuid ON server_clips (local_file_uuid);