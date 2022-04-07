use anchor_client::anchor_lang::ToAccountMetas;
use anchor_client::solana_sdk::instruction::Instruction;
use anchor_client::solana_sdk::pubkey::Pubkey;
use anchor_client::solana_sdk::signature::Signer;
use anchor_client::solana_sdk::system_program::ID as system_program;
use anchor_client::solana_sdk::sysvar::rent::ID as rent;
use anchor_spl::token::ID as token_program;
use anyhow::Result;
use clap::Subcommand;
use jet_staking::state::{StakeAccount, StakePool};
use jet_staking::{accounts, instruction, PoolConfig};
use spl_associated_token_account::get_associated_token_address;

use crate::config::{Config, ConfigOverride};
use crate::macros::*;
use crate::program::*;
use crate::pubkey::*;
use crate::terminal::Spinner;

/// Staking program based subcommand enum variants.
#[derive(Debug, Subcommand)]
pub enum StakingCommand {
    /// Deposit to a stake pool from your account.
    Add {
        /// (Optional) The amount of token to stake in the pool. The program
        /// by default will attempt to stake as much as possible if
        /// no amount is provided.
        #[clap(short, long)]
        amount: Option<u64>,
        /// Stake pool to deposit.
        #[clap(short, long)]
        pool: Pubkey,
    },
    /// Close a stake account.
    CloseAccount {
        #[clap(long)]
        pool: Pubkey,
        #[clap(long)]
        receiver: Option<Pubkey>,
    },
    /// Create a new stake account.
    CreateAccount {
        #[clap(long)]
        pool: Pubkey,
    },
    /// Create a new staking pool.
    CreatePool {
        #[clap(long)]
        seed: String,
        #[clap(long)]
        realm: Pubkey,
        #[clap(long)]
        token_mint: Pubkey,
        #[clap(long)]
        unbond_period: u64,
    },
    /// Withdraw bonded stake funds from a pool.
    WithdrawBonded {
        #[clap(long)]
        amount: u64,
        #[clap(long)]
        pool: Pubkey,
        #[clap(long)]
        receiver: Option<Pubkey>,
    },
    /// Withdraw bonded stake funds from a pool.
    WithdrawUnbonded {
        #[clap(long)]
        pool: Pubkey,
        #[clap(long)]
        rent_receiver: Option<Pubkey>,
        #[clap(long)]
        token_receiver: Option<Pubkey>,
        #[clap(long = "account")]
        unbonding_account: Pubkey,
    },
}

/// The main entry point and handler for all staking
/// program interaction commands.
pub fn entry(
    overrides: &ConfigOverride,
    program_id: &Pubkey,
    subcmd: &StakingCommand,
) -> Result<()> {
    let cfg = overrides.transform(*program_id)?;
    match subcmd {
        StakingCommand::Add { amount, pool } => process_add_stake(&cfg, amount, pool),
        StakingCommand::CloseAccount { pool, receiver } => {
            process_close_account(&cfg, pool, receiver)
        }
        StakingCommand::CreateAccount { pool } => process_create_account(&cfg, pool),
        StakingCommand::CreatePool {
            seed,
            realm,
            token_mint,
            unbond_period,
        } => process_create_pool(&cfg, seed.clone(), realm, token_mint, *unbond_period),
        StakingCommand::WithdrawBonded {
            amount,
            pool,
            receiver,
        } => process_withdraw_bonded(&cfg, *amount, pool, receiver),
        StakingCommand::WithdrawUnbonded {
            pool,
            rent_receiver,
            token_receiver,
            unbonding_account,
        } => {
            process_withdraw_unbonded(&cfg, pool, rent_receiver, token_receiver, unbonding_account)
        }
    }
}

