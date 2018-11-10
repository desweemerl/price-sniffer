use std::error::Error;
use std::fmt;


#[derive(Clone,Debug)]
pub enum AppError {
    Config(String),
    Db(String),
    RegEx(String),
    ParseError(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AppError::Config(msg)     => write!(f, "Configuration Error: {}", msg),
            AppError::Db(msg)         => write!(f, "DB Error: {}", msg),
            AppError::RegEx(msg)      => write!(f, "RegEx Error: {}", msg),
            AppError::ParseError(msg) => write!(f, "Parse Error: {}", msg),
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

impl From<curl::Error> for AppError {
    fn from(err: curl::Error) -> AppError {
        AppError::RegEx(err.to_string().clone())
    }
}

impl From<ini::ini::Error> for AppError {
    fn from(err: ini::ini::Error) -> AppError {
        AppError::Config(err.to_string().clone())
    }
}
