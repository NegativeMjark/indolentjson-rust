// Copyright 2016 Erik Johnston
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

extern crate indolentjson;

use indolentjson::compact::*;
use indolentjson::parse::*;

use std::str;

const TEST_STRING : &'static [u8] = br#"{
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
}"#;

/// Enumerates the given nodes and prints their types.
fn enumerate_and_print(compacted: &[u8], nodes: &[Node]) {
    enumerate_internal(compacted, nodes.len(), &mut nodes.iter(), 0, false);
}


fn enumerate_internal<'a, I>(compacted: &[u8], iterations: usize, nodes: &mut I, depth: usize, is_object: bool)
    where I: Iterator<Item=&'a Node>
{
    let mut offset = 0;
    for idx in 0..iterations {
        let node = match nodes.next() {
            Some(n) => n,
            None => return,
        };

        let size_of_node = node.length_in_bytes as usize;

        if !is_object || idx % 2 == 0 {
            print!("{}", &"\t\t\t\t\t\t\t"[..depth]);
        }

        if node.children > 0 {
            println!("{} ", match compacted[offset] {
                b'{' => "object ->",
                b'[' => "array ->",
                _ => unreachable!(),  // Only arrays or objects can have children
            });
            enumerate_internal(
                &compacted[offset+1..],
                node.children as usize,
                nodes,
                depth + 1,
                compacted[offset] == b'{',
            );
        } else {
            print!("{} ", match compacted[offset] {
                b't' => "TRUE",
                b'f' => "FALSE",
                b'n' => "NULL",
                b'"' => "string",
                b'{' => "object (empty)",
                b'[' => "array (empty)",
                b'-' | b'0'...b'9' => "number",
                _ => unreachable!(),
            });

            if !is_object || idx % 2 == 1 {
                println!("");
            }
        }

        offset += size_of_node + 1;
    }
}


fn main() {
    let mut compacted: Vec<u8> = Vec::new();
    let mut nodes: Vec<Node> = Vec::new();
    let mut parse_stack: Vec<Stack> = Vec::new();

    compact(TEST_STRING, &mut compacted).unwrap();
    parse(&compacted[..], &mut nodes, &mut parse_stack).unwrap();

    enumerate_and_print(&compacted, &nodes);
}
