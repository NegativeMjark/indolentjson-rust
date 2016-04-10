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


/// Parsed JSON is stored as a byte array of compact JSON and an array of nodes.
/// Each node represents a JSON object, array or value.
///
/// The ``children`` field is the total number of children under this node.
/// The child nodes come directly after the parent node in the array.
/// This means that the next sibling of a node can be found at an offset
/// of ``children + 1`` into the array.
///
/// The ``length_in_bytes`` gives the length of the value in bytes.
/// This allows the offset of a node to be computed as follows.
/// The root node is at offset 0.
/// The first child node starts at its parent offset + 1.
/// Each subsequent child node starts 1 byte after the end of previous node.
///
/// JSON scalars never have children and are identified by their first byte.
///
///  * JSON true values start with ``b't' == 0x74``.
///  * JSON false values start with ``b'f' == 0x66``.
///  * JSON null values start with ``b'n' == 0x6E``.
///  * JSON string values start with ``b'"' == 0x22``.
///  * JSON number values start with either ``b'-' == 0x2B``
///    or one of ``b"0123456789"` == [0x30..0x39]``.
///
/// JSON arrays start with ``b'[' == 0x5B``. The direct children of the node
///      are the elements of the array.
///
/// JSON objects start with ``b'{' == 0x7B``. The direct children of the node
/// alternate between JSON string keys and their values.
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
        // If the input is two bytes long then it is an empty object
        // or an empty array.
        push_node(output, 2);
        return true;
    }
    'node_end: loop {
        match stack.pop() {
            // On the first iteration of the loop the stack is empty.
            // So we fall through to start parsing the root array or
            // object.
            None => {},
            // Otherwise we've reached the end of an array or object.
            Some(offset_and_state) => {
                // Grab a copy of the length to appease the borrow checker.
                let output_len = output.len() as u32;
                let offset = offset_and_state >> 1;
                // The stack held the where the node was in the output array.
                // Grab a mutable copy of the node so we can fill it out with
                // the number of children and update the length of the node.
                // This lookup should never fail.
                let node = match output.get_mut(offset as usize) {
                    Some(node) => node,
                    None => return false
                };
                // The node length contined the number of bytes left in the
                // iterator when the node began. Subtracting the number of
                // bytes left now will give us the length.
                node.length_in_bytes -= iter.len() as u32;
                node.children = output_len - offset - 1;
                // Take a look at the next entry in the stack to get whether we
                // are parsing an array or parsing an object.
                // If the stack is empty then we've parsed the root object or
                // array and we can return.
                let prev_offset_and_state = match stack.last() {
                    Some(value) => value,
                    None => return true // <-- This is where the parser exits.
                };
                // Whether we were parsing an object or and array is stored in
                // the first bit of the stack entry.
                parsing_object = (prev_offset_and_state & 1) == 1;
                // We've finished parsing a node. There's either a comma b','
                // followed by more stuff in the outer node. or the outer node
                // is ending with a b']' or a b'}'.
                let input_char = match iter.next() {
                    None => return false,
                    Some(value) => *value
                };
                // If the node ends then jump to handling the end of a node.
                if input_char != b',' {
                    continue 'node_end;
                }
                // Otherwise fallthrough to parsing the start of a value.
            }
        };
        'value_start: loop {
            if parsing_object {
                // If we are parsing an object then parse the string key.
                let start = iter.len();
                // We can assume it's a string so skip the opening b'"'.
                if iter.next() == None {
                    return false;
                }
                // Loop through the characters until we find a closing b'"'.
                if !parse_string(&mut iter) {
                    return false;
                }
                // Add a node with the string.
                push_node(output, start - iter.len());
                // Skip over the b':'.
                if iter.next() == None {
                    return false;
                }
            }
            // Parse a JSON value.
            let start = iter.len();
            let input_char = match iter.next() {
                None => return false,
                Some(value) => *value
            };
            if input_char == b'{' {
                // This is the start of a JSON object.
                // Look at the next char to check if the object is empty.
                let peek_char = match iter.peek() {
                    None => return false,
                    Some(value) => **value
                };
                if peek_char == b'}' {
                    // The object was empty, output a 2 byte node.
                    push_node(output, 2);
                    // Consume the b'}' character.
                    let _ = iter.next();
                } else {
                    // The object is not empty.
                    // Add a placeholder node to the output vector recording
                    // where the start of the object was. The placeholder
                    // will be filled out with the correct info when the node
                    // ends.
                    // Add the index of the placeholder node in the output
                    // vector to the stack. Set the low bit to indicate we are
                    // parsing an object.
                    let index = output.len() as u32;
                    stack.push((index << 1) | 1);
                    push_node(output, start);
                    parsing_object = true;
                    // Jump to parsing the start of a value.
                    continue 'value_start;
                }
            } else if input_char == b'[' {
                // This is the start of a JSON array.
                // Look at the next char to check if the array is empty.
                let peek_char = match iter.peek() {
                    None => return false,
                    Some(value) => **value
                };
                if peek_char == b']' {
                    // The array is empty, output a 2 byte node.
                    push_node(output, 2);
                    // Consume the ']' character.
                    let _ = iter.next();
                } else {
                    // The array is not empty.
                    let index = output.len() as u32;
                    // Add a placeholder node to the output vector recording
                    // where the start of the array was. The placeholder
                    // will be filled out with the correct info when the node
                    // ends.
                    // Add the index of the placeholder node in the output
                    // vector to the stack. Leave the low bit unset to indicate
                    // we are parsing an array.
                    stack.push(index << 1);
                    push_node(output, start);
                    parsing_object = false;
                    // Jump to parsing the start of a value.
                    continue 'value_start;
                }
            } else if input_char == b'"' {
                // We are parsing a string. Loop until we see a closing b'"'.
                if !parse_string(&mut iter) {
                    return false;
                }
                push_node(output, start - iter.len());
            } else {
                // We are parsing a number or one of true, false or null.
                // Loop until we see a b',', a b'}', or a b']'.
                loop {
                    let input_char = match iter.next() {
                        None => return false,
                        Some(value) => *value
                    };
                    if input_char == b',' {
                        push_node(output, start - iter.len() - 1);
                        // Jump to parsing the start of a value.
                        continue 'value_start;
                    } else if (input_char & 0xDF) == b']' {
                        push_node(output, start - iter.len() - 1);
                        // Jump to parsing the end of a node.
                        continue 'node_end;
                    }
                }
            }
            // Strings, empty objects and empty arrays fall through to here to
            // handle the end of a value.
            // The next character is either a b',' if there is another value
            // to parse in the containing object or array or the character
            // is a b']' or a b'}' if the contaning object or array is ending.
            let input_char = match iter.next() {
                None => return false,
                Some(value) => *value
            };
            if input_char == b',' {
                // Jump to parsing the start of a value.
                continue 'value_start;
            } else {
                // Jump to parsing the end of a node.
                continue 'node_end;
            }
        }
    }
}


fn parse_string<'a, T: Iterator<Item=&'a u8>>(iter: &mut T) -> bool {
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
    return true;
}

fn push_node(output: &mut Vec<Node>, len : usize) {
    output.push( Node {
        children: 0,
        length_in_bytes: len as u32,
    });
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
