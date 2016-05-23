use std::borrow::Cow;


pub fn unescape_bytes<'a>(input: &'a [u8]) -> Cow<'a, [u8]> {
    let mut output = Cow::Borrowed(input);
    let mut iter = input.iter().enumerate();
    loop {
        let (idx, c) = match iter.next() {
            None => break,
            Some((idx, value)) => (idx, *value),
        };
        if c == b'\\' {
            let vec = into_buf(idx, &mut output);

            let escaped = match iter.next() {
                None => panic!("Invalid escape"),
                Some((_, value)) => *value,
            };
            match escaped {
                b'"' | b'\\' => vec.push(escaped),
                b'b' => vec.push(0x08),
                b'f' => vec.push(0x0C),
                b'n' => vec.push(0x0A),
                b'r' => vec.push(0x0D),
                b't' => vec.push(0x09),
                b'u' => {
                    // Skip the first two digits since they are zero.
                    if iter.next() == None || iter.next() == None {
                        panic!("Invalid escape")
                    }
                    let h2 = match iter.next() {
                        None => panic!("Invalid escape"),
                        Some((_, value)) => *value,
                    };
                    let h3 = match iter.next() {
                        None => panic!("Invalid escape"),
                        Some((_, value)) => *value,
                    };
                    let value = ((h2 - b'0') << 4) + ((h3 - b'0') & 0x1F);
                    if h3 > b'9' {
                        vec.push(value - 7);
                    } else {
                        vec.push(value);
                    }
                },
                _ => panic!("Invalid escape"),
            }
        }
    }

    output
}

const HEX : [u8 ; 16] = *b"0123456789ABCDEF";

pub fn escape_bytes<'a>(input: &'a [u8]) -> Cow<'a, [u8]> {
    let mut output = Cow::Borrowed(input);
    for (i, c) in input.iter().enumerate() {
        if *c < b' ' {
            let vec = into_buf(i, &mut output);
            vec.push(b'\\');
            match *c {
                0x08 => vec.push(b'b'),
                0x09 => vec.push(b't'),
                0x0A => vec.push(b'n'),
                0x0C => vec.push(b'f'),
                0x0D => vec.push(b'r'),
                _ => {
                    vec.extend(b"u00");
                    vec.push(b'0' + (c >> 4));
                    vec.push(HEX[(c & 0xF) as usize]);
                }
            }
            continue
        }
        if *c == b'\"' || *c == b'\\' {
            let vec = into_buf(i, &mut output);
            vec.push(b'\\');
            vec.push(*c);
        }
    }
    output
}

fn into_buf<'a, 'b>(idx: usize, cow: &'b mut Cow<'a, [u8]>) -> &'b mut Vec<u8> {
    if let Cow::Borrowed(input) = *cow {
        let mut vec = Vec::with_capacity(input.len() * 2);
        vec.extend(&input[..idx]);
        *cow = Cow::Owned(vec);
    }
    cow.to_mut()
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escape_control_characters() {
        assert_eq!(
            &br#"\u0000\u0001\u0002\u0003\u0004\u0005\u0006\u0007"#[..],
            &escape_bytes(
                b"\x00\x01\x02\x03\x04\x05\x06\x07"
            )[..]
        );
        assert_eq!(
            &br#"\b\t\n\u000B\f\r\u000E\u000F"#[..],
            &escape_bytes(
                b"\x08\x09\x0A\x0B\x0C\x0D\x0E\x0F"
            )[..]
        );
        assert_eq!(
            &br#"\u0010\u0011\u0012\u0013\u0014\u0015\u0016\u0017"#[..],
            &escape_bytes(
                b"\x10\x11\x12\x13\x14\x15\x16\x17"
            )[..]
        );
        assert_eq!(
            &br#"\u0018\u0019\u001A\u001B\u001C\u001D\u001E\u001F"#[..],
            &escape_bytes(
                b"\x18\x19\x1A\x1B\x1C\x1D\x1E\x1F"
            )[..]
        );
    }

    #[test]
    fn unescape_control_characters() {
        assert_eq!(
            &b"\x00\x01\x02\x03\x04\x05\x06\x07"[..],
            &unescape_bytes(
                br#"\u0000\u0001\u0002\u0003\u0004\u0005\u0006\u0007"#,
            )[..]
        );
        assert_eq!(
            &b"\x08\x09\x0A\x0B\x0C\x0D\x0E\x0F"[..],
            &unescape_bytes(
                br#"\b\t\n\u000B\f\r\u000E\u000F"#,
            )[..]
        );
        assert_eq!(
            &b"\x10\x11\x12\x13\x14\x15\x16\x17"[..],
            &unescape_bytes(
                br#"\u0010\u0011\u0012\u0013\u0014\u0015\u0016\u0017"#,
            )[..]
        );
        assert_eq!(
            &b"\x18\x19\x1A\x1B\x1C\x1D\x1E\x1F"[..],
            &unescape_bytes(
                br#"\u0018\u0019\u001A\u001B\u001C\u001D\u001E\u001F"#,
            )[..]
        );
    }

    #[test]
    fn escape_slash_and_quote() {
        assert_eq!(
            br#"\"\\"#, &escape_bytes(b"\"\\")[..]
        );
    }

    #[test]
    fn unescape_slash_and_quote() {
        assert_eq!(
            b"\"\\", &unescape_bytes(br#"\"\\"#)[..]
        );
    }
}
