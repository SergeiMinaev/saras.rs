use std::fmt;
use serde::{ Serialize,Deserialize };
use crate::validation::ValidationErrors;
use once_cell::sync::Lazy;
use async_lock::RwLock;


pub enum Error {
	Common,
	Auth,
	Validation,
	Database,
	Storage,
	HTTP,
	Decode,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let description = match self {
            Error::Common => "Common",
            Error::Auth => "Auth",
            Error::Validation => "Validation",
            Error::Database => "Database",
            Error::Storage => "Storage",
            Error::HTTP => "HTTP",
            Error::Decode => "Decode",
        };
        write!(f, "{}", description)
    }
}
