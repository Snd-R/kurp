use bytes::Bytes;
use image::ImageFormat;
use log::info;
use tokio::task;
use unicase::Ascii;
use warp::{Filter, http, Rejection};
use warp::http::{HeaderMap, HeaderValue, Response};
use warp::hyper::Method;
use warp::path::FullPath;
use warp_reverse_proxy::{errors, extract_request_data_filter, proxy_to_and_forward_response, QueryParameters};

use crate::{app_config, http_compression, upscaler};
use crate::http_compression::{Algorithm, compress};

pub async fn routes() {
    let config = app_config::get_global_config();
    let komga_upscale = warp::path!("api"/"v1"/"books" / String /"pages"/ i32)
        .map(move |_id, _page| (config.upstream_url.clone(), "".to_string()))
        .untuple_one()
        .and(extract_request_data_filter())
        .and_then(proxy_upscale_and_forward);

    let kavita_upscale = warp::path!("api"/"reader"/"image")
        .map(|| (config.upstream_url.clone(), "".to_string()))
        .untuple_one()
        .and(extract_request_data_filter())
        .and_then(proxy_upscale_and_forward);

    let regular_proxy = warp::any()
        .map(move || (config.upstream_url.clone(), "".to_string()))
        .untuple_one()
        .and(extract_request_data_filter())
        .and_then(proxy_to_and_forward_response);

    warp::serve(
        komga_upscale
            .or(kavita_upscale)
            .or(regular_proxy)
    ).run(([0, 0, 0, 0], config.port)).await;
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
    // TODO do not request compressed data to avoid decode and re-encode
    let response = proxy_to_and_forward_response(proxy_address, base_path, uri, params, method, headers, body).await?;
    let status = response.status();

    info!("{}: upstream response: {}",uri_str, response.status());
    if status == 304 || !status.is_success() {
        return Ok(response);
    }

    let mut content_type = response.headers().get("content-type").unwrap().to_str().unwrap();
    if content_type == "image/jpg" {
        content_type = "image/jpeg"
    }
    let image_format = ImageFormat::from_mime_type(content_type).unwrap();

    let encoding = response.headers().get("content-encoding");
    let body_to_decompress = response.body().clone();
    let decompressed = encoding
        .map(unwrap_encoding_header)
        .map(|algo| http_compression::decompress(body_to_decompress, algo));

    let to_upscale = match decompressed {
        None => response.body().clone(),
        Some(decompressed) => decompressed.await.unwrap()
    };

    let (upscaled, format) = task::spawn_blocking(move || {
        upscaler::UPSCALER.upscale(to_upscale, image_format)
    }).await.unwrap();

    let body_to_compress = upscaled.clone();
    let compressed = encoding
        .map(unwrap_encoding_header)
        .map(|algo| compress(body_to_compress, algo));

    let response_body = match compressed {
        None => upscaled,
        Some(compressed) => compressed.await.unwrap()
    };

    info!("{} finished upscaling", uri_str);
    response_to_upscaled_reply(response, response_body, format).await
        .map_err(warp::reject::custom)
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

fn unwrap_encoding_header(encoding: &HeaderValue) -> Algorithm {
    let encoding = encoding.to_str().unwrap();
    match encoding {
        "gzip" => Algorithm::Gzip,
        "deflate" => Algorithm::Deflate,
        "br" => Algorithm::Brotli,
        _ => panic!("unsupported compression algorithm")
    }
}