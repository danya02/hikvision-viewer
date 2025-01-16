use crate::ROOT_PATH;
use sqlx::SqlitePool;

pub enum ThumbnailType {
    /// The first frame of the video.
    FirstFrame = 0,
    /// A video clip where every frame is one minute apart.
    OneMinuteTimelapse = 1,
}

pub async fn make_thumbnails(db: &SqlitePool) -> anyhow::Result<()> {
    // Select all video clips that have a download UUID, but do not have a thumbnail of type 0.
    let missing_thumbs = sqlx::query!(
        "SELECT * FROM server_clips WHERE local_file_uuid IS NOT NULL AND NOT EXISTS (SELECT * FROM clip_thumbnail_images WHERE thumbnail_type = 0 AND clip_uuid = server_clips.local_file_uuid)"
    )
    .fetch_all(db).await?;

    let files = std::fs::read_dir(format!("{ROOT_PATH}/video"))
        .unwrap()
        .map(|v| v.unwrap())
        .map(|v| v.file_name().to_string_lossy().to_string())
        .collect::<Vec<String>>();

    for clip in missing_thumbs {
        let uuid = clip.local_file_uuid.unwrap();
        println!("Making thumbnail for clip {:?}", uuid);
        let file = files.iter().find(|v| v.contains(&uuid)).unwrap();
        std::process::Command::new("ffmpeg")
            .args([
                "-i",
                &format!("/clips/video/{}", file),
                "-frames:v",
                "1",
                "-vsync",
                "vfr",
                "-y",
                &format!("/clips/thumbnails/{uuid}_0.jpg"),
            ])
            .output()
            .unwrap();

        sqlx::query!(
            "INSERT INTO clip_thumbnail_images (thumbnail_type, clip_uuid) VALUES (0, ?)",
            uuid
        )
        .execute(db)
        .await
        .unwrap();
    }

    Ok(())
}
