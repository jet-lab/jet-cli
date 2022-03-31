use anchor_client::solana_sdk::commitment_config::CommitmentConfig;
use anchor_client::solana_sdk::pubkey::Pubkey;
use anchor_client::solana_sdk::signature::Signer;
use anchor_client::solana_sdk::system_program;
use anchor_client::Client;
use anchor_spl::token;
use anyhow::{anyhow, Result};
use clap::Subcommand;
use jet_staking::accounts;
use jet_staking::instruction as args;
use jet_staking::state::StakePool;
use std::rc::Rc;

use crate::config::ConfigOverride;
use crate::macros::*;

/// Staking program based subcommand enum variants.
#[derive(Debug, Subcommand)]
pub enum Command {
    #[clap(about = "Deposit to a stake pool from your account")]
    Add {
        amount: Option<u64>,
        pool: Pubkey,
        token_account: Pubkey,
    },
    #[clap(about = "Create a new staking account")]
    CreateAccount { pool: Pubkey },
}

/// The main entry point and handler for all staking
/// program interaction commands.
pub fn entry(cfg: &ConfigOverride, subcmd: &Command) -> Result<()> {
    match subcmd {
        Command::Add {
            amount,
            pool,
            token_account,
        } => add_stake(cfg, pool, token_account, amount),
        Command::CreateAccount { pool } => create_account(cfg, pool),
    }
}

/// The function handler for the staking subcommand that allows users to add
/// stake to their designated staking account from an owned token account.
fn add_stake(
    overrides: &ConfigOverride,
    pool: &Pubkey,
    token_account: &Pubkey,
    amount: &Option<u64>,
) -> Result<()> {
    let config = overrides.transform()?;

    let stake_account = find_staking_address(pool, &config.keypair.pubkey());

    let (program, signer) = program_client!(config, jet_staking::ID);

    assert_exists!(program, stake_account);

    let StakePool {
        stake_pool_vault, ..
    } = program.account(*pool)?;

    let sig = program
        .request()
        .accounts(accounts::AddStake {
            stake_pool: *pool,
            stake_pool_vault,
            stake_account,
            payer: signer.pubkey(),
            payer_token_account: *token_account,
            token_program: token::ID,
        })
        .args(args::AddStake { amount: *amount })
        .signer(signer.as_ref())
        .send()?;

    if config.verbose {
        println!("AddStake Signature: {}", sig);
    }

    Ok(())
}

/// The function handler for the staking subcommand that allows users to create a
/// new staking account for a designated pool for themselves.
fn create_account(overrides: &ConfigOverride, pool: &Pubkey) -> Result<()> {
    let config = overrides.transform()?;
    let signer_pubkey = config.keypair.pubkey();

    let auth = find_auth_address(&signer_pubkey);
    let stake_account = find_staking_address(pool, &signer_pubkey);

    let (program, signer) = program_client!(config, jet_staking::ID);

    assert_not_exists!(program, stake_account);

    let sig = program
        .request()
        .accounts(accounts::InitStakeAccount {
            owner: signer.pubkey(),
            auth,
            stake_pool: *pool,
            stake_account,
            payer: signer.pubkey(),
            system_program: system_program::ID,
        })
        .signer(signer.as_ref())
        .send()?;

    if config.verbose {
        println!("InitStakeAccount Signature: {}", sig);
    }

    Ok(())
}

/// Derive the public key of a `jet_auth::UserAuthentication` program account.
fn find_auth_address(owner: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[owner.as_ref()], &jet_auth::ID).0
}

/// Derive the public key of a `jet_staking::StakeAccount` program account.
fn find_staking_address(stake_pool: &Pubkey, owner: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[stake_pool.as_ref(), owner.as_ref()], &jet_staking::ID).0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derive_correct_auth_address() {
        let owner = Pubkey::default();
        let auth = find_auth_address(&owner);
        assert_eq!(
            auth.to_string(),
            "L2QDXAsEpjW1kmyCJSgJnifrMLa5UiG19AUFa83hZND"
        );
    }

    #[test]
    fn derive_correct_staking_address() {
        let pool = Pubkey::default();
        let owner = Pubkey::default();
        let staking = find_staking_address(&pool, &owner);
        assert_eq!(
            staking.to_string(),
            "3c7McYaJYNGR5jNyxgudWejKMebRZL4AoFPSuNKp9Dsq"
        );
    }
}
