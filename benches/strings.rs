#![feature(test)]

extern crate indolentjson;
extern crate test;

use indolentjson::strings::*;
use test::{black_box, Bencher};


#[bench]
fn noop_escape(b: &mut Bencher) {
    let test_string = black_box(b"test");

    b.bytes = test_string.len() as u64;
    b.iter(|| { escape_bytes(test_string) });
}

#[bench]
fn escape_controls(b: &mut Bencher) {
    let test_string = black_box(b"\t\n\r\\\"");

    b.bytes = test_string.len() as u64;
    b.iter(|| { escape_bytes(test_string) });
}

#[bench]
fn escape_mixed(b: &mut Bencher) {
    let test_string = black_box(
        b"This\nIsA\tMixture\x00OfStrings\x0cThat\"Need\\Escaping"
    );

    b.bytes = test_string.len() as u64;
    b.iter(|| { escape_bytes(test_string) });
}

#[bench]
fn noop_unescape(b: &mut Bencher) {
    let test_string = black_box(b"test");

    b.bytes = test_string.len() as u64;
    b.iter(|| { unescape_bytes(test_string) });
}

#[bench]
fn unescape_controls(b: &mut Bencher) {
    let test_string = black_box(br#"\t\n\r\\\""#);

    b.bytes = test_string.len() as u64;
    b.iter(|| { unescape_bytes(test_string) });
}

#[bench]
fn unescape_mixed(b: &mut Bencher) {
    let test_string = black_box(
        br#"This\nIsA\tMixture\u0000OfStrings\fThat\"Need\\Escaping"#
    );

    b.bytes = test_string.len() as u64;
    b.iter(|| { unescape_bytes(test_string) });
}
