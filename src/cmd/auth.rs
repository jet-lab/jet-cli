use anchor_client::solana_sdk::pubkey::Pubkey;
use anchor_client::solana_sdk::signer::Signer;
use anchor_client::solana_sdk::system_program;
use anyhow::Result;
use clap::Subcommand;
use jet_auth::{accounts, instruction as args, UserAuthentication};
use spinners::*;

use crate::config::ConfigOverride;
use crate::macros::{assert_not_exists, program_client};
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

fn create_account(overrides: &ConfigOverride, program_id: &Pubkey) -> Result<()> {
    let config = overrides.transform()?;
    request_approval(&config)?;

    let (program, signer) = program_client!(config, *program_id);

    let auth = find_auth_address(&signer.pubkey(), &program.id());

    assert_not_exists!(program, UserAuthentication, &auth);

    let sp = Spinner::new(Spinners::Dots, "Sending transaction".into());

    let signature = program
        .request()
        .accounts(accounts::CreateUserAuthentication {
            user: signer.pubkey(),
            payer: signer.pubkey(),
            auth,
            system_program: system_program::ID,
        })
        .args(args::CreateUserAuth {})
        .signer(signer.as_ref())
        .send()?;

    sp.stop_with_message("✅ Transaction confirmed!\n".into());

    println!("Pubkey: {}", auth);

    if config.verbose {
        println!("Signature: {}", signature);
    }

    Ok(())
}

/// Derive the public key of a `jet_auth::UserAuthentication` program account.
pub(crate) fn find_auth_address(owner: &Pubkey, program: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[owner.as_ref()], program).0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derive_correct_auth_address() {
        let auth = find_auth_address(&Pubkey::default(), &jet_auth::ID);
        assert_eq!(
            auth.to_string(),
            "L2QDXAsEpjW1kmyCJSgJnifrMLa5UiG19AUFa83hZND"
        );
    }
}
