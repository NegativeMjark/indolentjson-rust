/* Copyright 2016 Mark Haines
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

#[derive(PartialEq, Debug)]
pub struct Node {
    children: u32,
    length_in_bytes: u32,
}


pub fn parse(input: &[u8], output: &mut Vec<Node>, stack: &mut Vec<u32>) -> Result<(),()> {
    if parse_(input, output, stack) {
        Ok(())
    } else {
        Err(())
    }
}

fn parse_(input: &[u8], output: &mut Vec<Node>, stack: &mut Vec<u32>) -> bool {
    let mut iter = input.iter().peekable();
    let mut parsing_object = false;
    if iter.len() == 2 {
        output.push(Node {children: 0, length_in_bytes: 2});
        return true;
    }
    'node_end: loop {
        match stack.pop() {
            Some(offset_and_state) => {
                let output_index = output.len() as u32;
                let offset = offset_and_state >> 1;
                let node = match output.get_mut(offset as usize) {
                    Some(node) => node,
                    None => return false
                };
                node.length_in_bytes -= iter.len() as u32;
                node.children = output_index - offset - 1;
                let prev_offset_and_state = match stack.last() {
                    Some(value) => value,
                    None => return true
                };
                parsing_object = (prev_offset_and_state & 1) == 1;
                let input_char = match iter.next() {
                    None => return false,
                    Some(value) => *value
                };
                if input_char != b',' {
                    continue 'node_end;
                }
            },
            None => {}
        };
        'value_start: loop {
            if parsing_object {
                if iter.next() == None {
                    return false;
                }
                let start = iter.len() + 1;
                loop {
                    let input_char = match iter.next() {
                        None => return false,
                        Some(value) => *value
                    };
                    if input_char == b'"' {
                        break;
                    }
                    if input_char == b'\\' {
                        if iter.next() == None {
                            return false;
                        }
                    }
                }
                output.push(Node {
                    children: 0, length_in_bytes: (start - iter.len()) as u32
                });
                if iter.next() == None {
                    return false;
                }
            }
            let start = iter.len();
            let input_char = match iter.next() {
                None => return false,
                Some(value) => *value
            };
            if input_char == b'{' {
                let peek_char = match iter.peek() {
                    None => return false,
                    Some(value) => **value
                };
                if peek_char == b'}' {
                    output.push(Node {children: 0, length_in_bytes: 2});
                    let _ = iter.next();
                } else {
                    let index = output.len() as u32;
                    stack.push((index << 1) | 1);
                    output.push(Node {
                        children: 0, length_in_bytes: start as u32,
                    });
                    parsing_object = true;
                    continue 'value_start;
                }
            } else if input_char == b'[' {
                let peek_char = match iter.peek() {
                    None => return false,
                    Some(value) => **value
                };
                if peek_char == b']' {
                    output.push(Node {children: 0, length_in_bytes: 2});
                    let _ = iter.next();
                } else {
                    let index = output.len() as u32;
                    stack.push(index << 1);
                    output.push(Node {
                        children: 0, length_in_bytes: start as u32,
                    });
                    parsing_object = false;
                    continue 'value_start;
                }
            } else if input_char == b'"' {
                loop {
                    let input_char = match iter.next() {
                        None => return false,
                        Some(value) => *value
                    };
                    if input_char == b'"' {
                        break;
                    }
                    if input_char == b'\\' {
                        if iter.next() == None {
                            return false;
                        }
                    }
                }
                output.push(Node {
                    children: 0,
                    length_in_bytes: (start - iter.len()) as u32
                });
            } else {
                loop {
                    let input_char = match iter.next() {
                        None => return false,
                        Some(value) => *value
                    };
                    if input_char == b',' {
                        output.push(Node {
                            children: 0,
                            length_in_bytes: (start - iter.len() - 1) as u32
                        });
                        continue 'value_start;
                    } else if (input_char & 0xDF) == b']' {
                        output.push(Node {
                            children: 0,
                            length_in_bytes: (start - iter.len() - 1) as u32
                        });
                        continue 'node_end;
                    }
                }
            }
            let input_char = match iter.next() {
                None => return false,
                Some(value) => *value
            };
            if input_char == b',' {
                continue 'value_start;
            } else {
                continue 'node_end;
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::Node;

    fn parse(input: &[u8]) -> Vec<Node> {
        let mut output : Vec<Node> = Vec::new();
        let mut stack : Vec<u32> = Vec::new();
        super::parse(input, &mut output, &mut stack).unwrap();
        return output;
    }

    #[test]
    fn parse_empty_array() {
        let result = parse(b"[]");
        assert_eq!(vec![
            Node {children: 0, length_in_bytes: 2},
        ], result);
    }

    #[test]
    fn parse_nested_arrays() {
        let result = parse(b"[[[]],[]]");
        assert_eq!(vec![
            Node {children: 3, length_in_bytes: 9},
            Node {children: 1, length_in_bytes: 4},
            Node {children: 0, length_in_bytes: 2},
            Node {children: 0, length_in_bytes: 2},
        ], result);
    }

    #[test]
    fn parse_nested_objects() {
        let result = parse(br#"{"A":{"B":{}},"C":{}}"#);
        assert_eq!(vec![
            Node {children: 6, length_in_bytes: 21},
            Node {children: 0, length_in_bytes: 3},
            Node {children: 2, length_in_bytes: 8},
            Node {children: 0, length_in_bytes: 3},
            Node {children: 0, length_in_bytes: 2},
            Node {children: 0, length_in_bytes: 3},
            Node {children: 0, length_in_bytes: 2},
        ], result);
    }

    #[test]
    fn parse_scalars() {
        let result = parse(br#"[false,null,true]"#);
        assert_eq!(vec![
            Node {children: 3, length_in_bytes: 17},
            Node {children: 0, length_in_bytes: 5},
            Node {children: 0, length_in_bytes: 4},
            Node {children: 0, length_in_bytes: 4},
        ], result);
    }
}
