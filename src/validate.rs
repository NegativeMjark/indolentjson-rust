use parse::Node;

struct ValidateStack {
    end: u32,
    is_object: bool,
}


/// Validate that the result of parsing JSON is actually valid JSON.
///
/// For efficiency the parser takes a number of shortcuts based on the
/// assumption that the JSON is valid. These shortcuts mean that the parser
/// may successfully parse invalid JSON. For some use cases this may be
/// acceptable, for example if the application is going to convert the
/// to another format or the application knows that the JSON is valid.
///
/// However if the application is going to pass the JSON to another system,
/// and the JSON has come from an untrusted source then it should check that
/// the JSON was acutally valid.
///
/// Warning. This assumes that the JSON as been compacted before parsing.
/// Compacting the JSON may inaddvertently convert invalid JSON into valid
/// JSON. This may be unsuitable for applications that are expected to ensure
/// that their input is valid JSON, rather than just their output.
pub fn validate(json_bytes: &[u8], json_nodes: &[Node]) -> Result<(),()> {
    if validate_(json_bytes, json_nodes) {
        Ok(())
    } else {
        Err(())
    }
}

fn validate_(json_bytes: &[u8], json_nodes: &[Node]) -> bool {
    if json_nodes.len() == 1 {
        return validate_empty(json_bytes);
    }
    let mut stack: Vec<ValidateStack> = Vec::new();
    let mut expecting_key = false;
    let mut offset = 0;
    let mut end = 0;
    let mut is_object = false;
    for (index, node) in json_nodes.iter().enumerate() {
        if expecting_key {
            let start = offset;
            offset += node.length_in_bytes as usize;
            if !validate_key(&json_bytes[start..offset]) {
                return false;
            }
            if json_bytes[offset] != b':' {
                return false;
            }
            offset += 1;
            expecting_key = false;
        } else if node.children > 0 {
            is_object = match json_bytes[offset] {
                b'{' => true,
                b'[' => false,
                _ => return false,
            };
            end = index + node.children as usize;
            stack.push(ValidateStack { end: end as u32, is_object: is_object });
            expecting_key = is_object;
            offset += 1
        } else {
            let start = offset;
            offset += node.length_in_bytes as usize;
            if !validate_scalar(&json_bytes[start..offset]) {
                return false;
            }
            while index == end {
                if is_object {
                    if json_bytes[offset] != b'}' {
                        return false;
                    }
                } else {
                    if json_bytes[offset] != b']' {
                        return false;
                    }
                }
                offset += 1;
                let _ = stack.pop();
                let state = match stack.last() {
                    None => return true,
                    Some(value) => value
                };
                end = state.end as usize;
                is_object = state.is_object;
            }
            offset += 1;
            expecting_key = is_object;
        }
    }
    false
}

/// Validate an empty array or object at the start of the JSON.
/// The parser assumes that all JSON of length 2 contains a single
/// empty erray or object. We therefore need to check both bytes
/// to make sure that was the case.
fn validate_empty(bytes: &[u8]) -> bool {
    if bytes.len() != 2 {
        return false;
    }
    let char1 = bytes[0];
    let char2 = bytes[1];
    match char1 {
        b'{' => char2 == b'}',
        b'[' => char2 == b']',
        _ => false,
    }
}

/// Validate a JSON scalar value. This may be an empty array, an empty object,
/// a literal, a string, or a number.
fn validate_scalar(bytes: &[u8]) -> bool {
    if validate_empty(bytes) {
        // The bytes were an empty array or an empty object.
        return true;
    }
    if bytes.len() == 0 {
        // The parser doesn't check if scalar is empty.
        return false;
    }
    match bytes[0] {
        b'\"' => validate_string(bytes),
        // Literals must be lower case.
        b't' => bytes == b"true",
        b'f' => bytes == b"false",
        b'n' => bytes == b"null",
        // Numbers start with an optional '-' minus sign followed by either a
        // '0' or a '1'...'9' followed by zero or more digits.
        b'-' => validate_negative(&bytes[1..]),
        b'0' => validate_fraction(&bytes[1..]),
        b'1' ... b'9' => validate_digits(&bytes[1..]),
        _ => false,
    }
}


