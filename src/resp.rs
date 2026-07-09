use core::fmt;
use std::{io::BufRead, ops::Mul};

use crate::resp_result::{RESPError, RESPLength, RESPResult};

#[derive(Debug, PartialEq)]
pub enum RESP {
    Array(Vec<RESP>),
    BulkString(String),
    Null,
    SimpleString(String),
}

impl fmt::Display for RESP {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let data = match self {
            Self::Array(data) => {
                let mut output = String::from("*");
                output.push_str(format!("{}\r\n", data.len()).as_str());

                for elem in data.iter() {
                    output.push_str(elem.to_string().as_mut_str());
                }

                output
            }
            Self::BulkString(data) => format!("${}\r\n{}\r\n", data.len(), data),
            Self::Null => String::from("$-1\r\n"),
            Self::SimpleString(data) => format!("+{}\r\n", data),
        };
        write!(f, "{}", data)
    }
}

fn binary_extract_line(buffer: &[u8], index: &mut usize) -> RESPResult<Vec<u8>> {
    // the extracted bytes will sotred in this vector
    let mut output = Vec::new();

    // Exit with error if we try to read afte the end of buffer
    if *index >= buffer.len() {
        return Err(RESPError::OutOfBounds(*index));
    }

    // track the previous element we read
    let mut previous_elem: u8 = buffer[*index].clone();

    // flag that we found the charaters \r\n
    let mut seprator_flag: bool = false;

    // keep track of index from buffer in temprory index
    let mut final_index: usize = *index;

    // Now scan for \r\n and keep trac of current index and previous element
    for &elem in buffer[*index..].iter() {
        final_index += 1;

        // check if we passed the terminator \r\n
        if elem == b'\n' && previous_elem == b'\r' {
            // togglethr flag to signal that all is good
            seprator_flag = true;
            break;
        }

        // store the current element for the next loop
        previous_elem = elem.clone();
    }

    // if previous element is not found , then we are out of bounds
    if !seprator_flag {
        *index = final_index;
        return Err(RESPError::OutOfBounds(*index));
    }

    // copy the bytes from the buffer to the output vector
    // skip the final \r\n  by removing two bytes
    output.extend_from_slice(&buffer[*index..final_index - 2]);

    // make sure the index  is passed to the fn is updated with the final position and including the terminator \r\n
    *index = final_index;

    Ok(output)
}

fn binary_extract_line_as_string(buffer: &[u8], index: &mut usize) -> RESPResult<String> {
    // extract all possible bytes updating the index
    let line = binary_extract_line(buffer, index)?;

    // Convert the bytes to a UTF-8 String
    Ok(String::from_utf8(line)?)
}

// checks the firt character of the RESP buffer  is the given one and removes it.
pub fn resp_remove_type(value: char, buffer: &[u8], index: &mut usize) -> RESPResult<()> {
    // checkks weather the buffer contains the expected byte at the given index
    if buffer[*index] != value as u8 {
        return Err(RESPError::WrongType);
    }

    // increament the index to skip the type character
    *index += 1;

    Ok(())
}

// parse the simple string in the form of "+VALUE\r\n"
fn parse_simple_string(buffer: &[u8], index: &mut usize) -> RESPResult<RESP> {
    // Remove the type for the simple string
    resp_remove_type('+', buffer, index)?;

    // Read all possible bytes and convert them into UTF-8
    let line: String = binary_extract_line_as_string(buffer, index)?;

    // add the string to a custome RESP type
    Ok(RESP::SimpleString(line))
}

//select the correct parsing fn accroding to the data type found in the buffer at the given index
fn parser_router(
    buffer: &[u8],
    index: &mut usize,
) -> Option<fn(&[u8], &mut usize) -> RESPResult<RESP>> {
    // get the character at the current index
    // and associate it with the parsing function
    match buffer[*index] {
        b'+' => Some(parse_simple_string),
        b'$' => Some(parse_bulk_string),
        b'*' => Some(parse_array),
        _ => None,
    }
}

// Parse the bytes of the buffer at the given index and return a RESP data type
pub fn bytes_to_resp(buffer: &[u8], index: &mut usize) -> RESPResult<RESP> {
    // call the parsing function and manage the result
    match parser_router(buffer, index) {
        Some(parse_func) => {
            let result: RESP = parse_func(buffer, index)?;
            Ok(result)
        }
        None => Err(RESPError::Unknown),
    }
}

