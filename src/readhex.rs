pub fn read_hexdigits(h0: u8, h1: u8, h2: u8, h3: u8) -> u32 {
    // read the 4 hex digits
    let mut hex = ((h0 as u32) << 24)
            | ((h1 as u32) << 16)
            | ((h2 as u32) << 8)
            | (h3 as u32);
    // subtract '0'
    hex -= 0x30303030;
    // strip the higher bits, maps 'a' => 'A'
    hex &= 0x1F1F1F1F;
    let mask = hex & 0x10101010;
    // subtract 'A' - 10 - '9' - 9 = 7 from the letters.
    hex -= mask >> 1;
    hex += mask >> 4;
    // collect the nibbles
    hex |= hex >> 4;
    hex &= 0xFF00FF;
    hex |= hex >> 8;
    return hex & 0xFFFF;
}


#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;
    use test::black_box;

    pub fn read_hexdigit_4(input: &[u8], pos: usize) -> u32 {
        return read_hexdigits(
            input[pos], input[pos + 1], input[pos + 2], input[pos + 3]
        );
    }

    #[test]
    fn readhex_uppercase() {
        assert_eq!(0x0123, read_hexdigit_4("0123".as_bytes(), 0));
        assert_eq!(0x4567, read_hexdigit_4("4567".as_bytes(), 0));
        assert_eq!(0x89AB, read_hexdigit_4("89AB".as_bytes(), 0));
        assert_eq!(0xCDEF, read_hexdigit_4("CDEF".as_bytes(), 0));
    }

    #[test]
    fn readhex_lowercase() {
        assert_eq!(0x0123, read_hexdigit_4("0123".as_bytes(), 0));
        assert_eq!(0x4567, read_hexdigit_4("4567".as_bytes(), 0));
        assert_eq!(0x89AB, read_hexdigit_4("89ab".as_bytes(), 0));
        assert_eq!(0xCDEF, read_hexdigit_4("cdef".as_bytes(), 0));
    }

    #[bench]
    fn read_hexdigit(b: &mut Bencher) {
        let x = black_box("0123".as_bytes());
        b.bytes = x.len() as u64;
        b.iter(|| { read_hexdigit_4(x, 0) });
    }
}
