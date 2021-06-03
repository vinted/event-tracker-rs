use serde::Serialize;
use std::{fmt::Debug, time::SystemTime};

/// Event base
#[derive(Debug, Serialize)]
pub struct EventBase {
    /// Event name
    pub event: &'static str,

    /// Portal it's happening in
    pub portal: String,

    /// Current time in milliseconds since unix epoch
    pub time: u128,

    /// Debug pin
    pub debug_pin: Option<i32>,
}

/// Event to track
#[derive(Debug, Serialize)]
pub struct Event<T>
where
    T: Debug + Serialize,
{
    /// Event base
    #[serde(flatten)]
    pub base: EventBase,

    /// Additional tracking data
    #[serde(flatten)]
    pub tracking_data: T,
}

impl<T> Event<T>
where
    T: Debug + Serialize,
{
    /// Creates an instance of [`Event`]
    pub fn new(
        event: &'static str,
        portal: impl Into<String>,
        debug_pin: impl Into<Option<i32>>,
        tracking_data: T,
    ) -> Self {
        Self {
            base: EventBase {
                event,
                portal: portal.into(),
                time: SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .expect("SystemTime before UNIX_EPOCH")
                    .as_millis(),
                debug_pin: debug_pin.into(),
            },
            tracking_data,
        }
    }
}
