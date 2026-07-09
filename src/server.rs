use std::fmt;
use crate::RESP;

#[derive(Debug, PartialEq)]
pub enum ServerError {
    CommandError,
}

impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ServerError::CommandError => write!(f, "Error while processing!"),
        }
    }
}

pub type ServerResult<T> = Result<T, ServerError>;


// process an incomming request and return a result
pub fn process_request(request: RESP) -> ServerResult<RESP> {
    // check if the request is expressed using a RESP array and extract the elements.
    let elements = match request {
        RESP::Array(v) => v,
        _ => {
            return Err(ServerError::CommandError);
        }
    };

    // the vectors that contains all the command we need to process
    let mut command = Vec::new();

    // checked that each elem of the array is a bulk string, extract the content, and add it to the vector
    for elem in elements.iter() {
        match elem {
            RESP::BulkString(v) => command.push(v),
            _ => {
                return Err(ServerError::CommandError);
            }
        }
    }

    // match the first element of the vector with the code that implements that command.
    match command[0].to_lowercase().as_str() {
        "ping" => Ok(RESP::SimpleString(String::from("PONG"))),
        "echo" => Ok(RESP::BulkString(command[1].clone())),
        _ => {
            return Err(ServerError::CommandError);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    // test fn "process_request" processes a PING request and that it respond with PONG
    fn test_process_request_ping() {
        let request = RESP::Array(vec![RESP::BulkString(String::from("PING"))]);

        let output = process_request(request).unwrap();

        assert_eq!(output, RESP::SimpleString(String::from("PONG")));
    }

    #[test]
    // test the fn "process_request" return the correct error when given request that does not contain the RESP array
    fn test_process_request_not_array() {
        let request = RESP::BulkString(String::from("PING"));

        let error = process_request(request).unwrap_err();

        assert_eq!(error, ServerError::CommandError);
    }

    #[test]
    // test the function "process_request" return the correct error when it is given the correct
    //  RESP array but the content of the array is not a bulk array
    fn test_process_request_not_bulkstrings() {
        let request = RESP::Array(vec![RESP::SimpleString(String::from("PING"))]);

        let error = process_request(request).unwrap_err();
        assert_eq!(error, ServerError::CommandError);
    }

    #[test]
    // test the function "process_request" process an echo request and it is responds with a copy of the input 
    fn test_process_request_echo() {
        let request = RESP::Array(vec![RESP::BulkString(String::from("ECHO")),
                    RESP::BulkString(String::from("42"))]);

        let outptut = process_request(request).unwrap();

        assert_eq!(outptut, RESP::BulkString(String::from("42")));
    }
}