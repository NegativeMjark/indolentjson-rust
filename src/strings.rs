use std::borrow::Cow;


pub fn unescape_bytes<'a>(input: &'a [u8]) -> Option<Cow<'a, [u8]>> {
    if let Some(pos) = input.iter().position(|c| *c == b'\\') {
        let mut output = Vec::with_capacity(input.len() * 2);
        output.extend_from_slice(&input[..pos]);

        let mut iter = input.iter().skip(pos);
        loop {
            let c = match iter.next() {
                Some(value) => *value,
                None => break,
            };
            if c == b'\\' {
                let escaped = match iter.next() {
                    Some(value) => *value,
                    None => return None,
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
                            return None
                        }
                        let h2 = match iter.next() {
                            Some(value) => *value,
                            None => return None,
                        };
                        let h3 = match iter.next() {
                            Some(value) => *value,
                            None => return None,
                        };
                        let value = ((h2 - b'0') << 4) + ((h3 - b'0') & 0x1F);
                        if h3 > b'9' {
                            output.push(value - 7);
                        } else {
                            output.push(value);
                        }
                    },
                    _ => return None,
                }
            } else {
                output.push(c);
            }
        }

        Some(Cow::Owned(output))
    } else {
        Some(Cow::Borrowed(input))
    }
}

const HEX : [u8 ; 16] = *b"0123456789ABCDEF";

pub fn escape_bytes<'a>(input: &'a [u8]) -> Cow<'a, [u8]> {
    if let Some(pos) = input.iter().position(|c| *c < b' ' || *c == b'\"' || *c == b'\\') {
        let mut output = Vec::with_capacity(input.len() * 2);
        output.extend_from_slice(&input[..pos]);

        for c in input.iter().skip(pos) {
            if *c < b' ' {
                output.push(b'\\');
                match *c {
                    0x08 => output.push(b'b'),
                    0x09 => output.push(b't'),
                    0x0A => output.push(b'n'),
                    0x0C => output.push(b'f'),
                    0x0D => output.push(b'r'),
                    _ => {
                        output.extend(b"u00");
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
        Cow::Owned(output)
    } else {
        Cow::Borrowed(input)
    }
}


#[cfg(all(feature = "quickcheck_test", test))]
mod quickcheck_test {
    use super::*;
    use quickcheck::TestResult;

    #[quickcheck]
    fn escape_unescape(xs: Vec<u8>) -> bool {
        &xs[..] == unescape_bytes(&escape_bytes(&xs)).unwrap().as_ref()
    }

    #[quickcheck]
    fn unescape_escape(xs: Vec<u8>) -> TestResult {
        if let Some(v1) = unescape_bytes(&xs) {
            let v2 = escape_bytes(&v1);
            if let Some(v3) = unescape_bytes(&v2) {
                let v4 = escape_bytes(&v3);
                return TestResult::from_bool(v4.as_ref() == &v2[..]);
            }
        }
        return TestResult::discard()
    }
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
        assert_eq!(
            &br#"\u0000 "#[..],
            &escape_bytes(
                b"\x00 "
            )[..]
        );
        assert_eq!(
            &br#" \u0000"#[..],
            &escape_bytes(
                b" \x00"
            )[..]
        );
    }

    #[test]
    fn unescape_control_characters() {
        assert_eq!(
            &b"\x00\x01\x02\x03\x04\x05\x06\x07"[..],
            &unescape_bytes(
                br#"\u0000\u0001\u0002\u0003\u0004\u0005\u0006\u0007"#,
            ).unwrap()[..]
        );
        assert_eq!(
            &b"\x08\x09\x0A\x0B\x0C\x0D\x0E\x0F"[..],
            &unescape_bytes(
                br#"\b\t\n\u000B\f\r\u000E\u000F"#,
            ).unwrap()[..]
        );
        assert_eq!(
            &b"\x10\x11\x12\x13\x14\x15\x16\x17"[..],
            &unescape_bytes(
                br#"\u0010\u0011\u0012\u0013\u0014\u0015\u0016\u0017"#,
            ).unwrap()[..]
        );
        assert_eq!(
            &b"\x18\x19\x1A\x1B\x1C\x1D\x1E\x1F"[..],
            &unescape_bytes(
                br#"\u0018\u0019\u001A\u001B\u001C\u001D\u001E\u001F"#,
            ).unwrap()[..]
        );
        assert_eq!(
            &b"\x00 "[..],
            &unescape_bytes(
                br#"\u0000 "#,
            ).unwrap()[..]
        );
        assert_eq!(
            &b" \x00"[..],
            &unescape_bytes(
                br#" \u0000"#,
            ).unwrap()[..]
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
            b"\"\\", &unescape_bytes(br#"\"\\"#).unwrap()[..]
        );
    }

    #[test]
    fn invalid_escape() {
        assert!(unescape_bytes(br#"\p"#).is_none());
        assert!(unescape_bytes(br#"\"#).is_none());
        assert!(unescape_bytes(br#"\u"#).is_none());
        assert!(unescape_bytes(br#"\u0"#).is_none());
        assert!(unescape_bytes(br#"\u00"#).is_none());
        assert!(unescape_bytes(br#"\u000"#).is_none());
    }
}
