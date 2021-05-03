use event_tracker::*;
use serde::Serialize;

#[tokio::main]
async fn main() {
    let addr = "0.0.0.0:5005".parse().expect("valid addr");

    let udp_relay = Udp::new(addr);

    let _ = set_relay(udp_relay);

    track_events(1_000)
}

fn track_events(iterations: i32) {
    #[derive(Debug, Serialize)]
    struct SearchEvent {
        iteration: i32,
    }

    for iteration in 1..iterations {
        let event = Event::new("event", "FR", SearchEvent { iteration });

        let _ = track(event);
    }
}