/// Validate a JSON object key. We need to check that it starts with a '"'
/// since the parser assumes that the character following a '{' or a '.' is
/// a '"'.
fn validate_key(bytes: &[u8]) -> bool {
    if bytes.len() == 0 {
        return false;
    }
    match bytes[0] {
        b'\"' => validate_string(bytes),
        _ => false,
    }
}

/// Validate a JSON string. The parser checks for the starting and ending '"'.
/// So we just need to check that there isn't any illegal control characters,
/// and the escapes are valid.
fn validate_string(bytes: &[u8]) -> bool {
    if bytes.len() < 2 {
        // This shouldn't happen since the parser must have found an opening
        // and a closing '"'.
        return false;
    }
    let mut iter = bytes[1..bytes.len()-1].iter();
    loop {
        let c = match iter.next() {
            None => return true,
            Some(value) => *value,
        };
        if c == b'\\' {
            let escaped = match iter.next() {
                // There should always be another character since the parser
                // checks for escapes at the end of strings.
                None => return false,
                Some(value) => *value,
            };
            match escaped {
                // We only need to check that the chacacter is valid. We don't
                // need to check the contents of a '\u' escape because
                // the '\u' escapes are parsed and either removed or
                // regenerated when the JSON is compacted.
                // We disallow '/' even though the spec allows it as a JSON
                // escape because it should be removed when the JSON is
                // compacted.
                b'"' | b'\\' => continue,
                b'b' | b'f' | b'n' | b'r' | b't' | b'u' => continue,
                _ => return false,
            }
        } else if c < b' ' {
            // Check for control characters less than b' ' == 0x20.
            return false
        }
    }
}


/// Validate a negative number checking the bytes after the '-' sign.
fn validate_negative(bytes: &[u8]) -> bool {
    if bytes.len() == 0 {
        return false;
    }
    match bytes[0] {
        // The '-' must be followed by either a single '0' or '1'...'9'
        // followed by zero or more digits.
        b'0' => validate_fraction(&bytes[1..]),
        b'1' ... b'9' => validate_digits(&bytes[1..]),
        _ => false,
    }
}

/// Validate the digits of number up to a decimal point or an exponent.
fn validate_digits(bytes: &[u8]) -> bool {
    for (index, byte) in bytes.iter().enumerate() {
        match *byte {
            b'0' ... b'9' => continue,
            // If this isn't a digit then it must be a '.' or a 'e' or 'E'.
            _ => return validate_fraction(&bytes[index..]),
        }
    }
    // There wasn't a decimal point or an exponent.
    true
}

/// Validate the number from a decimal point onwards.
fn validate_fraction(bytes: &[u8]) -> bool {
    if bytes.len() == 0 {
        // The number had no fraction or exponent.
        return true;
    }
    if bytes.len() < 2 {
        // The number must have a digit after the decimal point or exponent.
        return false;
    }
    if bytes[0] != b'.' {
        // The number has no fraction but it does have an exponent.
        return validate_exponent(bytes);
    }
    let digits = &bytes[1..];
    for (index, byte) in digits.iter().enumerate() {
        match *byte {
            b'0' ... b'9' => continue,
            // If this isn't a digit then it must be an exponent.
            _ => return validate_exponent(&digits[index..]),
        }
    }
    // There wasn't an exponent.
    true
}


// Validate an exponent.
fn validate_exponent(bytes: &[u8]) -> bool {
    if bytes.len() < 2 {
        // Exponents must have a digit after the 'e' or 'E'.
        return false;
    }
    match bytes[0] {
        // Exponents must start with either a 'e' or a 'E'.
        b'E' | b'e' => {},
        _ => return false,
    }
    let offset = match bytes[1] {
        // Exponents may be optionally prefixed by a '+' or a '-'.
        // They still must contain at least one digit.
        b'+' | b'-' => {
            if bytes.len() < 3 {
                return false;
            }
            2
        }
        _ => 1,
    };
    for byte in bytes[offset..].iter() {
        match *byte {
            b'0' ... b'9' => continue,
            _ => return false,
        }
    }
    true
}

