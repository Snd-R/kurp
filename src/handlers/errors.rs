use log::error;
use warp::{Rejection, Reply};
use warp::filters::body::BodyDeserializeError;
use warp::http::StatusCode;

use crate::models::errors::ApiError;

pub async fn handle_rejection(err: Rejection) -> Result<impl Reply, Rejection> {
    let code;
    let message;

    if err.is_not_found() {
        return Err(err);
    } else if let Some(e) = err.find::<warp::reject::MethodNotAllowed>() {
        code = StatusCode::METHOD_NOT_ALLOWED;
        message = e.to_string();
    } else if let Some(e) = err.find::<BodyDeserializeError>() {
        error!("Could not read JSON data: {:?}", err);
        code = StatusCode::BAD_REQUEST;
        message = e.to_string();
    } else if let Some(e) = err.find::<ApiError>() {
        error!("API Error: {}", e);
        code = e.code;
        message = e.message.to_owned();
    } else {
        error!("Unhandled rejection: {:?}", err);
        code = StatusCode::INTERNAL_SERVER_ERROR;
        message = "INTERNAL_SERVER_ERROR".to_owned();
    }

    Ok(ApiError::new(message, code).into_response())
}
