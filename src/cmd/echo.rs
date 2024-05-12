use crate::{
    BulkString,
    cmd::CommandError, RespArray, RespFrame,
};

use super::{CommandExecutor, Echo, extract_args, validate_command};

impl CommandExecutor for Echo {
    fn execute(self, _backend: &crate::Backend) -> RespFrame {
        RespFrame::BulkString(BulkString::new(self.message))
    }
}

impl TryFrom<RespArray> for Echo {
    type Error = CommandError;
    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        validate_command(&value, &["ECHO"], Some(1))?;

        let mut args = extract_args(value, 1)?.into_iter();
        match args.next() {
            Some(RespFrame::BulkString(message)) => Ok(Echo {
                message: message.try_into()?,
            }),
            _ => Err(CommandError::InvalidArgument("Invalid message".to_string())),
        }
    }
}
