use std::net::SocketAddr;
use std::time::Duration;

use axum::Router;
use axum::routing::{any, get, patch, post};
use futures::FutureExt;
use log::info;
use tokio::sync::broadcast::Receiver;
use tokio::time::sleep;

use crate::app_state::AppState;
use crate::handlers::config::{get_config, update_config};
use crate::handlers::komga::{check_tags_on_book_metadata_update, check_tags_on_series_metadata_update};
use crate::handlers::proxy::{kavita_ws_proxy_handler, proxy_handler};
use crate::handlers::upscale::{upscale_kavita, upscale_komga};

pub async fn start(state: AppState, mut shutdown_rx: Receiver<()>) {
    let config = state.config.clone();
    let mut force_shutdown_rx = state.shutdown_tx.subscribe();

    let routes = make_routes(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    let server = axum::Server::bind(&addr)
        .serve(routes.into_make_service())
        .with_graceful_shutdown(async move { shutdown_rx.recv().await.unwrap() });

    let force_shutdown = force_shutdown_rx.recv()
        .then(|_| async { sleep(Duration::from_secs(1)).await; });

    tokio::select! {
        _ = server => { info!("Graceful server shutdown") }
        _ = force_shutdown  => { info!("Forced server shutdown") }
    }
}

fn make_routes(state: AppState) -> Router {
    let config = state.config.clone();

    let mut routes = Router::new()
        .route("/api/v1/books/:book_id/pages/:page_number", get(upscale_komga))
        .route("/api/reader/image", get(upscale_kavita))
        .route("/hubs/messages", get(kavita_ws_proxy_handler))
        .route("/", any(proxy_handler))
        .route("/*any", any(proxy_handler));

    if let Some(_) = &config.upscale_tag {
        routes = routes
            .route("/api/v1/series/:series_id/metadata", patch(check_tags_on_series_metadata_update))
            .route("/api/v1/books/:book_id/metadata", patch(check_tags_on_book_metadata_update))
    }

    if config.allow_config_updates {
        routes = routes
            .route("/kurp/config", get(get_config))
            .route("/kurp/config", post(update_config));
    }
    routes.with_state(state)
}