pub fn binary_extract_bytes(
    buffer: &[u8],
    index: &mut usize,
    lenght: usize,
) -> RESPResult<Vec<u8>> {
    // the output vector will contain the extrated bytes
    let mut output = Vec::new();

    // check if we are allowed to read the requested amountof bytes
    if *index + lenght > buffer.len() {
        return Err(RESPError::OutOfBounds(buffer.len()));
    }

    // copy the bytes from the buffer into the output vector
    output.extend_from_slice(&buffer[*index..*index + lenght]);

    // update the index
    *index += lenght;

    Ok(output)
}

// Extracts a single line from a RESP buffer and interprets it as length
// The type used for the number is RESPLength
pub fn resp_extract_length(buffer: &[u8], index: &mut usize) -> RESPResult<RESPLength> {
    // extract all bytes unitl a terminator is found and transform them into a string.
    let line: String = binary_extract_line_as_string(buffer, index)?;

    // convert the string into RESPLength
    let length: RESPLength = line.parse()?;

    Ok(length)
}

// Parse a bulk string in the form `$NUM\r\nVALUE\r\n`.
fn parse_bulk_string(buffer: &[u8], index: &mut usize) -> RESPResult<RESP> {
    // Remove the type for a bulk string.
    resp_remove_type('$', buffer, index)?;

    // read bytes from the buffer and interpret them as the length of the bulk string
    let length = resp_extract_length(buffer, index)?;

    // if the length is -1 we are looking at an empty bulk string
    if length == -1 {
        return Ok(RESP::Null);
    }

    // if the length is negative there is an error.
    if length < -1 {
        return Err(RESPError::IncorrectLength(length));
    }

    // Read all possible bytes from the buffer
    let bytes = binary_extract_bytes(buffer, index, length as usize)?;

    // convert the bytes into UTF-8
    let data: String = String::from_utf8(bytes)?;

    //Increament the index to skip the \r\n.
    *index += 2;

    Ok(RESP::BulkString(data))
}

// parse an array in the form `*NUM\r\nELEM1\r\nELEM2\2n...`
fn parse_array(buffer: &[u8], index: &mut usize) -> RESPResult<RESP> {
    // remove the type for an array
    resp_remove_type('*', buffer, index)?;

    // reads bytes from the buffer and interpret them as the number of elements in the array
    let length = resp_extract_length(buffer, index)?;

    // if the length is negative there is an error
    if length < 0 {
        return Err(RESPError::IncorrectLength(length));
    }

    // output vector will contain the elements extracted from array
    let mut data = Vec::new();

    // Extract all elements
    for _ in 0..length {
        // automatically detect the type of elements and parse it
        match parser_router(buffer, index) {
            Some(parse_func) => {
                let array_element: RESP = parse_func(buffer, index)?;

                // store the parse elem in vector
                data.push(array_element);
            }
            None => return Err(RESPError::Unknown),
        }
    }
    Ok(RESP::Array(data))
}

#[cfg(test)]
mod tests {
    use std::{
        ops::{Index, IndexMut},
        process::Output,
    };

    use crate::resp_result::RESPError;

    use super::*;

    #[test]
    fn test_binary_extract_line() {
        let buffer = "OK\r\n".as_bytes();
        let mut index: usize = 0;

        let output = binary_extract_line(buffer, &mut index).unwrap();

        assert_eq!(output, "OK".as_bytes());
        assert_eq!(index, 4);
    }

    #[test]
    fn test_binary_extract_line_longer_string() {
        let buffer = "ECHO\r\n".as_bytes();
        let mut index: usize = 0;

        let output = binary_extract_line(buffer, &mut index).unwrap();

        assert_eq!(output, "ECHO".as_bytes());
        assert_eq!(index, 6);
    }

    #[test]
    fn test_binary_extract_line_empty_buffer() {
        let buffer = "".as_bytes();
        let mut index: usize = 0;

        match binary_extract_line(buffer, &mut index) {
            Err(RESPError::OutOfBounds(index)) => {
                assert_eq!(index, 0);
            }
            _ => panic!(),
        }
    }

    #[test]
    // read the buffer that doesn't contain the terminator \r\n
    fn test_binaryxtract_line_no_separator() {
        let buffer = "OK".as_bytes();
        let mut index: usize = 0;

        match binary_extract_line(buffer, &mut index) {
            Err(RESPError::OutOfBounds(index)) => {
                assert_eq!(index, 2);
            }
            _ => panic!(),
        }
    }

    #[test]
    // Test that the function binary_extract_line
    // returns the correct error when we try to
    // read bytes starting with an index that is
    // already greater than the length of the buffer.
    fn test_binary_extract_line_index_too_advanced() {
        let buffer = "OK".as_bytes();
        let mut index: usize = 1;
        match binary_extract_line(buffer, &mut index) {
            Err(RESPError::OutOfBounds(index)) => {
                assert_eq!(index, 2);
            }
            _ => panic!(),
        }
    }

