use headers::{Authorization, HeaderMap, HeaderMapExt};
use headers::authorization::Bearer;
use log::info;

use crate::models::errors::HttpError;
use crate::models::kavita::{KavitaChapter, KavitaSeriesMetadata, KavitaVolume};

pub struct KavitaClient {
    client: reqwest::Client,
    base_uri: String,
}

impl KavitaClient {
    pub fn new(client: reqwest::Client, base_uri: String) -> Self {
        Self { client, base_uri }
    }

    pub async fn get_chapter(
        &self,
        chapter_id: &u32,
        auth: Authorization<Bearer>,
    ) -> Result<KavitaChapter, HttpError> {
        let mut headers = HeaderMap::new();
        headers.typed_insert(auth);

        let result = self.client.get(format!("{}/api/series/chapter", self.base_uri))
            .query(&[("chapterId", chapter_id)])
            .headers(headers)
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

    pub async fn get_volume(
        &self,
        volume_id: &u32,
        auth: Authorization<Bearer>,
    ) -> Result<KavitaVolume, HttpError> {
        let mut headers = HeaderMap::new();
        headers.typed_insert(auth);

        let result = self.client.get(format!("{}/api/series/volume", self.base_uri))
            .query(&[("volumeId", volume_id)])
            .headers(headers)
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

    pub async fn get_series_metadata(
        &self,
        series_id: &u32,
        auth: Authorization<Bearer>,
    ) -> Result<KavitaSeriesMetadata, HttpError> {
        let mut headers = HeaderMap::new();
        headers.typed_insert(auth);

        let result = self.client.get(format!("{}/api/series/metadata", self.base_uri))
            .query(&[("seriesId", series_id)])
            .headers(headers)
            .send().await
            .map_err(|err| HttpError { message: err.to_string() })?;

        info!("GET {} {}", result.url(), result.status());
        if !result.status().is_success() {
            return Err(HttpError { message: format!("{}, {}", result.status(), result.url()) });
        }

        let json = result.json::<KavitaSeriesMetadata>().await
            .map_err(|err| HttpError { message: err.to_string() })?;

        Ok(json)
    }
}