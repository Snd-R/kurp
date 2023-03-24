use std::env;
use std::fs;
use std::path::PathBuf;

use config::{Config, ConfigError, Environment, File};
use realcugan_ncnn_vulkan_rs::RealCuganModelType;
use serde_derive::{Deserialize, Serialize};
use waifu2x_ncnn_vulkan_rs::ModelType;

#[derive(Serialize, Deserialize, Debug)]
#[allow(unused)]
pub struct AppConfig {
    pub port: u16,
    pub upstream_url: String,
    pub upscale: bool,
    pub return_format: Format,
    pub size_threshold_enabled: bool,
    pub size_threshold: u32,
    pub size_threshold_png: u32,
    pub upscaler: EnabledUpscaler,
    pub waifu2x: Waifu2xConfig,
    pub realcugan: RealCuganConfig,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub enum Format {
    Png,
    Jpeg,
    WebP,
    Original,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum EnabledUpscaler {
    Waifu2x,
    Realcugan,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(remote = "ModelType")]
enum ModelTypeDef {
    Cunet,
    Upconv7AnimeStyleArtRgb,
    Upconv7Photo,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(remote = "RealCuganModelType")]
enum RealCuganModelTypeDef {
    Nose,
    Pro,
    Se,
}

#[derive(Serialize, Deserialize, Debug)]
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

#[derive(Serialize, Deserialize, Debug)]
pub struct RealCuganConfig {
    pub gpuid: i32,
    pub scale: u32,
    pub noise: i32,

    #[serde(with = "RealCuganModelTypeDef")]
    pub model: RealCuganModelType,
    pub tile_size: u32,
    pub sync_gap: u32,
    pub tta_mode: bool,
    pub num_threads: i32,
    pub models_path: String,
}

impl AppConfig {
    pub fn new() -> Result<Self, ConfigError> {
        let config_dir = AppConfig::get_config_directory();

        let models_default_dir = config_dir.join("models");
        let mut waifu2x_config = config::Map::new();
        waifu2x_config.insert("gpuid".to_string(), "0");
        waifu2x_config.insert("scale".to_string(), "2");
        waifu2x_config.insert("noise".to_string(), "-1");
        waifu2x_config.insert("model".to_string(), "Cunet");
        waifu2x_config.insert("tile_size".to_string(), "0");
        waifu2x_config.insert("tta_mode".to_string(), "false");
        waifu2x_config.insert("num_threads".to_string(), "2");
        waifu2x_config.insert("models_path".to_string(), models_default_dir.to_str().unwrap());

        let mut realcugan_config = config::Map::new();
        realcugan_config.insert("gpuid".to_string(), "0");
        realcugan_config.insert("scale".to_string(), "2");
        realcugan_config.insert("noise".to_string(), "-1");
        realcugan_config.insert("model".to_string(), "Se");
        realcugan_config.insert("tile_size".to_string(), "0");
        realcugan_config.insert("sync_gap".to_string(), "3");
        realcugan_config.insert("tta_mode".to_string(), "false");
        realcugan_config.insert("num_threads".to_string(), "2");
        realcugan_config.insert("models_path".to_string(), models_default_dir.to_str().unwrap());


        let mut config = Config::builder();
        if config_dir.join("config.yml").exists() {
            config = config.add_source(File::from(config_dir.join("config.yml")))
        }

        config = config.add_source(Environment::with_prefix("kurp"))
            .set_default("port", "3030")?
            .set_default("upstream_url", "http://localhost:8080")?
            .set_default("upscale", true)?
            .set_default("return_format", "WebP")?
            .set_default("size_threshold_enabled", "true")?
            .set_default("size_threshold", "500")?
            .set_default("size_threshold_png", "1000")?
            .set_default("waifu2x", waifu2x_config)?
            .set_default("realcugan", realcugan_config)?
            .set_default("upscaler", "Waifu2x")?;

        config.build()?.try_deserialize()
    }

    pub fn write_config(config: AppConfig) {
        let yaml = serde_yaml::to_string(&config).unwrap();
        let config_path = AppConfig::get_config_directory().join("config.yml");
        fs::write(config_path, yaml).expect("Unable to write file");
    }

    fn get_config_directory() -> PathBuf {
        let current_dir = env::current_dir().expect("can't read current dir");
        let dir_env = env::var("KURP_CONF_DIR");
        let config_dir: PathBuf = dir_env.map(|path| { PathBuf::from(path) })
            .unwrap_or_else(|_| { current_dir.clone() });

        fs::create_dir_all(&config_dir).expect("can't create config directory");

        config_dir
    }
}
