use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use tokio_tungstenite::tungstenite::Error;
use tokio_tungstenite::tungstenite::handshake::client::Response;

pub struct WebsocketProxyClient {
    base_url: String,
}

impl WebsocketProxyClient {
    pub fn new(base_url: String) -> Self {
        Self { base_url }
    }

    pub async fn spawn_client(&self, path_and_query: &str) -> Result<(WebSocketStream<MaybeTlsStream<TcpStream>>, Response), Error> {
        let url = format!("{}{}", self.base_url, path_and_query);
        Ok(connect_async(url).await?)
    }
}