use std::env;
use std::fs;
use std::path::PathBuf;

use config::{Config, ConfigError, Environment, File};
use once_cell::sync::OnceCell;
use serde_derive::Deserialize;
use waifu2x_ncnn_vulkan_rs::ModelType;

pub static CONFIG: OnceCell<AppConfig> = OnceCell::new();

pub fn get_global_config() -> &'static AppConfig {
    CONFIG.get_or_init(|| AppConfig::new().unwrap())
}

#[derive(Deserialize)]
#[allow(unused)]
pub struct AppConfig {
    pub komga_url: String,
    pub upscale: bool,
    pub return_format: Format,
    pub size_threshold_enabled: bool,
    pub size_threshold: u32,
    pub size_threshold_png: u32,
    pub waifu2x: Waifu2xConfig,
}

#[derive(Debug, Deserialize)]
pub enum Format {
    Png,
    Jpeg,
    WebP,
    Original,
}

#[derive(Deserialize)]
#[serde(remote = "ModelType")]
enum ModelTypeDef {
    Cunet,
    Upconv7AnimeStyleArtRgb,
    Upconv7Photo,
}

#[derive(Deserialize)]
pub struct Waifu2xConfig {
    pub gpuid: i32,
    pub scale: u32,
    pub noise: i32,

    #[serde(with = "ModelTypeDef")]
    pub model: ModelType,
    pub tile_size: u32,
    pub tta_mode: bool,
    pub num_threads: i32,
    pub models_path: String,
}

impl AppConfig {
    pub fn new() -> Result<Self, ConfigError> {
        let current_dir = env::current_dir().expect("can't read current dir");
        let dir_env = env::var("KURP_CONF_DIR");
        let config_dir: PathBuf = dir_env.map(|path| { PathBuf::from(path) })
            .unwrap_or_else(|_| { current_dir.clone() });

        fs::create_dir_all(&config_dir).expect("can't create config directory");


        let models_default_dir = current_dir.join("models");
        let mut waifu2x_config = config::Map::new();
        waifu2x_config.insert("gpuid".to_string(), "0");
        waifu2x_config.insert("scale".to_string(), "2");
        waifu2x_config.insert("noise".to_string(), "-1");
        waifu2x_config.insert("model".to_string(), "Cunet");
        waifu2x_config.insert("tile_size".to_string(), "0");
        waifu2x_config.insert("tta_mode".to_string(), "false");
        waifu2x_config.insert("num_threads".to_string(), "1");
        waifu2x_config.insert("models_path".to_string(), models_default_dir.to_str().unwrap());


        let mut config = Config::builder();
        if config_dir.join("config.yml").exists() {
            config = config.add_source(File::from(config_dir.join("config.yml")))
        }

        config = config.add_source(Environment::with_prefix("kurp"))
            .set_default("komga_url", "http://localhost:8080")?
            .set_default("upscale", true)?
            .set_default("return_format", "WebP")?
            .set_default("size_threshold_enabled", "true")?
            .set_default("size_threshold", "500")?
            .set_default("size_threshold_png", "1000")?
            .set_default("waifu2x", waifu2x_config)?;

        config.build()?.try_deserialize()
    }
}