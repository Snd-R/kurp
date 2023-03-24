use std::convert::Infallible;
use std::ops::Deref;
use std::sync::Arc;

use tokio::sync::broadcast::Sender;

use crate::config::app_config::AppConfig;

pub async fn update_config(tx: Arc<Sender<bool>>, new_config: AppConfig) -> Result<impl warp::Reply, Infallible> {
    AppConfig::write_config(new_config);
    tx.send(true).unwrap();

    Ok(warp::reply())
}

pub async fn get_config(config: Arc<AppConfig>) -> Result<impl warp::Reply, Infallible> {
    Ok(warp::reply::json(config.deref()))
}
