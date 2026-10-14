use std::fmt;

pub fn decode(data: &str) -> Result<Vec<u8>, Error> {
    base32::decode(base32::Alphabet::Rfc4648Lower { padding: false }, data).ok_or(Error)
}

pub fn encode(data: &[u8]) -> String {
    base32::encode(base32::Alphabet::Rfc4648Lower { padding: false }, data)
}

#[derive(Clone, Copy, Debug)]
pub struct Error;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid base32")
    }
}

impl std::error::Error for Error {}
