use std::ops::Deref;

use bytes::{Buf, BytesMut};
use lazy_static::lazy_static;

use crate::{RespDecode, RespEncode, RespError};

use super::{parse_length, CRLF_LEN};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub enum BulkString {
    Normal(Vec<u8>),
    Null,
}

#[allow(dead_code)]
const EMPTY_BULK_STRING_BIN: &[u8] = b"$0\r\n\r\n";
const NULL_BULK_STRING_BIN: &[u8] = b"$-1\r\n";
const NULL_BULK_STRING_LEN: usize = NULL_BULK_STRING_BIN.len();

lazy_static! {
    static ref NULL_BULK_STRING_VEC: Vec<u8> = Vec::from(NULL_BULK_STRING_BIN);
}

// - Normal bulk string: "$<length>\r\n<data>\r\n"
// - Null bulk string: "$-1\r\n"
impl RespEncode for BulkString {
    fn encode(self) -> Vec<u8> {
        match self {
            BulkString::Normal(data) => {
                let mut buf = Vec::with_capacity(data.len() + 16);
                buf.extend_from_slice(&format!("${}\r\n", data.len()).into_bytes());
                buf.extend_from_slice(&data);
                buf.extend_from_slice(b"\r\n");
                buf
            }
            BulkString::Null => NULL_BULK_STRING_BIN.to_vec(),
        }
    }
}

impl RespDecode for BulkString {
    const PREFIX: &'static str = "$";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        // Null bulk string is the minimum encoding of a bulk string.
        if buf.len() < NULL_BULK_STRING_LEN {
            return Err(RespError::NotComplete);
        }

        // Check if the frame is a null bulk string.
        if buf.len() == NULL_BULK_STRING_LEN && buf.starts_with(NULL_BULK_STRING_BIN) {
            buf.advance(NULL_BULK_STRING_LEN);
            return Ok(BulkString::Null);
        }

        // Decode the normal bulk string.
        let (end, len) = parse_length(buf, Self::PREFIX)?;
        let remained = &buf[end + CRLF_LEN..];
        if remained.len() < len + CRLF_LEN {
            return Err(RespError::NotComplete);
        }
        buf.advance(end + CRLF_LEN);
        let data = buf.split_to(len + CRLF_LEN);
        Ok(BulkString::Normal(data[..len].to_vec()))
    }

    fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
        let (end, len) = parse_length(buf, Self::PREFIX)?;
        Ok(end + CRLF_LEN + len + CRLF_LEN)
    }
}

impl BulkString {
    pub fn new(s: impl Into<Vec<u8>>) -> Self {
        let s = s.into();
        if s == NULL_BULK_STRING_BIN {
            Self::Null
        } else {
            BulkString::Normal(s)
        }
    }

    pub fn to_ascii_uppercase(&self) -> Self {
        match self {
            BulkString::Normal(data) => BulkString::Normal(data.to_ascii_uppercase()),
            BulkString::Null => BulkString::Null,
        }
    }
}

impl TryInto<String> for BulkString {
    type Error = std::string::FromUtf8Error;
    fn try_into(self) -> Result<String, Self::Error> {
        match self {
            BulkString::Normal(data) => String::from_utf8(data),
            BulkString::Null => String::from_utf8(NULL_BULK_STRING_BIN.to_vec()),
        }
    }
}

impl AsRef<[u8]> for BulkString {
    fn as_ref(&self) -> &[u8] {
        match self {
            BulkString::Normal(data) => data.as_ref(),
            BulkString::Null => NULL_BULK_STRING_BIN,
        }
    }
}

impl Deref for BulkString {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        match self {
            BulkString::Normal(data) => data,
            BulkString::Null => NULL_BULK_STRING_VEC.as_ref(),
        }
    }
}

impl<T> From<T> for BulkString
where
    T: Into<Vec<u8>>,
{
    fn from(s: T) -> Self {
        BulkString::new(s)
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use crate::RespFrame;

    use super::*;

    #[test]
    fn test_bulk_string_encode() {
        let frame: RespFrame = BulkString::new(b"hello".to_vec()).into();
        assert_eq!(frame.encode(), b"$5\r\nhello\r\n");
    }

    #[test]
    fn test_bulk_string_encode_empty() {
        let frame: RespFrame = BulkString::new(b"".to_vec()).into();
        assert_eq!(frame.encode(), EMPTY_BULK_STRING_BIN);
    }

    #[test]
    fn test_bulk_string_encode_null() {
        let frame: RespFrame = BulkString::Null.into();
        assert_eq!(frame.encode(), NULL_BULK_STRING_BIN);
    }

    #[test]
    fn test_bulk_string_decode() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"$5\r\nhello\r\n");

        let frame = BulkString::decode(&mut buf)?;
        assert_eq!(frame, BulkString::new(b"hello"));

        buf.extend_from_slice(b"$5\r\nhello");
        let ret = BulkString::decode(&mut buf);
        assert_eq!(ret.unwrap_err(), RespError::NotComplete);

        buf.extend_from_slice(b"\r\n");
        let frame = BulkString::decode(&mut buf)?;
        assert_eq!(frame, BulkString::new(b"hello"));

        Ok(())
    }

    #[test]
    fn test_bulk_string_decode_empty() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(EMPTY_BULK_STRING_BIN);

        let frame = BulkString::decode(&mut buf)?;
        assert_eq!(frame, BulkString::new(b""));

        Ok(())
    }

    #[test]
    fn test_bulk_string_decode_null() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(NULL_BULK_STRING_BIN);

        let frame = BulkString::decode(&mut buf)?;
        assert_eq!(frame, BulkString::Null);

        Ok(())
    }
}
