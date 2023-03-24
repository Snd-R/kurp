use std::sync::Arc;
use std::time::Duration;

use futures::future::select_all;
use futures::FutureExt;
use log::info;
use tokio::sync::broadcast;
use tokio::time::sleep;
use warp::Filter;

use crate::config::app_config::AppConfig;
use crate::handlers::errors::handle_rejection;
use crate::routes::config::config_routes;
use crate::routes::proxy::proxy_route;
use crate::routes::upscale::upscale_routes;
use crate::upscaler::upscale_actor::UpscaleActorHandle;

pub async fn start(
    config: Arc<AppConfig>,
    upscaler: UpscaleActorHandle,
) {
    let (tx, mut graceful_shutdown_rx) = broadcast::channel::<bool>(10);
    let shutdown_tx = Arc::new(tx);
    let mut force_shutdown_rx = shutdown_tx.subscribe();

    let api = config_routes(shutdown_tx, config.clone())
        .or(upscale_routes(config.clone(), upscaler.clone()))
        .recover(handle_rejection)
        .or(proxy_route(config.clone()));

    let (addr, server) = warp::serve(api)
        .bind_with_graceful_shutdown(([0, 0, 0, 0], config.port),
                                     async move { graceful_shutdown_rx.recv().await.unwrap(); });

    // FIXME existing connections will not be closed
    let force_shutdown = force_shutdown_rx.recv()
        .then(|_| async move { sleep(Duration::from_secs(1)).await; })
        .boxed();
    info!( "listening on http://{}", addr);

    select_all(vec![server.boxed(), force_shutdown]).await;
}