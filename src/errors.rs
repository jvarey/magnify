use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub(crate) enum ConnectionError {
    ConnectionDoesNotExistError,
    ConnectionExistsError,
    NoConnectionsError,
    NoDefaultConnectionError,
}

impl fmt::Display for ConnectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConnectionError::ConnectionDoesNotExistError => {
                write!(f, "There is no connection by that name")
            }
            ConnectionError::ConnectionExistsError => {
                write!(f, "A connection with this name already exists")
            }
            ConnectionError::NoDefaultConnectionError => {
                write!(
                    f,
                    "There are no connections saved that are flagged as default"
                )
            }
            ConnectionError::NoConnectionsError => {
                write!(f, "There are no saved connections")
            }
        }
    }
}

impl Error for ConnectionError {}
