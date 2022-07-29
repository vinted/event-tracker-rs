# Event Tracker

A Rust port of [vinted/event-tracker](https://github.com/vinted/event-tracker)

## Installation

Add this line to your application's `Cargo.toml`:

```toml
vinted_event_tracker = { git = "https://github.com/vinted/event-tracker-rs" }
```

## Usage

Configure the crate in your application executable, e.g. `src/main.rs` or `src/bin/executable_name.rs`.

```rust
use serde::Serialize;
use vinted_event_tracker::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let udp_relay = Udp::bind("0.0.0.0:5005")?;

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

    let event = "event_name";
    let portal = "fr";
    let debug_pin = Some(1234);
    let search_event = SearchEvent {
        total: 123,
        timed_out: false,
        query: "shoes",
    };

    let event = Event::new(event, portal, debug_pin, search_event);

    let _ = track(event);
}
```
