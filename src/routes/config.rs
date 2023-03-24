use std::sync::Arc;

use tokio::sync::broadcast::Sender;
use warp::{Filter, Rejection, Reply};

use crate::config::app_config::AppConfig;
use crate::handlers;
use crate::routes::filters::{json_body, with_channel, with_config};

fn update_config(tx: Arc<Sender<bool>>) -> impl Filter<Extract=(impl Reply, ), Error=Rejection> + Clone {
    return warp::path!("kurp"/"config")
        .and(warp::post())
        .and(with_channel(tx))
        .and(json_body())
        .and_then(handlers::config::update_config);
}

fn get_config(config: Arc<AppConfig>) -> impl Filter<Extract=(impl Reply, ), Error=Rejection> + Clone {
    return warp::path!("kurp"/"config")
        .and(warp::get())
        .and(with_config(config))
        .and_then(handlers::config::get_config);
}

pub fn config_routes(tx: Arc<Sender<bool>>, config: Arc<AppConfig>) -> impl Filter<Extract=(impl Reply, ), Error=Rejection> + Clone {
    update_config(tx)
        .or(get_config(config))
}