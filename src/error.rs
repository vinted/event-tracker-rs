/// Crate level error enum
#[derive(Debug)]
pub enum Error {
    /// Occurs when an event cannot be serialized
    SerdeJson(serde_json::Error),

    /// I/O error
    Io(std::io::Error),

    /// FromStr error
    AddrParse(std::net::AddrParseError),

    /// FromStr error
    NoRemoteAddr,
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SerdeJson(e) => e.fmt(f),
            Self::Io(e) => e.fmt(f),
            Self::AddrParse(e) => e.fmt(f),
            Self::NoRemoteAddr => "no remote address specified".fmt(f),
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Self::SerdeJson(error)
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<std::net::AddrParseError> for Error {
    fn from(error: std::net::AddrParseError) -> Self {
        Self::AddrParse(error)
    }
}
