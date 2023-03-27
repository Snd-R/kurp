use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct KavitaChapter {
    pub id: u32,
    #[serde(rename = "volumeId")]
    pub volume_id: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct KavitaVolume {
    pub id: u32,
    #[serde(rename = "seriesId")]
    pub series_id: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct KavitaSeriesMetadata {
    pub tags: Vec<KavitaTag>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct KavitaTag {
    pub id: u32,
    pub title: String,
}
