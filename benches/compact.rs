#![feature(test)]

extern crate indolentjson;
extern crate test;

use indolentjson::compact::*;
use test::{black_box, Bencher};

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
