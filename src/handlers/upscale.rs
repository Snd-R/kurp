use std::future::Future;
use std::path::Path;
use std::sync::Arc;

use axum::extract::State;
use axum::http::{HeaderMap, HeaderValue, Request, Response, StatusCode};
use bytes::Bytes;
use hyper::Body;
use hyper::body::to_bytes;
use image::ImageFormat;
use log::info;
use moka::future::Cache;
use once_cell::sync::Lazy;
use regex::Regex;
use unicase::Ascii;

use crate::app_state::AppState;
use crate::http_compression;
use crate::http_compression::{Algorithm, compress};
use crate::models::errors::HttpError;
use crate::upscaler::upscale_actor::UpscaleActorHandle;

pub async fn upscale_komga(
    State(state): State<AppState>,
    req: Request<Body>,
) -> Result<Response<Body>, StatusCode> {
    let uri = req.uri().clone();
    let tag_checker = state.upscale_tag_checker.clone();
    let cookie = req.headers().get("Cookie")
        .ok_or(StatusCode::BAD_REQUEST)?
        .clone();

    let upscale_condition = || async {
        let book_id = uri.path().split("/").collect::<Vec<&str>>().windows(2)
            .find(|path| path[0] == "books")
            .map(|path| path[1])
            .unwrap();
        tag_checker.komga_contains_upscale_tag(book_id, cookie.to_str().unwrap()).await
    };

    upscale(state, req, upscale_condition).await
}

pub async fn upscale_kavita(
    State(state): State<AppState>,
    req: Request<Body>,
) -> Result<Response<Body>, StatusCode> {
    upscale(state, req, || async { Ok(true) }).await
}

pub async fn upscale<F, Fut>(
    state: AppState,
    request: Request<Body>,
    upscale_condition: F,
) -> Result<Response<Body>, StatusCode>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output=Result<bool, HttpError>>
{
    let request = to_proxy_request(state.upscale_call_history_cache.clone(), request);
    let request_path = request.uri().path_and_query()
        .map(|path| path.to_string())
        .unwrap_or("/".to_string());

    let uri_str = format!("{} {}", request.method().as_str(), request.uri().path());

    let response = state.proxy_client.proxy_request(request).await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;
    info!("{}: upstream response: {}",uri_str, response.status());

    if response.status() == 304 || !response.status().is_success() {
        return Ok(response);
    }
    let should_upscale = upscale_condition().await.map_err(|_| StatusCode::BAD_GATEWAY)?;
    if !should_upscale { return Ok(response); }

    let upscaled = upscale_response(response, state.upscaler).await;
    info!("{} finished upscaling", uri_str);
    state.upscale_call_history_cache.insert(request_path, ()).await;
    Ok(upscaled)
}

async fn upscale_response(
    response: Response<Body>,
    upscaler: UpscaleActorHandle,
) -> Response<Body> {
    let status = response.status();
    let headers = response.headers().clone();
    let mut content_type = headers.get("content-type").unwrap().to_str().unwrap();
    if content_type == "image/jpg" {
        content_type = "image/jpeg"
    }
    let image_format = ImageFormat::from_mime_type(content_type).unwrap();

    let encoding = headers.get("content-encoding");
    let response_bytes = to_bytes(response).await.unwrap();

    let decompressed = encoding
        .map(unwrap_encoding_header)
        .map(|algo| http_compression::decompress(response_bytes.clone(), algo));

    let to_upscale = match decompressed {
        None => response_bytes,
        Some(decompressed) => decompressed.await.unwrap()
    };

    let (upscaled, format) = upscaler.upscale(to_upscale, image_format).await.unwrap();

    let body_to_compress = upscaled.clone();
    let compressed = encoding
        .map(unwrap_encoding_header)
        .map(|algo| compress(body_to_compress, algo));

    let response_body = match compressed {
        None => upscaled,
        Some(compressed) => compressed.await.unwrap()
    };

    to_response(status, response_body, &headers, format)
}

fn to_response(
    status: StatusCode,
    bytes: Bytes,
    headers: &HeaderMap<HeaderValue>,
    format: ImageFormat,
) -> Response<Body> {
    let mime_type = match format {
        ImageFormat::Png => { Some(("image/png", "png")) }
        ImageFormat::Jpeg => { Some(("image/jpeg", "jpeg")) }
        ImageFormat::WebP => { Some(("image/webp", "webp")) }
        _ => { None }
    };
    let mut builder = Response::builder();
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
        .unwrap()
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

fn to_proxy_request(
    call_cache: Arc<Cache<String, ()>>,
    req: Request<Body>,
) -> Request<Body> {
    let request_path = req.uri().path_and_query()
        .map(|path| path.to_string())
        .unwrap_or("/".to_string());

    let (parts, body) = req.into_parts();

    let headers = remove_uncached_conditional_headers(
        parts.headers,
        call_cache.clone(),
        request_path,
    );

    let mut builder = Request::builder()
        .method(parts.method)
        .uri(parts.uri)
        .version(parts.version)
        .extension(parts.extensions);
    builder.headers_mut().unwrap().extend(headers);

    builder.body(body).unwrap()
}

fn is_conditional_header(header_name: &str) -> bool {
    static CONDITIONAL_HEADERS: Lazy<Vec<Ascii<&'static str>>> = Lazy::new(|| {
        vec![Ascii::new("If-Modified-Since"), Ascii::new("If-None-Match")]
    });
    CONDITIONAL_HEADERS.iter().any(|h| h == &header_name)
}

fn remove_uncached_conditional_headers(
    headers: HeaderMap<HeaderValue>,
    call_cache: Arc<Cache<String, ()>>,
    request_path: String,
) -> HeaderMap<HeaderValue> {
    if call_cache.get(&request_path).is_some() {
        return headers;
    }

    headers.iter()
        .filter_map(|(k, v)|
            if is_conditional_header(k.as_str()) {
                None
            } else {
                Some((k.clone(), v.clone()))
            }).collect()
}
