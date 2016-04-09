#![feature(test)]

extern crate indolentjson;
extern crate test;

use indolentjson::compact::*;
use indolentjson::parse::*;
use test::{black_box, Bencher};

const TEST_STRING : &'static str = r#"{
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


#[bench]
fn benchmark_parse_compact(b : &mut Bencher) {
    let test_string = black_box(TEST_STRING.as_bytes());
    let mut compacted : Vec<u8> = Vec::new();
    let mut parsed : Vec<Node> = Vec::new();
    let mut parse_stack : Vec<u32> = Vec::new();
    b.bytes = test_string.len() as u64;
    b.iter(|| {
        compacted.clear();
        compact(test_string, &mut compacted).unwrap();
        parsed.clear();
        parse(&compacted[..], &mut parsed, &mut parse_stack).unwrap();
    });
}


#[bench]
fn benchmark_parse(b : &mut Bencher) {
    let test_string = black_box(TEST_STRING.as_bytes());
    let mut compacted : Vec<u8> = Vec::new();
    let mut parsed : Vec<Node> = Vec::new();
    let mut parse_stack : Vec<u32> = Vec::new();
    compact(test_string, &mut compacted).unwrap();
    b.bytes = compacted.len() as u64;
    b.iter(|| {
        parsed.clear();
        parse(&compacted[..], &mut parsed, &mut parse_stack).unwrap();
    });
}


