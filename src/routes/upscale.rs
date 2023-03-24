use std::sync::Arc;

use warp::{Filter, Rejection, Reply};
use warp_reverse_proxy::extract_request_data_filter;

use crate::config::app_config::AppConfig;
use crate::handlers::upscale::upscale;
use crate::routes::filters::{with_config, with_upscaler};
use crate::upscaler::upscale_actor::UpscaleActorHandle;

fn komga_upscale(
    config: Arc<AppConfig>,
    upscaler: UpscaleActorHandle,
) -> impl Filter<Extract=(impl Reply, ), Error=Rejection> + Clone {
    return warp::path!("api"/"v1"/"books" / String /"pages"/ i32)
        .map(|_id, _page| ()).untuple_one()
        .and(with_config(config))
        .and(with_upscaler(upscaler))
        .and(extract_request_data_filter())
        .and_then(upscale);
}

fn kavita_upscale(
    config: Arc<AppConfig>,
    upscaler: UpscaleActorHandle,
) -> impl Filter<Extract=(impl Reply, ), Error=Rejection> + Clone {
    return warp::path!("api"/"reader"/"image")
        .and(with_config(config))
        .and(with_upscaler(upscaler))
        .and(extract_request_data_filter())
        .and_then(upscale);
}

pub fn upscale_routes(
    config: Arc<AppConfig>,
    upscaler: UpscaleActorHandle,
) -> impl Filter<Extract=(impl Reply, ), Error=Rejection> + Clone {
    komga_upscale(config.clone(), upscaler.clone())
        .or(kavita_upscale(config, upscaler))
}
