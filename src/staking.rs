use anchor_client::solana_sdk::commitment_config::CommitmentConfig;
use anchor_client::solana_sdk::pubkey::Pubkey;
use anchor_client::solana_sdk::signature::Signer;
use anchor_client::solana_sdk::system_program;
use anchor_client::Client;
use anchor_spl::token;
use anyhow::Result;
use clap::Subcommand;
use jet_staking::accounts;
use jet_staking::instruction as args;
use jet_staking::state::StakePool;
use std::rc::Rc;

use crate::config::ConfigOverride;
use crate::{program_client, send_and_log};

/// Staking program based subcommand enum variants.
#[derive(Debug, Subcommand)]
pub enum Command {
    #[clap(about = "Deposit to a stake pool from your account")]
    Add {
        amount: Option<u64>,
        pool: Pubkey,
        token_account: Pubkey,
    },
    #[clap(about = "Close your staking account")]
    CloseAccount { pool: Pubkey },
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
        Command::CloseAccount { pool } => close_account(cfg, pool),
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

    let StakePool {
        stake_pool_vault, ..
    } = program.account(*pool)?;

    send_and_log!(
        program,
        accounts::AddStake {
            stake_pool: *pool,
            stake_pool_vault,
            stake_account,
            payer: program.payer(),
            payer_token_account: *token_account,
            token_program: token::ID,
        },
        args::AddStake { amount: *amount },
        signer
    )
}

/// The function handler for the staking subcommand that allows user to
/// close their staking account and receive the rent funds back.
fn close_account(overrides: &ConfigOverride, pool: &Pubkey) -> Result<()> {
    let config = overrides.transform()?;

    let stake_account = find_staking_address(pool, &config.keypair.pubkey());

    let (program, signer) = program_client!(config, jet_staking::ID);

    send_and_log!(
        program,
        accounts::CloseStakeAccount {
            owner: program.payer(),
            closer: program.payer(),
            stake_account,
        },
        signer
    )
}

/// The function handler for the staking subcommand that allows users to create a
/// new staking account for a designated pool for themselves.
fn create_account(overrides: &ConfigOverride, pool: &Pubkey) -> Result<()> {
    let config = overrides.transform()?;

    let auth = find_auth_address(&config.keypair.pubkey());
    let stake_account = find_staking_address(pool, &config.keypair.pubkey());

    let (program, signer) = program_client!(config, jet_staking::ID);

    send_and_log!(
        program,
        accounts::InitStakeAccount {
            owner: program.payer(),
            auth,
            stake_pool: *pool,
            stake_account,
            payer: program.payer(),
            system_program: system_program::ID,
        },
        signer
    )
}

/// Derive the public key of a `jet_auth::UserAuthentication` program account.
fn find_auth_address(owner: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[owner.as_ref()], &jet_auth::ID).0
}

/// Derive the public key of a `jet_staking::StakeAccount` program account.
fn find_staking_address(stake_pool: &Pubkey, owner: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[stake_pool.as_ref(), owner.as_ref()], &jet_staking::ID).0
}
