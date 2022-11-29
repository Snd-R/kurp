use bytes::Bytes;
use image::ImageFormat;
use log::info;
use tokio::task;
use unicase::Ascii;
use warp::{Filter, http, Rejection};
use warp::http::{HeaderMap, Response};
use warp::hyper::Method;
use warp::path::FullPath;
use warp_reverse_proxy::{errors, extract_request_data_filter, proxy_to_and_forward_response, QueryParameters};

use crate::{app_config, upscaler};

pub async fn routes() {
    let config = app_config::get_global_config();
    let image_upscale = warp::path!("api"/"v1"/"books" / String /"pages"/ u8)
        .map(move |_id, _page| (config.komga_url.clone(), "".to_string()))
        .untuple_one()
        .and(extract_request_data_filter())
        .and_then(proxy_upscale_and_forward);

    let regular_proxy = warp::any()
        .map(move || (config.komga_url.clone(), "".to_string()))
        .untuple_one()
        .and(extract_request_data_filter())
        .and_then(proxy_to_and_forward_response);

    warp::serve(image_upscale.or(regular_proxy)).run(([0, 0, 0, 0], config.port)).await;
}

async fn proxy_upscale_and_forward(
    proxy_address: String,
    base_path: String,
    uri: FullPath,
    params: QueryParameters,
    method: Method,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Response<Bytes>, Rejection> {
    let uri_str = format!("{} {}", method, uri.as_str());
    let response = proxy_to_and_forward_response(proxy_address, base_path, uri, params, method, headers, body).await?;
    let status = response.status();

    info!("{}: upstream response: {}",uri_str, response.status());
    if status == 304 || !status.is_success() {
        return Ok(response);
    }

    let content_type = response.headers().get("content-type").unwrap().to_str().unwrap();
    let image_format = ImageFormat::from_mime_type(content_type).unwrap();

    let to_upscale = response.body().clone();
    let (upscaled, format) = task::spawn_blocking(move || {
        upscaler::upscale(to_upscale, image_format)
    }).await.unwrap();
    info!("{} done upscaling", uri_str);

    response_to_upscaled_reply(response, upscaled, format).await.map_err(warp::reject::custom)
}

async fn response_to_upscaled_reply(
    response: Response<Bytes>,
    bytes: Bytes,
    format: ImageFormat,
) -> Result<Response<Bytes>, errors::Error> {
    let mime_type = match format {
        ImageFormat::Png => { Some("image/png") }
        ImageFormat::Jpeg => { Some("image/jpeg") }
        ImageFormat::WebP => { Some("image/webp") }
        _ => { None }
    };
    let mut builder = http::Response::builder();
    for (k, v) in response.headers() {
        if Ascii::new("Content-Length") == k {
            builder = builder.header("Content-Length", bytes.len())
        } else if Ascii::new("Content-Type") == k && mime_type.is_some() {
            builder = builder.header("Content-Type", mime_type.unwrap())
        } else {
            builder = builder.header(k, v);
        }
    }
    builder
        .status(response.status())
        .body(bytes)
        .map_err(errors::Error::HTTP)
}
