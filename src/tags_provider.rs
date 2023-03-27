use std::sync::Arc;
use std::time::Duration;

use moka::future::Cache;

use crate::clients::kavita_client::KavitaClient;
use crate::clients::komga_client::KomgaClient;
use crate::models::errors::HttpError;

pub struct UpscaleTagChecker {
    upscale_tag: Option<String>,
    komga: Arc<KomgaClient>,
    kavita: Arc<KavitaClient>,
    cache: Cache<String, bool>,
}

impl UpscaleTagChecker {
    pub fn new(upscale_tag: Option<String>, komga: Arc<KomgaClient>, kavita: Arc<KavitaClient>) -> Self {
        let cache = Cache::builder()
            .time_to_live(Duration::from_secs(3 * 60))
            .build();

        Self { upscale_tag, komga, kavita, cache }
    }

    pub async fn kavita_contains_upscale_tag(&self, book_id: &str, cookie: &str) -> Result<bool, HttpError> {
        match &self.upscale_tag {
            None => Ok(false),
            Some(upscale_tag) => {
                match self.cache.get(book_id) {
                    None => Ok(self.check_kavita_tags(upscale_tag, book_id, cookie).await?),
                    Some(contains) => Ok(contains)
                }
            }
        }
    }

    pub async fn komga_contains_upscale_tag(&self, book_id: &str, cookie: &str) -> Result<bool, HttpError> {
        match &self.upscale_tag {
            None => Ok(false),
            Some(upscale_tag) => {
                match self.cache.get(book_id) {
                    None => Ok(self.check_komga_tags(upscale_tag, book_id, cookie).await?),
                    Some(contains) => Ok(contains)
                }
            }
        }
    }

    async fn check_komga_tags(&self, upscale_tag: &String, book_id: &str, cookie: &str) -> Result<bool, HttpError> {
        let book = self.komga.get_book(book_id, cookie).await?;
        let series = self.komga.get_series(&book.series_id, cookie).await?;
        let contains_tag = book.metadata.tags.contains(upscale_tag) || series.metadata.tags.contains(upscale_tag);
        self.cache.insert(book_id.to_string(), contains_tag).await;

        Ok(contains_tag)
    }

    async fn check_kavita_tags(&self, upscale_tag: &String, book_id: &str, cookie: &str) -> Result<bool, HttpError> {
        let book = self.komga.get_book(book_id, cookie).await?;
        let series = self.komga.get_series(&book.series_id, cookie).await?;
        let contains_tag = book.metadata.tags.contains(upscale_tag) || series.metadata.tags.contains(upscale_tag);
        self.cache.insert(book_id.to_string(), contains_tag).await;

        Ok(contains_tag)
    }
}