#[cfg(test)]
mod test {

    fn validate(input: &[u8]) -> bool {
        let mut parsed : Vec<::parse::Node> = Vec::new();
        let mut stack : Vec<::parse::Stack> = Vec::new();
        ::parse::parse(input, &mut parsed, &mut stack).unwrap();
        match super::validate(input, parsed.as_slice()) {
            Ok(()) => true,
            Err(()) => false
        }
    }

    #[test]
    fn validate_mismatched_brackets() {
        assert_eq!(true, validate(b"{}"));
        assert_eq!(true, validate(b"[]"));
        assert_eq!(true, validate(b"[0]"));
        assert_eq!(true, validate(br#"{"":1}"#));

        assert_eq!(false, validate(b"{]"));
        assert_eq!(false, validate(b"[}"));
        assert_eq!(false, validate(b"[0}"));
        assert_eq!(false, validate(br#"{"":1]"#));
    }

    #[test]
    fn validate_invalid_brackets() {
        assert_eq!(false, validate(b"{@"));
        assert_eq!(false, validate(b"[@"));
        assert_eq!(false, validate(br#"[""@"#));
    }

    #[test]
    fn validate_empty() {
        assert_eq!(false, validate(b"[,]"));
    }

    #[test]
    fn validate_invalid_object_key() {
        assert_eq!(false, validate(br#"{@":1}"#));
        assert_eq!(false, validate(br#"{""@1}"#));
    }

    #[test]
    fn validate_literals() {
        assert_eq!(true, validate(b"[false]"));
        assert_eq!(true, validate(b"[true]"));
        assert_eq!(true, validate(b"[null]"));

        assert_eq!(false, validate(b"[fslae]"));
        assert_eq!(false, validate(b"[ture]"));
        assert_eq!(false, validate(b"[nlul]"));

        assert_eq!(false, validate(b"[FALSE]"));
        assert_eq!(false, validate(b"[TRUE]"));
        assert_eq!(false, validate(b"[NULL]"));
    }

    #[test]
    fn validate_strings() {
        assert_eq!(true, validate(br#"["\"\\\b\f\n\r\t\u0000"]"#));
        assert_eq!(false, validate(br#"["\/"]"#));
        assert_eq!(false, validate(br#"["\g"]"#));
    }

    #[test]
    fn validate_whitespace() {
        assert_eq!(false, validate(b"[\"\n\"]"));
    }

    #[test]
    fn validate_decimals() {
        assert_eq!(true, validate(b"[0,1,2,3,4,5,6,7,8,9]"));
        assert_eq!(true, validate(b"[10,21,32,43,54,65,76,87,98]"));

        assert_eq!(false, validate(b"[00]"));
        assert_eq!(false, validate(b"[1A]"));
    }

    #[test]
    fn validate_fractions() {
        assert_eq!(true, validate(b"[0.0,0.01,0.123456789]"));
        assert_eq!(false, validate(b"[0.]"));
        assert_eq!(false, validate(b"[0A0]"));
    }

    #[test]
    fn validate_exponent() {
        assert_eq!(true, validate(b"[0e0,0e1,1e00,1e10,1e99]"));
        assert_eq!(true, validate(b"[0E0,0E1,1E00,1E10,1E99]"));
        assert_eq!(true, validate(b"[0.0e-0]"));
        assert_eq!(true, validate(b"[0.0e+0]"));

        assert_eq!(false, validate(b"[0eA]"));
        assert_eq!(false, validate(b"[0e0.1]"));
    }

    #[test]
    fn validate_negative() {
        assert_eq!(true, validate(b"[-0,-1,-0.0,-0.0e1]"));

        assert_eq!(false, validate(b"[+1]"));
        assert_eq!(false, validate(b"[-00]"));
    }

    #[test]
    fn validate_nested() {
        assert_eq!(true, validate(br#"[{"":[]},[],{}]"#));
    }
}