    #[test]
    // return the error, when read a buffer that contains only \r
    fn test_binary_extract_line_half_separator() {
        let buffer = "OK\r".as_bytes();
        let mut index: usize = 0;

        match binary_extract_line(buffer, &mut index) {
            Err(RESPError::OutOfBounds(index)) => {
                assert_eq!(index, 3);
            }
            _ => panic!(),
        }
    }

    #[test]
    // handle the error, when read try to read a buffer that contains only \n
    fn test_binary_extract_line_incorrect_separator() {
        let buffer = "OK\n".as_bytes();
        let mut index: usize = 0;

        match binary_extract_line(buffer, &mut index) {
            Err(RESPError::OutOfBounds(index)) => {
                assert_eq!(index, 3);
            }
            _ => panic!(),
        }
    }

    #[test]
    //converts a buffer with a terminator to a string
    fn test_extract_line_as_string() {
        let buffer = "OK\r\n".as_bytes();
        let mut index: usize = 0;

        let output = binary_extract_line_as_string(buffer, &mut index).unwrap();

        assert_eq!(output, String::from("OK"));
        assert_eq!(index, 4);
    }

    #[test]
    //test the fn binary_extract_line_as_string
    // return the correct error when we try to read a buffer that contains an invalid UTF-8 sequence of bytes
    fn test_binary_extract_line_as_string_invalid_utf8() {
        let buffer: Vec<u8> = vec![0xFF, 0xFE, b'\r', b'\n'];
        let mut index: usize = 0;

        let error = binary_extract_line_as_string(&buffer, &mut index).unwrap_err();

        assert_eq!(error, RESPError::FromUtf8);
    }

    #[test]
    //test that the function resp_remove_type
    // checks and return the given type from a buffer
    // and updates the index
    fn test_resp_remove_type() {
        let buffer = "+OK\r\n".as_bytes();
        let mut index: usize = 0;

        resp_remove_type('+', buffer, &mut index).unwrap();

        assert_eq!(index, 1);
    }

    #[test]
    // test that the fn remove resp_remove_type returns the correct error when we try
    // to read a given type from a buffer that contains a different type.
    fn test_resp_remove_type_error() {
        let buffer = "*OK\r\n".as_bytes();
        let mut index: usize = 0;

        let error = resp_remove_type('+', buffer, &mut index).unwrap_err();

        assert_eq!(index, 0);
        assert_eq!(error, RESPError::WrongType);
    }

    #[test]
    // test the fn parse_simple_string return the correct RESP variant when we read a buffer that contains a RESP simple string
    fn test_parse_simple_string() {
        let buffer = "+OK\r\n".as_bytes();
        let mut index: usize = 0;

        let output = parse_simple_string(buffer, &mut index).unwrap();

        assert_eq!(output, RESP::SimpleString(String::from("OK")));
        assert_eq!(index, 5);
    }

    #[test]
    // test the parsing process end-to-end, checking
    // that the function bytes_to_resp can parse a buffer with a RESP smiple string
    fn test_to_bytes_simple_string() {
        let buffer = "+OK\r\n".as_bytes();
        let mut index: usize = 0;

        let output = bytes_to_resp(buffer, &mut index).unwrap();

        assert_eq!(output, RESP::SimpleString(String::from("OK")));
        assert_eq!(index, 5);
    }

    #[test]
    //test the aring process end-to-end, checking that the fn bytes_to_resp can return the correct error when used to parse a buffer with an uunknown data type
    fn test_bytes_to_resp_unknown() {
        let buffer = "?OK\r\n".as_bytes();
        let mut index: usize = 0;

        let error = bytes_to_resp(buffer, &mut index).unwrap_err();

        assert_eq!(error, RESPError::Unknown);
        assert_eq!(index, 0);
    }

    #[test]
    // test the function binary_extract_bytes correctly from the requested number of bytes from the buffer , updating the index
    fn test_binary_extract_bytes() {
        let buffer = "SOMEBYTES".as_bytes();
        let mut index: usize = 0;

        let output = binary_extract_bytes(buffer, &mut index, 6).unwrap();

        assert_eq!(output, "SOMEBY".as_bytes().to_vec());
        assert_eq!(index, 6);
    }

    #[test]
    // test the fn binary_extract_bytes return the correct error when we try to read more bytes than are available in the buffer
    fn test_binary_extract_bytes_out_of_bounds() {
        let buffer = "SOMEBYTES".as_bytes();
        let mut index: usize = 0;

        let error = binary_extract_bytes(buffer, &mut index, 10).unwrap_err();

        assert_eq!(error, RESPError::OutOfBounds(9));
        assert_eq!(index, 0);
    }

