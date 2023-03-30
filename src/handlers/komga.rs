use axum::extract::{Path, State};
use axum::http::{Request, Response, StatusCode};
use hyper::{Body, body};

use crate::app_state::AppState;
use crate::models::komga::{KomgaBookMetadataUpdate, KomgaSeriesMetadataUpdate};

pub async fn check_tags_on_series_metadata_update(
    State(state): State<AppState>,
    request: Request<Body>,
) -> Result<Response<Body>, StatusCode> {
    let (parts, body) = request.into_parts();
    let bytes = body::to_bytes(body).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let json: KomgaSeriesMetadataUpdate = serde_json::from_slice(&bytes)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let request = Request::from_parts(parts, Body::from(bytes));
    let response = state.proxy_client.proxy_request(request).await.map_err(|_| StatusCode::BAD_GATEWAY)?;

    if let Some(_) = json.tags {
        state.upscale_tag_checker.invalidate_cache();
        state.upscale_call_history_cache.invalidate_all();
    }

    Ok(response)
}

pub async fn check_tags_on_book_metadata_update(
    State(state): State<AppState>,
    request: Request<Body>,
) -> Result<Response<Body>, StatusCode> {
    let (parts, body) = request.into_parts();
    let bytes = body::to_bytes(body).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let json: KomgaBookMetadataUpdate = serde_json::from_slice(&bytes)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let request = Request::from_parts(parts, Body::from(bytes));
    let response = state.proxy_client.proxy_request(request).await.map_err(|_| StatusCode::BAD_GATEWAY)?;

    if let Some(_) = json.tags {
        state.upscale_tag_checker.invalidate_cache();
        state.upscale_call_history_cache.invalidate_all();
    }

    Ok(response)
}
