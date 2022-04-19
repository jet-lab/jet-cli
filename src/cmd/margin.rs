use anchor_client::solana_client::rpc_filter::{Memcmp, MemcmpEncodedBytes, RpcFilterType};
use anchor_client::solana_sdk::pubkey::Pubkey;
use anchor_client::solana_sdk::signer::Signer;
use anchor_client::solana_sdk::system_program::ID as system_program;
use anyhow::Result;
use clap::Subcommand;
use jet_margin::{accounts, instruction, MarginAccount};

use crate::config::{Config, ConfigOverride};
use crate::macros::{assert_exists, assert_not_exists};
use crate::program::{create_program_client, send_with_approval};
use crate::pubkey::derive_margin_account;
use crate::terminal::{print_serialized, DisplayOptions};

/// Margin program based subcommand enum variants.
#[derive(Debug, Subcommand)]
pub enum MarginCommand {
    /// Get the account data for a user's margin account.
    Account {
        /// Base-58 public key of the margin account.
        address: Option<Pubkey>,
        /// Output data as serialized JSON.
        #[clap(long)]
        json: bool,
        /// Base-58 public key of the owner to use to derive.
        #[clap(long, conflicts_with = "address")]
        owner: Option<Pubkey>,
        /// Formatted data output.
        #[clap(long)]
        pretty: bool,
    },
    /// Check the health of the positions in a margin account.
    Check {
        /// Base-58 public key of the margin account.
        address: Pubkey,
    },
    /// Close your margin account.
    CloseAccount {
        /// (Optional) The public key to receive the rent.
        #[clap(long)]
        receiver: Option<Pubkey>,
        /// The numerical seed for the account to close.
        #[clap(short, long)]
        seed: u16,
    },
    /// Create a new margin account.
    CreateAccount {
        /// The numerical seed for the new account.
        #[clap(short, long)]
        seed: u16,
    },
}

/// The main entry point and handler for all margin
/// program interaction commands.
pub fn entry(
    overrides: &ConfigOverride,
    program_id: &Pubkey,
    subcmd: &MarginCommand,
) -> Result<()> {
    let cfg = overrides.transform(*program_id)?;
    match subcmd {
        MarginCommand::Account {
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
        MarginCommand::Check { address } => process_check_health(&cfg, address),
        MarginCommand::CloseAccount { receiver, seed } => {
            process_close_account(&cfg, receiver, *seed)
        }
        MarginCommand::CreateAccount { seed } => process_create_account(&cfg, *seed),
    }
}

/// The function handler to fetch the margin program account data for the derive public key
/// and display it in the terminal for the user to observe or parse.
fn process_get_account(
    cfg: &Config,
    address: &Option<Pubkey>,
    owner: &Option<Pubkey>,
    display: DisplayOptions,
) -> Result<()> {
    let (program, signer) = create_program_client(cfg);
    let owner_pk = owner.unwrap_or(signer.pubkey());

    if let Some(addr) = address {
        return print_serialized(program.account::<MarginAccount>(*addr)?, &display);
    }

    let margins: Vec<MarginAccount> = program
        .accounts(vec![
            RpcFilterType::DataSize(8 + std::mem::size_of::<MarginAccount>() as u64),
            RpcFilterType::Memcmp(Memcmp {
                offset: 16,
                bytes: MemcmpEncodedBytes::Bytes(owner_pk.to_bytes().to_vec()),
                encoding: None,
            }),
        ])?
        .iter()
        .map(|acc| acc.1)
        .collect();

    print_serialized(margins, &display)
}

/// The function handler to verify the health of the positions in a margin
/// account via the `jet_margin::VerifyHealthy` transaction.
fn process_check_health(cfg: &Config, address: &Pubkey) -> Result<()> {
    let (program, _) = create_program_client(cfg);

    send_with_approval(
        cfg,
        program
            .request()
            .accounts(accounts::VerifyHealthy {
                margin_account: *address,
            })
            .args(instruction::VerifyHealthy {}),
        Some(vec!["jet_margin::VerifyHealthy"]),
    )?;

    println!("Healthy!");

    Ok(())
}

/// The function handler to allow user's to close their margin account and receive back rent.
fn process_close_account(cfg: &Config, receiver: &Option<Pubkey>, seed: u16) -> Result<()> {
    let (program, signer) = create_program_client(cfg);
    let rent_receiver = receiver.unwrap_or(signer.pubkey());

    // Derive the target margin account and assert that it exists on-chain
    let margin_account = derive_margin_account(&signer.pubkey(), seed, &program.id());
    assert_exists!(&program, MarginAccount, &margin_account);

    // Build and send `jet_margin::CloseAccount` transaction
    send_with_approval(
        cfg,
        program
            .request()
            .accounts(accounts::CloseAccount {
                owner: signer.pubkey(),
                receiver: rent_receiver,
                margin_account,
            })
            .args(instruction::CloseAccount {})
            .signer(signer.as_ref()),
        Some(vec!["jet_margin::CloseAccount"]),
    )?;

    Ok(())
}

/// The function handler for a user to create a new margin account for themselves.
fn process_create_account(cfg: &Config, seed: u16) -> Result<()> {
    let (program, signer) = create_program_client(cfg);

    // Derive the public key for the margin account and assert that is does not already exist
    let margin_account = derive_margin_account(&signer.pubkey(), seed, &program.id());
    assert_not_exists!(&program, MarginAccount, &margin_account);

    // Build and send the `jet_margin::CreateAccount` transaction
    send_with_approval(
        cfg,
        program
            .request()
            .accounts(accounts::CreateAccount {
                owner: signer.pubkey(),
                payer: signer.pubkey(),
                margin_account,
                system_program,
            })
            .args(instruction::CreateAccount { seed })
            .signer(signer.as_ref()),
        Some(vec!["jet_margin::CreateAccount"]),
    )?;

    println!("Pubkey: {}", margin_account);

    Ok(())
}
