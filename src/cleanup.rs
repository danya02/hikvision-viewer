use std::path::Path;

use sqlx::SqlitePool;

pub async fn cleanup_files(db: &SqlitePool, root_path: &Path) -> anyhow::Result<()> {
    // For every file in root_path/video, extract the UUID by regex,
    // then delete the file if it does not exist in the database.
    let mut files = tokio::fs::read_dir(root_path.join("video")).await?;

    let mut files_seen = 0;
    let mut files_deleted = 0;
    while let Some(file) = files.next_entry().await? {
        if !file.file_type().await?.is_file() {
            continue;
        }
        files_seen += 1;

        let filename = file.file_name().to_string_lossy().to_string();
        let uuid =
            regex::Regex::new(r"[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}")
                .unwrap()
                .find(&filename)
                .unwrap()
                .as_str();

        if sqlx::query!("SELECT * FROM server_clips WHERE local_file_uuid = ?", uuid)
            .fetch_optional(db)
            .await?
            .is_none()
        {
            files_deleted += 1;
            println!(
                "File {filename} is not in the database, deleting. {files_deleted}/{files_seen}"
            );
            tokio::fs::remove_file(file.path()).await?;
        }
    }

    // Do the same for the thumbnails dir
    let mut thumbs_files = tokio::fs::read_dir(root_path.join("thumbnails")).await?;
    while let Some(thumb) = thumbs_files.next_entry().await? {
        if !thumb.file_type().await?.is_file() {
            continue;
        }
        files_seen += 1;

        let filename = thumb.file_name().to_string_lossy().to_string();
        let uuid =
            regex::Regex::new(r"[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}")
                .unwrap()
                .find(&filename)
                .unwrap()
                .as_str();

        if sqlx::query!(
            "SELECT * FROM clip_thumbnail_images WHERE clip_uuid = ?",
            uuid
        )
        .fetch_optional(db)
        .await?
        .is_none()
        {
            files_deleted += 1;
            println!(
                "File {filename} is not in the database, deleting. {files_deleted}/{files_seen}"
            );
            tokio::fs::remove_file(thumb.path()).await?;
        }
    }

    Ok(())
}
