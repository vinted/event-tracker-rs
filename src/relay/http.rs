use super::Relay;
use crate::EventBase;
use bytes::Bytes;
use futures_channel::mpsc::{self, Receiver, Sender};
use futures_util::StreamExt;
use reqwest::{header, Client, Url};

const DEFAULT_BUFFER: usize = 512;

/// A [`Relay`] that will print events to HTTP listener
#[derive(Debug, Clone)]
pub struct Http {
    sender: Sender<HttpEvent>,
}

type HttpEvent = (EventBase, Bytes);

impl Http {
    /// Creates an instance of [`Http`] [`Relay`]
    pub fn new(url: Url) -> Self {
        let (sender, mut receiver) = mpsc::channel::<HttpEvent>(DEFAULT_BUFFER);

        let client = Client::new();

        let task = Box::pin(async move {
            handle_http_connection(client, url, &mut receiver).await;
        });

        let _ = tokio::spawn(task);

        Self { sender }
    }
}

async fn handle_http_connection(client: Client, url: Url, receiver: &mut Receiver<HttpEvent>) {
    while let Some((event_base, bytes)) = receiver.next().await {
        let mut req = client
            .post(url.clone())
            .body(bytes)
            .header(header::CONTENT_TYPE, "application/json")
            .header("X-Local-Time", event_base.time.to_string())
            .header("X-Platform", "web")
            .header("X-Portal", event_base.portal);

        if let Some(debug_pin) = event_base.debug_pin {
            req = req.header("X-Debug-Pin", debug_pin);
        }

        match req.send().await {
            Ok(response) => {
                let status = response.status();

                if !status.is_success() {
                    let status_code = status.as_u16();
                    let response_body = response.text().await;

                    match response_body {
                        Ok(body) => {
                            tracing::error!(%status_code, %body, "Couldn't complete HTTP request successfully");
                        }
                        Err(ref error) => {
                            tracing::error!(%status_code, %error, "Couldn't complete HTTP request successfully");
                        }
                    }
                }
            }
            Err(ref error) => {
                tracing::error!(%error, "Couldn't send data to HTTP relay");

                break;
            }
        }
    }
}

impl Relay for Http {
    fn transport(&self, event_base: EventBase, event: Bytes) -> crate::Result<()> {
        if let Err(ref error) = self.sender.clone().try_send((event_base, event)) {
            tracing::error!(%error, "Couldn't send data to HTTP relay");
        }

        Ok(())
    }
}
