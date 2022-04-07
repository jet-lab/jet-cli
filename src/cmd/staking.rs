use anchor_client::anchor_lang::ToAccountMetas;
use anchor_client::solana_sdk::account_info::AccountInfo;
use anchor_client::solana_sdk::instruction::Instruction;
use anchor_client::solana_sdk::pubkey::Pubkey;
use anchor_client::solana_sdk::signature::Signer;
use anchor_client::solana_sdk::system_program::ID as system_program;
use anchor_client::solana_sdk::sysvar::rent::ID as rent;
use anchor_client::Program;
use anchor_spl::token::ID as token_program;
use anyhow::{anyhow, Result};
use clap::Subcommand;
use jet_staking::state::{StakeAccount, StakePool};
use jet_staking::{accounts, instruction, spl_governance as jet_spl_governance, PoolConfig};
use spl_associated_token_account::{create_associated_token_account, get_associated_token_address};
use spl_governance::state::realm::{get_realm_data, Realm};
use spl_governance::state::token_owner_record::get_token_owner_record_address;

use crate::config::{Config, ConfigOverride};
use crate::macros::*;
use crate::program::*;
use crate::pubkey::*;
use crate::terminal::Spinner;

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
        skip_mint_votes: bool,
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
    #[clap(about = "Withdraw bonded stake funds from a pool")]
    WithdrawBonded {
        #[clap(long)]
        amount: u64,
        #[clap(long)]
        pool: Pubkey,
        #[clap(long)]
        receiver: Option<Pubkey>,
    },
    #[clap(about = "Withdraw bonded stake funds from a pool")]
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
pub fn entry(overrides: &ConfigOverride, program_id: &Pubkey, subcmd: &Command) -> Result<()> {
    let cfg = overrides.transform()?;
    match subcmd {
        Command::Add {
            amount,
            pool,
            realm,
            skip_mint_votes,
        } => process_add_stake(&cfg, program_id, amount, pool, realm, *skip_mint_votes),
        Command::CloseAccount { pool, receiver } => {
            process_close_account(&cfg, program_id, pool, receiver)
        }
        Command::CreateAccount { pool } => process_create_account(&cfg, program_id, pool),
        Command::CreatePool {
            seed,
            token_mint,
            unbond_period,
        } => process_create_pool(&cfg, program_id, seed.clone(), token_mint, *unbond_period),
        Command::WithdrawBonded {
            amount,
            pool,
            receiver,
        } => process_withdraw_bonded(&cfg, program_id, *amount, pool, receiver),
        Command::WithdrawUnbonded {
            pool,
            rent_receiver,
            token_receiver,
            unbonding_account,
        } => process_withdraw_unbonded(
            &cfg,
            program_id,
            pool,
            rent_receiver,
            token_receiver,
            unbonding_account,
        ),
    }
}

