use std::sync::Arc;

use warp::{Filter, Rejection, Reply};
use warp_reverse_proxy::{extract_request_data_filter, proxy_to_and_forward_response};

use crate::config::app_config::AppConfig;

pub fn proxy_route(
    config: Arc<AppConfig>,
) -> impl Filter<Extract=(impl Reply, ), Error=Rejection> + Clone {
    return warp::any()
        .map(move || (config.upstream_url.clone(), "".to_string())).untuple_one()
        .and(extract_request_data_filter())
        .and_then(proxy_to_and_forward_response);
}