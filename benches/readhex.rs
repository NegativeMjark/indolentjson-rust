#![feature(test)]

extern crate indolentjson;
extern crate test;

use indolentjson::readhex::*;
use test::{black_box, Bencher};

#[bench]
fn read_hexdigit(b: &mut Bencher) {
    let x = black_box(b"0123");
    b.bytes = x.len() as u64;
    b.iter(|| { read_hexdigits(x[0], x[1], x[2], x[3]) });
}
