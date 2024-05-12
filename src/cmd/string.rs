use crate::{
    cmd::{CommandError, StringGet},
    RespArray, RespFrame, RespNull,
};

use super::{CommandExecutor, extract_args, RESP_OK, StringSet, validate_command};

impl CommandExecutor for StringGet {
    fn execute(self, backend: &crate::Backend) -> RespFrame {
        backend.string_get(&self.key).unwrap_or_else(|| RespFrame::Null(RespNull))
    }
}

impl TryFrom<RespArray> for StringGet {
    type Error = CommandError;
    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        validate_command(&value, &["GET"], Some(1))?;

        let mut args = extract_args(value, 1)?.into_iter();
        match args.next() {
            Some(RespFrame::BulkString(key)) => Ok(StringGet {
                key: key.try_into()?,
            }),
            _ => Err(CommandError::InvalidArgument("Invalid key".to_string())),
        }
    }
}

impl CommandExecutor for StringSet {
    fn execute(self, backend: &crate::Backend) -> RespFrame {
        backend.string_set(self.key, self.value);
        RESP_OK.clone()
    }
}

impl TryFrom<RespArray> for StringSet {
    type Error = CommandError;
    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        validate_command(&value, &["SET"], Some(2))?;

        let mut args = extract_args(value, 1)?.into_iter();
        match (args.next(), args.next()) {
            (Some(RespFrame::BulkString(key)), Some(value)) => Ok(StringSet {
                key: key.try_into()?,
                value,
            }),
            _ => Err(CommandError::InvalidArgument(
                "Invalid key or value".to_string(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use bytes::BytesMut;

    use crate::{Backend, RespArray, RespDecode, RespFrame};

    use super::{CommandExecutor, RESP_OK, StringGet, StringSet};

    #[test]
    fn test_get_from_resp_array() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*2\r\n$3\r\nget\r\n$5\r\nhello\r\n");

        let frame = RespArray::decode(&mut buf)?;

        let result: StringGet = frame.try_into()?;
        assert_eq!(result.key, "hello");

        Ok(())
    }

    #[test]
    fn test_set_from_resp_array() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*3\r\n$3\r\nset\r\n$5\r\nhello\r\n$5\r\nworld\r\n");

        let frame = RespArray::decode(&mut buf)?;

        let result: StringSet = frame.try_into()?;
        assert_eq!(result.key, "hello");
        assert_eq!(result.value, RespFrame::BulkString(b"world".into()));

        Ok(())
    }

    #[test]
    fn test_set_get_command() -> Result<()> {
        let backend = Backend::new();
        let cmd = StringSet {
            key: "hello".to_string(),
            value: RespFrame::BulkString(b"world".into()),
        };
        let result = cmd.execute(&backend);
        assert_eq!(result, RESP_OK.clone());

        let cmd = StringGet {
            key: "hello".to_string(),
        };
        let result = cmd.execute(&backend);
        assert_eq!(result, RespFrame::BulkString(b"world".into()));

        Ok(())
    }
}
