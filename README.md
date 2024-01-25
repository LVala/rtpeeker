# RTPeeker

![CI](https://img.shields.io/github/actions/workflow/status/LVala/rtpeeker/ci.yml)
![crates.io](https://img.shields.io/crates/v/rtpeeker)

RTP streams analysis and visualization tool.

_Work in progress..._

## Installation

Supports Linux and MacOS.

1. RTPeeker depends on `libpcap`, make sure to install it:

```shell
# installed on MacOS by default

# for Ubuntu
sudo apt install libpcap-dev

# for Arch
sudo pacman -S libpcap
```

2. Install RTPeeker using the [Rust toolchain](https://www.rust-lang.org/tools/install):

```shell
cargo install --locked rtpeeker
```

3. Run RTPeeker:

```shell
rtpeeker --help
```
