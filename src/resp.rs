#[derive(Debug, PartialEq)]
pub enum Value {
  SimpleString(Vec<u8>),
}

pub fn decode(bytes: &[u8]) -> Option<Value> {
  decode_value(bytes).map(|(value, _rest)| value)
}

pub fn decode_value(bytes: &[u8]) -> Option<(Value, &[u8])> {
  if bytes.is_empty() {
    return None
  }

  match bytes[0] {
    b'+' => decode_simple_string(&bytes[1..]),
    _ => {
      println!("unexpected type specifier: {:?}", bytes[0]);
      None
    }
  }
}

fn decode_simple_string(bytes: &[u8]) -> Option<(Value, &[u8])> {
  // we may need to test if `b` is LF(\n) for RESP 3.0
  let Some(end) = bytes.iter().position(|b| *b == b'\r') else {
    println!("simple string is not terminated with CR");
    return None
  };

  // TODO: ensure that bytes ends with CR and LF

  let value = Value::SimpleString(bytes[..end].to_vec());
  let rest = &bytes[(end+1)..]; // skip LF
  Some((value, rest))
}

pub fn encode(value: &Value) -> Vec<u8> {
  match value {
    Value::SimpleString(bytes) => encode_simple_string(bytes),
    // _ => panic!("unsupported value"),
  }
}

fn encode_simple_string(bytes: &Vec<u8>) -> Vec<u8> {
  let mut v = Vec::new();
  v.push(b'+');

  for b in bytes {
    v.push(*b);
  }

  v.push(b'\r');
  v.push(b'\n');

  v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_simple_string() {
      let bytes = b"+OK\r\n";
      let result = decode(bytes);
      assert_eq!(result, Some(Value::SimpleString(b"OK".to_vec())))
    }

    #[test]
    fn test_encode_simple_string() {
      let value = Value::SimpleString(b"OK".to_vec());
      let result = encode(&value);
      assert_eq!(result, b"+OK\r\n".to_vec())
    }
}
