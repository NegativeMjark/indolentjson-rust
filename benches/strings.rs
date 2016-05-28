#![feature(test)]

extern crate indolentjson;
extern crate test;

use indolentjson::strings::*;
use test::Bencher;


#[bench]
fn noop_escape(b: &mut Bencher) {
    b.iter(|| {
        escape_bytes(b"test")
    });
}

#[bench]
fn escape_controls(b: &mut Bencher) {
    b.iter(|| {
        escape_bytes(b"\t\n\r\\")
    });
}

#[bench]
fn noop_unescape(b: &mut Bencher) {
    b.iter(|| {
        unescape_bytes(b"test")
    });
}

#[bench]
fn unescape_controls(b: &mut Bencher) {
    b.iter(|| {
        unescape_bytes(br#"\t\n\r\\"#)
    });
}
