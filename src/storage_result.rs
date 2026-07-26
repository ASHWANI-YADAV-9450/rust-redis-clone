use std::{fmt, string};

#[derive(Debug, PartialEq)]
pub enum StorageError {
    IncorrectRequest,
    CommandNotAvailable(String),
    CommandInternalError(String),
    CommandSyntaxError(String)
}

impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StorageError::IncorrectRequest => {
                write!(f, "The client sent an incorrect request!")
            }
            StorageError::CommandNotAvailable(c) => {
                write!(f, "The request command {} is not available!", c)
            }
            StorageError::CommandInternalError(string) => {
                write!(f, "Internal error while processing {}!", string)
            } 
            StorageError::CommandSyntaxError(string) => {
                write!(f, "Syntax error while processing {}!", string)
            }
        }
    }
}

pub type StorageResult<T>  = Result<T, StorageError>;