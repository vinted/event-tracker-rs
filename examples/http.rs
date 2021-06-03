use serde::Serialize;
use std::time::Duration;
use tokio::time::sleep;
use tracing::Level;
use vinted_event_tracker::*;

#[tokio::main(worker_threads = 1)]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .finish();

    let url = "http://0.0.0.0:8888".parse().expect("valid addr");

    let http_relay = Http::new(url);

    if let Err(ref error) = set_relay(http_relay) {
        tracing::error!(%error, "Couldn't set HTTP relay");
    }

    track_events(1_000);

    // Needed on standalone example to wait until all events have been sent
    sleep(Duration::from_secs(10)).await;
}

fn track_events(iterations: i32) {
    #[derive(Debug, Serialize)]
    struct SearchEvent {
        iteration: i32,
    }

    for iteration in 1..iterations {
        let event = Event::new("event", "portal", SearchEvent { iteration });

        if let Err(ref error) = track(event) {
            tracing::error!(%error, "Couldn't track an event");
        }
    }
}
