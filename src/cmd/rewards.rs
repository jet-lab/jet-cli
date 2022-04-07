use anchor_client::solana_sdk::pubkey::Pubkey;
use anchor_client::solana_sdk::signature::Signer;
use anchor_spl::token::ID as token_program;
use anyhow::Result;
use clap::Subcommand;
use jet_rewards::state::Airdrop;
use jet_rewards::{accounts, instruction};
use jet_staking::state::StakePool;

use crate::config::{Config, ConfigOverride};
use crate::program::{create_program_client, send_with_approval};
use crate::pubkey::{derive_stake_account, derive_voter_weight_record};

/// Rewards program based subcommand enum variants.
#[derive(Debug, Subcommand)]
pub enum RewardsCommand {
    /// Claim governance rewards airdrop.
    ClaimAirdrop {
        /// The public key of the target airdrop.
        airdrop: Pubkey,
    },
}

/// The main entry point and handler for all rewards
/// program interaction commands.
pub fn entry(
    overrides: &ConfigOverride,
    program_id: &Pubkey,
    subcmd: &RewardsCommand,
) -> Result<()> {
    let cfg = overrides.transform(*program_id)?;
    match subcmd {
        RewardsCommand::ClaimAirdrop { airdrop } => process_claim_airdrop(&cfg, airdrop),
    }
}

fn process_claim_airdrop(cfg: &Config, airdrop: &Pubkey) -> Result<()> {
    // Instantiate program clients for both jet_rewards and jet_staking programs
    let (rewards_program, signer) = create_program_client(cfg);
    let (staking_program, _) =
        create_program_client(&Config::from_with_program(cfg, jet_staking::ID)); // TODO: make configurable override (?)

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
        Some(vec!["jet_rewards::AirdropClaim"]),
    )
}
