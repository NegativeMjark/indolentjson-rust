use readhex::*;

const HEX : [u8 ; 16] = [
    b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9',
    b'A', b'B', b'C', b'D', b'E', b'F'
];

pub fn compact_vector(input: &[u8], output: &mut Vec<u8>) -> bool {
    let mut iter = input.iter();

    loop {
        let input_char = match iter.next() {
            None => return true,
            Some(value) => *value,
        };
        if input_char <= b' ' { // Whitespace '\n', '\r', '\t', ' '
            continue;
        }
        output.push(input_char);
        if input_char == b'\"' { // Double Quote '\"'
            loop {
                let input_char = match iter.next() {
                    None => return false,
                    Some(value) => *value,
                };
                if input_char == b'\\' { // Back Slash '\\'
                    let input_char = match iter.next() {
                        None => return false,
                        Some(value) => *value,
                    };
                    if input_char == b'u' { // Unicode escape "u"
                        let h0 = match iter.next() {
                            None => return false,
                            Some(value) => *value
                        };
                        let h1 = match iter.next() {
                            None => return false,
                            Some(value) => *value
                        };
                        let h2 = match iter.next() {
                            None => return false,
                            Some(value) => *value
                        };
                        let h3 = match iter.next() {
                            None => return false,
                            Some(value) => *value
                        };
                        let escaped = read_hexdigits(h0, h1, h2, h3);
                        if escaped < 0x20 {
                            output.push(b'\\');
                            match escaped {
                                0x08 => output.push(b'b'),
                                0x09 => output.push(b't'),
                                0x0A => output.push(b'n'),
                                0x0C => output.push(b'f'),
                                0x0D => output.push(b'r'),
                                _ => {
                                    output.push(b'u');
                                    output.push(b'0');
                                    output.push(b'0');
                                    output.push(b'0' + (escaped as u8 >> 4));
                                    output.push(HEX[(escaped & 0xF) as usize]);
                                },
                            }
                        } else if escaped < 0x80 {
                            if escaped as u8 == b'\"' || escaped as u8 == b'\\' {
                                output.push(b'\\');
                            }
                            output.push(escaped as u8);
                        } else if escaped < 0x800 {
                            output.push((escaped >> 6) as u8 | 0xC0);
                            output.push((escaped as u8 & 0x3F) | 0x80);
                        } else {
                            if escaped < 0xD800 || escaped >= 0xE000 {
                                output.push((escaped >> 12) as u8 | 0xE0);
                                output.push(((escaped >> 6) & 0x3F) as u8 | 0x80);
                                output.push((escaped as u8 & 0x3F) | 0x80);
                            } else {
                                // surrogate pair
                                match iter.next() {
                                    None => return false,
                                    _ => {;}
                                }
                                match iter.next() {
                                    None => return false,
                                    _ => {;}
                                }
                                let h0 = match iter.next() {
                                    None => return false,
                                    Some(value) => *value
                                };
                                let h1 = match iter.next() {
                                    None => return false,
                                    Some(value) => *value
                                };
                                let h2 = match iter.next() {
                                    None => return false,
                                    Some(value) => *value
                                };
                                let h3 = match iter.next() {
                                    None => return false,
                                    Some(value) => *value
                                };
                                let surrogate = read_hexdigits(h0, h1, h2, h3);
                                let codepoint = 0x10000 + (
                                    ((escaped & 0x3FF) << 10) | (surrogate & 0x3FF)
                                );
                                output.push((codepoint >> 18) as u8 | 0xF0);
                                output.push(((codepoint >> 12) & 0x3F) as u8 | 0x80);
                                output.push(((codepoint >> 6) & 0x3F) as u8 | 0x80);
                                output.push((codepoint as u8 & 0x3F) | 0x80);
                            }
                        }

                    } else if input_char == b'/' { // Forward Slash '/'
                        output.push(input_char);
                    } else {
                        output.push(b'\\');
                        output.push(input_char);
                    }
                } else {
                    output.push(input_char);
                }
                if input_char == b'\"' {
                    break;
                }
            }
        }
    }
}

pub fn compact(input_json: &str) -> String {
    let mut output : Vec<u8> = Vec::with_capacity(input_json.as_bytes().len());
    if compact_vector(input_json.as_bytes(), &mut output) {
        return String::from_utf8(output).unwrap();
    } else {
        return "FAIL".to_string();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::black_box;
    use test::Bencher;

    #[test]
    fn compact_json_object() {
        assert_eq!("{}", compact("{ }"));
    }

    #[test]
    fn compact_json_string() {
        assert_eq!(
            r#"{"key":"\"hello / world\""}"#,
            compact(r#" { "key" : "\"hello \/ world\"" } "#)
        );
    }

    #[test]
    fn compact_unicode_escapes() {
        assert_eq!(
            r#"["\u0000\u0001\u0002\u0003\u0004\u0005\u0006\u0007"]"#,
            compact(r#"["\u0000\u0001\u0002\u0003\u0004\u0005\u0006\u0007"]"#)
        );
        assert_eq!(
            r#"["\b\t\n\u000B\f\r\u000E\u000F"]"#,
            compact(r#"["\u0008\u0009\u000A\u000B\u000C\u000D\u000E\u000F"]"#)
        );
        assert_eq!(
            r#"["\u0010\u0011\u0012\u0013\u0014\u0015\u0016\u0017"]"#,
            compact(r#"["\u0010\u0011\u0012\u0013\u0014\u0015\u0016\u0017"]"#)
        );
        assert_eq!(
            r#"["\u0018\u0019\u001A\u001B\u001C\u001D\u001E\u001F"]"#,
            compact(r#"["\u0018\u0019\u001A\u001B\u001C\u001D\u001E\u001F"]"#)
        );
    }

    #[test]
    fn compact_unicode_utf8() {
        assert_eq!(
            "[\"a\\\\B\\\"\"]",
            compact("[\"\\u0061\\u005C\\u0042\\u0022\"]")
        );
        assert_eq!("[\"\u{120}\"]", compact("[\"\\u0120\"]"));
        assert_eq!("[\"\u{FFF}\"]", compact("[\"\\u0FFF\"]"));
        assert_eq!("[\"\u{1820}\"]", compact("[\"\\u1820\"]"));
        assert_eq!("[\"\u{FFFF}\"]", compact("[\"\\uFFFF\"]"));
        assert_eq!("[\"\u{20820}\"]", compact("[\"\\uD842\\uDC20\"]"));
        assert_eq!("[\"\u{10FFFF}\"]", compact("[\"\\uDBFF\\uDFFF\"]"));
    }

    #[bench]
    fn benchmark_compact(b : &mut Bencher) {
        let test_string = black_box(r#"{
            "A longish bit of JSON": true,
            "containing": {
                "whitespace": " ",
                "unicode escapes ": "\uFFFF\u0FFF\u007F\uDBFF\uDFFF",
                "other sorts of esacpes": "\b\t\n\f\r\"\\\/",
                "unicode escapes for the other sorts of escapes":
                    "\u0008\u0009\u000A\u000C\u000D\u005C\u0022",
                "numbers": [0, 1, 1e4, 1.0, -1.0e7 ],
                "and more": [ true, false, null ]
            }
        }"#.as_bytes());
        let mut output : Vec<u8> = Vec::with_capacity(test_string.len());
        b.bytes = test_string.len() as u64;
        b.iter(|| { output.clear(); compact_vector(test_string, &mut output) });
    }
}
