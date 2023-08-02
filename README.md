# RTPeeker

_Work in progress_

## How to setup

Install Trunk
```console
cargo install --locked trunk
```

1) With on-change reloading in the client

Run the server
```console
cargo run
```

Run Trunk development server
```console
cd client
trunk serve serve
```

Trunk server will proxy WebSocket connections to the main backend.

2) As a single app

Compile client to WASM
```console
cd client
trunk build --release
```

Run the server
```console
cd ..
cargo run
```
