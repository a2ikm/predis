use std::{fmt, io::{Bytes, Read}};

#[derive(Debug, PartialEq)]
pub enum Value {
  SimpleString(Vec<u8>),
}

#[derive(Debug, PartialEq)]
pub enum DecodeError {
  IOError,
  UnexpectedEOF,
  UnexpectedType,
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
          DecodeError::IOError => write!(f, "io error"),
          DecodeError::UnexpectedEOF => write!(f, "unexpected eof"),
          DecodeError::UnexpectedType => write!(f, "unexpected type"),
        }
    }
}

impl std::error::Error for DecodeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

pub fn decode(bytes: &[u8]) -> Result<Value, DecodeError> {
  decode_value(&mut bytes.bytes())
}

fn decode_value(bytes: &mut Bytes<&[u8]>) -> Result<Value, DecodeError> {
  match bytes.next() {
    Some(Ok(b)) => {
      match b {
        b'+' => decode_simple_string(bytes),
        _ => Err(DecodeError::UnexpectedType),
      }
    },
    Some(Err(e)) => {
      println!("decode_value error: {:?}", e);
      Err(DecodeError::IOError)
    },
    None => {
      Err(DecodeError::UnexpectedEOF)
    }
  }
}

fn decode_simple_string(bytes: &mut Bytes<&[u8]>) -> Result<Value, DecodeError> {
  let mut v: Vec<u8> = Vec::new();
  loop {
    match bytes.next() {
      Some(Ok(b'\r')) => {
        bytes.next(); // drop LF(\n)
        break;
      },
      Some(Ok(b)) => {
        v.push(b);
      },
      Some(Err(e)) => {
        println!("decode_simple_string error: {:?}", e);
        return Err(DecodeError::IOError)
      },
      None => {
        return Err(DecodeError::UnexpectedEOF)
      }
    }
  }
  Ok(Value::SimpleString(v))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_simple_string() {
        let bytes = b"+OK\r\n";
        let result = decode(bytes);
        assert_eq!(result, Ok(Value::SimpleString(b"OK".to_vec())))
    }
}
