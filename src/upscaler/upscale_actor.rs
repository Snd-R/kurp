use std::convert::Infallible;

use bytes::Bytes;
use image::ImageFormat;
use tokio::sync::{mpsc, oneshot};

use crate::models::errors::UpscaleError;
use crate::upscaler::upscaler::Upscaler;

struct UpscaleActor {
    receiver: mpsc::Receiver<UpscaleMessage>,
    upscaler: Option<Box<dyn Upscaler>>,
}

enum UpscaleMessage {
    Upscale {
        image: (Bytes, ImageFormat),
        respond_to: oneshot::Sender<Result<(Bytes, ImageFormat), UpscaleError>>,
    },
    Update {
        upscaler: Box<dyn Upscaler>,
        respond_to: oneshot::Sender<Result<(), Infallible>>,
    },
    Deinitialize {
        respond_to: oneshot::Sender<Result<(), Infallible>>,
    },
}

impl UpscaleActor {
    fn new_uninitialized(receiver: mpsc::Receiver<UpscaleMessage>) -> Self {
        UpscaleActor { receiver, upscaler: None }
    }

    fn handle_message(&mut self, msg: UpscaleMessage) {
        match msg {
            UpscaleMessage::Upscale { image, respond_to } => {
                let (bytes, format) = image;

                let result = match &self.upscaler {
                    Some(upscaler) => { Ok(upscaler.upscale(bytes, format)) }
                    None => { Err(UpscaleError { message: "Upscaler is not initialized".to_string() }) }
                };

                respond_to.send(result).unwrap();
            }

            UpscaleMessage::Update { upscaler, respond_to } => {
                self.upscaler = Some(upscaler);
                respond_to.send(Ok(())).unwrap()
            }
            UpscaleMessage::Deinitialize { respond_to } => {
                self.upscaler = None;
                respond_to.send(Ok(())).unwrap()
            }
        }
    }
}

async fn run_upscale_actor(mut actor: UpscaleActor) {
    while let Some(msg) = actor.receiver.recv().await {
        actor.handle_message(msg)
    }
}

#[derive(Clone)]
pub struct UpscaleActorHandle {
    sender: mpsc::Sender<UpscaleMessage>,
}

impl UpscaleActorHandle {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel(100);
        let actor = UpscaleActor::new_uninitialized(receiver);
        tokio::spawn(run_upscale_actor(actor));

        Self { sender }
    }

    pub async fn upscale(&self, image: Bytes, format: ImageFormat) -> Result<(Bytes, ImageFormat), UpscaleError> {
        let (send, recv) = oneshot::channel();
        let msg = UpscaleMessage::Upscale {
            image: (image, format),
            respond_to: send,
        };

        let _ = self.sender.send(msg).await;
        recv.await.expect("Actor task has been killed")
    }


    pub async fn init(&self, upscaler: Box<dyn Upscaler>) {
        let (send, recv) = oneshot::channel();
        let msg = UpscaleMessage::Update { upscaler, respond_to: send };
        let _ = self.sender.send(msg).await;

        let _ = recv.await.expect("Actor task has been killed");
    }

    pub async fn deinitialize(&self) {
        let (send, recv) = oneshot::channel();
        let _ = self.sender.send(UpscaleMessage::Deinitialize { respond_to: send }).await;

        let _ = recv.await.expect("Actor task has been killed");
    }
}