# Event Tracker

An abstraction for event tracking written in Rust.

## Installation

Add event tracker to your dependencies in `Cargo.toml`:

```toml
vinted_event_tracker = { git = "https://github.com/vinted/event-tracker-rs" }
```

## Usage

Configure the crate in your application executable.

```rust
use serde::Serialize;
use vinted_event_tracker::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let udp_relay = UdpRelay::bind("0.0.0.0:5005").await?;

    set_relay(udp_relay)?;

    Ok(())
}
```

In your library code, create an event structure and use it for tracking

```rust
use serde::Serialize;
use vinted_event_tracker::*;

fn track_search_event() {
    #[derive(Debug, Serialize)]
    struct SearchEvent<'a> {
        total: i32,
        timed_out: bool,
        query: &'a str,
    }

    let search_event = SearchEvent {
        total: 123,
        timed_out: false,
        query: "shoes",
    };

    Event::new("event_name", "portal", search_event).track();
}
```
