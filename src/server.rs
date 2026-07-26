use std::{ sync::{Arc, Mutex}};
use crate::storage::Storage;
use crate::storage_result::{StorageError, StorageResult};
use crate::RESP;





// process an incomming request and return a result
pub fn process_request(request: RESP, storage: Arc<Mutex<Storage>>) -> StorageResult<RESP> {
    // check if the request is expressed using a RESP array and extract the elements.
    let elements = match request {
        RESP::Array(v) => v,
        _ => {
            return Err(StorageError::IncorrectRequest);
        }
    };

    // the vectors that contains all the command we need to process
    let mut command = Vec::new();

    // checked that each elem of the array is a bulk string, extract the content, and add it to the vector
    for elem in elements.iter() {
        match elem {
            RESP::BulkString(v) => command.push(v.clone()),
            _ => {
                return Err(StorageError::IncorrectRequest);
            }
        }
    }

    // Acquire a lock on the storoge;
    let mut guard = storage.lock().unwrap();

    // process the command contained in the request.
    let response = guard.process_command(&command);

    // return the response
    response
}

#[cfg(test)]
mod tests {
    use crate::storage;

use super::*;

    #[test]
    // test fn "process_request" processes a PING request and that it respond with PONG
    fn test_process_request_ping() {
        let request = RESP::Array(vec![RESP::BulkString(String::from("PING"))]);
        let storage = Arc::new(Mutex::new(Storage::new()));

        let output = process_request(request, storage).unwrap();

        assert_eq!(output, RESP::SimpleString(String::from("PONG")));
    }

    #[test]
    // test the fn "process_request" return the correct error when given request that does not contain the RESP array
    fn test_process_request_not_array() {
        let request = RESP::BulkString(String::from("PING"));
        let storage = Arc::new(Mutex::new(Storage::new()));

        let error = process_request(request, storage).unwrap_err();

        assert_eq!(error, StorageError::IncorrectRequest);
    }

    #[test]
    // test the function "process_request" return the correct error when it is given the correct
    //  RESP array but the content of the array is not a bulk array
    fn test_process_request_not_bulkstrings() {
        let request = RESP::Array(vec![RESP::SimpleString(String::from("PING"))]);
        let storage = Arc::new(Mutex::new(Storage::new()));

        let error = process_request(request, storage).unwrap_err();
        assert_eq!(error, StorageError::IncorrectRequest);
    }

    #[test]
    // test the function "process_request" process an echo request and it is responds with a copy of the input 
    fn test_process_request_echo() {
        let request = RESP::Array(vec![RESP::BulkString(String::from("ECHO")),
                    RESP::BulkString(String::from("42"))]);

        let storage = Arc::new(Mutex::new(Storage::new()));

        let outptut = process_request(request, storage).unwrap();

        assert_eq!(outptut, RESP::BulkString(String::from("42")));
    }
}