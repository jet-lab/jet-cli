use anchor_client::solana_sdk::pubkey::Pubkey;
use anchor_client::solana_sdk::signer::Signer;
use anchor_client::solana_sdk::system_program::ID as system_program;
use anyhow::Result;
use clap::Subcommand;
use jet_auth::{accounts, instruction, UserAuthentication};

use crate::config::{Config, Overrides};
use crate::macros::*;
use crate::program::{create_program_client, send_with_approval};
use crate::pubkey::derive_auth_account;
use crate::terminal::{print_serialized, DisplayOptions};

/// Auth program based subcommand enum variants.
#[derive(Debug, Subcommand)]
pub enum AuthCommand {
    /// Get the account data for the user's auth account.
    Account {
        /// Base-58 public key of the account.
        address: Option<Pubkey>,
        /// Output data as serialized JSON.
        #[clap(long)]
        json: bool,
        /// Base-58 public key of the account owner.
        #[clap(long, conflicts_with = "address")]
        owner: Option<Pubkey>,
        /// Formatted data output.
        #[clap(long)]
        pretty: bool,
    },
    /// Create a new auth account.
    CreateAccount {},
    /// Derive the public key of an auth account.
    Derive {
        /// Base-58 override of the account owner.
        #[clap(long)]
        owner: Option<Pubkey>,
    },
}

/// The main entry point and handler for all auth
/// program interaction commands.
pub fn entry(overrides: &Overrides, program_id: &Pubkey, subcmd: &AuthCommand) -> Result<()> {
    let cfg = Config::new(overrides, *program_id)?;
    match subcmd {
        AuthCommand::Account {
            address,
            json,
            owner,
            pretty,
        } => process_get_account(
            &cfg,
            address,
            owner,
            DisplayOptions::from_args(*json, *pretty),
        ),
        AuthCommand::CreateAccount {} => process_create_account(&cfg),
        AuthCommand::Derive { owner } => process_derive(&cfg, owner),
    }
}

/// The function handler to get the deserialized data for the derived
/// user auth account and display in the terminal for the user to observe.
fn process_get_account(
    cfg: &Config,
    address: &Option<Pubkey>,
    owner: &Option<Pubkey>,
    display: DisplayOptions,
) -> Result<()> {
    let (program, signer) = create_program_client(cfg);
    let owner_pk = owner.unwrap_or(signer.pubkey());
    let auth_account = address.unwrap_or(derive_auth_account(&owner_pk, &program.id()));
    print_serialized(
        program.account::<UserAuthentication>(auth_account)?,
        &display,
    )
}

/// The function handler for the auth subcommand that allows
/// users to create a new authentication account for themselves.
fn process_create_account(cfg: &Config) -> Result<()> {
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
        vec!["jet_auth::CreateUserAuthentication"],
    )?;

    println!("Pubkey: {}", auth);

    Ok(())
}

/// The function handler to derive the public key of a `jet_auth::UserAuthentication` program account.
fn process_derive(cfg: &Config, owner: &Option<Pubkey>) -> Result<()> {
    let acc_owner = owner.unwrap_or(cfg.keypair.pubkey());
    let pk = derive_auth_account(&acc_owner, &cfg.program_id);
    println!("{}", pk);
    Ok(())
}
