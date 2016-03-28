use readhex::*;

const HEX : [u8 ; 16] = [
    0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39,
    0x41, 0x42, 0x43, 0x44, 0x45, 0x46
];

fn compact_vector(input: &[u8], output: &mut Vec<u8>) -> bool {
    let mut input_pos = 0;
    let input_end = input_pos + input.len();

    loop {
        if input_pos == input_end {
            return true;
        }
        let input_char = input[input_pos];
        input_pos += 1;
        if input_char <= b' ' { // Whitespace '\n', '\t', ' '
            continue;
        }
        output.push(input_char);
        if input_char == b'\"' { // Double Quote '\"'
            loop {
                if input_pos == input_end {
                    return false;
                }
                let input_char = input[input_pos];
                input_pos += 1;
                if input_char == b'\\' { // Back Slash '\\'
                    if input_pos == input_end {
                        return false;
                    }
                    let input_char = input[input_pos];
                    input_pos += 1;
                    if input_char == b'u' { // Unicode escape "u"
                        if input_end - input_pos < 4 {
                            return false;
                        }
                        let escaped = read_hexdigit_4(input, input_pos);
                        input_pos += 4;
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
                                if input_end - input_pos < 6 {
                                    return false;
                                }
                                let surrogate = read_hexdigit_4(input, input_pos + 2);
                                input_pos += 6;
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


    

}