/// The function handler for the staking subcommand that allows users to add
/// stake to their designated staking account from an owned token account.
fn process_add_stake(cfg: &Config, amount: &Option<u64>, pool: &Pubkey) -> Result<()> {
    let (program, signer) = create_program_client(cfg);
    let mut req = program.request();
    let mut ix_names = Vec::<&str>::new();

    let mut sp = Spinner::new("Finding stake pool accounts");
    assert_exists!(&program, StakePool, pool);

    let StakePool {
        stake_pool_vault,
        max_voter_weight_record,
        token_mint,
        ..
    } = program.account(*pool)?;

    sp.finish_with_message("Stake pool accounts retrieved");

    sp = Spinner::new("Preprending required instructions");
    let payer_token_account = get_associated_token_address(&signer.pubkey(), &token_mint);
    let stake_account = derive_stake_account(pool, &signer.pubkey(), &program.id());
    let voter_weight_record = derive_voter_weight_record(&stake_account, &program.id());

    if !account_exists(&program, &stake_account)? {
        req = req.instruction(Instruction::new_with_borsh(
            program.id(),
            &instruction::InitStakeAccount {},
            accounts::InitStakeAccount {
                owner: signer.pubkey(),
                auth: derive_auth_account(&signer.pubkey(), &jet_auth::ID), // TODO: genericize auth program ID (?)
                stake_pool: *pool,
                stake_account,
                voter_weight_record,
                payer: signer.pubkey(),
                system_program,
            }
            .to_account_metas(None),
        ));

        ix_names.push("jet_staking::InitStakeAccount");
    }

    req = req.instruction(Instruction::new_with_borsh(
        program.id(),
        &instruction::AddStake { amount: *amount },
        accounts::AddStake {
            stake_pool: *pool,
            stake_pool_vault,
            stake_account,
            payer: signer.pubkey(),
            payer_token_account,
            voter_weight_record,
            max_voter_weight_record,
            token_program,
        }
        .to_account_metas(None),
    ));

    ix_names.push("jet_staking::AddStake");
    sp.finish_with_message("Instruction bytes compiled");

    send_with_approval(cfg, req.signer(signer.as_ref()), Some(ix_names))
}

/// The function handler for a user closing their staking account.
fn process_close_account(cfg: &Config, pool: &Pubkey, receiver: &Option<Pubkey>) -> Result<()> {
    let (program, signer) = create_program_client(cfg);

    // Derive the public key of the `jet_staking::StakeAccount` that
    // is being closed in the instruction call and assert that is exists
    let stake_account = derive_stake_account(pool, &signer.pubkey(), &program.id());
    let voter_weight_record = derive_voter_weight_record(&stake_account, &program.id());

    assert_exists!(&program, StakeAccount, &stake_account);

    let closer = match receiver {
        Some(pk) => *pk,
        None => signer.pubkey(),
    };

    // Build and send the `jet_staking::CloseStakeAccount` transaction
    send_with_approval(
        cfg,
        program
            .request()
            .accounts(accounts::CloseStakeAccount {
                owner: signer.pubkey(),
                closer,
                voter_weight_record,
                stake_account,
            })
            .args(instruction::CloseStakeAccount {})
            .signer(signer.as_ref()),
        None,
    )
}

/// The function handler for the staking subcommand that allows users to create a
/// new staking account for a designated pool for themselves.
fn process_create_account(cfg: &Config, pool: &Pubkey) -> Result<()> {
    let (program, signer) = create_program_client(cfg);

    // Derive the public keys for the user's `jet_auth::UserAuthentication`
    // and `jet_staking::StakeAccount` program accounts and assert that the
    // stake account does not exist since this command creates one
    let auth = derive_auth_account(&signer.pubkey(), &program.id());
    let stake_account = derive_stake_account(pool, &signer.pubkey(), &program.id());
    let voter_weight_record = derive_voter_weight_record(&stake_account, &program.id());

    assert_not_exists!(&program, StakeAccount, &stake_account);

    // Build and send the `jet_staking::InitStakeAccount` transaction
    send_with_approval(
        cfg,
        program
            .request()
            .accounts(accounts::InitStakeAccount {
                owner: signer.pubkey(),
                auth,
                stake_pool: *pool,
                stake_account,
                payer: signer.pubkey(),
                voter_weight_record,
                system_program,
            })
            .args(instruction::InitStakeAccount {})
            .signer(signer.as_ref()),
        None,
    )?;

    println!("Pubkey: {}", stake_account);

    Ok(())
}

