use log::LevelFilter;

mod routes;
mod app_config;
mod upscaler;

#[tokio::main]

async fn main() {
    env_logger::Builder::new()
        .filter_level(LevelFilter::Info)
        .parse_default_env()
        .init();
    routes::routes().await
}