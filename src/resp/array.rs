use std::iter::IntoIterator;
use std::ops::Deref;

use bytes::{Buf, BytesMut};
use lazy_static::lazy_static;

use crate::{RespDecode, RespEncode, RespError, RespFrame};

use super::{BUF_CAP, calc_total_length, CRLF_LEN, parse_length};

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum RespArray {
    Normal(Vec<RespFrame>),
    Null,
}

const EMPTY_ARRAY_BIN: &[u8] = b"*0\r\n";
const EMPTY_ARRAY_LEN: usize = EMPTY_ARRAY_BIN.len();
const NULL_ARRAY_BIN: &[u8] = b"*-1\r\n";
const NULL_ARRAY_LEN: usize = NULL_ARRAY_BIN.len();

// - Normal array: "*<number-of-elements>\r\n<element-1>...<element-n>"
// - Null array: "*-1\r\n"
impl RespEncode for RespArray {
    fn encode(self) -> Vec<u8> {
        match self {
            RespArray::Normal(frames) => {
                let mut buf = Vec::with_capacity(BUF_CAP);
                buf.extend_from_slice(&format!("*{}\r\n", frames.len()).into_bytes());
                for frame in frames {
                    buf.extend_from_slice(&frame.encode());
                }
                buf
            }
            RespArray::Null => NULL_ARRAY_BIN.to_vec(),
        }
    }
}

impl RespDecode for RespArray {
    const PREFIX: &'static str = "*";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        // Empty array is the minimum encoding of an array.
        if buf.len() < EMPTY_ARRAY_LEN {
            return Err(RespError::NotComplete);
        }

        // Check if the frame is a null array.
        if buf.len() == NULL_ARRAY_LEN && buf.starts_with(NULL_ARRAY_BIN) {
            buf.advance(NULL_ARRAY_LEN);
            return Ok(RespArray::Null);
        }

        // Decode the normal array.
        let (end, len) = parse_length(buf, Self::PREFIX)?;
        let total_len = calc_total_length(buf, end, len, Self::PREFIX)?;

        if buf.len() < total_len {
            return Err(RespError::NotComplete);
        }
        buf.advance(end + CRLF_LEN);
        let mut frames = Vec::with_capacity(len);
        for _ in 0..len {
            frames.push(RespFrame::decode(buf)?);
        }

        Ok(RespArray::new(frames))
    }

    fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
        let (end, len) = parse_length(buf, Self::PREFIX)?;
        calc_total_length(buf, end, len, Self::PREFIX)
    }
}

impl IntoIterator for RespArray {
    type Item = RespFrame;
    type IntoIter = std::vec::IntoIter<RespFrame>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            RespArray::Normal(frames) => frames.into_iter(),
            RespArray::Null => Vec::new().into_iter(),
        }
    }
}

impl RespArray {
    // Empty array is different from null array.
    // Caller should explicitly use RespArray::Null to create a null array.
    pub fn new(s: impl Into<Vec<RespFrame>>) -> Self {
        RespArray::Normal(s.into())
    }

    pub fn empty() -> Self {
        RespArray::new(Vec::new())
    }
}

lazy_static! {
    static ref NULL_ARRAY_FRAME: Vec<RespFrame> = Vec::new();
}

impl Deref for RespArray {
    type Target = Vec<RespFrame>;

    fn deref(&self) -> &Self::Target {
        match self {
            RespArray::Normal(frames) => frames,
            RespArray::Null => NULL_ARRAY_FRAME.deref(),
        }
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use crate::BulkString;

    use super::*;

    #[test]
    fn test_array_encode() {
        let frame: RespFrame = RespArray::new(vec![
            BulkString::new("set".to_string()).into(),
            BulkString::new("hello".to_string()).into(),
            BulkString::new("world".to_string()).into(),
        ])
            .into();
        assert_eq!(
            &frame.encode(),
            b"*3\r\n$3\r\nset\r\n$5\r\nhello\r\n$5\r\nworld\r\n"
        );
    }

    #[test]
    fn test_array_encode_empty() {
        let frame: RespFrame = RespArray::new(vec![]).into();
        assert_eq!(&frame.encode(), EMPTY_ARRAY_BIN);
    }

    #[test]
    fn test_array_encode_null() {
        let frame: RespFrame = RespArray::Null.into();
        assert_eq!(frame.encode(), NULL_ARRAY_BIN);
    }

    #[test]
    fn test_array_decode() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*2\r\n$3\r\nset\r\n$5\r\nhello\r\n");

        let frame = RespArray::decode(&mut buf)?;
        assert_eq!(frame, RespArray::new([b"set".into(), b"hello".into()]));

        buf.extend_from_slice(b"*2\r\n$3\r\nset\r\n");
        let ret = RespArray::decode(&mut buf);
        assert_eq!(ret.unwrap_err(), RespError::NotComplete);

        buf.extend_from_slice(b"$5\r\nhello\r\n");
        let frame = RespArray::decode(&mut buf)?;
        assert_eq!(frame, RespArray::new([b"set".into(), b"hello".into()]));

        Ok(())
    }

    #[test]
    fn test_array_decode_empty() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(EMPTY_ARRAY_BIN);

        let frame = RespArray::decode(&mut buf)?;
        assert_eq!(frame, RespArray::new(vec![]));

        Ok(())
    }

    #[test]
    fn test_array_decode_null() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(NULL_ARRAY_BIN);

        let frame = RespArray::decode(&mut buf)?;
        assert_eq!(frame, RespArray::Null);

        Ok(())
    }
}
