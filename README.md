# GeekTime Rust Camp Assignment #02

## Requirements

- The original implementation of the course can be found [here](https://github.com/tyr-rust-bootcamp/02-simple-redis).

### Support More Commands

- [ECHO](https://redis.io/docs/latest/commands/echo/)
- [HMGET](https://redis.io/docs/latest/commands/hmget/)
- [SADD](https://redis.io/docs/latest/commands/sadd/) & [SISMEMBER](https://redis.io/docs/latest/commands/sismember/)

### Refactoring

- Delete `NullBulkString` and `NullArray`.
- Refactor `BulkString` and `RespArray` so that they can handle the two NULL types above.
