pub mod cleanup;
pub mod isapi;
pub mod thumbnails;

use std::path::Path;

use cleanup::cleanup_files;
use diqwest::WithDigestAuth;
use isapi::CMSearchResult;
use sqlx::SqlitePool;
use thumbnails::make_thumbnails;

#[derive(Clone)]
pub struct HiwatchConfig {
    pub url: String,
    pub user: String,
    pub password: String,
}

const ROOT_PATH: &str = "/mnt";

#[tokio::main]
async fn main() {
    println!("Hello, world!");
    dotenvy::dotenv().unwrap();

    let url = std::env::var("DATABASE_URL").unwrap();
    let db = sqlx::SqlitePool::connect(&url).await.unwrap();
    sqlx::migrate!().run(&db).await.unwrap();

    cleanup_files(&db, &Path::new(ROOT_PATH)).await.unwrap();

    let conf = HiwatchConfig {
        url: std::env::var("HIWATCH_URL").unwrap(),
        user: std::env::var("HIWATCH_USERNAME").unwrap(),
        password: std::env::var("HIWATCH_PASSWORD").unwrap(),
    };
    fetch_new_clip_meta(&conf, &db).await.unwrap();

    // For every clip that was not yet downloaded, download it
    let clips_to_download = sqlx::query!(
        "SELECT * FROM server_clips WHERE local_file_uuid IS NULL ORDER BY start_unix_time ASC"
    )
    .fetch_all(&db)
    .await
    .unwrap();

    for clip in clips_to_download {
        println!("Downloading clip {:?}", clip.media_url);
        download_clip(&clip.media_url, &conf, &db).await.unwrap();
    }

    make_thumbnails(&db).await.unwrap();
}

async fn fetch_new_clip_meta(conf: &HiwatchConfig, db: &SqlitePool) -> anyhow::Result<()> {
    let client = reqwest::Client::new();

    // Figure out the start date by finding the most recent clip.
    let start_date = sqlx::query!(
        "SELECT start_unix_time FROM server_clips ORDER BY start_unix_time DESC LIMIT 1"
    )
    .fetch_optional(db)
    .await?;

    // Convert to chrono
    let start_date = start_date
        .map(|v| v.start_unix_time)
        // Camera was deployed in 2024, so no records exist before then: unix date for start of 2024.
        .unwrap_or(1704056400);
    let mut start_date = chrono::DateTime::from_timestamp(start_date, 0).unwrap();

    // start a few days before, to be safe that we have not missed any days in the meantime.
    start_date = start_date - chrono::Duration::days(5);

    while start_date < chrono::Utc::now() {
        // Collect clips for the current day.

        let start_date_formatted = start_date.to_rfc3339();
        let end_date = start_date + chrono::Duration::days(1);
        let end_date_formatted = end_date.to_rfc3339();

        println!("Query: {start_date_formatted} -- {end_date_formatted}");
        let response = client
            .post(&format!("{}/ISAPI/ContentMgmt/search", conf.url))
            .body(format!(
                r"
            <CMSearchDescription>
            <searchID>00000000-0000-0000-0000-000000000000</searchID>
            <trackList>
                <trackID>101</trackID>
                <trackID>201</trackID>
            </trackList>
            <timeSpanList>
            <timeSpan>
            <startTime>{start_date_formatted}</startTime>
            <endTime>{end_date_formatted}</endTime>
            </timeSpan>
            </timeSpanList>
            <maxResults>1000</maxResults>
            <searchResultPostion>0</searchResultPostion>
            <metadataList>
            <metadataDescriptor>//recordType.meta.std-cgi.com</metadataDescriptor>
            </metadataList>
            </CMSearchDescription>"
            ))
            .send_with_digest_auth(&conf.user, &conf.password)
            .await?;

        let results = response.text().await?;
        // println!("results: {}", results);
        // remove the first line with <?xml ... ?>
        let results = results.lines().skip(1).collect::<Vec<_>>().join("\n");
        let parsed_results: CMSearchResult = serde_xml_rs::from_str(&results).unwrap();
        // println!("parsed_results: {:?}", parsed_results);

        let matches = parsed_results
            .match_list
            .unwrap_or_default()
            .matches
            .unwrap_or_default();

        let mut new_matches = 0;
        for match_item in &matches {
            let timestamp = match_item.time_span.start_time.timestamp();
            // Try inserting it into the database. If error, it already exists there.
            let result = sqlx::query!(
                "INSERT INTO server_clips (media_url, start_unix_time) VALUES (?, ?)",
                match_item.media_segment_descriptor.playback_uri,
                timestamp,
            )
            .execute(db)
            .await;
            if result.is_ok() {
                new_matches += 1;
            }
        }

        println!("Matches: {}, New matches: {new_matches}", matches.len());

        // If there were any results, then set the start of the new search to be the most recent clip's start.
        // This ensures that the next search we do contains any clips that were not seen yet.
        if matches.len() > 0 {
            start_date = matches.last().unwrap().time_span.start_time;
        } else {
            // If the search range is empty, advance it by a day.
            start_date = start_date + chrono::Duration::days(1);
        }
    }

    Ok(())
}

async fn download_clip(
    clip_uri: &str,
    conf: &HiwatchConfig,
    db: &SqlitePool,
) -> anyhow::Result<()> {
    use diqwest::WithDigestAuth;
    use futures_util::StreamExt;
    use tokio::io::AsyncWriteExt;

    let client = reqwest::Client::new();
    let response = client
        .post(&format!("{}/ISAPI/ContentMgmt/download", conf.url))
        .body(format!(
            r#"
    <downloadRequest version="1.0" xmlns="http://www.isapi.org/ver20/XMLSchema">
    <playbackURI>{clip_uri}</playbackURI>
    </downloadRequest>
    "#,
        ))
        .send_with_digest_auth(&conf.user, &conf.password)
        .await?;

    let uuid = uuid::Uuid::new_v4();
    // Sanitize the URL into a file path, then append a UUID to it.
    // Store the mapping from UUID to original URL in a separate table.
    let sanitized_url = clip_uri
        .replace('/', "_")
        .replace('\\', "_")
        .replace(':', "_")
        .replace('?', "_")
        .replace('*', "_")
        .replace('"', "_")
        .replace('<', "_")
        .replace('>', "_")
        .replace('&', "_")
        .replace('|', "_");
    let clip_uuid = uuid.to_string();

    let mut file = tokio::fs::File::create(format!(
        "{ROOT_PATH}/video/{sanitized_url}_{clip_uuid}.hevc"
    ))
    .await?;
    let mut stream = response.bytes_stream();

    let mut bytes = 0;
    let mut mb = 0;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        file.write_all(&chunk).await?;
        bytes += chunk.len();
        if bytes / 1024 / 1024 != mb {
            mb = bytes / 1024 / 1024;
            println!("Downloaded {mb} MB");
        }
    }

    // Record the new UUID in the table
    sqlx::query!(
        "UPDATE server_clips SET local_file_uuid = ? WHERE media_url = ?",
        clip_uuid,
        clip_uri,
    )
    .execute(db)
    .await?;

    Ok(())
}
