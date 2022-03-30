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
use std::{path::PathBuf, rc::Rc};

use crate::{default_keypair_path, load_keypair, parse_cluster};

#[derive(Debug, Subcommand)]
pub enum Command {
    Add {
        #[clap(short, long)]
        keypair: Option<PathBuf>,
        #[clap(short, long)]
        pool: Pubkey,
        #[clap(short, long)]
        token_account: Pubkey,
        #[clap(short, long)]
        url: String,
    },
    CreateAccount {
        #[clap(short, long)]
        keypair: Option<PathBuf>,
        #[clap(short, long)]
        pool: Pubkey,
        #[clap(short, long)]
        url: String,
    },
}

pub fn entry(subcmd: &Command) -> Result<()> {
    match subcmd {
        Command::Add {
            keypair,
            pool,
            token_account,
            url,
        } => add_stake(url, keypair, pool, token_account),
        Command::CreateAccount { keypair, pool, url } => create_account(url, keypair, pool),
    }
}

fn add_stake(
    url: &String,
    keypair_path: &Option<PathBuf>,
    pool: &Pubkey,
    token_account: &Pubkey,
) -> Result<()> {
    let keypair = match keypair_path {
        Some(p) => load_keypair(p.to_owned())?,
        None => load_keypair(default_keypair_path()?)?,
    };

    let (account, _) = Pubkey::find_program_address(
        &[pool.as_ref(), keypair.pubkey().as_ref()],
        &jet_staking::ID,
    );

    let program = Client::new_with_options(
        parse_cluster(url)?,
        Rc::new(keypair),
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

fn create_account(url: &String, keypair_path: &Option<PathBuf>, pool: &Pubkey) -> Result<()> {
    let keypair = match keypair_path {
        Some(p) => load_keypair(p.to_owned())?,
        None => load_keypair(default_keypair_path()?)?,
    };

    let (auth, _) = Pubkey::find_program_address(&[keypair.pubkey().as_ref()], &jet_auth::ID);

    let (account, _) = Pubkey::find_program_address(
        &[pool.as_ref(), keypair.pubkey().as_ref()],
        &jet_staking::ID,
    );

    let program = Client::new_with_options(
        parse_cluster(url)?,
        Rc::new(keypair),
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
