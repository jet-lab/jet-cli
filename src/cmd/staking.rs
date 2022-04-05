use anchor_client::anchor_lang::ToAccountMetas;
use anchor_client::solana_sdk::instruction::Instruction;
use anchor_client::solana_sdk::pubkey::Pubkey;
use anchor_client::solana_sdk::signature::Signer;
use anchor_client::solana_sdk::system_program::ID as system_program;
use anchor_client::solana_sdk::sysvar::rent::ID as rent;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::ID as token_program;
use anyhow::{anyhow, Result};
use clap::Subcommand;
use jet_staking::state::{StakeAccount, StakePool};
use jet_staking::{accounts, instruction, spl_governance as jet_spl_governance, PoolConfig};
use spinners::*;
use spl_associated_token_account::{create_associated_token_account, get_associated_token_address};
use spl_governance::state::realm::get_realm_data;
use spl_governance::state::token_owner_record::get_token_owner_record_address;
use std::str::FromStr;

use crate::config::ConfigOverride;
use crate::macros::*;
use crate::pubkey::find_auth_address;
use crate::pubkey::*;
use crate::terminal::request_approval;

#[derive(Debug, PartialEq)]
pub enum WithdrawType {
    Bonded,
    Unbonded,
}

impl FromStr for WithdrawType {
    type Err = anyhow::Error;

    fn from_str(val: &str) -> Result<Self> {
        match val {
            "b" | "bonded" => Ok(WithdrawType::Bonded),
            "u" | "unbonded" => Ok(WithdrawType::Unbonded),
            _ => Err(anyhow!("unknown withdraw type string to parse")),
        }
    }
}

/// Staking program based subcommand enum variants.
#[derive(Debug, Subcommand)]
pub enum Command {
    #[clap(about = "Deposit to a stake pool from your account")]
    Add {
        #[clap(long)]
        amount: Option<u64>,
        #[clap(long)]
        pool: Pubkey,
        #[clap(long)]
        realm: Pubkey,
        #[clap(long)]
        token_account: Pubkey,
    },
    #[clap(about = "Close a stake account")]
    CloseAccount {
        #[clap(long)]
        pool: Pubkey,
        #[clap(long)]
        receiver: Option<Pubkey>,
    },
    #[clap(about = "Create a new stake account")]
    CreateAccount {
        #[clap(long)]
        pool: Pubkey,
    },
    #[clap(about = "Create a new staking pool")]
    CreatePool {
        #[clap(long)]
        seed: String,
        #[clap(long)]
        token_mint: Pubkey,
        #[clap(long)]
        unbond_period: u64,
    },
    #[clap(about = "Withdraw staked funds from a pool")]
    Withdraw {
        #[clap(long)]
        amount: Option<u64>,
        #[clap(long)]
        pool: Pubkey,
        #[clap(long)]
        rent_receiver: Option<Pubkey>,
        #[clap(long)]
        token_receiver: Option<Pubkey>,
        #[clap(long)]
        unbonding_account: Option<Pubkey>,
        #[clap(long = "type")]
        withdraw_type: WithdrawType,
    },
}

/// The main entry point and handler for all staking
/// program interaction commands.
pub fn entry(cfg: &ConfigOverride, program_id: &Pubkey, subcmd: &Command) -> Result<()> {
    match subcmd {
        Command::Add {
            amount,
            pool,
            realm,
            token_account,
        } => add_stake(cfg, program_id, amount, pool, realm, token_account),
        Command::CloseAccount { pool, receiver } => close_account(cfg, program_id, pool, receiver),
        Command::CreateAccount { pool } => create_account(cfg, program_id, pool),
        Command::CreatePool {
            seed,
            token_mint,
            unbond_period,
        } => create_pool(cfg, program_id, seed.clone(), token_mint, *unbond_period),
        Command::Withdraw {
            amount,
            pool,
            rent_receiver,
            token_receiver,
            unbonding_account,
            withdraw_type,
        } => withdraw(
            cfg,
            program_id,
            amount,
            pool,
            rent_receiver,
            token_receiver,
            unbonding_account,
            withdraw_type,
        ),
    }
}

