use std::fmt;
use std::num;
use std::string::FromUtf8Error;

use crate::resp_result::RESPError::WrongType;


#[derive(Debug, PartialEq)]
pub enum RESPError {
    FromUtf8,
    OutOfBounds(usize),
    Unknown,
    WrongType,
    IncorrectLength(RESPLength),
    ParseInt,
}



impl fmt::Display for RESPError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RESPError::FromUtf8 => write!(f, "Cannot convert from UTF-8"),
            RESPError::OutOfBounds(index) => write!(f, "Out of bounds at index {}", index),
            RESPError::WrongType => write!(f, "Wrong prefix for RESP type"),
            RESPError::Unknown => write!(f, "Unknown formate for RESP stirng",),
            RESPError::IncorrectLength(length) => write!(f, "Incorrect length {}", length),
            RESPError::ParseInt => write!(f, "Cannot parse string into integer"),

        }
    }
}

impl From<FromUtf8Error> for RESPError {
    fn from(_err: FromUtf8Error) -> Self {
        Self::FromUtf8
    }
}

impl From<num::ParseIntError> for RESPError {
    fn from(_err: num::ParseIntError) -> Self {
        Self::ParseInt
    }
}


pub type RESPResult<T> = Result<T, RESPError>;
pub type RESPLength = i32;