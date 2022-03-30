use anchor_client::solana_sdk::commitment_config::CommitmentConfig;
use anchor_client::solana_sdk::pubkey::Pubkey;
use anchor_client::solana_sdk::signature::Signer;
use anchor_client::solana_sdk::system_program;
use anchor_client::Client;
use anchor_spl::token;
use anyhow::Result;
use clap::Subcommand;
use jet_staking::accounts::{AddStake, InitStakeAccount};
use jet_staking::state::StakePool;
use std::rc::Rc;

use crate::config::ConfigOverride;

#[derive(Debug, Subcommand)]
pub enum Command {
    Add { pool: Pubkey, token_account: Pubkey },
    CreateAccount { pool: Pubkey },
}

pub fn entry(cfg: &ConfigOverride, subcmd: &Command) -> Result<()> {
    match subcmd {
        Command::Add {
            pool,
            token_account,
        } => add_stake(cfg, pool, token_account),
        Command::CreateAccount { pool } => create_account(cfg, pool),
    }
}

fn add_stake(overrides: &ConfigOverride, pool: &Pubkey, token_account: &Pubkey) -> Result<()> {
    let config = overrides.transform()?;

    let (account, _) = Pubkey::find_program_address(
        &[pool.as_ref(), config.keypair.pubkey().as_ref()],
        &jet_staking::ID,
    );

    let program = Client::new_with_options(
        config.cluster,
        Rc::new(config.keypair),
        CommitmentConfig::confirmed(),
    )
    .program(jet_staking::ID);

    let pool_data: StakePool = program.account(*pool)?;

    let sig = program
        .request()
        .accounts(AddStake {
            stake_pool: *pool,
            stake_pool_vault: pool_data.stake_pool_vault,
            stake_account: account,
            payer: program.payer(),
            payer_token_account: *token_account,
            token_program: token::ID,
        })
        .send()?;

    println!("Signature: {}", sig);

    Ok(())
}

fn create_account(overrides: &ConfigOverride, pool: &Pubkey) -> Result<()> {
    let config = overrides.transform()?;

    let (auth, _) =
        Pubkey::find_program_address(&[config.keypair.pubkey().as_ref()], &jet_auth::ID);

    let (account, _) = Pubkey::find_program_address(
        &[pool.as_ref(), config.keypair.pubkey().as_ref()],
        &jet_staking::ID,
    );

    let program = Client::new_with_options(
        config.cluster,
        Rc::new(config.keypair),
        CommitmentConfig::confirmed(),
    )
    .program(jet_staking::ID);

    let sig = program
        .request()
        .accounts(InitStakeAccount {
            owner: program.payer(),
            auth,
            stake_pool: *pool,
            stake_account: account,
            payer: program.payer(),
            system_program: system_program::ID,
        })
        .send()?;

    println!("Signature: {}", sig);

    Ok(())
}
