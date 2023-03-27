use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct KomgaSeries {
    pub id: String,
    pub metadata: KomgaSeriesMetadata,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct KomgaSeriesMetadata {
    pub tags: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct KomgaBook {
    pub id: String,
    #[serde(rename = "seriesId")]
    pub series_id: String,
    pub metadata: KomgaBookMetadata,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct KomgaBookMetadata {
    pub tags: Vec<String>,
}
