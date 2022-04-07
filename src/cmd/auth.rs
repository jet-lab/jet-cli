use anchor_client::solana_sdk::pubkey::Pubkey;
use anchor_client::solana_sdk::signer::Signer;
use anchor_client::solana_sdk::system_program::ID as system_program;
use anyhow::Result;
use clap::Subcommand;
use jet_auth::{accounts, instruction, UserAuthentication};

use crate::config::{Config, ConfigOverride};
use crate::macros::*;
use crate::program::{create_program_client, send_with_approval};
use crate::pubkey::derive_auth_account;

/// Auth program based subcommand enum variants.
#[derive(Debug, Subcommand)]
pub enum AuthCommand {
    /// Create a new auth account.
    CreateAccount {},
}

/// The main entry point and handler for all auth
/// program interaction commands.
pub fn entry(overrides: &ConfigOverride, program_id: &Pubkey, subcmd: &AuthCommand) -> Result<()> {
    let cfg = overrides.transform(*program_id)?;
    match subcmd {
        AuthCommand::CreateAccount {} => create_account(&cfg),
    }
}

/// The function handler for the auth subcommand that allows
/// users to create a new authentication account for themselves.
fn create_account(cfg: &Config) -> Result<()> {
    let (program, signer) = create_program_client(cfg);

    // Derive the public key of the new authentication account
    // and ensure that it does not already exist
    let auth = derive_auth_account(&signer.pubkey(), &program.id());
    assert_not_exists!(&program, UserAuthentication, &auth);

    // Build and send the `jet_auth::CreateUserAuthentication` transaction
    send_with_approval(
        cfg,
        program
            .request()
            .accounts(accounts::CreateUserAuthentication {
                user: signer.pubkey(),
                payer: signer.pubkey(),
                auth,
                system_program,
            })
            .args(instruction::CreateUserAuth {})
            .signer(signer.as_ref()),
        Some(vec!["jet_auth::CreateUserAuthentication"]),
    )?;

    println!("Pubkey: {}", auth);

    Ok(())
}
