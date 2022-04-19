use serde::Serialize;
use vinted_event_tracker::*;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let addr = "0.0.0.0:5005".parse().expect("valid addr");

    let udp_relay = Udp::bind(addr).await.expect("valid udp relay");

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
        let event = Event::new("event", "portal", 1234, SearchEvent { iteration });

        if let Err(ref error) = track(event) {
            tracing::error!(%error, "Couldn't track an event");
        }
    }
}