/// The function handler for the staking subcommand that allows users to add
/// stake to their designated staking account from an owned token account.
fn add_stake(
    overrides: &ConfigOverride,
    program_id: &Pubkey,
    amount: &Option<u64>,
    pool: &Pubkey,
    realm: &Pubkey,
    token_account: &Pubkey,
) -> Result<()> {
    let config = overrides.transform()?;
    request_approval(&config)?;

    let (program, signer) = program_client!(config, *program_id);

    let mut sp = Spinner::new(Spinners::Dots, "Finding stake account and pool".into());

    // Derive the stake account address for the user and assert
    // the existence of the stake account and stake pool program account
    let stake_account = find_stake_account_address(pool, &signer.pubkey(), &program.id());

    assert_exists!(program, StakeAccount, &stake_account);

    let StakePool {
        stake_pool_vault,
        stake_vote_mint,
        ..
    } = program.account(*pool)?;

    sp.stop_with_message("✅ Stake account and pool found".into());

    sp = Spinner::new(Spinners::Dots, "Building prerequisite instructions".into());

    // Create the instruction for `jet_staking::AddStake` to prepend the vote minting
    let add_stake_ix = Instruction::new_with_borsh(
        jet_staking::ID,
        &instruction::AddStake { amount: *amount },
        accounts::AddStake {
            stake_pool: *pool,
            stake_pool_vault,
            stake_account,
            payer: signer.pubkey(),
            payer_token_account: *token_account,
            token_program,
        }
        .to_account_metas(None),
    );

    // Derive the public key of the user's voter token account and if the account doesn't exist,
    // prepend the transaction with the creation of the associated token account for them
    let voter_token_account = get_associated_token_address(&signer.pubkey(), &stake_vote_mint);

    let mut req = program.request();

    assert_exists!(program, AssociatedToken, &voter_token_account, {
        req = req.instruction(create_associated_token_account(
            &signer.pubkey(),
            &signer.pubkey(),
            &stake_vote_mint,
        ));
    },);

    // Append the instruction for `jet_staking::AddStake` to the transaction
    req = req.instruction(add_stake_ix);

    // Read and deserialize the realm account bytes from on-chain
    let realm_data = fetch_realm!(program, &jet_spl_governance::ID, realm);

    // Derive the public keys for the governance token owner record
    // and the relevant governance token vault accounts.
    let governance_owner_record = get_token_owner_record_address(
        &jet_spl_governance::ID,
        realm,
        &realm_data.community_mint,
        &signer.pubkey(),
    );

    let governance_vault = find_governance_vault_address(realm, &realm_data.community_mint);

    sp.stop_with_message("✅ Instruction bytes compiled".into());

    sp = Spinner::new(Spinners::Dots, "Sending transaction".into());

    // Build and send the remaining of the transaction from the
    // `jet_staking::MintVotes` instruction and send it
    let signature = req
        .accounts(accounts::MintVotes {
            owner: signer.pubkey(),
            stake_pool: *pool,
            stake_pool_vault,
            stake_vote_mint,
            stake_account,
            voter_token_account,
            governance_realm: *realm,
            governance_vault,
            governance_owner_record,
            payer: signer.pubkey(),
            governance_program: jet_spl_governance::ID,
            token_program,
            system_program,
            rent,
        })
        .args(instruction::MintVotes { amount: None })
        .signer(signer.as_ref())
        .send()?;

    sp.stop_with_message("✅ Transaction confirmed!\n".into());

    if config.verbose {
        println!("Signature: {}", signature);
    }

    Ok(())
}

/// The function handler for a user closing their staking account.
fn close_account(
    overrides: &ConfigOverride,
    program_id: &Pubkey,
    pool: &Pubkey,
    receiver: &Option<Pubkey>,
) -> Result<()> {
    let config = overrides.transform()?;
    request_approval(&config)?;

    let (program, signer) = program_client!(config, *program_id);

    // Derive the public key of the `jet_staking::StakeAccount` that
    // is being closed in the instruction call and assert that is exists
    let stake_account = find_stake_account_address(pool, &signer.pubkey(), &program.id());

    assert_exists!(program, StakeAccount, &stake_account);

    let closer = match receiver {
        Some(pk) => *pk,
        None => signer.pubkey(),
    };

    let sp = Spinner::new(Spinners::Dots, "Sending transaction".into());

    // Build and send the `jet_staking::CloseStakeAccount` transaction
    let signature = program
        .request()
        .accounts(accounts::CloseStakeAccount {
            owner: signer.pubkey(),
            closer,
            stake_account,
        })
        .args(instruction::CloseStakeAccount {})
        .signer(signer.as_ref())
        .send()?;

    sp.stop_with_message("✅ Transaction confirmed!\n".into());

    if config.verbose {
        println!("Signature: {}", signature);
    }

    Ok(())
}

/// The function handler for the staking subcommand that allows users to create a
/// new staking account for a designated pool for themselves.
fn create_account(overrides: &ConfigOverride, program_id: &Pubkey, pool: &Pubkey) -> Result<()> {
    let config = overrides.transform()?;
    request_approval(&config)?;

    let (program, signer) = program_client!(config, *program_id);

    // Derive the public keys for the user's `jet_auth::UserAuthentication`
    // and `jet_staking::StakeAccount` program accounts and assert that the
    // stake account does not exist since this command creates one
    let auth = find_auth_address(&signer.pubkey(), &program.id());
    let stake_account = find_stake_account_address(pool, &signer.pubkey(), &program.id());

    assert_not_exists!(program, StakeAccount, &stake_account);

    let sp = Spinner::new(Spinners::Dots, "Sending transaction".into());

    // Build and send the `jet_staking::InitStakeAccount` transaction
    let signature = program
        .request()
        .accounts(accounts::InitStakeAccount {
            owner: signer.pubkey(),
            auth,
            stake_pool: *pool,
            stake_account,
            payer: signer.pubkey(),
            system_program,
        })
        .args(instruction::InitStakeAccount {})
        .signer(signer.as_ref())
        .send()?;

    sp.stop_with_message("✅ Transaction confirmed!\n".into());

    if config.verbose {
        println!("Signature: {}\n", signature);
    }

    println!("Pubkey: {}", stake_account);

    Ok(())
}

