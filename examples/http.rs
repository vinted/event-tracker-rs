use serde::Serialize;
use vinted_event_tracker::*;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let url = "https://0.0.0.0:9999".parse().expect("valid url");

    let http_relay = Http::new(url);

    if let Err(ref error) = set_relay(http_relay) {
        tracing::error!(%error, "Couldn't set HTTP relay");
    }

    track_events(5);
}

fn track_events(iterations: i32) {
    #[derive(Debug, Serialize)]
    struct SearchEvent {
        iteration: i32,
    }

    for iteration in 1..iterations {
        let event = Event::new("system.test", "fr", Some(1234), SearchEvent { iteration });

        if let Err(ref error) = track(event) {
            tracing::error!(%error, "Couldn't track an event");
        }
    }
}
