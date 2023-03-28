use log::{debug, info};

use crate::models::errors::HttpError;
use crate::models::komga::{KomgaBook, KomgaSeries};

pub struct KomgaClient {
    client: reqwest::Client,
    base_uri: String,
}

impl KomgaClient {
    pub fn new(client: reqwest::Client, base_uri: String) -> Self {
        Self { client, base_uri }
    }

    pub async fn get_book(&self, book_id: &str, cookie: &str) -> Result<KomgaBook, HttpError> {
        let result = self.client.get(format!("{}/api/v1/books/{}", &self.base_uri, book_id))
            .header("Cookie", cookie)
            .send().await
            .map_err(|err| HttpError { message: err.to_string() })?;
        info!("GET {} {}", result.url(), result.status());
        if !result.status().is_success() {
            return Err(HttpError { message: format!("{}, {}", result.status(), result.url()) });
        }

        let json = result.json::<KomgaBook>().await
            .map_err(|err| HttpError { message: err.to_string() })?;

        Ok(json)
    }
    pub async fn get_series(&self, series_id: &str, cookie: &str) -> Result<KomgaSeries, HttpError> {
        let result = self.client.get(format!("{}/api/v1/series/{}", &self.base_uri, series_id))
            .header("cookie", cookie)
            .send().await
            .map_err(|err| HttpError { message: err.to_string() })?;

        info!("GET {} {}", result.url(), result.status());
        if !result.status().is_success() {
            return Err(HttpError { message: format!("{}, {}", result.status(), result.url()) });
        }

        let json = result.json::<KomgaSeries>().await
            .map_err(|err| HttpError { message: err.to_string() })?;

        Ok(json)
    }
}