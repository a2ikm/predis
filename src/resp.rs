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

fn decode_size(bytes: &[u8]) -> Option<(usize, &[u8])> {
  let Some(end) = bytes.iter().position(|b| !b.is_ascii_digit()) else {
    println!("ASCII digit sequence for size is not terminated");
    return None
  };

  let size_bytes = &bytes[..end];
  let rest = &bytes[end..];

  let Ok(size_str) = std::str::from_utf8(size_bytes) else {
    println!("invalid UTF-8 sequence for size");
    return None
  };

  let Ok(size) = size_str.parse::<usize>() else {
    println!("failed to parse size string: {}", size_str);
    return None
  };

  Some((size, rest))
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
      assert_eq!(decode(b"+OK\r\n"), Some(Value::SimpleString(b"OK".to_vec())))
    }

    #[test]
    fn test_decode_size() {
      assert_eq!(decode_size(b""), None);
      assert_eq!(decode_size(b"\r"), None);
      assert_eq!(decode_size(b"\r\n"), None);
      assert_eq!(decode_size(b"0"), None);
      assert_eq!(decode_size(b"0\r\nrest"), Some((0usize, &b"\r\nrest"[..])));
      assert_eq!(decode_size(b"1\r\nrest"), Some((1usize, &b"\r\nrest"[..])));
      assert_eq!(decode_size(b"10\r\nrest"), Some((10usize, &b"\r\nrest"[..])));
    }

    #[test]
    fn test_encode_simple_string() {
      let value = Value::SimpleString(b"OK".to_vec());
      let result = encode(&value);
      assert_eq!(result, b"+OK\r\n".to_vec())
    }
}
