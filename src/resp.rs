#[derive(Debug, PartialEq)]
pub enum Value {
    SimpleString(Vec<u8>),
    Error(Vec<u8>),
    Integer(i64),
    Array(Vec<Value>),
    BulkString(Vec<u8>),
}

pub fn decode(bytes: &[u8]) -> Option<Value> {
    let (value, rest) = decode_value(bytes)?;

    if !rest.is_empty() {
        println!("malformed, rest = {:?}", rest);
        return None;
    }

    Some(value)
}

pub fn decode_value(bytes: &[u8]) -> Option<(Value, &[u8])> {
    if bytes.is_empty() {
        return None;
    }

    let rest = &bytes[1..];

    match bytes[0] {
        b'+' => decode_simple_string(rest),
        b'-' => decode_error(rest),
        b':' => decode_integer(rest),
        b'*' => decode_array(rest),
        b'$' => decode_bulk_string(rest),
        _ => {
            println!("unexpected type specifier: {:?}", bytes[0]);
            None
        }
    }
}

fn decode_simple_string(bytes: &[u8]) -> Option<(Value, &[u8])> {
    let (string_bytes, rest) = split_with_crlf(bytes)?;

    let value = Value::SimpleString(string_bytes.to_vec());
    Some((value, rest))
}

fn decode_error(bytes: &[u8]) -> Option<(Value, &[u8])> {
    let (string_bytes, rest) = split_with_crlf(bytes)?;

    let value = Value::Error(string_bytes.to_vec());
    Some((value, rest))
}

fn decode_integer(bytes: &[u8]) -> Option<(Value, &[u8])> {
    let (integer_bytes, rest) = split_with_crlf(bytes)?;

    let Ok(integer_str) = std::str::from_utf8(integer_bytes) else {
        println!("invalid UTF-8 sequence for integer");
        return None;
    };

    let Ok(integer) = integer_str.parse::<i64>() else {
        println!("failed to parse integer string: {}", integer_str);
        return None;
    };

    let value = Value::Integer(integer);
    Some((value, rest))
}

fn decode_array(bytes: &[u8]) -> Option<(Value, &[u8])> {
    let Some((size, mut rest)) = decode_size(bytes) else {
        println!("array size is not given");
        return None;
    };

    let mut items = Vec::with_capacity(size);
    for i in 0..size {
        println!("array loop: {:?}: {:?}", i, bytes);
        let Some((item, rest2)) = decode_value(rest) else {
            println!("failed to decode array item");
            return None;
        };
        items.push(item);
        rest = rest2;
    }

    let value = Value::Array(items);
    Some((value, rest))
}

fn decode_bulk_string(bytes: &[u8]) -> Option<(Value, &[u8])> {
    let Some((size, rest)) = decode_size(bytes) else {
        println!("bulkstring size is not given");
        return None;
    };

    let string = rest[..size].to_vec();
    let rest = consume_crlf(&rest[size..])?;

    let value = Value::BulkString(string);
    Some((value, rest))
}

fn decode_size(bytes: &[u8]) -> Option<(usize, &[u8])> {
    let (size_bytes, rest) = split_with_crlf(bytes)?;

    let Ok(size_str) = std::str::from_utf8(size_bytes) else {
        println!("invalid UTF-8 sequence for size");
        return None;
    };

    let Ok(size) = size_str.parse::<usize>() else {
        println!("failed to parse size string: {}", size_str);
        return None;
    };

    Some((size, rest))
}

#[test]
fn test_decode_size() {
    assert_eq!(decode_size(b""), None);
    assert_eq!(decode_size(b"\r"), None);
    assert_eq!(decode_size(b"\r\n"), None);
    assert_eq!(decode_size(b"0"), None);
    assert_eq!(decode_size(b"0\r\nrest"), Some((0usize, &b"rest"[..])));
    assert_eq!(decode_size(b"1\r\nrest"), Some((1usize, &b"rest"[..])));
    assert_eq!(decode_size(b"10\r\nrest"), Some((10usize, &b"rest"[..])));
}

fn split_with_crlf(bytes: &[u8]) -> Option<(&[u8], &[u8])> {
    // TODO: we may need to test if `b` is LF(\n) for RESP 3.0

    let Some(end) = bytes.iter().position(|b| *b == b'\r') else {
        println!("CR is not found");
        return None;
    };

    let head = &bytes[..end];
    let rest = consume_crlf(&bytes[end..])?;
    Some((head, rest))
}

