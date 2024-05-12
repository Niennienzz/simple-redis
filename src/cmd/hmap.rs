use crate::{BulkString, cmd::CommandError, RespArray, RespFrame};

use super::{CommandExecutor, extract_args, HashGet, HashGetAll, HashSet, RESP_OK, validate_command};

impl CommandExecutor for HashGet {
    fn execute(self, backend: &crate::Backend) -> RespFrame {
        backend.hash_get(&self.key, &self.field).unwrap_or_else(|| RespFrame::Null(crate::RespNull))
    }
}

impl TryFrom<RespArray> for HashGet {
    type Error = CommandError;
    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        validate_command(&value, &["hget"], Some(2))?;

        let mut args = extract_args(value, 1)?.into_iter();
        match (args.next(), args.next()) {
            (Some(RespFrame::BulkString(key)), Some(RespFrame::BulkString(field))) => Ok(HashGet {
                key: key.try_into()?,
                field: field.try_into()?,
            }),
            _ => Err(CommandError::InvalidArgument(
                "Invalid key or field".to_string(),
            )),
        }
    }
}

impl CommandExecutor for HashGetAll {
    fn execute(self, backend: &crate::Backend) -> RespFrame {
        let hmap = backend.hmap.get(&self.key);

        match hmap {
            Some(hmap) => {
                let mut data = Vec::with_capacity(hmap.len());
                for v in hmap.iter() {
                    let key = v.key().to_owned();
                    data.push((key, v.value().clone()));
                }
                if self.sort {
                    data.sort_by(|a, b| a.0.cmp(&b.0));
                }
                let ret = data
                    .into_iter()
                    .flat_map(|(k, v)| vec![BulkString::from(k).into(), v])
                    .collect::<Vec<RespFrame>>();

                RespArray::new(ret).into()
            }
            None => RespArray::new([]).into(),
        }
    }
}

impl TryFrom<RespArray> for HashGetAll {
    type Error = CommandError;
    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        validate_command(&value, &["hgetall"], Some(1))?;

        let mut args = extract_args(value, 1)?.into_iter();
        match args.next() {
            Some(RespFrame::BulkString(key)) => Ok(HashGetAll {
                key: key.try_into()?,
                sort: false,
            }),
            _ => Err(CommandError::InvalidArgument("Invalid key".to_string())),
        }
    }
}

impl CommandExecutor for HashSet {
    fn execute(self, backend: &crate::Backend) -> RespFrame {
        backend.hash_set(self.key, self.field, self.value);
        RESP_OK.clone()
    }
}

impl TryFrom<RespArray> for HashSet {
    type Error = CommandError;
    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        validate_command(&value, &["hset"], Some(3))?;

        let mut args = extract_args(value, 1)?.into_iter();
        match (args.next(), args.next(), args.next()) {
            (Some(RespFrame::BulkString(key)), Some(RespFrame::BulkString(field)), Some(value)) => {
                Ok(HashSet {
                    key: key.try_into()?,
                    field: field.try_into()?,
                    value,
                })
            }
            _ => Err(CommandError::InvalidArgument(
                "Invalid key, field or value".to_string(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use bytes::BytesMut;

    use crate::{BulkString, RespArray, RespDecode, RespFrame};

    use super::{CommandExecutor, HashGet, HashGetAll, HashSet, RESP_OK};

    #[test]
    fn test_hget_from_resp_array() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*3\r\n$4\r\nhget\r\n$3\r\nmap\r\n$5\r\nhello\r\n");

        let frame = RespArray::decode(&mut buf)?;

        let result: HashGet = frame.try_into()?;
        assert_eq!(result.key, "map");
        assert_eq!(result.field, "hello");

        Ok(())
    }

    #[test]
    fn test_hgetall_from_resp_array() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*2\r\n$7\r\nhgetall\r\n$3\r\nmap\r\n");

        let frame = RespArray::decode(&mut buf)?;

        let result: HashGetAll = frame.try_into()?;
        assert_eq!(result.key, "map");

        Ok(())
    }

    #[test]
    fn test_hset_from_resp_array() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*4\r\n$4\r\nhset\r\n$3\r\nmap\r\n$5\r\nhello\r\n$5\r\nworld\r\n");

        let frame = RespArray::decode(&mut buf)?;

        let result: HashSet = frame.try_into()?;
        assert_eq!(result.key, "map");
        assert_eq!(result.field, "hello");
        assert_eq!(result.value, RespFrame::BulkString(b"world".into()));

        Ok(())
    }

    #[test]
    fn test_hset_hget_hgetall_commands() -> Result<()> {
        let backend = crate::Backend::new();
        let cmd = HashSet {
            key: "map".to_string(),
            field: "hello".to_string(),
            value: RespFrame::BulkString(b"world".into()),
        };
        let result = cmd.execute(&backend);
        assert_eq!(result, RESP_OK.clone());

        let cmd = HashSet {
            key: "map".to_string(),
            field: "hello1".to_string(),
            value: RespFrame::BulkString(b"world1".into()),
        };
        cmd.execute(&backend);

        let cmd = HashGet {
            key: "map".to_string(),
            field: "hello".to_string(),
        };
        let result = cmd.execute(&backend);
        assert_eq!(result, RespFrame::BulkString(b"world".into()));

        let cmd = HashGetAll {
            key: "map".to_string(),
            sort: true,
        };
        let result = cmd.execute(&backend);

        let expected = RespArray::new([
            BulkString::from("hello").into(),
            BulkString::from("world").into(),
            BulkString::from("hello1").into(),
            BulkString::from("world1").into(),
        ]);
        assert_eq!(result, expected.into());
        Ok(())
    }
}
