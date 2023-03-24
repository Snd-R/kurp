use std::sync::Arc;

use log::LevelFilter;

use EnabledUpscaler::Realcugan;

use crate::config::app_config::{AppConfig, EnabledUpscaler};
use crate::config::app_config::EnabledUpscaler::Waifu2x;
use crate::upscaler::upscale_actor::UpscaleActorHandle;
use crate::upscaler::upscaler::{RealCuganUpscaler, Upscaler, Waifu2xUpscaler};

mod config;
mod upscaler;
mod http_compression;
mod server;
mod models;
mod routes;
mod handlers;

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
        server::start(config, upscale_actor.clone()).await;
    }
}