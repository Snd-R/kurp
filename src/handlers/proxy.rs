use axum::extract::State;
use axum::http::{Request, StatusCode};
use axum::response::IntoResponse;
use hyper::Body;

use crate::app_state::AppState;

pub async fn proxy_route(
    State(state): State<AppState>,
    req: Request<Body>,
) -> impl IntoResponse {
    match state.proxy_client.proxy_request(req).await {
        Ok(resp) => resp.into_response(),
        Err(_) => StatusCode::BAD_GATEWAY.into_response()
    }
}