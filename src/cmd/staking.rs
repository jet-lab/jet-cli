use anchor_client::anchor_lang::ToAccountMetas;
use anchor_client::solana_sdk::commitment_config::CommitmentConfig;
use anchor_client::solana_sdk::instruction::Instruction;
use anchor_client::solana_sdk::pubkey::Pubkey;
use anchor_client::solana_sdk::signature::Signer;
use anchor_client::solana_sdk::{system_program, sysvar};
use anchor_spl::token;
use anyhow::{anyhow, Result};
use clap::Subcommand;
use jet_staking::spl_governance as jet_spl_governance;
use jet_staking::state::StakePool;
use jet_staking::{accounts, instruction as args};
use spinners::*;
use spl_associated_token_account::{create_associated_token_account, get_associated_token_address};
use spl_governance::state::realm::get_realm_data;
use spl_governance::state::token_owner_record::get_token_owner_record_address;

use crate::config::ConfigOverride;
use crate::macros::*;
use crate::terminal::request_approval;

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
    #[clap(about = "Create a new staking account")]
    CreateAccount {
        #[clap(long)]
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
            realm,
            token_account,
        } => add_stake(cfg, amount, pool, realm, token_account),
        Command::CreateAccount { pool } => create_account(cfg, pool),
    }
}

/// The function handler for the staking subcommand that allows users to add
/// stake to their designated staking account from an owned token account.
fn add_stake(
    overrides: &ConfigOverride,
    amount: &Option<u64>,
    pool: &Pubkey,
    realm: &Pubkey,
    token_account: &Pubkey,
) -> Result<()> {
    let config = overrides.transform()?;
    request_approval(&config)?;

    let (program, signer) = program_client!(config, jet_staking::ID);

    let mut sp = Spinner::new(Spinners::Dots, "Finding stake account and pool".into());

    // Derive the stake account address for the user and assert
    // the existence of the stake account and stake pool program account
    let stake_account = find_stake_account_address(pool, &signer.pubkey());
    assert_exists!(program, &stake_account);
    assert_exists!(program, pool);

    let StakePool {
        stake_pool_vault,
        stake_vote_mint,
        ..
    } = program.account(*pool)?;

    sp.stop_with_symbol("✅");

    sp = Spinner::new(Spinners::Dots, "Building prerequisite instructions".into());

    // Create the instruction for `jet_staking::AddStake` to prepend the vote minting
    let add_stake_ix = Instruction::new_with_borsh(
        jet_staking::ID,
        &args::AddStake { amount: *amount },
        accounts::AddStake {
            stake_pool: *pool,
            stake_pool_vault,
            stake_account,
            payer: signer.pubkey(),
            payer_token_account: *token_account,
            token_program: token::ID,
        }
        .to_account_metas(None),
    );

    // Derive the public key of the user's voter token account and if the account doesn't exist,
    // prepend the transaction with the creation of the associated token account for them
    let voter_token_account = get_associated_token_address(&signer.pubkey(), &stake_vote_mint);

    let mut req = program.request();

    if program
        .rpc()
        .get_account_with_commitment(&voter_token_account, CommitmentConfig::confirmed())?
        .value
        .is_none()
    {
        req = req.instruction(create_associated_token_account(
            &signer.pubkey(),
            &signer.pubkey(),
            &stake_vote_mint,
        ));
    }

    // Append the instruction for `jet_staking::AddStake` to the transaction
    req = req.instruction(add_stake_ix);

    // Read and deserialize the realm account bytes from on-chain
    let realm_data = fetch_realm!(program, realm)?;

    // Derive the public keys for the governance token owner record
    // and the relevant governance token vault accounts.
    let governance_owner_record = get_token_owner_record_address(
        &jet_spl_governance::ID,
        realm,
        &realm_data.community_mint,
        &signer.pubkey(),
    );

    let governance_vault = find_governance_vault_address(realm, &realm_data.community_mint);

    sp.stop_with_symbol("✅");

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
            token_program: token::ID,
            system_program: system_program::ID,
            rent: sysvar::rent::ID,
        })
        .args(args::MintVotes { amount: None })
        .signer(signer.as_ref())
        .send()?;

    sp.stop_with_message("✅ Transaction confirmed!".into());

    if config.verbose {
        println!("Signature: {}", signature);
    }

    Ok(())
}

/// The function handler for the staking subcommand that allows users to create a
/// new staking account for a designated pool for themselves.
fn create_account(overrides: &ConfigOverride, pool: &Pubkey) -> Result<()> {
    let config = overrides.transform()?;
    let signer_pubkey = config.keypair.pubkey();
    request_approval(&config)?;

    // Derive the public keys for the user's `jet_auth::UserAuthentication`
    // and `jet_staking::StakeAccount` program accounts and assert that the
    // stake account does not exist since this command creates one
    let auth = find_auth_address(&signer_pubkey);
    let stake_account = find_stake_account_address(pool, &signer_pubkey);

    let (program, signer) = program_client!(config, jet_staking::ID);

    assert_not_exists!(program, &stake_account);

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
            system_program: system_program::ID,
        })
        .signer(signer.as_ref())
        .send()?;

    sp.stop_with_message("✅ Transaction confirmed!".into());

    if config.verbose {
        println!("Signature: {}", signature);
    }

    Ok(())
}

/// Derive the public key of a `jet_auth::UserAuthentication` program account.
fn find_auth_address(owner: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[owner.as_ref()], &jet_auth::ID).0
}

/// Derive the public key of a governance token vault program account.
fn find_governance_vault_address(realm: &Pubkey, mint: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &["governance".as_ref(), realm.as_ref(), mint.as_ref()],
        &jet_spl_governance::ID,
    )
    .0
}

/// Derive the public key of a `jet_staking::StakeAccount` program account.
fn find_stake_account_address(stake_pool: &Pubkey, owner: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[stake_pool.as_ref(), owner.as_ref()], &jet_staking::ID).0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derive_correct_auth_address() {
        let auth = find_auth_address(&Pubkey::default());
        assert_eq!(
            auth.to_string(),
            "L2QDXAsEpjW1kmyCJSgJnifrMLa5UiG19AUFa83hZND"
        );
    }

    #[test]
    fn derive_correct_governance_vault_address() {
        let vault = find_governance_vault_address(&Pubkey::default(), &Pubkey::default());
        assert_eq!(
            vault.to_string(),
            "J3JXRJuUMRRASSVc6jrvGQL3UwnPDR6F6x42rCak4ex6"
        );
    }

    #[test]
    fn derive_correct_staking_address() {
        let staking = find_stake_account_address(&Pubkey::default(), &Pubkey::default());
        assert_eq!(
            staking.to_string(),
            "3c7McYaJYNGR5jNyxgudWejKMebRZL4AoFPSuNKp9Dsq"
        );
    }
}
