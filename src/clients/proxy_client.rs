use axum::http::{HeaderMap, HeaderValue, Request};
use axum::response::Response;
use hyper::Body;
use once_cell::sync::Lazy;
use unicase::Ascii;

use crate::models::errors::ProxyError;

#[derive(Clone)]
pub struct ProxyClient {
    client: reqwest::Client,
    base_url: String,
}

impl ProxyClient {
    pub fn new(client: reqwest::Client, base_url: String) -> Self {
        Self { client, base_url }
    }

    pub async fn proxy_request(
        &self,
        req: Request<Body>,
    ) -> Result<Response<Body>, ProxyError> {
        let request = self.to_proxy_request(req)?;
        let response = self.call_proxy_request(request).await?;

        self.response_to_reply(response).await
    }

    async fn response_to_reply(
        &self,
        response: reqwest::Response,
    ) -> Result<Response<Body>, ProxyError> {
        let mut builder = Response::builder();
        for (k, v) in self.remove_hop_headers(response.headers()).iter() {
            builder = builder.header(k, v);
        }
        let status = response.status();
        let body = Body::wrap_stream(response.bytes_stream());
        builder
            .status(status)
            .body(body)
            .map_err(|err| ProxyError { message: err.to_string() })
    }

    fn is_hop_header(&self, header_name: &str) -> bool {
        static HOP_HEADERS: Lazy<Vec<Ascii<&'static str>>> = Lazy::new(|| {
            vec![
                Ascii::new("Connection"),
                Ascii::new("Keep-Alive"),
                Ascii::new("Proxy-Authenticate"),
                Ascii::new("Proxy-Authorization"),
                Ascii::new("Te"),
                Ascii::new("Trailers"),
                Ascii::new("Transfer-Encoding"),
                Ascii::new("Upgrade"),
            ]
        });

        HOP_HEADERS.iter().any(|h| h == &header_name)
    }

    fn remove_hop_headers(&self, headers: &HeaderMap<HeaderValue>) -> HeaderMap<HeaderValue> {
        headers
            .iter()
            .filter_map(|(k, v)| {
                if self.is_hop_header(k.as_str()) {
                    None
                } else {
                    Some((k.clone(), v.clone()))
                }
            })
            .collect()
    }

    fn to_proxy_request(
        &self,
        request: Request<Body>,
    ) -> Result<reqwest::Request, ProxyError> {
        let url = if let Some(path) = request.uri().path_and_query() {
            format!("{}{}", self.base_url, path.as_str())
        } else {
            format!("{}{}", self.base_url, request.uri().path())
        };

        let headers = self.remove_hop_headers(request.headers());

        self.client
            .request(request.method().clone(), url)
            .headers(headers)
            .body(request.into_body())
            .build()
            .map_err(|err| ProxyError { message: err.to_string() })
    }

    async fn call_proxy_request(&self, request: reqwest::Request) -> Result<reqwest::Response, ProxyError> {
        self.client
            .execute(request)
            .await
            .map_err(|err| ProxyError { message: err.to_string() })
    }
}