#[test]
fn test_split_with_crlf() {
    assert_eq!(split_with_crlf(b""), None);
    assert_eq!(split_with_crlf(b"\r"), None);
    assert_eq!(split_with_crlf(b"\r\n"), Some((&b""[..], &b""[..])));
    assert_eq!(split_with_crlf(b"123\r\n"), Some((&b"123"[..], &b""[..])));
    assert_eq!(split_with_crlf(b"\r\n456"), Some((&b""[..], &b"456"[..])));
    assert_eq!(
        split_with_crlf(b"123\r\n456"),
        Some((&b"123"[..], &b"456"[..]))
    );
    assert_eq!(
        split_with_crlf(b"123\r\n456\r\n789"),
        Some((&b"123"[..], &b"456\r\n789"[..]))
    );
    assert_eq!(
        split_with_crlf(b"123\r\n\r\n789"),
        Some((&b"123"[..], &b"\r\n789"[..]))
    );
}

fn consume_crlf(bytes: &[u8]) -> Option<&[u8]> {
    // TODO: we may need to test if `b` is LF(\n) for RESP 3.0

    if bytes.len() >= 2 && bytes[0] == b'\r' && bytes[1] == b'\n' {
        Some(&bytes[2..]) // skip CR+LF
    } else {
        None
    }
}

#[test]
fn test_consume_crlf() {
    assert_eq!(consume_crlf(b""), None);
    assert_eq!(consume_crlf(b"\r"), None);
    assert_eq!(consume_crlf(b"\n"), None);
    assert_eq!(consume_crlf(b"123\r\n"), None);
    assert_eq!(consume_crlf(b"\r\n"), Some(&b""[..]));
    assert_eq!(consume_crlf(b"\r\n456"), Some(&b"456"[..]));
}

pub fn encode(value: &Value) -> Vec<u8> {
    match value {
        Value::SimpleString(bytes) => encode_simple_string(bytes),
        _ => panic!("unsupported value"),
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
    fn test_decode() {
        // Malformed
        assert_eq!(decode(b""), None);
        assert_eq!(decode(b"+OK\r\n+OK\r\n"), None);

        // SimpleString
        assert_eq!(
            decode(b"+OK\r\n"),
            Some(Value::SimpleString(b"OK".to_vec()))
        );

        // Error
        assert_eq!(
            decode(b"-ERR unknown command 'foobar'\r\n"),
            Some(Value::Error(b"ERR unknown command 'foobar'".to_vec()))
        );

        // Integer
        assert_eq!(decode(b":"), None);
        assert_eq!(decode(b":\r\n"), None);
        assert_eq!(decode(b":123\r\n"), Some(Value::Integer(123i64)));

        // Array
        assert_eq!(decode(b"*0\r\n"), Some(Value::Array(vec![])));
        assert_eq!(
            decode(b"*1\r\n+OK\r\n"),
            Some(Value::Array(vec![Value::SimpleString(b"OK".to_vec())]))
        );
        assert_eq!(
            decode(b"*2\r\n+OK\r\n+NG\r\n"),
            Some(Value::Array(vec![
                Value::SimpleString(b"OK".to_vec()),
                Value::SimpleString(b"NG".to_vec())
            ]))
        );

        // BulkString
        assert_eq!(decode(b"$"), None);
        assert_eq!(decode(b"$0"), None);
        assert_eq!(decode(b"$0\r"), None);
        assert_eq!(decode(b"$0\n"), None);
        assert_eq!(decode(b"$0\r\n"), None);
        assert_eq!(decode(b"$0\r\n\r\n"), Some(Value::BulkString(b"".to_vec())));
        assert_eq!(
            decode(b"$1\r\na\r\n"),
            Some(Value::BulkString(b"a".to_vec()))
        );
        assert_eq!(
            decode(b"$4\r\na\r\nb\r\n"),
            Some(Value::BulkString(b"a\r\nb".to_vec()))
        );
    }

    #[test]
    fn test_encode_simple_string() {
        let value = Value::SimpleString(b"OK".to_vec());
        let result = encode(&value);
        assert_eq!(result, b"+OK\r\n".to_vec())
    }
}
