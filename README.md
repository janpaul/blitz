# Blitz

A fast, lightweight in-memory key-value store built in Rust. Inspired by Redis, but not a clone — Blitz is designed for
ultra-low-latency use cases like game state on a LAN, with both TCP and UDP support.

## Features

- In-memory key-value storage
- TCP server with simple text-based protocol
- Lightweight and fast — no external dependencies
- UDP support
- Key expiration
- Lists (`PUSHR`, `POPR`, ...)
- sets *(planned)*

## Getting Started

```bash
git clone https://github.com/yourname/blitz
cd blitz
cargo run
```

By default, Blitz listens on `127.0.0.1:6379`.

## Connecting

Use any TCP client, for example `nc`:

```bash
nc 127.0.0.1 6379
```

## Commands

| Command                             | Description                                      |
|-------------------------------------|--------------------------------------------------|
| `SET <key> <value>`                 | Store a value under a key                        |
| `GET <key>`                         | Retrieve the value for a key                     |
| `DEL <key>`                         | Delete a key and return its value                |
| `EXISTS <key>`                      | Returns `1` if a key exists, otherwise `0`       |
| `TYPE <key>`                        | Returns the type of the key (string,number,list) |
| `RENAME <old> <new>`, `REN`, `MOVE` | Renames key `old` to `new`                       |
| `INCR <key>`                        | Increments the value of the key                  |
| `DECR <key>`                        | Decrements the value of the key                  |
| `ADD <key> <number>`                | Adds `number` to the value of the key            |
| `SUB <key>`                         | Substracts `number` from the value of the key    |
| `EXPIRE <key> <seconds>`, `EXP`     | The `key` expires in `seconds`                   |
| `TTL <key>`                         | Gets the expiration of the key, in seconds       |
| `MGET <k1> <kn>`                    | Get the values of the specified keys             |
| `MSET <k1> <v1> <kn> <vn>`          | Set the values of the specified keys             |
| `PUSHR <list> <value>`              | Push `value` to the right side of the `list`     |
| `PUSHL <list> <value>`              | Push `value` to the left side of the `list`      |
| `POPR <list>`                       | Gets the value of the right side of the list     |
| `POPL <list>`                       | Gets the value of the left side of the list      |
| `LLEN <list>`                       | Gets the number of items in the list             |
| `LRANGE <list> <start> <stop>`      | Gets the items in the list in specified range    |
| `LIST`, `LS`, `KEYS`                | List all keys currently in storage               |
| `PING`                              | Health check, returns `PONG`                     |
| `CLEAR`                             | Remove all keys from storage                     |
| `QUIT`, `BYE`                       | Close the connection                             |

### Examples

```
SET player1.pos 10,20
+OK

GET player1.pos
10,20

DEL player1.pos
10,20

LIST
player2.pos
player3.pos

CLEAR
+OK

QUIT
+BYE

LRANGE player1.inventory 0 -1
```

## Responses

| Response | Meaning                         |
|----------|---------------------------------|
| `OK`     | Command succeeded               |
| `BYE`    | Connection closing (after QUIT) |
| `NIL`    | Key not found                   |
| `NOK`    | Unknown or malformed command    |

## UDP Datagram Protocol

Blitz exposes a binary UDP interface on port 6380, designed for ultra-low-latency use cases like game state on a LAN.
Unlike the TCP interface, there is no connection overhead — just fire a packet and get a response. This comes with some
limitations: no pipelining, no transactions, and a much simpler command set (only GET and SET). Also, there is a hard
limit to the length of the key and value (255 bytes each) to fit within a single UDP datagram.

### Packet Format

#### Request

| Byte(s) | Field        | Description                             |
|---------|--------------|-----------------------------------------|
| 0       | Command      | 0x01 = GET, 0x02 = SET                  |
| 1       | Key length   | Length of the key in bytes              |
| 2..n    | Key          | UTF-8 key                               |
| n+1     | Value length | Length of the value in bytes (SET only) |
| n+2..   | Value        | UTF-8 value (SET only)                  |

#### Response

| Byte(s) | Field        | Description                       |
|---------|--------------|-----------------------------------|
| 0       | Status       | 0x00 = OK, 0x01 = NIL, 0x02 = NOK |
| 1       | Value length | Length of the value in bytes      |
| 1..n    | Value        | UTF-8 value (GET only)            |

### Example (Python)

```python
import socket

s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
addr = ("127.0.0.1", 6380)

# SET player1.pos = "10,20"
key = b"player1.pos"
value = b"10,20"
packet = bytes([0x02, len(key)]) + key + bytes([len(value)]) + value
s.sendto(packet, addr)
print(s.recvfrom(256))  # (b'\x00', ...)

# GET player1.pos
packet = bytes([0x01, len(key)]) + key
s.sendto(packet, addr)
data, _ = s.recvfrom(256)
print(data[2:].decode())  # 10,20
````

### Notes

- Maximum key size: 255 bytes
- Maximum value size: 255 bytes
- TCP and UDP share the same in-memory storage — a value written via UDP is immediately readable via TCP and vice versa.

## Project Status

Blitz is a work in progress, built as a Rust learning project. Expect breaking changes.

## License

MIT
