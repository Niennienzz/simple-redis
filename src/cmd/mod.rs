use enum_dispatch::enum_dispatch;
use lazy_static::lazy_static;
use thiserror::Error;

use crate::{Backend, RespArray, RespError, RespFrame, SimpleString};

mod echo;
mod hmap;
mod set;
mod string;

lazy_static! {
    static ref RESP_OK: RespFrame = SimpleString::new("OK").into();
    static ref RESP_UNSUPPORTED: RespFrame = SimpleString::new("UNSUPPORTED").into();
}

#[derive(Error, Debug)]
pub enum CommandError {
    #[error("Invalid command: {0}")]
    InvalidCommand(String),
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    #[error("{0}")]
    RespError(#[from] RespError),
    #[error("Utf8 error: {0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),
}

#[enum_dispatch]
pub trait CommandExecutor {
    fn execute(self, backend: &Backend) -> RespFrame;
}

#[enum_dispatch(CommandExecutor)]
#[derive(Debug)]
pub enum Command {
    Echo(Echo),
    StringGet(StringGet),
    StringSet(StringSet),
    HashGet(HashGet),
    HashSet(HashSet),
    HashGetAll(HashGetAll),
    HashMultiGet(HashMultiGet),
    SetAdd(SetAdd),
    SetIsMember(SetIsMember),
    SetMembers(SetMembers),

    Unsupported(Unsupported),
}

#[derive(Debug)]
pub struct Echo {
    message: String,
}

#[derive(Debug)]
pub struct StringGet {
    key: String,
}

#[derive(Debug)]
pub struct StringSet {
    key: String,
    value: RespFrame,
}

#[derive(Debug)]
pub struct HashGet {
    key: String,
    field: String,
}

#[derive(Debug)]
pub struct HashSet {
    key: String,
    field: String,
    value: RespFrame,
}

#[derive(Debug)]
pub struct HashGetAll {
    key: String,
    sort: bool,
}

#[derive(Debug)]
pub struct HashMultiGet {
    key: String,
    fields: Vec<String>,
}

#[derive(Debug)]
pub struct SetAdd {
    key: String,
    members: Vec<String>,
}

#[derive(Debug)]
pub struct SetIsMember {
    key: String,
    member: String,
}

#[derive(Debug)]
pub struct SetMembers {
    key: String,
}

#[derive(Debug)]
pub struct Unsupported;

impl TryFrom<RespFrame> for Command {
    type Error = CommandError;
    fn try_from(v: RespFrame) -> Result<Self, Self::Error> {
        match v {
            RespFrame::Array(array) => array.try_into(),
            _ => Err(CommandError::InvalidCommand(
                "Command must be an Array".to_string(),
            )),
        }
    }
}

impl TryFrom<RespArray> for Command {
    type Error = CommandError;
    fn try_from(v: RespArray) -> Result<Self, Self::Error> {
        match v.first() {
            Some(RespFrame::BulkString(ref cmd)) => match cmd.to_ascii_uppercase().as_ref() {
                b"ECHO" => Ok(Echo::try_from(v)?.into()),
                b"GET" => Ok(StringGet::try_from(v)?.into()),
                b"SET" => Ok(StringSet::try_from(v)?.into()),
                b"HGET" => Ok(HashGet::try_from(v)?.into()),
                b"HSET" => Ok(HashSet::try_from(v)?.into()),
                b"HGETALL" => Ok(HashGetAll::try_from(v)?.into()),
                b"HMGET" => Ok(HashMultiGet::try_from(v)?.into()),
                b"SADD" => Ok(SetAdd::try_from(v)?.into()),
                b"SISMEMBER" => Ok(SetIsMember::try_from(v)?.into()),
                b"SMEMBERS" => Ok(SetMembers::try_from(v)?.into()),
                _ => Ok(Unsupported.into()),
            },
            _ => Err(CommandError::InvalidCommand(
                "Command must have a BulkString as the first argument".to_string(),
            )),
        }
    }
}

impl CommandExecutor for Unsupported {
    fn execute(self, _: &Backend) -> RespFrame {
        RESP_UNSUPPORTED.clone()
    }
}

// If n_args is None, then we do not check the number of arguments.
fn validate_command(
    value: &RespArray,
    names: &[&'static str],
    n_args: Option<usize>,
) -> Result<(), CommandError> {
    if let Some(n_args) = n_args {
        if value.len() != n_args + names.len() {
            return Err(CommandError::InvalidArgument(format!(
                "{} command must have exactly {} argument",
                names.join(" "),
                n_args
            )));
        }
    }

    for (i, name) in names.iter().enumerate() {
        match value[i] {
            RespFrame::BulkString(ref cmd) => {
                if cmd.to_ascii_uppercase().as_ref() != name.as_bytes() {
                    return Err(CommandError::InvalidCommand(format!(
                        "Invalid command: expected {}, got {}",
                        name,
                        String::from_utf8_lossy(cmd.as_ref())
                    )));
                }
            }
            _ => {
                return Err(CommandError::InvalidCommand(
                    "Command must have a BulkString as the first argument".to_string(),
                ));
            }
        }
    }
    Ok(())
}

fn extract_args(value: RespArray, start: usize) -> Result<Vec<RespFrame>, CommandError> {
    let mut args = Vec::new();
    for arg in value.into_iter().skip(start) {
        args.push(arg);
    }
    Ok(args)
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use bytes::BytesMut;

    use crate::{RespArray, RespDecode, RespFrame, RespNull};

    use super::{Backend, Command, CommandExecutor};

    #[test]
    fn test_command() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*2\r\n$3\r\nget\r\n$5\r\nhello\r\n");

        let frame = RespArray::decode(&mut buf)?;

        let cmd: Command = frame.try_into()?;

        let backend = Backend::new();

        let ret = cmd.execute(&backend);
        assert_eq!(ret, RespFrame::Null(RespNull));

        Ok(())
    }
}
