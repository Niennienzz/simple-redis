# GeekTime Rust Camp Assignment #02

## Requirements

- The original implementation of the course can be found [here](https://github.com/tyr-rust-bootcamp/02-simple-redis).

### Support More Commands

- [ECHO](https://redis.io/docs/latest/commands/echo/)
- [HMGET](https://redis.io/docs/latest/commands/hmget/)
- [SADD](https://redis.io/docs/latest/commands/sadd/), [SISMEMBER](https://redis.io/docs/latest/commands/sismember/),
  and [SMEMBERS](https://redis.io/docs/latest/commands/smembers/)

### Refactoring

- Delete `NullBulkString` and `NullArray`.
- Refactor `BulkString` and `RespArray` so that they can handle the two NULL types above.

---

## How to Run

- Use the `cargo run` command to start the server.
- The server will listen on `6500` port.
- Then, use a Redis client (i.e., [Redis CLI](https://redis.io/docs/latest/develop/connect/cli/)) to connect.

```bash
redis-cli -p 6500
```

- Now, you can use the Redis commands mentioned above

```bash
127.0.0.1:6500> ECHO "Hello World!"
"Hello World!"
```

```bash
127.0.0.1:6500> SADD my_set 1 2 3
(integer) 3

127.0.0.1:6500> SISMEMBER my_set 1
(integer) 1

127.0.0.1:6500> SISMEMBER my_set 100
(integer) 0

127.0.0.1:6500> SMEMBERS my_set
1) "3"
2) "1"
3) "2"
```

```bash
127.0.0.1:6500> HMGET my_hash k1 k2 k3
1) (nil)
2) (nil)
3) (nil)

127.0.0.1:6500> HSET my_hash k1 v1
OK

127.0.0.1:6500> HSET my_hash k2 v2
OK

127.0.0.1:6500> HMGET my_hash k1 k2 k3
1) "v1"
2) "v2"
3) (nil)
```
