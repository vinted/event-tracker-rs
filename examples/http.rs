use serde::Serialize;
use vinted_event_tracker::*;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let relay = HttpRelay::new("https://0.0.0.0:9999".parse().unwrap());

    set_relay(relay).unwrap();

    track_events(1_000);
}

fn track_events(iterations: i32) {
    #[derive(Debug, Serialize)]
    struct SearchEvent {
        iteration: i32,
    }

    for iteration in 1..iterations {
        vinted_event_tracker::track("event", "portal", Some(1234), SearchEvent { iteration });
    }
}
