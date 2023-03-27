use log::info;

use crate::models::errors::HttpError;
use crate::models::kavita::{KavitaChapter, KavitaVolume};

pub struct KavitaClient {
    client: reqwest::Client,
    base_uri: String,
}

impl KavitaClient {
    pub fn new(client: reqwest::Client, base_uri: String) -> Self {
        Self { client, base_uri }
    }

    pub async fn get_chapter(&self, chapter_id: &str, cookie: &str) -> Result<KavitaChapter, HttpError> {
        let result = self.client.get("/api/series/chapter/")
            .query(&[("chapterId", chapter_id)])
            .header("cookie", cookie)
            .send().await
            .map_err(|err| HttpError { message: err.to_string() })?;

        info!("GET {} {}", result.url(), result.status());
        if !result.status().is_success() {
            return Err(HttpError { message: format!("{}, {}", result.status(), result.url()) });
        }

        let json = result.json::<KavitaChapter>().await
            .map_err(|err| HttpError { message: err.to_string() })?;

        Ok(json)
    }

    pub async fn get_volume(&self, volume_id: &str, cookie: &str) -> Result<KavitaVolume, HttpError> {
        let result = self.client.get("/api/series/volume")
            .query(&[("volumeId", volume_id)])
            .header("cookie", cookie)
            .send().await
            .map_err(|err| HttpError { message: err.to_string() })?;

        info!("GET {} {}", result.url(), result.status());
        if !result.status().is_success() {
            return Err(HttpError { message: format!("{}, {}", result.status(), result.url()) });
        }

        let json = result.json::<KavitaVolume>().await
            .map_err(|err| HttpError { message: err.to_string() })?;

        Ok(json)
    }

    pub async fn get_series_metadata(&self, series_id: &str, cookie: &str) -> Result<KavitaVolume, HttpError> {
        let result = self.client.get("/api/series/metadata")
            .query(&[("seriesId", series_id)])
            .header("cookie", cookie)
            .send().await
            .map_err(|err| HttpError { message: err.to_string() })?;

        info!("GET {} {}", result.url(), result.status());
        if !result.status().is_success() {
            return Err(HttpError { message: format!("{}, {}", result.status(), result.url()) });
        }

        let json = result.json::<KavitaVolume>().await
            .map_err(|err| HttpError { message: err.to_string() })?;

        Ok(json)
    }
}