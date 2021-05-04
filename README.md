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
let addr = "0.0.0.0:5005".parse().expect("valid addr");
let udp_relay = vinted_event_tracker::relay::Udp::new(addr);
let _ = vinted_event_tracker::set_relay(udp_relay);
```

In your library code, create an event structure and use it for tracking

```rust
#[derive(Debug, Serialize)]
struct SearchEvent {
    total: i32,
    timed_out: bool,
    query: String,
}

let search_event = SearchEvent {
    total: 123,
    timed_out: false,
    query: "shoes".to_string(),
};

let event = vinted_event_tracker::Event::new("event", "portal", search_event);

let _ = vinted_event_tracker::track(event);
```

Tracker implementations:

- `vinted_event_tracker::relay::Noop`
- `vinted_event_tracker::relay::Udp`
