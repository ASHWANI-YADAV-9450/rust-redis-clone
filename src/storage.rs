use std::collections::HashMap;

use crate::{resp::RESP, storage_result::{StorageError, StorageResult}};

#[derive(Debug, PartialEq)]
pub enum StorageValue {
    String(String),
}

pub struct Storage {
    store: HashMap<String, StorageValue>,
}

impl Storage {
    pub fn new() -> Self {
        let store: HashMap<String, StorageValue> = HashMap::new();

        Self {store: store}
    }
    // process an incoming command with its parameters.
    pub fn process_command(&mut self, command: &Vec<String>) -> StorageResult<RESP> {
        match command[0].to_lowercase().as_str() {
            "ping" => self.command_ping(&command),
            "echo" => self.command_echo(&command),
            _ => Err(StorageError::CommandNotAvailable(command[0].clone())),
        }
    }
    
    // the command `PING` reponds with as simple string that contains the value `PONG`
    fn command_ping(&self, _command: &Vec<String>) -> StorageResult<RESP> {
        Ok(RESP::SimpleString("PONG".to_string()))
    }


    // the command `ECHO` responds with a bulk string that contains the same value passed to `ECHO`
    fn command_echo(&self, command: &Vec<String>) -> StorageResult<RESP> {
        Ok(RESP::BulkString(command[1].clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    // test new storage can be created and that it is eempty
    fn test_create_new() {
        let storage: `Storage  = Storage::new();


    }
}