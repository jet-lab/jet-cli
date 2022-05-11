<div align="center">
  <h1>Jet CLI</div>

  <p align="center">
    <a target="_blank" href="https://github.com/jet-lab/jet-cli/actions/workflows/test.yaml"><img alt="Test" src="https://img.shields.io/github/workflow/status/jet-lab/jet-cli/Test?label=Test&logo=github"></a>
    <a target="_blank" href="https://github.com/jet-lab/jet-cli/tree/master/Cargo.toml"><img alt="Version" src="https://img.shields.io/github/v/release/jet-lab/jet-cli?color=orange&label=jet-cli" /></a>
  </p>

  <p align="center">
    <em>
      The Jet Protocol CLI is geared towards users of all types who wish to interact with our range of Solana programs through a traditional terminal interface. The goal is to provide equivalently high-leveled abstraction from the on-chain programs as a traditional web-application interface would for the user-base that wishes to interact through their command line or via for use in automated or scripting environments.
    </em>
  </p>
</div>


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
