use std::fmt;
use std::io;

use reqwest::Error as ReqwestError;

#[derive(Debug)]
pub enum RawstErr {
    HttpError(ReqwestError),
    FileError(io::Error),
    Unknown(ReqwestError),
    InvalidThreadCount,
    InvalidArgs,
    BadRequest,
    Unauthorized,
    Forbidden,
    NotFound,
    InternalServerError,
    Unreachable,
}

impl fmt::Display for RawstErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {

            RawstErr::HttpError(err) => write!(f, "HTTP Error: {}", err),
            RawstErr::FileError(err) => write!(f, "File Error: {}", err),
            RawstErr::InvalidThreadCount => write!(f, "Invalid number of threads"),
            RawstErr::InvalidArgs => write!(f, "Invalid Arguments or No Arguments"),
            RawstErr::BadRequest => write!(f, "Bad Request: The server cannot or will not process the request due to something that is perceived to be a client error."),
            RawstErr::Unauthorized => write!(f, "Unauthorized: The request has not been applied because it lacks valid authentication credentials for the target resource."),
            RawstErr::Forbidden => write!(f, "Forbidden: The server understood the request, but it refuses to authorize it."),
            RawstErr::NotFound => write!(f, "Not Found: The server has not found anything matching the Request-URI."),
            RawstErr::InternalServerError => write!(f, "Internal Server Error: The server encountered an unexpected condition which prevented it from fulfilling the request."),
            RawstErr::Unreachable => write!(f, "Unreachable: The request was not able to reach the server"),
            RawstErr::Unknown(err) => write!(f, "Unknow Error: {}", err)

        }
    }
}

impl std::error::Error for RawstErr {}
