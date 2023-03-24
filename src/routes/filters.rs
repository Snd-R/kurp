use std::convert::Infallible;
use std::sync::Arc;

use tokio::sync::broadcast::Sender;
use warp::{Filter, Rejection};

use crate::config::app_config::AppConfig;
use crate::upscaler::upscale_actor::UpscaleActorHandle;

pub fn with_config(config: Arc<AppConfig>) -> impl Filter<Extract=(Arc<AppConfig>, ), Error=Infallible> + Clone {
    warp::any().map(move || config.clone())
}

pub fn with_upscaler(upscaler: UpscaleActorHandle) -> impl Filter<Extract=(UpscaleActorHandle, ), Error=Infallible> + Clone {
    warp::any().map(move || upscaler.clone())
}

pub fn with_channel(tx: Arc<Sender<bool>>) -> impl Filter<Extract=(Arc<Sender<bool>>, ), Error=Infallible> + Clone {
    warp::any().map(move || tx.clone())
}

pub fn json_body() -> impl Filter<Extract=(AppConfig, ), Error=Rejection> + Clone {
    warp::body::content_length_limit(1024 * 16).and(warp::body::json())
}
