use std::str::FromStr;
use std::sync::Arc;

use hyper::Uri;
use log::LevelFilter;
use moka::future::Cache;
use ractor::Actor;
use reqwest::redirect::Policy;
use tokio::sync::broadcast;

use crate::app_state::AppState;
use crate::clients::kavita_client::KavitaClient;
use crate::clients::komga_client::KomgaClient;
use crate::clients::proxy_client::ProxyClient;
use crate::clients::websocket_proxy_client::WebsocketProxyClient;
use crate::config::app_config::AppConfig;
use crate::tags_provider::UpscaleTagChecker;
use crate::upscaler::upscale_actor::{UpscaleSupervisorActor, UpscaleSupervisorMessage};

mod config;
mod upscaler;
mod http_compression;
mod models;
mod handlers;
mod clients;
mod tags_provider;
mod app_state;
mod server;


#[tokio::main]
async fn main() {
    env_logger::Builder::new()
        .filter_level(LevelFilter::Info)
        .parse_default_env()
        .init();

    let (upscale_actor, _) = Actor::spawn(None, UpscaleSupervisorActor, ())
        .await
        .expect("Failed to start Upscale Actor!");

    loop {
        upscale_actor.send_message(UpscaleSupervisorMessage::Destroy)
            .expect("Failed to send Upscaler Destroy message");
        let config = Arc::new(AppConfig::new().unwrap());
        upscale_actor.send_message(UpscaleSupervisorMessage::Init(config.clone()))
            .expect("Failed to send Upscaler Init message");

        let (tx, graceful_shutdown_rx) = broadcast::channel::<()>(10);

        let reqwest_client = reqwest::Client::builder()
            .redirect(Policy::none())
            .build()
            .expect("Reqwest client couldn't build");


        let upstream_url = Uri::from_str(config.upstream_url.as_str()).unwrap();
        let upstream_url_str = upstream_url.to_string().strip_suffix("/").unwrap().to_string();
        let komga_client = Arc::new(KomgaClient::new(reqwest_client.clone(), upstream_url_str.clone()));
        let kavita_client = Arc::new(KavitaClient::new(reqwest_client.clone(), upstream_url_str.clone()));

        let tag_provider = Arc::new(UpscaleTagChecker::new(
            config.upscale_tag.clone(),
            komga_client,
            kavita_client,
        ));

        let upscale_call_cache = Cache::new(1_000);
        let proxy_client = ProxyClient::new(reqwest_client, upstream_url_str);
        let ws_url = Uri::builder()
            .scheme("ws")
            .authority(upstream_url.authority().unwrap().as_str())
            .path_and_query(upstream_url.path())
            .build().unwrap();
        let ws_url_str = ws_url.to_string().strip_suffix("/").unwrap().to_string();
        let websocket_proxy_client = WebsocketProxyClient::new(ws_url_str);

        let state = AppState {
            config,
            upscaler: upscale_actor.clone(),
            proxy_client: Arc::new(proxy_client),
            websocket_proxy_client: Arc::new(websocket_proxy_client),
            upscale_call_history_cache: Arc::new(upscale_call_cache),
            upscale_tag_checker: tag_provider,
            shutdown_tx: tx,
        };
        server::start(state, graceful_shutdown_rx).await;
    }
}