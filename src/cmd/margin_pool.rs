// Copyright (C) 2022 JET PROTOCOL HOLDINGS, LLC.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use anchor_client::solana_client::rpc_filter::RpcFilterType;
use anchor_client::solana_sdk::pubkey::Pubkey;
use anchor_spl::token::ID as token_program;
use anyhow::Result;
use clap::Subcommand;
use jet_margin::MarginAccount;
use jet_margin_pool::{accounts, instruction, MarginPool};

use crate::config::{Config, Overrides};
use crate::macros::assert_exists;
use crate::program::{create_program_client, send_with_approval};
use crate::pubkey::derive_margin_pool;
use crate::terminal::{print_serialized, DisplayOptions};

/// Margin pool program based subcommand enum variants.
#[derive(Debug, Subcommand)]
pub enum MarginPoolCommand {
    /// Borrow funds from a margin pool.
    Borrow {
        /// The token amount to borrow from the pool.
        amount: u64,
        /// Margin account to sign the borrow.
        #[clap(long)]
        account: Pubkey,
        /// Account to receive borrowed tokens.
        #[clap(long)]
        deposit_account: Pubkey,
        /// Account to receive the loan notes.
        #[clap(long)]
        loan_account: Pubkey,
        /// Target margin pool.
        #[clap(long)]
        pool: Pubkey,
    },
    /// Deposit into an existing margin pool.
    Deposit {
        /// The token amount to deposit into the pool.
        amount: u64,
        /// Margin account that owns the source token account.
        #[clap(long)]
        account: Pubkey,
        /// Destination token account address.
        #[clap(long)]
        destination: Pubkey,
        /// Target margin pool.
        #[clap(long)]
        pool: Pubkey,
        /// Fund source token account.
        #[clap(long)]
        source: Pubkey,
    },
    /// Derive the public key of a margin pool.
    Derive {
        /// The underlying token mint address.
        #[clap(long)]
        token_mint: Pubkey,
    },
    /// Get the account data for a margin pool or all that exist.
    Get {
        /// Public key of specific pool to get.
        address: Option<Pubkey>,
        /// Output data as serialized JSON.
        #[clap(long)]
        json: bool,
        /// Formatted data output.
        #[clap(long)]
        pretty: bool,
        /// Token mint to derive margin pool.
        #[clap(long, conflicts_with = "address")]
        token_mint: Option<Pubkey>,
    },
}

/// The main entry point and handler for all margin pool
/// program interaction commands.
pub fn entry(overrides: &Overrides, program_id: &Pubkey, subcmd: &MarginPoolCommand) -> Result<()> {
    let cfg = Config::new(overrides, *program_id)?;
    match subcmd {
        MarginPoolCommand::Borrow {
            amount,
            account,
            deposit_account,
            loan_account,
            pool,
        } => process_borrow(&cfg, *amount, account, deposit_account, loan_account, pool),
        MarginPoolCommand::Deposit {
            amount,
            account,
            destination,
            pool,
            source,
        } => process_deposit(&cfg, *amount, account, destination, pool, source),
        MarginPoolCommand::Derive { token_mint } => process_derive(&cfg, token_mint),
        MarginPoolCommand::Get {
            address,
            json,
            pretty,
            token_mint,
        } => process_get(
            &cfg,
            address,
            token_mint,
            DisplayOptions::from_args(*json, *pretty),
        ),
    }
}

/// The function handler to allow users to borrow tokens from a margin pool.
fn process_borrow(
    cfg: &Config,
    amount: u64,
    margin_account: &Pubkey,
    deposit_account: &Pubkey,
    loan_account: &Pubkey,
    margin_pool: &Pubkey,
) -> Result<()> {
    let (program, _) = create_program_client(cfg);

    assert_exists!(&program, MarginAccount, margin_account);
    assert_exists!(&program, MarginPool, margin_pool);

    let pool = program.account::<MarginPool>(*margin_pool)?;

    send_with_approval(
        cfg,
        program
            .request()
            .accounts(accounts::MarginBorrow {
                margin_account: *margin_account,
                margin_pool: *margin_pool,
                loan_note_mint: pool.loan_note_mint,
                deposit_note_mint: pool.deposit_note_mint,
                loan_account: *loan_account,
                deposit_account: *deposit_account,
                token_program,
            })
            .args(instruction::MarginBorrow { amount }),
        vec!["jet_margin_pool::MarginBorrow"],
    )
}

/// The function handler to allow users to deposit token funds into a margin pool.
fn process_deposit(
    cfg: &Config,
    amount: u64,
    destination_account: &Pubkey,
    margin_account: &Pubkey,
    margin_pool: &Pubkey,
    source_account: &Pubkey,
) -> Result<()> {
    let (program, signer) = create_program_client(cfg);

    assert_exists!(&program, MarginAccount, margin_account);
    assert_exists!(&program, MarginPool, margin_pool);

    let pool = program.account::<MarginPool>(*margin_pool)?;

    send_with_approval(
        cfg,
        program
            .request()
            .accounts(accounts::Deposit {
                margin_pool: *margin_pool,
                vault: pool.vault,
                deposit_note_mint: pool.deposit_note_mint,
                depositor: *margin_account,
                source: *source_account,
                destination: *destination_account,
                token_program,
            })
            .args(instruction::Deposit { amount })
            .signer(signer.as_ref()),
        vec!["jet_margin_pool::Deposit"],
    )
}

/// The function handler to derive the public key of a `jet_margin_pool::MarginPool`.
fn process_derive(cfg: &Config, token_mint: &Pubkey) -> Result<()> {
    println!("{}", derive_margin_pool(token_mint, &cfg.program_id));
    Ok(())
}

/// The function handler for fetching and displaying program account data for all existing
/// or a specific margin pool account.
fn process_get(
    cfg: &Config,
    address: &Option<Pubkey>,
    token_mint: &Option<Pubkey>,
    display: DisplayOptions,
) -> Result<()> {
    let (program, _) = create_program_client(cfg);

    if let Some(addr) = address {
        return print_serialized(program.account::<MarginPool>(*addr)?, &display);
    } else if let Some(tm) = token_mint {
        let pool = derive_margin_pool(tm, &program.id());
        return print_serialized(program.account::<MarginPool>(pool)?, &display);
    }

    let pools: Vec<MarginPool> = program
        .accounts::<MarginPool>(vec![RpcFilterType::DataSize(
            8 + std::mem::size_of::<MarginPool>() as u64,
        )])?
        .iter()
        .map(|acc| acc.1.to_owned())
        .collect();

    print_serialized(pools, &display)
}