/// The function handler for the staking subcommand that allows a user
/// to create a new staking pool with the appropriate mints from a seed.
fn process_create_pool(
    cfg: &Config,
    seed: String,
    realm: &Pubkey,
    token_mint: &Pubkey,
    unbond_period: u64,
) -> Result<()> {
    let (program, signer) = create_program_client(cfg);

    // Derive the public keys needed for a new staking pool and
    // ensure that the pool address doesn't already exist
    let StakePoolAddresses {
        pool,
        vault,
        collateral_mint,
    } = derive_stake_pool(&seed, &program.id());
    let max_voter_weight_record = derive_max_voter_weight_record(realm, &program.id());

    assert_not_exists!(&program, StakePool, &pool);

    // Build and send the `jet_staking::instruction::InitPool` transaction
    send_with_approval(
        cfg,
        program
            .request()
            .accounts(accounts::InitPool {
                payer: signer.pubkey(),
                authority: signer.pubkey(),
                token_mint: *token_mint,
                stake_pool: pool,
                stake_collateral_mint: collateral_mint,
                stake_pool_vault: vault,
                max_voter_weight_record,
                token_program,
                system_program,
                rent,
            })
            .args(instruction::InitPool {
                seed,
                config: PoolConfig {
                    governance_realm: *realm,
                    unbond_period,
                },
            })
            .signer(signer.as_ref()),
        None,
    )?;

    println!("Pubkey: {}", pool);

    Ok(())
}

/// The function handler for the staking subcommand that allows users to
/// withdraw bonded stake from the designated pool.
fn process_withdraw_bonded(
    cfg: &Config,
    amount: u64,
    pool: &Pubkey,
    receiver: &Option<Pubkey>,
) -> Result<()> {
    let (program, signer) = create_program_client(cfg);

    let token_receiver = match receiver {
        Some(pk) => *pk,
        None => signer.pubkey(),
    };

    let StakePool {
        stake_pool_vault, ..
    } = program.account(*pool)?;

    // Build and send the `jet_staking::instruction::WithdrawBonded` transaction
    send_with_approval(
        cfg,
        program
            .request()
            .accounts(accounts::WithdrawBonded {
                authority: signer.pubkey(),
                stake_pool: *pool,
                token_receiver,
                stake_pool_vault,
                token_program,
            })
            .args(instruction::WithdrawBonded { amount })
            .signer(signer.as_ref()),
        None,
    )
}

/// The function handler for the staking subcommand that allows users to
/// withdraw unbonded stake from the designated pool.
fn process_withdraw_unbonded(
    cfg: &Config,
    pool: &Pubkey,
    rent_receiver: &Option<Pubkey>,
    token_receiver: &Option<Pubkey>,
    unbonding_account: &Pubkey,
) -> Result<()> {
    let (program, signer) = create_program_client(cfg);

    let stake_account = derive_stake_account(pool, &signer.pubkey(), &program.id());

    let rent_closer = match rent_receiver {
        Some(pk) => *pk,
        None => signer.pubkey(),
    };

    let token_closer = match token_receiver {
        Some(pk) => *pk,
        None => signer.pubkey(),
    };

    let StakePool {
        stake_pool_vault, ..
    } = program.account(*pool)?;

    // Build and send the `jet_staking::instruction::WithdrawUnbonded` transaction
    send_with_approval(
        cfg,
        program
            .request()
            .accounts(accounts::WithdrawUnbonded {
                owner: signer.pubkey(),
                closer: rent_closer,
                token_receiver: token_closer,
                stake_account,
                stake_pool: *pool,
                stake_pool_vault,
                unbonding_account: *unbonding_account,
                token_program,
            })
            .args(instruction::WithdrawUnbonded {})
            .signer(signer.as_ref()),
        None,
    )
}
