pub fn unescape_bytes(input: &[u8]) -> Vec<u8> {
    let mut output = Vec::new();
    let mut iter = input.iter();
    loop {
        let c = match iter.next() {
            None => return output,
            Some(value) => *value,
        };
        if c == b'\\' {
            let escaped = match iter.next() {
                None => panic!("Invalid escape"),
                Some(value) => *value,
            };
            match escaped {
                b'"' | b'\\' => output.push(escaped),
                b'b' => output.push(0x08),
                b'f' => output.push(0x0C),
                b'n' => output.push(0x0A),
                b'r' => output.push(0x0D),
                b't' => output.push(0x09),
                b'u' => {
                    // Skip the first two digits since they are zero.
                    if iter.next() == None || iter.next() == None {
                        panic!("Invalid escape")
                    }
                    let h2 = match iter.next() {
                        None => panic!("Invalid escape"),
                        Some(value) => *value,
                    };
                    let h3 = match iter.next() {
                        None => panic!("Invalid escape"),
                        Some(value) => *value,
                    };
                    let value = ((h2 - b'0') << 4) + ((h3 - b'0') & 0x1F);
                    if h3 > b'9' {
                        output.push(value - 7);
                    } else {
                        output.push(value);
                    }
                },
                _ => return output,
            }
        } else {
            output.push(c);
        }
    }
}

const HEX : [u8 ; 16] = *b"0123456789ABCDEF";

pub fn escape_bytes(input: &[u8]) -> Vec<u8> {
    let mut output = Vec::new();
    for c in input.iter() {
        if *c < b' ' {
            output.push(b'\\');
            match *c {
                0x08 => output.push(b'b'),
                0x09 => output.push(b't'),
                0x0A => output.push(b'n'),
                0x0C => output.push(b'f'),
                0x0D => output.push(b'r'),
                _ => {
                    output.push(b'u');
                    output.push(b'0');
                    output.push(b'0');
                    output.push(b'0' + (c >> 4));
                    output.push(HEX[(c & 0xF) as usize]);
                }
            }
            continue
        }
        if *c == b'\"' || *c == b'\\' {
            output.push(b'\\');
        }
        output.push(*c);
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escape_control_characters() {
        assert_eq!(
            r#"\u0000\u0001\u0002\u0003\u0004\u0005\u0006\u0007"#,
            String::from_utf8(escape_bytes(
                b"\x00\x01\x02\x03\x04\x05\x06\x07"
            )).unwrap()
        );
        assert_eq!(
            r#"\b\t\n\u000B\f\r\u000E\u000F"#,
            String::from_utf8(escape_bytes(
                b"\x08\x09\x0A\x0B\x0C\x0D\x0E\x0F"
            )).unwrap()
        );
        assert_eq!(
            r#"\u0010\u0011\u0012\u0013\u0014\u0015\u0016\u0017"#,
            String::from_utf8(escape_bytes(
                b"\x10\x11\x12\x13\x14\x15\x16\x17"
            )).unwrap()
        );
        assert_eq!(
            r#"\u0018\u0019\u001A\u001B\u001C\u001D\u001E\u001F"#,
            String::from_utf8(escape_bytes(
                b"\x18\x19\x1A\x1B\x1C\x1D\x1E\x1F"
            )).unwrap()
        );
    }

    #[test]
    fn unescape_control_characters() {
        assert_eq!(
            "\x00\x01\x02\x03\x04\x05\x06\x07",
            String::from_utf8(unescape_bytes(
                br#"\u0000\u0001\u0002\u0003\u0004\u0005\u0006\u0007"#,
            )).unwrap()
        );
        assert_eq!(
            "\x08\x09\x0A\x0B\x0C\x0D\x0E\x0F",
            String::from_utf8(unescape_bytes(
                br#"\b\t\n\u000B\f\r\u000E\u000F"#,
            )).unwrap()
        );
        assert_eq!(
            "\x10\x11\x12\x13\x14\x15\x16\x17",
            String::from_utf8(unescape_bytes(
                br#"\u0010\u0011\u0012\u0013\u0014\u0015\u0016\u0017"#,
            )).unwrap()
        );
        assert_eq!(
            "\x18\x19\x1A\x1B\x1C\x1D\x1E\x1F",
            String::from_utf8(unescape_bytes(
                br#"\u0018\u0019\u001A\u001B\u001C\u001D\u001E\u001F"#,
            )).unwrap()
        );
    }

    #[test]
    fn escape_slash_and_quote() {
        assert_eq!(
            r#"\"\\"#, String::from_utf8(escape_bytes(b"\"\\")).unwrap()
        );
    }

    #[test]
    fn unescape_slash_and_quote() {
        assert_eq!(
            "\"\\", String::from_utf8(unescape_bytes(br#"\"\\"#)).unwrap()
        );
    }
}
