use bincode::{Decode, Encode};

#[derive(Encode, Decode, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version(u16);

impl Version {
    pub const fn from_raw(v: u16) -> Self {
        Self(v)
    }

    pub const fn new(major: u16, minor: u16) -> Self {
        assert!(major < (1 << 10));
        assert!(minor < (1 << 6));
        Self((major << 6) | minor)
    }

    pub const fn into_raw(self) -> u16 {
        self.0
    }

    pub const fn major(self) -> u16 {
        self.0 >> 6
    }

    pub const fn minor(self) -> u16 {
        self.0 & 0x3F
    }
}

impl core::fmt::Debug for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Version")
            .field("major", &self.major())
            .field("minor", &self.minor())
            .finish()
    }
}

impl core::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}.{}", self.major(), self.minor()))
    }
}

const fn const_parse_u16(v: &str, radix: u32) -> u16 {
    if radix < 2 || radix > 36 {
        panic!("Invalid radix")
    }

    let radix = radix as u16;

    let mut val = 0u16;

    let b = v.as_bytes();

    let mut i = 0;

    while i < b.len() {
        let b = b[i];

        let d = match b {
            b'0'..=b'9' => b - b'0',
            b'A'..=b'Z' => (b - b'A') + 10,
            b'a'..=b'z' => (b - b'a') + 10,
            _ => panic!("Expected a digit"),
        } as u16;

        if d < radix {
            let v = match val.checked_mul(radix) {
                Some(v) => v.checked_add(d),
                None => None,
            };

            match v {
                Some(v) => val = v,
                None => panic!("Out of Range value"),
            }
        } else {
            panic!("Out of Range digit")
        }
        i += 1;
    }
    val
}

const VERSION_MAJOR: u16 = const_parse_u16(core::env!("CARGO_PKG_VERSION_MAJOR"), 10);
const VERSION_MINOR: u16 = const_parse_u16(core::env!("CARGO_PKG_VERSION_MINOR"), 10);

pub const CURRENT: Version = Version::new(VERSION_MAJOR, VERSION_MINOR);
