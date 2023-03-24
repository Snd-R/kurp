use std::fmt;
use std::fmt::{Debug, Display, Formatter};

use serde::ser::{Serializer, SerializeStruct};
use serde::Serialize;
use warp::http::StatusCode;
use warp::reply::Response;

#[derive(Debug, Clone)]
pub struct ApiError {
    pub message: String,
    pub code: StatusCode,
}

#[derive(Debug, Clone)]
pub struct UpscaleError {
    pub message: String,
}

impl Display for UpscaleError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Display for ApiError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "({} - {})", self.code.as_str(), self.message)
    }
}

impl Serialize for ApiError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        let mut state = serializer.serialize_struct("ApiError", 2)?;
        state.serialize_field("message", &self.message)?;
        state.serialize_field("code", &self.code.as_u16())?;
        state.end()
    }
}

impl ApiError {
    pub fn new<S: AsRef<str>>(message: S, code: StatusCode) -> Self {
        Self {
            message: message.as_ref().to_string(),
            code,
        }
    }
}

impl warp::reject::Reject for ApiError {}

impl warp::Reply for ApiError {
    fn into_response(self) -> Response {
        let json = warp::reply::json(&self);
        warp::reply::with_status(json, self.code).into_response()
    }
}