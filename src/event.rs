use serde::Serialize;
use std::{fmt::Debug, time::SystemTime};

/// Event to track
#[derive(Debug, Serialize)]
pub struct Event<T>
where
    T: Debug + Serialize,
{
    /// Event name
    pub event: &'static str,

    /// Portal it's happening in
    pub portal: String,

    /// Current time in milliseconds since unix epoch
    pub time: u128,

    /// Additional tracking data
    #[serde(flatten)]
    pub tracking_data: T,
}

impl<T> Event<T>
where
    T: Debug + Serialize,
{
    /// Creates an instance of [`Event`]
    pub fn new(event: &'static str, portal: impl Into<String>, tracking_data: T) -> Self {
        Self {
            event,
            portal: portal.into(),
            time: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .expect("SystemTime before UNIX_EPOCH")
                .as_millis(),
            tracking_data,
        }
    }
}
