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

use anchor_client::solana_sdk::pubkey::Pubkey;
use anchor_spl::token::ID as token_program;
use anyhow::Result;
use clap::Subcommand;
use jet_margin::MarginAccount;
use jet_margin_pool::{accounts, instruction, MarginPool};

use crate::config::{Config, Overrides};
use crate::macros::assert_exists;
use crate::program::{create_program_client, send_with_approval};

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
