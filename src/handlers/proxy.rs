use axum::extract::{State, WebSocketUpgrade};
use axum::extract::ws::{CloseFrame, Message, WebSocket};
use axum::http::{Request, StatusCode};
use axum::response::{IntoResponse, Response};
use futures::{sink::SinkExt, stream::StreamExt};
use hyper::Body;
use log::error;
use tokio::net::TcpStream;
use tokio_tungstenite::{MaybeTlsStream, tungstenite, WebSocketStream};

use crate::app_state::AppState;

pub async fn proxy_handler(
    State(state): State<AppState>,
    req: Request<Body>,
) -> impl IntoResponse {
    match state.proxy_client.proxy_request(req).await {
        Ok(resp) => resp.into_response(),
        Err(_) => StatusCode::BAD_GATEWAY.into_response()
    }
}

pub async fn kavita_ws_proxy_handler(
    State(state): State<AppState>,
    ws: WebSocketUpgrade,
    request: Request<Body>,
) -> Result<Response, StatusCode> {
    let (upstream_socket, _) = match state.websocket_proxy_client.spawn_client(request.uri().path_and_query().unwrap().as_str()).await {
        Ok(ok) => { ok }
        Err(err) => {
            error!("{}", err.to_string());
            return Err(StatusCode::BAD_GATEWAY);
        }
    };

    Ok(ws.on_upgrade(move |socket| handle_socket(socket, upstream_socket)))
}

async fn handle_socket(
    incoming_socket: WebSocket,
    upstream_socket: WebSocketStream<MaybeTlsStream<TcpStream>>,
) {
    let (mut incoming_sender, mut incoming_receiver) = incoming_socket.split();
    let (mut upstream_sender, mut upstream_receiver) = upstream_socket.split();

    let mut incoming_send_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = incoming_receiver.next().await {
            upstream_sender.send(to_tungstenite(msg)).await.unwrap();
        }
    });

    let mut upstream_send_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = upstream_receiver.next().await {
            let axum_message = match from_tungstenite(msg) {
                Some(msg) => msg,
                None => continue
            };

            incoming_sender.send(axum_message).await.unwrap();
        }
    });

    tokio::select! {
        _ = (&mut incoming_send_task) => {
            upstream_send_task.abort();
        },
        _ = (&mut upstream_send_task) => {
            incoming_send_task.abort();
        }
    }
}

fn from_tungstenite(message: tungstenite::Message) -> Option<Message> {
    match message {
        tungstenite::Message::Text(text) => Some(Message::Text(text)),
        tungstenite::Message::Binary(binary) => Some(Message::Binary(binary)),
        tungstenite::Message::Ping(ping) => Some(Message::Ping(ping)),
        tungstenite::Message::Pong(pong) => Some(Message::Pong(pong)),
        tungstenite::Message::Close(Some(close)) => Some(Message::Close(Some(CloseFrame {
            code: close.code.into(),
            reason: close.reason,
        }))),
        tungstenite::Message::Close(None) => Some(Message::Close(None)),
        // we can ignore `Frame` frames as recommended by the tungstenite maintainers
        // https://github.com/snapview/tungstenite-rs/issues/268
        tungstenite::Message::Frame(_) => None,
    }
}

fn to_tungstenite(message: Message) -> tungstenite::Message {
    match message {
        Message::Text(text) => tungstenite::Message::Text(text),
        Message::Binary(binary) => tungstenite::Message::Binary(binary),
        Message::Ping(ping) => tungstenite::Message::Ping(ping),
        Message::Pong(pong) => tungstenite::Message::Pong(pong),
        Message::Close(Some(close)) => tungstenite::Message::Close(Some(tungstenite::protocol::CloseFrame {
            code: close.code.into(),
            reason: close.reason,
        })),
        Message::Close(None) => tungstenite::Message::Close(None),
    }
}
