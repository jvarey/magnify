use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub(crate) enum ConnectionError {
    ConnectionDoesNotExist,
    ConnectionExists,
    NoConnections,
    NoDefaultConnection,
    WriteConnection,
}

impl fmt::Display for ConnectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConnectionError::ConnectionDoesNotExist => {
                write!(f, "There is no connection by that name")
            }
            ConnectionError::ConnectionExists => {
                write!(f, "A connection with this name already exists")
            }
            ConnectionError::NoDefaultConnection => {
                write!(
                    f,
                    "There are no connections saved that are flagged as default"
                )
            }
            ConnectionError::NoConnections => {
                write!(f, "There are no saved connections")
            }
            ConnectionError::WriteConnection => {
                write!(f, "Could not write connections file")
            }
        }
    }
}

impl Error for ConnectionError {}
