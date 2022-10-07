use serde::Serialize;
use vinted_event_tracker::*;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let udp_relay = UdpRelay::bind("0.0.0.0:5005")
        .await
        .expect("valid udp relay");

    if let Err(ref error) = set_relay(udp_relay) {
        tracing::error!(%error, "Couldn't set UDP relay");
    }

    track_events(1_000)
}

fn track_events(iterations: i32) {
    #[derive(Debug, Serialize)]
    struct SearchEvent {
        iteration: i32,
    }

    for iteration in 1..iterations {
        if let Err(ref error) = Event::track("test", "portal", None, SearchEvent { iteration }) {
            tracing::error!(%error, "Couldn't track an event");
        }
    }
}
