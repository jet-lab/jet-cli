use anchor_client::solana_sdk::commitment_config::CommitmentConfig;
use anchor_client::solana_sdk::pubkey::Pubkey;
use anchor_client::solana_sdk::signature::Signer;
use anchor_client::solana_sdk::system_program;
use anchor_client::Client;
use anchor_spl::token;
use anyhow::Result;
use clap::Subcommand;
use jet_staking::accounts;
use jet_staking::instruction;
use jet_staking::state::StakePool;
use std::rc::Rc;

use crate::config::ConfigOverride;
use crate::program_client;

/// Staking program based subcommand enum variants.
#[derive(Debug, Subcommand)]
pub enum Command {
    Add {
        amount: Option<u64>,
        pool: Pubkey,
        token_account: Pubkey,
    },
    CreateAccount {
        pool: Pubkey,
    },
    Unbond {
        amount: Option<u64>,
        pool: Pubkey,
    },
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
        Command::Unbond { amount, pool } => unbond_stake(cfg, pool, amount),
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

    let sig = program
        .request()
        .accounts(accounts::AddStake {
            stake_pool: *pool,
            stake_pool_vault,
            stake_account,
            payer: program.payer(),
            payer_token_account: *token_account,
            token_program: token::ID,
        })
        .args(instruction::AddStake {
            amount: *amount,
        })
        .signer(signer.as_ref())
        .send()?;

    println!("Signature: {}", sig);

    Ok(())
}

/// The function handler for the staking subcommand that allows users to create a
/// new staking account for a designated pool for themselves.
fn create_account(overrides: &ConfigOverride, pool: &Pubkey) -> Result<()> {
    let config = overrides.transform()?;

    let auth = find_auth_address(&config.keypair.pubkey());
    let stake_account = find_staking_address(pool, &config.keypair.pubkey());

    let (program, signer) = program_client!(config, jet_staking::ID);

    let sig = program
        .request()
        .accounts(accounts::InitStakeAccount {
            owner: program.payer(),
            auth,
            stake_pool: *pool,
            stake_account,
            payer: program.payer(),
            system_program: system_program::ID,
        })
        .signer(signer.as_ref())
        .send()?;

    println!("Signature: {}", sig);

    Ok(())
}

/// The function handler for the staking subcommand that allows users to unbond
/// existing staking from the pool back into their account.
fn unbond_stake(overrides: &ConfigOverride, pool: &Pubkey, amount: &Option<u64>) -> Result<()> {
    let config = overrides.transform()?;

    let stake_account = find_staking_address(pool, &config.keypair.pubkey());
    let unbonding_account = find_unbonding_address(&stake_account, 0); // FIXME:

    let (program, signer) = program_client!(config, jet_staking::ID);

    let StakePool {
        stake_pool_vault, ..
    } = program.account(*pool)?;

    let sig = program
        .request()
        .accounts(accounts::UnbondStake {
            owner: program.payer(),
            payer: program.payer(),
            stake_account,
            stake_pool: *pool,
            stake_pool_vault,
            unbonding_account,
            system_program: system_program::ID,
        })
        .args(instruction::UnbondStake {
            seed: 0, // FIXME:,
            amount: *amount,
        })
        .signer(signer.as_ref())
        .send()?;

    println!("Signature: {}", sig);

    Ok(())
}

fn find_auth_address(owner: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[owner.as_ref()], &jet_staking::ID).0
}

fn find_staking_address(stake_pool: &Pubkey, owner: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[stake_pool.as_ref(), owner.as_ref()], &jet_staking::ID).0
}

fn find_unbonding_address(stake_account: &Pubkey, seed: u32) -> Pubkey {
    Pubkey::find_program_address(
        &[stake_account.as_ref(), seed.to_le_bytes().as_ref()],
        &jet_staking::ID,
    )
    .0
}
