use std::path::Path;

use bytes::Bytes;
use image::ImageFormat;
use log::info;
use regex::Regex;
use tokio::task;
use unicase::Ascii;
use warp::{Filter, http, Rejection};
use warp::http::{HeaderMap, HeaderValue, Response};
use warp::hyper::{body, StatusCode};
use warp::hyper::{Body, Method};
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
) -> Result<Response<Body>, Rejection> {
    let uri_str = format!("{} {}{}", method, uri.as_str(), params.as_deref().map(|query| "?".to_string() + query).unwrap_or_default());
    // TODO do not request compressed data to avoid decode and re-encode
    let response = proxy_to_and_forward_response(proxy_address, base_path, uri, params, method, headers, body).await?;
    let status = response.status();

    info!("{}: upstream response: {}",uri_str, response.status());
    if status == 304 || !status.is_success() {
        return Ok(response);
    }

    let headers = response.headers().clone();
    let mut content_type = headers.get("content-type").unwrap().to_str().unwrap();
    if content_type == "image/jpg" {
        content_type = "image/jpeg"
    }
    let image_format = ImageFormat::from_mime_type(content_type).unwrap();

    let encoding = headers.get("content-encoding");
    let response_bytes = body::to_bytes(response).await.unwrap();

    let decompressed = encoding
        .map(unwrap_encoding_header)
        .map(|algo| http_compression::decompress(response_bytes.clone(), algo));

    let to_upscale = match decompressed {
        None => response_bytes,
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
    response_to_upscaled_reply(status, response_body, &headers, format).await
        .map_err(warp::reject::custom)
}

async fn response_to_upscaled_reply(
    status: StatusCode,
    bytes: Bytes,
    headers: &HeaderMap<HeaderValue>,
    format: ImageFormat,
) -> Result<Response<Body>, errors::Error> {
    let mime_type = match format {
        ImageFormat::Png => { Some(("image/png", "png")) }
        ImageFormat::Jpeg => { Some(("image/jpeg", "jpeg")) }
        ImageFormat::WebP => { Some(("image/webp", "webp")) }
        _ => { None }
    };
    let mut builder = http::Response::builder();
    for (k, v) in headers {
        if Ascii::new("Content-Length") == k {
            builder = builder.header("Content-Length", bytes.len())
        } else if Ascii::new("Content-Type") == k && mime_type.is_some() {
            builder = builder.header("Content-Type", mime_type.unwrap().0)
        } else if Ascii::new("Content-Disposition") == k && mime_type.is_some() {
            let new_value: String = v.to_str().unwrap().split("; ")
                .map(|param| if param.starts_with("filename=") {
                    with_new_file_extension(param, mime_type.unwrap().1)
                } else if param.starts_with("filename*=") {
                    with_new_file_extension(param, mime_type.unwrap().1)
                } else { param.to_string() })
                .collect::<Vec<String>>().join("; ");
            builder = builder.header("Content-Disposition", new_value)
        } else {
            builder = builder.header(k, v);
        }
    }
    builder
        .status(status)
        .body(Body::from(bytes))
        .map_err(errors::Error::Http)
}

fn with_new_file_extension(name: &str, extension: &str) -> String {
    let regex = Regex::new(r"(filename\*=UTF-8''|filename=)(.+\b)").unwrap();
    let captures = regex.captures(name).unwrap();
    let param_name = captures.get(1).unwrap().as_str();
    let filename = captures.get(2).unwrap().as_str();
    let new_filename = Path::new(filename).with_extension(extension)
        .into_os_string().into_string().unwrap();
    format!("{}{}", param_name, new_filename)
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