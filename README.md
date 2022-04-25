# Jet CLI

[![Test](https://img.shields.io/github/workflow/status/jet-lab/jet-cli/Test?label=Test&logo=github)](https://github.com/jet-lab/jet-cli/tree/master/Cargo.toml)
[![Version](https://img.shields.io/github/v/release/jet-lab/jet-cli?color=orange&label=jet-cli)](https://github.com/jet-lab/jet-cli/tree/master/Cargo.toml)

## Installation

### Download Pre-built Binary (Recommended)

Each [release](https://github.com/jet-lab/jet-cli/releases) of the CLI contains a pre-built binary in a `.tar.gz` for the following targets:

- `aarch64-apple-darwin`
- `x86_64-apple-darwin`
- `x86_64-unknown-linux-gnu`

> If you require a new build target to be added to the release builds, please create an issue on this repository stating so as the preferred request method.

### Build from Source

```sh
cargo install --git https://github.com/jet-lab/jet-cli --tag <TAG> jet-cli --locked
```
