#!/bin/bash

GOVERNANCE_REPO_PATH=""
PROGRAM_TO_DEPLOY=""

function validate_repo_path {
  if [ -z "$GOVERNANCE_REPO_PATH" ]; then
    echo "Error: expected the '--governance' argument but was not found."
    exit 1
  elif [ ! -d "$GOVERNANCE_REPO_PATH" ]; then
    echo "Error: value for '--governance' does not point to valid directory."
    exit 1
  fi
}

function validate_program {
  if [ -z "$PROGRAM_TO_DEPLOY" ]; then
    if [ ! -d "$GOVERNANCE_REPO_PATH/programs/$PROGRAM_TO_DEPLOY" ]; then
      echo "Error: value for '--program' does not exist in the repository."
      exit 1
    fi
  fi
}

function deploy {
  local program="$1"
  echo "Deploying jet_$program..."
  solana deploy \
    --url l \
    --keypair ./authority.json \
    $GOVERNANCE_REPO_PATH/target/deploy/jet_$program.so \
    $GOVERNANCE_REPO_PATH/target/deploy/jet_$program-keypair.json
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --governance)
      GOVERNANCE_REPO_PATH=$2
      shift 2
      ;;

    --program)
      PROGRAM_TO_DEPLOY=$2
      shift 2
      ;;

    *)
      break
      ;;
  esac
done

validate_repo_path
validate_program

solana-keygen new \
  --no-bip39-passphrase \
  --outfile ./authority.json \
  --silent \
  --force

solana-keygen new \
  --no-bip39-passphrase \
  --outfile ./user.json \
  --silent \
  --force

solana airdrop -u l 50 $(solana-keygen pubkey ./authority.json)
solana airdrop -u l 10 $(solana-keygen pubkey ./user.json)

if [ -z "$PROGRAM_TO_DEPLOY" ]; then
  deploy "auth"
  deploy "rewards"
  deploy "staking"
else
  deploy $PROGRAM_TO_DEPLOY
fi
