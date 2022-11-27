use std::io::Cursor;
use std::sync::Mutex;

use bytes::Bytes;
use image::ImageFormat;
use log::info;
use once_cell::sync::Lazy;
use waifu2x_ncnn_vulkan_rs::Waifu2x;

use crate::app_config;
use crate::app_config::Format;

pub static WAIFU2X: Lazy<Mutex<Waifu2x>> = Lazy::new(|| {
    let config = app_config::get_global_config();
    Mutex::new(
        Waifu2x::new(config.waifu2x.gpuid,
                     config.waifu2x.noise,
                     config.waifu2x.scale,
                     config.waifu2x.model,
                     config.waifu2x.tile_size,
                     config.waifu2x.tta_mode,
                     config.waifu2x.num_threads,
                     config.waifu2x.models_path.clone())
    )
});

pub fn upscale(input: Bytes, image_format: ImageFormat) -> (Bytes, ImageFormat) {
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

    let waifu = WAIFU2X.lock().unwrap();
    let image = image::io::Reader::new(Cursor::new(input)).with_guessed_format().unwrap().decode().unwrap();
    let upscaled = waifu.proc_image(image);
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