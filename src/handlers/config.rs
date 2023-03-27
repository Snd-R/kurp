use std::ops::Deref;

use axum::extract::State;
use axum::Json;
use axum::response::IntoResponse;
use hyper::StatusCode;

use crate::app_state::AppState;
use crate::config::app_config::AppConfig;

pub async fn update_config(
    State(state): State<AppState>,
    Json(new_config): Json<AppConfig>,
) -> impl IntoResponse {
    AppConfig::write_config(new_config);
    state.shutdown_tx.send(()).unwrap();

    StatusCode::OK
}

pub async fn get_config(
    State(state): State<AppState>
) -> impl IntoResponse {
    Json(state.config.deref().clone())
}
