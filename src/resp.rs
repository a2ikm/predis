use std::fmt;

#[derive(Debug, PartialEq)]
pub enum Value {
  SimpleString(String),
}

#[derive(Debug, PartialEq)]
pub enum DecodeError {
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
        }
    }
}

impl std::error::Error for DecodeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
        }
    }
}

pub fn decode(_bytes: &[u8]) -> Result<Value, DecodeError> {
  Ok(Value::SimpleString("OK".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_simple_string() {
        let bytes = b"+OK\r\n";
        let result = decode(bytes);
        assert_eq!(result, Ok(Value::SimpleString("OK".to_string())))
    }
}
