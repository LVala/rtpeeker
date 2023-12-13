# RTPeeker

_Work in progress_

## Installation

1. [Install Rust and cargo](https://www.rust-lang.org/tools/install)

2. Add WASM32 target
```console
rustup target add wasm32-unknown-unknown
```

3. Install native dependencies, for Ubuntu/Debian:
```console
sudo apt install build-essential pkg-config libssl-dev libpcap-dev
```

4. Install Trunk, WASM bundling tool
```console
cargo install --locked trunk
```

5. Install RTPeeker
```console
cargo install --locked --git https://github.com/LVala/rtpeeker --branch easier-install rtpeeker
```

6. Run the app
```console
rtpeeker --help
```

## Usage

List local network interfaces
```console
rtpeeker list
```

Capture from interface "en0" and file `./rtp.pcap`
```console
rtpeeker run -i en0 -f ./rtp.pcap
```

Apply capture filter (the same as in Wireshark or tcpdump)
```console
rtpeeker run -i en0 -c "src 192.0.0.5"
```

Show help explaining these options
```console
rtpeeker --help
```
