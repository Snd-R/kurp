use std::fmt;
use std::fmt::{Debug, Display, Formatter};

#[derive(Debug, Clone)]
pub struct UpscaleError {
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct ProxyError {
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct HttpError {
    pub message: String,
}

impl Display for UpscaleError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result { write!(f, "{}", self.message) }
}

impl Display for ProxyError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result { write!(f, "{}", self.message) }
}
