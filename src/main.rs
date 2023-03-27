#![feature(result_option_inspect)]

use std::sync::Arc;

use log::LevelFilter;
use moka::future::Cache;
use reqwest::redirect::Policy;
use tokio::sync::broadcast;

use EnabledUpscaler::Realcugan;

use crate::app_state::AppState;
use crate::clients::kavita_client::KavitaClient;
use crate::clients::komga_client::KomgaClient;
use crate::clients::proxy_client::ProxyClient;
use crate::config::app_config::{AppConfig, EnabledUpscaler};
use crate::config::app_config::EnabledUpscaler::Waifu2x;
use crate::tags_provider::UpscaleTagChecker;
use crate::upscaler::upscale_actor::UpscaleActorHandle;
use crate::upscaler::upscaler::{RealCuganUpscaler, Upscaler, Waifu2xUpscaler};

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

    let upscale_actor = UpscaleActorHandle::new();
    loop {
        upscale_actor.deinitialize().await;

        let config = Arc::new(AppConfig::new().unwrap());

        let upscaler: Box<dyn Upscaler> = match config.upscaler {
            Waifu2x => Box::new(Waifu2xUpscaler::new(config.clone())),
            Realcugan => Box::new(RealCuganUpscaler::new(config.clone()))
        };

        upscale_actor.init(upscaler).await;
        let (tx, graceful_shutdown_rx) = broadcast::channel::<()>(10);

        let reqwest_client = reqwest::Client::builder()
            .redirect(Policy::none())
            .build()
            .expect("Reqwest client couldn't build");


        let upstream_url = config.upstream_url.clone();
        let komga_client = Arc::new(KomgaClient::new(reqwest_client.clone(), upstream_url.clone()));
        let kavita_client = Arc::new(KavitaClient::new(reqwest_client.clone(), upstream_url.clone()));

        let tag_provider = Arc::new(UpscaleTagChecker::new(
            config.upscale_tag.clone(),
            komga_client,
            kavita_client,
        ));

        let upscale_call_cache = Cache::new(1_000);

        let state = AppState {
            config,
            upscaler: upscale_actor.clone(),
            proxy_client: Arc::new(ProxyClient::new(reqwest_client, upstream_url)),
            upscale_call_history_cache: Arc::new(upscale_call_cache),
            upscale_tag_checker: tag_provider,
            shutdown_tx: tx,
        };
        server::start(state, graceful_shutdown_rx).await;
    }
}