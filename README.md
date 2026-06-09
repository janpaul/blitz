# Blitz

A fast, lightweight in-memory key-value store built in Rust. Inspired by Redis, but not a clone — Blitz is designed for ultra-low-latency use cases like game state on a LAN, with both TCP and UDP support (UDP coming soon).

## Features

- In-memory key-value storage
- TCP server with simple text-based protocol
- Lightweight and fast — no external dependencies
- UDP support *(planned)*
- Key expiration *(planned)*
- Lists and sets *(planned)*

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

| Command | Description |
|---|---|
| `SET <key> <value>` | Store a value under a key |
| `GET <key>` | Retrieve the value for a key |
| `DEL <key>` | Delete a key and return its value |
| `LIST` | List all keys currently in storage |
| `CLEAR` | Remove all keys from storage |
| `QUIT` | Close the connection |

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
```

## Responses

| Response | Meaning |
|--------|---|
| `OK`  | Command succeeded |
| `BYE`  | Connection closing (after QUIT) |
| `NIL`  | Key not found |
| `NOK`  | Unknown or malformed command |

## Project Status

Blitz is a work in progress, built as a Rust learning project. Expect breaking changes.

## License

MIT
