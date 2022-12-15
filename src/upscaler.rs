use std::io::Cursor;
use std::sync::Mutex;

use bytes::Bytes;
use image::{DynamicImage, ImageFormat};
use log::info;
use once_cell::sync::Lazy;
use realcugan_ncnn_vulkan_rs::RealCugan;
use waifu2x_ncnn_vulkan_rs::Waifu2x;

use crate::app_config;
use crate::app_config::{EnabledUpscaler, Format, RealCuganConfig, Waifu2xConfig};

pub static UPSCALER: Lazy<Box<dyn Upscaler + Send + Sync>> = Lazy::new(|| {
    let config = app_config::get_global_config();
    match config.upscaler {
        EnabledUpscaler::Waifu2x => Box::new(Waifu2xUpscaler::new(&config.waifu2x)),
        EnabledUpscaler::Realcugan => Box::new(RealCuganUpscaler::new(&config.realcugan))
    }
});

struct Waifu2xUpscaler {
    waifu2x: Mutex<Waifu2x>,
}

impl Waifu2xUpscaler {
    pub fn new(config: &Waifu2xConfig) -> Self {
        let waifu2x = Mutex::new(Waifu2x::new(
            config.gpuid,
            config.noise,
            config.scale,
            config.model,
            config.tile_size,
            config.tta_mode,
            config.num_threads,
            config.models_path.clone(),
        ));

        Self { waifu2x }
    }
}

struct RealCuganUpscaler {
    realcugan: Mutex<RealCugan>,
}

impl RealCuganUpscaler {
    pub fn new(config: &RealCuganConfig) -> Self {
        let realcugan = Mutex::new(RealCugan::new(
            config.gpuid,
            config.noise,
            config.scale,
            config.model,
            config.tile_size,
            config.sync_gap,
            config.tta_mode,
            config.num_threads,
            config.models_path.clone(),
        ));
        Self {
            realcugan
        }
    }
}

pub trait Upscaler {
    fn upscale(&self, input: Bytes, image_format: ImageFormat) -> (Bytes, ImageFormat) {
        let config = app_config::get_global_config();
        if config.size_threshold_enabled {
            let input_kb = (input.len() / 1024) as u32;
            let threshold = if image_format == ImageFormat::Png {
                config.size_threshold_png
            } else {
                config.size_threshold
            };
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
}

impl Upscaler for Waifu2xUpscaler {
    fn upscale_image(&self, image: DynamicImage) -> DynamicImage {
        self.waifu2x.lock().unwrap().proc_image(image)
    }
}

impl Upscaler for RealCuganUpscaler {
    fn upscale_image(&self, image: DynamicImage) -> DynamicImage {
        self.realcugan.lock().unwrap().proc_image(image)
    }
}