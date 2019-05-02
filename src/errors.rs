use std::error::Error;
use std::io;
use std::fmt;

use yaml_rust::scanner::ScanError;


#[derive(Clone,Debug)]
pub enum AppError {
    Config(String),
    Db(String),
    RegEx(String),
    ParseError(String),
    Request(String),
    IoError(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AppError::Config(msg)     => write!(f, "Configuration Error: {}", msg),
            AppError::Db(msg)         => write!(f, "DB Error: {}", msg),
            AppError::RegEx(msg)      => write!(f, "RegEx Error: {}", msg),
            AppError::ParseError(msg) => write!(f, "Parse Error: {}", msg),
            AppError::Request(msg)    => write!(f, "Request Error: {}", msg),
            AppError::IoError(msg)    => write!(f, "IO Error: {}", msg),
        }
    }
}

impl Error for AppError {}

impl From<postgres::Error> for AppError {
    fn from(err: postgres::Error) -> AppError {
        AppError::Db(err.as_db().unwrap().message.clone())
    }
}

impl From<regex::Error> for AppError {
    fn from(err: regex::Error) -> AppError {
        AppError::RegEx(err.to_string().clone())
    }
}

impl From<rust_decimal::Error> for AppError {
    fn from(err: rust_decimal::Error) -> AppError {
        AppError::ParseError(err.to_string().clone())
    }
}

impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> AppError {
        AppError::Request(err.to_string().clone())
    }
}

impl From<io::Error> for AppError {
    fn from(err: io::Error) -> AppError {
        AppError::IoError(err.to_string().clone())
    }
}

impl From<ScanError> for AppError {
    fn from(err: ScanError) -> AppError {
        AppError::Config(err.to_string().clone())
    }
}
