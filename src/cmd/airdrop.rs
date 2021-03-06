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

use anchor_client::solana_client::rpc_filter::{Memcmp, MemcmpEncodedBytes, RpcFilterType};
use anchor_client::solana_sdk::pubkey::Pubkey;
use anchor_client::solana_sdk::signature::Signer;
use anchor_spl::token::ID as token_program;
use anyhow::Result;
use clap::Subcommand;
use jet_rewards::state::Airdrop;
use jet_rewards::{accounts, instruction};
use jet_staking::state::StakePool;

use super::staking::DEFAULT_STAKE_POOL;

use crate::config::{Config, Overrides};
use crate::program::{create_program_client, send_with_approval};
use crate::pubkey::{derive_stake_account, derive_voter_weight_record};
use crate::terminal::{print_serialized, DisplayOptions};

/// Rewards program based subcommand enum variants for airdrops.
#[derive(Debug, Subcommand)]
pub enum AirdropCommand {
    /// Get account data for airdrop account.
    Account {
        /// Base-58 public key of the account.
        #[clap(value_parser)]
        address: Pubkey,
        /// Output data as serialized JSON.
        #[clap(long, value_parser)]
        json: bool,
        /// Formatted data output.
        #[clap(long, value_parser)]
        pretty: bool,
    },
    /// Claim rewards airdrop.
    Claim {
        /// The public key of the target airdrop.
        #[clap(value_parser)]
        airdrop: Pubkey,
    },
    /// List all airdrops for a stake pool.
    List {
        /// Output data as serialized JSON.
        #[clap(long, value_parser)]
        json: bool,
        /// Formatted data output.
        #[clap(long, value_parser)]
        pretty: bool,
        /// The stake pool associated with the airdrop(s).
        #[clap(long, value_parser, default_value = DEFAULT_STAKE_POOL)]
        stake_pool: Pubkey,
    },
}

/// The main entry point and handler for all rewards
/// program interaction commands.
pub fn entry(overrides: &Overrides, program_id: &Pubkey, subcmd: &AirdropCommand) -> Result<()> {
    let cfg = Config::new(overrides, *program_id)?;
    match subcmd {
        AirdropCommand::Account {
            address,
            json,
            pretty,
        } => process_get_account(&cfg, address, DisplayOptions::from_args(*json, *pretty)),
        AirdropCommand::Claim { airdrop } => process_claim(&cfg, airdrop),
        AirdropCommand::List {
            json,
            pretty,
            stake_pool,
        } => process_list(&cfg, stake_pool, DisplayOptions::from_args(*json, *pretty)),
    }
}

/// The function handler to get the airdrop program account of the argued public key
/// and display the content in the terminal for observation.
fn process_get_account(cfg: &Config, address: &Pubkey, display: DisplayOptions) -> Result<()> {
    let (program, _) = create_program_client(cfg);
    print_serialized(program.account::<Airdrop>(*address)?, &display)
}

/// The function handler to allow a user to claim a rewards airdrop
/// that they provide the public key for.
fn process_claim(cfg: &Config, airdrop: &Pubkey) -> Result<()> {
    // Instantiate program clients for both jet_rewards and jet_staking programs
    let (rewards_program, signer) = create_program_client(cfg);
    let (staking_program, _) = create_program_client(&cfg.clone_with_program(jet_staking::ID)); // TODO: make configurable override (?)

    // Fetch the required program account data to retrieve PDAs required for instructions
    let Airdrop {
        reward_vault,
        stake_pool,
        ..
    } = rewards_program.account(*airdrop)?;

    let StakePool {
        stake_pool_vault,
        max_voter_weight_record,
        ..
    } = staking_program.account(stake_pool)?;

    // Derive public keys that required remotely stored PDAs
    let stake_account = derive_stake_account(&stake_pool, &signer.pubkey(), &staking_program.id());
    let voter_weight_record = derive_voter_weight_record(&stake_account, &staking_program.id());

    // Build and send the `jet_rewards::AirdropClaim` instruction
    send_with_approval(
        cfg,
        rewards_program
            .request()
            .accounts(accounts::AirdropClaim {
                airdrop: *airdrop,
                reward_vault,
                recipient: signer.pubkey(),
                receiver: signer.pubkey(),
                stake_pool,
                stake_pool_vault,
                stake_account,
                voter_weight_record,
                max_voter_weight_record,
                staking_program: staking_program.id(),
                token_program,
            })
            .args(instruction::AirdropClaim {})
            .signer(signer.as_ref()),
        vec!["jet_rewards::AirdropClaim"],
    )
}

/// The function handler for retrieving and displaying the list of airdrop accounts
/// discovered via their stake pool association with the provided public key.
fn process_list(cfg: &Config, pool: &Pubkey, display: DisplayOptions) -> Result<()> {
    let (program, _) = create_program_client(cfg);

    let filters = vec![
        RpcFilterType::DataSize(8 + std::mem::size_of::<Airdrop>() as u64),
        RpcFilterType::Memcmp(Memcmp {
            offset: 112,
            bytes: MemcmpEncodedBytes::Bytes(pool.to_bytes().to_vec()),
            encoding: None,
        }),
    ];

    let mut deserialized = Vec::<Airdrop>::new();
    for airdrop in program.accounts_lazy::<Airdrop>(filters)? {
        deserialized.push(airdrop?.1);
    }

    print_serialized(deserialized, &display)
}
