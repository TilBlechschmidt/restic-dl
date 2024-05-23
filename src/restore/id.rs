use rand::{distributions::Alphanumeric, thread_rng, Rng};
use serde::{Deserialize, Serialize};
use std::{fmt, io::Read, path::Path, str::FromStr};
use thiserror::Error;

const BYTE_COUNT: usize = 32;
const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                         abcdefghijklmnopqrstuvwxyz\
                         0123456789";

#[derive(Debug, Error, PartialEq, Eq)]
pub enum RestoreIdParseError {
    #[error("expected `{BYTE_COUNT}` characters, got `{0}`")]
    InvalidLength(usize),

    #[error("encountered invalid character `{0}`")]
    InvalidCharacter(char),
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize)]
#[serde(into = "String", try_from = "&str")]
pub struct RestoreId([u8; BYTE_COUNT]);

impl RestoreId {
    pub fn new() -> Self {
        let mut rng = thread_rng();
        Self([0; BYTE_COUNT].map(|_| rng.sample(Alphanumeric)))
    }

    fn to_str(&self) -> &str {
        std::str::from_utf8(&self.0).expect("RestoreId to be valid UTF-8")
    }
}

impl AsRef<str> for RestoreId {
    fn as_ref(&self) -> &str {
        self.to_str()
    }
}

impl AsRef<Path> for RestoreId {
    fn as_ref(&self) -> &Path {
        self.to_str().as_ref()
    }
}

impl fmt::Display for RestoreId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_ref())
    }
}

impl From<RestoreId> for String {
    fn from(id: RestoreId) -> Self {
        id.to_string()
    }
}

impl FromStr for RestoreId {
    type Err = RestoreIdParseError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let mut bytes = input.as_bytes();

        if bytes.len() != BYTE_COUNT {
            return Err(RestoreIdParseError::InvalidLength(bytes.len()));
        }

        let invalid_char = bytes.iter().find(|c| !CHARSET.contains(c));

        if let Some(c) = invalid_char {
            return Err(RestoreIdParseError::InvalidCharacter(*c as char));
        }

        let mut id = [0; BYTE_COUNT];

        bytes
            .read_exact(&mut id)
            .expect("id length check to short-circuit");

        Ok(Self(id))
    }
}

impl TryFrom<&str> for RestoreId {
    type Error = RestoreIdParseError;

    fn try_from(input: &str) -> Result<Self, Self::Error> {
        input.parse()
    }
}

#[cfg(test)]
mod does {
    use super::{RestoreIdParseError::*, *};

    #[test]
    fn survive_a_roundtrip() {
        let id = RestoreId::new();
        let sid: &str = id.as_ref();
        let rid: RestoreId = sid.parse().unwrap();
        assert_eq!(id, rid);
    }

    #[test]
    fn parse_valid_ids() {
        assert!("HeyThisIsATestWeWillSeeIfItWorks"
            .parse::<RestoreId>()
            .is_ok());
    }

    #[test]
    fn reject_short_ids() {
        assert_eq!("Hi".parse::<RestoreId>(), Err(InvalidLength(2)));
    }

    #[test]
    fn reject_long_ids() {
        assert_eq!(
            "0123456789012345678901234567890123456789".parse::<RestoreId>(),
            Err(InvalidLength(40))
        );
    }

    #[test]
    fn reject_invalid_chars() {
        assert_eq!(
            "0123456789012345678901234567890-".parse::<RestoreId>(),
            Err(InvalidCharacter('-'))
        );
    }

    #[test]
    fn serialize_to_string() {
        let id = RestoreId::new();
        let sid = serde_json::to_string(&id).unwrap();
        assert_eq!(format!("\"{id}\""), sid);
    }

    #[test]
    fn deserialize_from_string() {
        serde_json::from_str::<RestoreId>("\"HeyThisIsATestWeWillSeeIfItWorks\"").unwrap();
    }
}
