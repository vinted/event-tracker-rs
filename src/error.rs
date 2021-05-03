/// Crate level error enum
#[derive(Debug)]
pub enum Error {
    /// Occurs when a relay has already been initialized
    RelayAlreadyInitialized,

    /// Occurs when an event cannot be serialized
    Serde(serde_json::Error),
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RelayAlreadyInitialized => write!(
                f,
                "attempted to set relay after the relay was already initialized"
            ),
            Self::Serde(e) => write!(f, "{}", e),
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Self::Serde(error)
    }
}
