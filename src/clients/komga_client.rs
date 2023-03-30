use headers::{Authorization, Cookie, HeaderMap, HeaderMapExt};
use headers::authorization::Basic;
use log::info;

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

    pub async fn get_book(
        &self,
        book_id: &str,
        cookie: Option<Cookie>,
        auth: Option<Authorization<Basic>>,
    ) -> Result<KomgaBook, HttpError> {
        let result = self.client.get(format!("{}/api/v1/books/{}", &self.base_uri, book_id))
            .headers(self.auth_header(cookie, auth)?)
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
    pub async fn get_series(
        &self,
        series_id: &str,
        cookie: Option<Cookie>,
        auth: Option<Authorization<Basic>>,
    ) -> Result<KomgaSeries, HttpError> {
        let result = self.client.get(format!("{}/api/v1/series/{}", &self.base_uri, series_id))
            .headers(self.auth_header(cookie, auth)?)
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

    fn auth_header(
        &self,
        cookie: Option<Cookie>,
        auth: Option<Authorization<Basic>>,
    ) -> Result<HeaderMap, HttpError> {
        let mut headers = HeaderMap::new();
        match auth {
            Some(auth) => { headers.typed_insert(auth) }
            None => {
                match cookie {
                    Some(cookie) => { headers.typed_insert(cookie) }
                    None => { return Err(HttpError { message: "No auth header".to_string() }); }
                }
            }
        };

        Ok(headers)
    }
}