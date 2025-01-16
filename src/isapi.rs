#[derive(serde::Deserialize, Clone, Debug)]
pub struct CMSearchResult {
    #[serde(rename = "searchID")]
    pub search_id: String,

    #[serde(rename = "responseStatus")]
    pub response_status: bool,

    #[serde(rename = "responseStatusStrg")]
    pub response_status_strg: String,

    #[serde(rename = "numOfMatches")]
    pub num_of_matches: u64,

    /// Missing if status is NO MATCHES
    #[serde(rename = "matchList")]
    pub match_list: Option<IsapiCmSearchResultMatchList>,
}

#[derive(serde::Deserialize, Clone, Debug, Default)]
pub struct IsapiCmSearchResultMatchList {
    /// Cannot be deserialized if list is empty, so use Option
    /// (to the consumer: unwrap_or_default)
    #[serde(rename = "$value")]
    pub matches: Option<Vec<IsapiCmSearchResultMatchItem>>,
}

#[derive(serde::Deserialize, Clone, Debug)]
pub struct IsapiCmSearchResultMatchItem {
    #[serde(rename = "sourceID")]
    pub source_id: String,

    #[serde(rename = "trackID")]
    pub track_id: String,

    #[serde(rename = "timeSpan")]
    pub time_span: IsapiCmSearchResultMatchTimeSpan,

    #[serde(rename = "mediaSegmentDescriptor")]
    pub media_segment_descriptor: IsapiCmSearchResultMatchMediaSegmentDescriptor,
}

#[derive(serde::Deserialize, Clone, Debug)]
pub struct IsapiCmSearchResultMatchTimeSpan {
    #[serde(rename = "startTime")]
    pub start_time: chrono::DateTime<chrono::Utc>,

    #[serde(rename = "endTime")]
    pub end_time: chrono::DateTime<chrono::Utc>,
}

#[derive(serde::Deserialize, Clone, Debug)]
pub struct IsapiCmSearchResultMatchMediaSegmentDescriptor {
    #[serde(rename = "contentType")]
    pub content_type: String,

    #[serde(rename = "codecType")]
    pub codec_type: String,

    #[serde(rename = "playbackURI")]
    pub playback_uri: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_item_deserialize() {
        let xml = r#"
        <searchMatchItem>
<sourceID>{00000000-0000-0000-0000-000000000000}</sourceID>
<trackID>101</trackID>
<timeSpan>
<startTime>2024-11-26T12:43:11Z</startTime>
<endTime>2024-11-26T12:44:09Z</endTime>
</timeSpan>
<mediaSegmentDescriptor>
<contentType>video</contentType>
<codecType>H.264-BP</codecType>
<playbackURI>rtsp://10.22.0.11/Streaming/tracks/101/?starttime=20241126T124311Z&amp;endtime=20241126T124409Z&amp;name=00000000022006913&amp;size=3895472</playbackURI>
</mediaSegmentDescriptor>
<metadataMatches>
<metadataDescriptor>recordType.meta.hikvision.com/allEvent</metadataDescriptor>
</metadataMatches>
</searchMatchItem>
<searchMatchItem>
"#;

        let result: IsapiCmSearchResultMatchItem = serde_xml_rs::from_str(xml).unwrap();
        println!("{:#?}", result);
    }

    #[test]
    fn test_list_deserialize() {
        let xml = r#"
<matchList>
<searchMatchItem>
<sourceID>{00000000-0000-0000-0000-000000000000}</sourceID>
<trackID>101</trackID>
<timeSpan>
<startTime>2024-11-26T12:43:11Z</startTime>
<endTime>2024-11-26T12:44:09Z</endTime>
</timeSpan>
<mediaSegmentDescriptor>
<contentType>video</contentType>
<codecType>H.264-BP</codecType>
<playbackURI>rtsp://10.22.0.11/Streaming/tracks/101/?starttime=20241126T124311Z&amp;endtime=20241126T124409Z&amp;name=00000000022006913&amp;size=3895472</playbackURI>
</mediaSegmentDescriptor>
<metadataMatches>
<metadataDescriptor>recordType.meta.hikvision.com/allEvent</metadataDescriptor>
</metadataMatches>
</searchMatchItem>
<searchMatchItem>
<sourceID>{00000000-0000-0000-0000-000000000000}</sourceID>
<trackID>101</trackID>
<timeSpan>
<startTime>2024-11-26T12:44:35Z</startTime>
<endTime>2024-11-26T12:45:43Z</endTime>
</timeSpan>
<mediaSegmentDescriptor>
<contentType>video</contentType>
<codecType>H.264-BP</codecType>
<playbackURI>rtsp://10.22.0.11/Streaming/tracks/101/?starttime=20241126T124435Z&amp;endtime=20241126T124543Z&amp;name=00000000022007013&amp;size=4442408</playbackURI>
</mediaSegmentDescriptor>
<metadataMatches>
<metadataDescriptor>recordType.meta.hikvision.com/allEvent</metadataDescriptor>
</metadataMatches>
</searchMatchItem>
</matchList>
        "#;

        let result: IsapiCmSearchResultMatchList = serde_xml_rs::from_str(xml).unwrap();
        println!("{:#?}", result);
    }
}
