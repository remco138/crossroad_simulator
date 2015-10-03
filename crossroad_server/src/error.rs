use std::error::Error as StdError;
use std::io::Error as IoError;
use serde_json::error::Error as SerdeError;

use std::fmt;


// ------------------------------------------------------------------------------
// Crate Error/Result
// ------------------------------------------------------------------------------

pub type Result<T> = ::std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Io(IoError),
    Serde(SerdeError),
    SerdeJson(JsonError),
    Other(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Io(ref err) => fmt::Display::fmt(err, f),
            Error::Serde(ref err) => fmt::Display::fmt(err, f),
            Error::SerdeJson(ref err) => fmt::Display::fmt(err, f),
            Error::Other(ref err) => err.fmt(f),
        }
    }
}

impl From<IoError> for Error {
    fn from(err: IoError) -> Error {
        Error::Io(err)
    }   
}

impl From<JsonError> for Error {
    fn from(err: JsonError) -> Error {
        Error::SerdeJson(err)
    }
}


// ------------------------------------------------------------------------------
// JsonError
// ------------------------------------------------------------------------------

pub struct JsonError {
    pub error: SerdeError,
    pub json: String,
}

impl JsonError {
    pub fn new(json_str: &str, serde_error: SerdeError) -> JsonError {
        JsonError { json: json_str.to_string(), error: serde_error, }
    }
}

impl fmt::Debug for JsonError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\n\n{}\n{}\n", self.json, self.error)
    }
}
impl fmt::Display for JsonError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.error.fmt(f)
    }
}

impl StdError for JsonError {
    fn description(&self) -> &str {
       self.error.description()
    }
    fn cause(&self) -> Option<&StdError> {
        Some(self)
    }
}