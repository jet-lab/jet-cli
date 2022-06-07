<div align="center">
  <img height="125" src="https://293354890-files.gitbook.io/~/files/v0/b/gitbook-legacy-files/o/assets%2F-M_72skN1dye71puMdjs%2F-Miqzl5oK1cXXAkARfER%2F-Mis-yeKp1Krh7JOFzQG%2Fjet_logomark_color.png?alt=media&token=0b8dfc84-37d7-455d-9dfd-7bb59cee5a1a" />

  <h1>Jet CLI</div>

  <p align="center">
    <a target="_blank" href="https://github.com/jet-lab/jet-cli/actions/workflows/test.yaml">
      <img alt="Test" src="https://img.shields.io/github/workflow/status/jet-lab/jet-cli/Test?label=Test&logo=github">
    </a>
    <a target="_blank" href="https://github.com/jet-lab/jet-cli/tree/master/Cargo.toml">
      <img alt="Version" src="https://img.shields.io/github/v/release/jet-lab/jet-cli?color=orange&label=jet-cli" />
    </a>
    <a target="_blank" href="https://github.com/jet-lab/jet-cli/tree/master/LICENSE">
      <img alt="License" src="https://img.shields.io/badge/License-AGPL--3.0--or--later-blue" />
    </a>
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

### Checkout and Run from Source

```sh
# Clone Jet CLI
git clone https://github.com/jet-lab/jet-cli
cd jet-cli

# Install submodules
git submodule update

# Build and run (This may take a while)
cargo run

# Create a devnet margin account
cargo run margin create-account -u d --seed 0
```

### Create Margin Account

```sh
# Create a devnet margin account
jet-cli margin create-account -u d --seed 0

# Store the margin account pubkey in $account
account=$(jet-cli margin derive --seed 0)

echo $account
# 77tJm3j57zMaGR1bFDgWKeJphQarK3fkhB3VPT912zha

# Deposit into the pool (Requires the pool and token account pubkey)
jet-cli margin-pool deposit --account $account --pool $pool --source $source 1
```

# Troubleshooting

`` Error: Message("missing field `keypair_path`", Some(...))  ``

Ensure you have a solana config file at ~/.config/solana/cli/config.yml with contents like the following.

```yaml
---
json_rpc_url: "http://localhost:8899"
websocket_url: ""
keypair_path: ~/.config/solana/id.json
address_labels:
  "11111111111111111111111111111111": System Program
commitment: confirmed
```
