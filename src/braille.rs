#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pattern(u8);

impl Pattern {
    pub const EMPTY: Self = Self(0);

    pub const UTF8_LEN: usize = 3;

    pub const fn from_offset(offset: u8) -> Self {
        Self(offset)
    }

    pub const fn offset(self) -> u8 {
        self.0
    }

    pub const fn with(self, x: u32, y: u32) -> Self {
        Self::from_offset(self.offset() | mask(x, y))
    }

    pub const fn as_char(self) -> char {
        match char::from_u32(0x2800 + self.offset() as u32) {
            Some(c) => c,
            None => unreachable!(),
        }
    }

    pub fn encode_utf8(self, buf: &mut [u8; Self::UTF8_LEN]) -> &mut str {
        self.as_char().encode_utf8(buf)
    }
}

const fn mask(x: u32, y: u32) -> u8 {
    0b1_u8.wrapping_shl(0b0111_0101_0100_0011_0110_0010_0001_0000_u32 >> (16 * x + 4 * y))
}

impl Display for Pattern {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.encode_utf8(&mut [0; 3]))
    }
}

#[test]
fn mask_equiv() {
    fn mask_def(x: u32, y: u32) -> u8 {
        0b1 << match (x, y) {
            (0, 0) => 0,
            (0, 1) => 1,
            (0, 2) => 2,
            (0, 3) => 6,
            (1, 0) => 3,
            (1, 1) => 4,
            (1, 2) => 5,
            (1, 3) => 7,
            (_, _) => panic!(),
        }
    }
    for x in 0..2 {
        for y in 0..4 {
            println!("Testing mask({x}, {y})");
            assert_eq!(mask(x, y), mask_def(x, y));
        }
    }
}

#[test]
fn works() {
    let mut pattern = Pattern::EMPTY;
    assert_eq!(pattern.as_char(), '⠀');
    let mut step = |x, y| {
        pattern = pattern.with(x, y);
        pattern.as_char()
    };
    assert_eq!(step(1, 0), '⠈');
    assert_eq!(step(1, 1), '⠘');
    assert_eq!(step(1, 3), '⢘');
    assert_eq!(step(0, 2), '⢜');
    assert_eq!(step(0, 1), '⢞');
}

#[test]
fn to_str_equiv() {
    for offset in 0..=255_u8 {
        let pattern = Pattern::from_offset(offset);
        assert_eq!(*pattern.encode_utf8(&mut [0; 3]), pattern.as_char().to_string());
    }
}

use std::fmt::Display;
use std::fmt;
use std::fmt::Formatter;
use std::str;