    #[test]
    // test that the function parse_bulk_string returns the correct RESP variant
    // when we read the buffer that contains RESP bulk string
    fn test_parse_bulk_string() {
        let buffer = "$2\r\nOK\r\n".as_bytes();
        let mut index = 0;

        let output = parse_bulk_string(buffer, &mut index).unwrap();

        assert_eq!(output, RESP::BulkString(String::from("OK")));
        assert_eq!(index, 8);
    }

    #[test]
    // test that the fn parse_bulk_string return the correct RESP variant when we read a buffer that contains an empty RESP bulk string
    fn test_parse_bulk_string_empty() {
        let buffer = "$-1\r\n".as_bytes();
        let mut index: usize = 0;

        let output = parse_bulk_string(buffer, &mut index).unwrap();

        assert_eq!(output, RESP::Null);
        assert_eq!(index, 5);
    }

    #[test]
    // Test that the function parse_bulk_string returns the correct error when we try to
    // parse a RESP bulk string with an unparsable length.
    fn test_parse_bulk_string_unparsable_length() {
        let buffer = "$wrong\r\nOK\r\n".as_bytes();
        let mut index: usize = 0;
        let error = parse_bulk_string(buffer, &mut index).unwrap_err();
        assert_eq!(error, RESPError::ParseInt);
        assert_eq!(index, 8);
    }

    #[test]
    // Test that the function parse_bulk_string
    // returns the correct error when we try to
    // parse a RESP bulk string with a negative
    // length less than -1.
    fn test_parse_bulk_string_negative_length() {
        let buffer = "$-7\r\nOK\r\n".as_bytes();
        let mut index: usize = 0;
        let error = parse_bulk_string(buffer, &mut index).unwrap_err();
        assert_eq!(error, RESPError::IncorrectLength(-7));
        assert_eq!(index, 5);
    }

    #[test]
    // Test that the function parse_bulk_string
    // returns the correct error when we try to
    // parse a RESP bulk string but the buffer
    // doesn't contain enough bytes.
    fn test_parse_bulk_string_data_too_short() {
        let buffer = "$7\r\nOK\r\n".as_bytes();
        let mut index: usize = 0;
        let error = parse_bulk_string(buffer, &mut index).unwrap_err();
        assert_eq!(error, RESPError::OutOfBounds(8));
        assert_eq!(index, 4);
    }

    #[test]
    // Test the parsing process end-to-end, checking
    // that the function bytes_to_resp can parse
    // a buffer with a RESP bulk string.
    fn test_bytes_to_resp_bulk_string() {
        let buffer = "$2\r\nOK\r\n".as_bytes();
        let mut index: usize = 0;
        let output = bytes_to_resp(buffer, &mut index).unwrap();
        assert_eq!(output, RESP::BulkString(String::from("OK")));
        assert_eq!(index, 8);
    }

    #[test]
    // Test that the function parse_array
    // returns the correct RESP variant when
    // we read a buffer that contains an
    // array of two elements, a simple string
    // and a bulk string.
    fn test_parse_array() {
        let buffer = "*2\r\n+OK\r\n$5\r\nVALUE\r\n".as_bytes();
        let mut index: usize = 0;
        let output = parse_array(buffer, &mut index).unwrap();
        assert_eq!(
            output,
            RESP::Array(vec![
                RESP::SimpleString(String::from("OK")),
                RESP::BulkString(String::from("VALUE"))
            ])
        );
        assert_eq!(index, 20);
    }
    #[test]
    // Test that the function parse_array returns the correct error when we try to
    // parse an array with negative length.
    fn test_parse_array_invalid_length() {
        let buffer = "*-1\r\n+OK\r\n$5\r\nVALUE\r\n".as_bytes();
        let mut index: usize = 0;
        let error = parse_array(buffer, &mut index).unwrap_err();
        assert_eq!(error, RESPError::IncorrectLength(-1));
        assert_eq!(index, 5);
    }

    #[test]
    // Test the parsing process end-to-end, checking
    // that the function bytes_to_resp can parse
    // a buffer with a RESP array.
    fn test_bytes_to_resp_array() {
        let buffer = "*2\r\n+OK\r\n$5\r\nVALUE\r\n".as_bytes();
        let mut index: usize = 0;
        let output = bytes_to_resp(buffer, &mut index).unwrap();
        assert_eq!(
            output,
            RESP::Array(vec![
                RESP::SimpleString(String::from("OK")),
                RESP::BulkString(String::from("VALUE"))
            ])
        );
        assert_eq!(index, 20);
    }
}
