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

    pub fn read_hexdigit_4(input: &[u8], pos: usize) -> u32 {
        return read_hexdigits(
            input[pos], input[pos + 1], input[pos + 2], input[pos + 3]
        );
    }

    #[test]
    fn readhex_uppercase() {
        assert_eq!(0x0123, read_hexdigit_4(b"0123", 0));
        assert_eq!(0x4567, read_hexdigit_4(b"4567", 0));
        assert_eq!(0x89AB, read_hexdigit_4(b"89AB", 0));
        assert_eq!(0xCDEF, read_hexdigit_4(b"CDEF", 0));
    }

    #[test]
    fn readhex_lowercase() {
        assert_eq!(0x0123, read_hexdigit_4(b"0123", 0));
        assert_eq!(0x4567, read_hexdigit_4(b"4567", 0));
        assert_eq!(0x89AB, read_hexdigit_4(b"89ab", 0));
        assert_eq!(0xCDEF, read_hexdigit_4(b"cdef", 0));
    }
}
