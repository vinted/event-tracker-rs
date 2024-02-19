use crate::{Metadata, Relay};
use reqwest::{header, Client, Url};

/// A [`Relay`] that will print events to HTTP listener
#[derive(Debug, Clone)]
pub struct HttpRelay {
    client: Client,
    url: Url,
}

impl HttpRelay {
    /// Creates an instance of [HttpRelay]
    pub fn new(url: Url) -> Self {
        Self {
            client: Client::new(),
            url,
        }
    }
}

impl Relay for HttpRelay {
    fn transport(&self, metadata: Metadata, serialized_event: Vec<u8>) {
        let mut request = self
            .client
            .post(self.url.clone())
            .body(serialized_event)
            .header(header::CONTENT_TYPE, "application/json")
            .header("X-Local-Time", metadata.time.to_string())
            .header("X-Platform", "web")
            .header("X-Portal", metadata.portal);

        if let Some(debug_pin) = metadata.debug_pin {
            request = request.header("X-Debug-Pin", debug_pin);
        }

        let result = tokio::spawn(async move {
            let response = match request.send().await {
                Ok(response) => response,
                Err(error) => {
                    tracing::error!(%error, "Couldn't send data to HTTP relay");
                    return;
                }
            };

            let status = response.status();

            if status.is_client_error() || status.is_server_error() {
                let status_code = status.as_u16();
                let response_body = response.text().await;

                match response_body {
                    Ok(error) => {
                        tracing::error!(%status_code, %error, "Couldn't complete HTTP request successfully");
                    }
                    Err(error) => {
                        tracing::error!(%status_code, %error, "Couldn't complete HTTP request successfully");
                    }
                }
            }
        });

        drop(result);
    }
}
