use crate::{
    cmd::CommandError,
    RespArray, RespFrame, RespNullArray,
};

use super::{
    CommandExecutor, extract_args,
    SetAdd, SetIsMember, SetMembers, validate_command,
};

impl CommandExecutor for SetAdd {
    fn execute(self, backend: &crate::Backend) -> RespFrame {
        let ret = backend.set_add(self.key, self.members);
        RespFrame::Integer(ret)
    }
}

impl TryFrom<RespArray> for SetAdd {
    type Error = CommandError;
    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        validate_command(&value, &["sadd"], None)?;

        // Parse the key.
        let mut args = extract_args(value, 1)?.into_iter();
        let key = match args.next() {
            Some(RespFrame::BulkString(key)) => key.try_into()?,
            _ => return Err(CommandError::InvalidArgument("Invalid key".to_string())),
        };

        // Parse the members.
        let mut members = Vec::new();
        for arg in args {
            match arg {
                RespFrame::BulkString(member) => members.push(member.try_into()?),
                _ => return Err(CommandError::InvalidArgument("Invalid member".to_string())),
            }
        }

        Ok(SetAdd { key, members })
    }
}

impl CommandExecutor for SetIsMember {
    fn execute(self, backend: &crate::Backend) -> RespFrame {
        let ret = backend.set_is_member(&self.key, &self.member);
        RespFrame::Integer(ret)
    }
}

impl TryFrom<RespArray> for SetIsMember {
    type Error = CommandError;
    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        validate_command(&value, &["sismember"], None)?;

        let mut args = extract_args(value, 1)?.into_iter();
        match (args.next(), args.next()) {
            (Some(RespFrame::BulkString(key)), Some(RespFrame::BulkString(member))) => Ok(SetIsMember {
                key: key.try_into()?,
                member: member.try_into()?,
            }),
            _ => Err(CommandError::InvalidArgument(
                "Invalid key or member".to_string(),
            )),
        }
    }
}

impl CommandExecutor for SetMembers {
    fn execute(self, backend: &crate::Backend) -> RespFrame {
        let set = backend.set.get(&self.key);
        match set {
            Some(set) => {
                let members = set.iter().map(|v| {
                    RespFrame::BulkString(crate::resp::BulkString::new(v.clone()))
                }).collect();
                RespFrame::Array(RespArray(members))
            }
            None => RespFrame::NullArray(RespNullArray),
        }
    }
}

impl TryFrom<RespArray> for SetMembers {
    type Error = CommandError;
    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        validate_command(&value, &["smembers"], Some(1))?;

        let mut args = extract_args(value, 1)?.into_iter();
        match args.next() {
            Some(RespFrame::BulkString(key)) => Ok(SetMembers {
                key: key.try_into()?,
            }),
            _ => Err(CommandError::InvalidArgument("Invalid key".to_string())),
        }
    }
}
