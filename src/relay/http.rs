use super::Relay;
use crate::EventBase;
use bytes::Bytes;
use futures_channel::mpsc::{self, Receiver, Sender};
use futures_util::StreamExt;
use hyper::{client::HttpConnector, header, Body, Client, Method, Request, Uri};

const DEFAULT_BUFFER: usize = 512;

/// A [`Relay`] that will print events to HTTP listener
#[derive(Debug, Clone)]
pub struct Http {
    sender: Sender<HttpEvent>,
}

type HttpEvent = (EventBase, Bytes);

impl Http {
    /// Creates an instance of [`Http`] [`Relay`]
    pub fn new(url: Uri) -> Self {
        let (sender, mut receiver) = mpsc::channel::<HttpEvent>(DEFAULT_BUFFER);

        let client = Client::new();

        let task = Box::pin(async move {
            handle_http_connection(&client, url, &mut receiver).await;
        });

        let _ = tokio::spawn(task);

        Self { sender }
    }
}

async fn handle_http_connection(
    client: &Client<HttpConnector>,
    url: Uri,
    receiver: &mut Receiver<HttpEvent>,
) {
    while let Some((event_base, bytes)) = receiver.next().await {
        let req = Request::builder()
            .method(Method::POST)
            .uri(url.clone())
            .header(header::CONTENT_TYPE, "application/json")
            .header("X-Local-Time", event_base.time.to_string())
            .body(Body::from(bytes));

        match req {
            Ok(req) => {
                if let Err(ref error) = client.request(req).await {
                    tracing::error!(%error, "Couldn't send data to HTTP relay");

                    break;
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
