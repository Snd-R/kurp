use std::io::Cursor;
use std::sync::Arc;

use bytes::Bytes;
use image::{DynamicImage, ImageFormat};
use log::info;
use realcugan_ncnn_vulkan_rs::RealCugan;
use waifu2x_ncnn_vulkan_rs::Waifu2x;

use crate::config::app_config::{AppConfig, Format};

#[derive(Copy, Clone)]
pub struct UpscalerConfig {
    threshold_enabled: bool,
    threshold: u32,
    threshold_png: u32,
    return_format: Format,
}

pub trait Upscaler: Send {
    fn upscale(&self, input: Bytes, image_format: ImageFormat) -> (Bytes, ImageFormat) {
        let config = self.get_config();
        if config.threshold_enabled {
            let input_kb = (input.len() / 1024) as u32;
            let threshold = if image_format == ImageFormat::Png { config.threshold_png } else { config.threshold };
            if input_kb > threshold {
                info!("image size {} is bigger than threshold {}. skipping upscale", input_kb, threshold);
                return (input, image_format);
            }
        }

        let mut reader = image::io::Reader::new(Cursor::new(input.clone()));
        reader.set_format(image_format);
        let image = reader.decode()
            .or(image::io::Reader::new(Cursor::new(input))
                .with_guessed_format().unwrap().decode()
            ).unwrap();

        let upscaled = self.upscale_image(image);
        let mut buf = Cursor::new(Vec::new());

        let format_to = match config.return_format {
            Format::Png => { ImageFormat::Png }
            Format::Jpeg => { ImageFormat::Jpeg }
            Format::WebP => { ImageFormat::WebP }
            Format::Original => { image_format }
        };

        upscaled.write_to(&mut buf, format_to).expect("can't write image");
        (Bytes::from(buf.into_inner()), format_to)
    }

    fn upscale_image(&self, image: DynamicImage) -> DynamicImage;

    fn get_config(&self) -> UpscalerConfig;
}

pub struct Waifu2xUpscaler {
    config: UpscalerConfig,
    waifu2x: Waifu2x,
}

pub struct RealCuganUpscaler {
    config: UpscalerConfig,
    realcugan: RealCugan,
}

impl Waifu2xUpscaler {
    pub fn new(config: Arc<AppConfig>) -> Self {
        let waifu2x = Waifu2x::new(
            config.waifu2x.gpuid,
            config.waifu2x.noise,
            config.waifu2x.scale,
            config.waifu2x.model,
            config.waifu2x.tile_size,
            config.waifu2x.tta_mode,
            config.waifu2x.num_threads,
            config.waifu2x.models_path.clone(),
        );

        let upscaler_config = UpscalerConfig {
            threshold_enabled: config.size_threshold_enabled,
            threshold: config.size_threshold,
            threshold_png: config.size_threshold_png,
            return_format: config.return_format,
        };

        Self { config: upscaler_config, waifu2x }
    }
}

impl RealCuganUpscaler {
    pub fn new(config: Arc<AppConfig>) -> Self {
        let realcugan = RealCugan::new(
            config.realcugan.gpuid,
            config.realcugan.noise,
            config.realcugan.scale,
            config.realcugan.model,
            config.realcugan.tile_size,
            config.realcugan.sync_gap,
            config.realcugan.tta_mode,
            config.realcugan.num_threads,
            config.realcugan.models_path.clone(),
        );

        let upscaler_config = UpscalerConfig {
            threshold_enabled: config.size_threshold_enabled,
            threshold: config.size_threshold,
            threshold_png: config.size_threshold_png,
            return_format: config.return_format,
        };

        Self {
            config: upscaler_config,
            realcugan,
        }
    }
}


impl Upscaler for Waifu2xUpscaler {
    fn upscale_image(&self, image: DynamicImage) -> DynamicImage {
        self.waifu2x.proc_image(image)
    }

    fn get_config(&self) -> UpscalerConfig {
        self.config
    }
}

impl Upscaler for RealCuganUpscaler {
    fn upscale_image(&self, image: DynamicImage) -> DynamicImage {
        self.realcugan.proc_image(image)
    }

    fn get_config(&self) -> UpscalerConfig {
        self.config
    }
}