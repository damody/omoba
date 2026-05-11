# omb-ws-bridge

`omb-ws-bridge` lets the Web/WASM client talk to the existing native KCP backend. The browser connects over WebSocket; the bridge connects to `omb` over KCP and forwards the same framed protobuf bytes.

## Run

From repo root:

```powershell
cargo run --manifest-path omb-ws-bridge/Cargo.toml -- 127.0.0.1:50062 127.0.0.1:50061
```

Arguments:

- first: WebSocket bind address, default `127.0.0.1:50062`
- second: KCP backend address, default `127.0.0.1:50061`

Environment fallbacks:

- `OMBOBA_WS_ADDR`
- `OMB_KCP_ADDR`

## Frame Format

Each WebSocket binary message is one complete OMOBA frame:

```text
[1 byte tag][4 byte big-endian payload length][payload bytes]
```

Browser-to-bridge frames are validated and forwarded to KCP unchanged. KCP-to-browser frames are read from the KCP stream; if a KCP frame uses the compression bit (`0x80`), the bridge decompresses it and sends the browser an uncompressed frame with the base tag.

## Tag Mapping

- `0x01` `PlayerCommand`: browser to backend
- `0x02` `GameEvent`: backend to browser
- `0x03` `CommandAck`: backend to browser
- `0x04` `SubscribeRequest`: browser to backend
- `0x05` `GameStateRequest`: browser to backend
- `0x06` `GameStateResponse`: backend to browser
- `0x07` `ViewportUpdate`: browser to backend
- `0x10` `InputSubmit`: browser to backend
- `0x11` `TickBatch`: backend to browser
- `0x12` `StateHash`: backend to browser
- `0x13` `JoinRequest`: browser to backend
- `0x14` `GameStart`: backend to browser
- `0x15` `SnapshotReq`: browser to backend
- `0x16` `SnapshotResp`: backend to browser
- `0x17` `PingRequest`: browser to backend
- `0x18` `PingResponse`: backend to browser
