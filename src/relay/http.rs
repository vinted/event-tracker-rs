use crate::{Error, EventBase, Relay};
use bytes::Bytes;
use reqwest::{blocking::Client, header, Url};

/// A [`Relay`] that will print events to HTTP listener
#[derive(Debug, Clone)]
pub struct Http {
    client: Client,
    url: Url,
}

impl Http {
    /// Creates an instance of [`Http`] [`Relay`]
    pub fn new(url: Url) -> Self {
        Self {
            client: Client::new(),
            url,
        }
    }
}

impl Relay for Http {
    fn transport(&self, event_base: EventBase, bytes: Bytes) -> Result<(), Error> {
        let mut request = self
            .client
            .post(self.url.clone())
            .body(bytes)
            .header(header::CONTENT_TYPE, "application/json")
            .header("X-Local-Time", event_base.time.to_string())
            .header("X-Platform", "web")
            .header("X-Portal", event_base.portal);

        if let Some(debug_pin) = event_base.debug_pin {
            request = request.header("X-Debug-Pin", debug_pin);
        }

        let response = match request.send() {
            Ok(response) => response,
            Err(error) => {
                tracing::error!(%error, "Couldn't send data to HTTP relay");
                return Ok(());
            }
        };

        let status = response.status();

        if status.is_client_error() || status.is_server_error() {
            let status_code = status.as_u16();
            let response_body = response.text();

            match response_body {
                Ok(error) => {
                    tracing::error!(%status_code, %error, "Couldn't complete HTTP request successfully");
                }
                Err(error) => {
                    tracing::error!(%status_code, %error, "Couldn't complete HTTP request successfully");
                }
            }
        }

        Ok(())
    }
}