/// The function handler for the staking subcommand that allows a user
/// to create a new staking pool with the appropriate mints from a seed.
fn create_pool(
    overrides: &ConfigOverride,
    program_id: &Pubkey,
    seed: String,
    token_mint: &Pubkey,
    unbond_period: u64,
) -> Result<()> {
    let config = overrides.transform()?;
    request_approval(&config)?;

    let (program, signer) = program_client!(config, *program_id);

    // Derive the public keys needed for a new staking pool and
    // ensure that the pool address doesn't already exist
    let StakePoolAddresses {
        pool,
        vault,
        collateral_mint,
        vote_mint,
    } = find_stake_pool_addresses(&seed, &program.id());

    assert_not_exists!(program, StakePool, &pool);

    // Build and send the `jet_staking::instruction::InitPool` transaction
    let signature = program
        .request()
        .accounts(accounts::InitPool {
            payer: signer.pubkey(),
            authority: signer.pubkey(),
            token_mint: *token_mint,
            stake_pool: pool,
            stake_vote_mint: vote_mint,
            stake_collateral_mint: collateral_mint,
            stake_pool_vault: vault,
            token_program,
            system_program,
            rent,
        })
        .args(instruction::InitPool {
            seed,
            config: PoolConfig { unbond_period },
        })
        .signer(signer.as_ref())
        .send()?;

    if config.verbose {
        println!("Signature: {}\n", signature);
    }

    Ok(())
}

/// The function handler for the staking subcommand that allows users to either
/// withdraw bonded or unbonded stake from the designated pool based on the
/// provided value for the `--type` (`WithdrawType`) command argument.
fn withdraw(
    overrides: &ConfigOverride,
    program_id: &Pubkey,
    amount: &Option<u64>,
    pool: &Pubkey,
    rent_receiver: &Option<Pubkey>,
    token_receiver: &Option<Pubkey>,
    unbonding_account: &Option<Pubkey>,
    withdraw_type: &WithdrawType,
) -> Result<()> {
    // Return error for required missing arguments or log a warning
    // or any unused arguments provided based on the selected
    // stake withdraw type instruction provided
    if *withdraw_type == WithdrawType::Bonded {
        if amount.is_none() {
            return Err(anyhow!(
                "Error: argument '--amount' is required with bonded withdraws"
            ));
        }

        if rent_receiver.is_some() {
            println!("Warning: argument '--rent-receiver' is unused with bonded withdraws");
        }
    } else {
        if unbonding_account.is_none() {
            return Err(anyhow!(
                "Error: argument '--unbonding-account' is required with unbonded withdraws"
            ));
        }

        if amount.is_some() {
            println!("Warning: argument '--amount' is unused with unbonded withdraws");
        }
    }

    let config = overrides.transform()?;
    request_approval(&config)?;

    let (program, signer) = program_client!(config, *program_id);

    let mut req = program.request();

    let mut sp = Spinner::new(Spinners::Dots, "Deriving required accounts".into());

    let token_closer = match token_receiver {
        Some(pk) => *pk,
        None => signer.pubkey(),
    };

    let StakePool {
        stake_pool_vault, ..
    } = program.account(*pool)?;

    // Assign the transaction accounts and instruction arguments to
    // the request based on the provided `WithdrawType` argument
    req = match *withdraw_type {
        WithdrawType::Bonded => {
            assert!(amount.is_some());

            req.accounts(accounts::WithdrawBonded {
                authority: signer.pubkey(),
                stake_pool: *pool,
                token_receiver: token_closer,
                stake_pool_vault,
                token_program,
            })
            .args(instruction::WithdrawBonded {
                amount: amount.unwrap(),
            })
        }
        WithdrawType::Unbonded => {
            assert!(unbonding_account.is_some());

            let stake_account = find_stake_account_address(pool, &signer.pubkey(), &program.id());

            let rent_closer = match rent_receiver {
                Some(pk) => *pk,
                None => signer.pubkey(),
            };

            req.accounts(accounts::WithdrawUnbonded {
                owner: signer.pubkey(),
                closer: rent_closer,
                token_receiver: token_closer,
                stake_account,
                stake_pool: *pool,
                stake_pool_vault,
                unbonding_account: unbonding_account.unwrap(),
                token_program,
            })
            .args(instruction::WithdrawUnbonded {})
        }
    };

    sp.stop_with_message("✅ Required accounts found".into());

    sp = Spinner::new(Spinners::Dots, "Sending transaction".into());

    // Finalize and send the `jet_staking::instruction::WithdrawXXX` transaction
    let signature = req.signer(signer.as_ref()).send()?;

    sp.stop_with_message("✅ Transaction confirmed!\n".into());

    if config.verbose {
        println!("Signature: {}", signature);
    }

    Ok(())
}
