use anchor_client::solana_sdk::pubkey::Pubkey;
use anchor_client::solana_sdk::signer::Signer;
use anchor_client::solana_sdk::system_program::ID as system_program;
use anyhow::Result;
use clap::Subcommand;
use jet_auth::{accounts, instruction, UserAuthentication};
use spinners::*;

use crate::config::ConfigOverride;
use crate::macros::{assert_not_exists, program_client};
use crate::pubkey::find_auth_address;
use crate::terminal::request_approval;

/// Auth program based subcommand enum variants.
#[derive(Debug, Subcommand)]
pub enum Command {
    #[clap(about = "Create a new auth account")]
    CreateAccount {},
}

/// The main entry point and handler for all auth
/// program interaction commands.
pub fn entry(cfg: &ConfigOverride, program_id: &Pubkey, subcmd: &Command) -> Result<()> {
    match subcmd {
        Command::CreateAccount {} => create_account(cfg, program_id),
    }
}

/// The function handler for the auth subcommand that allows
/// users to create a new authentication account for themselves.
fn create_account(overrides: &ConfigOverride, program_id: &Pubkey) -> Result<()> {
    let config = overrides.transform()?;
    request_approval(&config)?;

    let (program, signer) = program_client!(config, *program_id);

    // Derive the public key of the new authentication account
    // and ensure that it does not already exist
    let auth = find_auth_address(&signer.pubkey(), &program.id());
    assert_not_exists!(program, UserAuthentication, &auth);

    let sp = Spinner::new(Spinners::Dots, "Sending transaction".into());

    // Build and send the `jet_auth::CreateUserAuthentication` transaction
    let signature = program
        .request()
        .accounts(accounts::CreateUserAuthentication {
            user: signer.pubkey(),
            payer: signer.pubkey(),
            auth,
            system_program,
        })
        .args(instruction::CreateUserAuth {})
        .signer(signer.as_ref())
        .send()?;

    sp.stop_with_message("✅ Transaction confirmed!\n".into());

    if config.verbose {
        println!("Signature: {}\n", signature);
    }

    println!("Pubkey: {}", auth);

    Ok(())
}