/// The function handler for the staking subcommand that allows users to add
/// stake to their designated staking account from an owned token account.
fn process_add_stake(
    cfg: &Config,
    program_id: &Pubkey,
    amount: &Option<u64>,
    pool: &Pubkey,
    realm: &Pubkey,
    skip_mint_votes: bool,
) -> Result<()> {
    let (program, signer) = create_program_client(cfg, *program_id);
    let mut req = program.request();
    let mut ix_names = Vec::<&str>::new();

    // Fetch the public keys for the required accounts
    // from the received stake pool
    let mut sp = Spinner::new("Finding stake pool accounts");

    assert_exists!(&program, StakePool, pool);

    let StakePool {
        stake_pool_vault,
        stake_vote_mint,
        token_mint,
        ..
    } = program.account(*pool)?;

    sp.finish_with_message("Stake pool accounts retrieved");

    //
    sp = Spinner::new("Parsing governance realm data");

    let realm_data = parse_realm(&program, realm, &jet_spl_governance::ID)?;

    let governance_owner_record = get_token_owner_record_address(
        &jet_spl_governance::ID,
        realm,
        &realm_data.community_mint,
        &signer.pubkey(),
    );

    let governance_vault = derive_governance_vault(realm, &realm_data.community_mint);

    sp.finish_with_message("Realm discovered");

    let payer_token_account = get_associated_token_address(&signer.pubkey(), &token_mint);
    let voter_token_account = get_associated_token_address(&signer.pubkey(), &stake_vote_mint);
    let stake_account = derive_stake_account(pool, &signer.pubkey(), &program.id());

    sp = Spinner::new("Preprending required instructions");

    if !account_exists(&program, &stake_account)? {
        req = req.instruction(Instruction::new_with_borsh(
            program.id(),
            &instruction::InitStakeAccount {},
            accounts::InitStakeAccount {
                owner: signer.pubkey(),
                auth: derive_auth_account(&signer.pubkey(), &jet_auth::ID), // FIXME: genericize auth program ID
                stake_pool: *pool,
                stake_account,
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
            token_program,
        }
        .to_account_metas(None),
    ));

    ix_names.push("jet_staking::AddStake");

    if !skip_mint_votes {
        if !account_exists(&program, &voter_token_account)? {
            req = req.instruction(create_associated_token_account(
                &signer.pubkey(),
                &signer.pubkey(),
                &stake_vote_mint,
            ));

            ix_names.push("associated_token_program::Create");
        }

        req = req.instruction(Instruction::new_with_borsh(
            program.id(),
            &instruction::MintVotes { amount: None }, // FIXME: amount omitted to do full amount possible (?)
            accounts::MintVotes {
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
            }
            .to_account_metas(None),
        ));

        ix_names.push("jet_staking::MintVotes");

        req = req.instruction(anchor_spl::token::spl_token::instruction::close_account(
            &token_program,
            &voter_token_account,
            &signer.pubkey(),
            &signer.pubkey(),
            &[&signer.pubkey()],
        )?);

        ix_names.push("token_program::CloseAccount");
    }

    sp.finish_with_message("Instruction bytes compiled");

    send_with_approval(cfg, req.signer(signer.as_ref()), Some(ix_names))
}

/// The function handler for a user closing their staking account.
fn process_close_account(
    cfg: &Config,
    program_id: &Pubkey,
    pool: &Pubkey,
    receiver: &Option<Pubkey>,
) -> Result<()> {
    let (program, signer) = create_program_client(cfg, *program_id);

    // Derive the public key of the `jet_staking::StakeAccount` that
    // is being closed in the instruction call and assert that is exists
    let stake_account = derive_stake_account(pool, &signer.pubkey(), &program.id());

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
                stake_account,
            })
            .args(instruction::CloseStakeAccount {})
            .signer(signer.as_ref()),
        None,
    )
}

/// The function handler for the staking subcommand that allows users to create a
/// new staking account for a designated pool for themselves.
fn process_create_account(cfg: &Config, program_id: &Pubkey, pool: &Pubkey) -> Result<()> {
    let (program, signer) = create_program_client(cfg, *program_id);

    // Derive the public keys for the user's `jet_auth::UserAuthentication`
    // and `jet_staking::StakeAccount` program accounts and assert that the
    // stake account does not exist since this command creates one
    let auth = derive_auth_account(&signer.pubkey(), &program.id());
    let stake_account = derive_stake_account(pool, &signer.pubkey(), &program.id());

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
    program_id: &Pubkey,
    seed: String,
    token_mint: &Pubkey,
    unbond_period: u64,
) -> Result<()> {
    let (program, signer) = create_program_client(cfg, *program_id);

    // Derive the public keys needed for a new staking pool and
    // ensure that the pool address doesn't already exist
    let StakePoolAddresses {
        pool,
        vault,
        collateral_mint,
        vote_mint,
    } = derive_stake_pool(&seed, &program.id());

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
            .signer(signer.as_ref()),
        None,
    )
}

/// The function handler for the staking subcommand that allows users to
/// withdraw bonded stake from the designated pool.
fn process_withdraw_bonded(
    cfg: &Config,
    program_id: &Pubkey,
    amount: u64,
    pool: &Pubkey,
    receiver: &Option<Pubkey>,
) -> Result<()> {
    let (program, signer) = create_program_client(cfg, *program_id);

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
    program_id: &Pubkey,
    pool: &Pubkey,
    rent_receiver: &Option<Pubkey>,
    token_receiver: &Option<Pubkey>,
    unbonding_account: &Pubkey,
) -> Result<()> {
    let (program, signer) = create_program_client(cfg, *program_id);

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

/// Reads the account of the argued governance realm
/// public key and deserialize the account data bytes into a usable
/// instance of the `spl_governance::state::realm::Realm` struct.
fn parse_realm(program: &Program, realm: &Pubkey, gov_program: &Pubkey) -> Result<Realm> {
    let client = program.rpc();
    let account = client.get_account_with_commitment(realm, client.commitment())?;

    if account.value.is_none() {
        return Err(anyhow!(
            "realm {} not found for program {}",
            realm,
            gov_program
        ));
    }

    let acc_info = account.value.as_ref().unwrap();
    get_realm_data(
        gov_program,
        &AccountInfo::new(
            realm,
            false,
            false,
            &mut acc_info.lamports.clone(),
            &mut acc_info.data.clone(),
            &acc_info.owner,
            false,
            acc_info.rent_epoch,
        ),
    )
    .map_err(Into::into)
}
