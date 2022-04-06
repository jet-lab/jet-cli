use anchor_client::solana_sdk::pubkey::Pubkey;
use anchor_client::Program;
use anyhow::Result;

/// Checks whether the account for the argued public key exists.
pub fn account_exists(program: &Program, public_key: &Pubkey) -> Result<bool> {
    let client = program.rpc();
    let info = client.get_account_with_commitment(public_key, client.commitment())?;
    Ok(info.value.is_some())
}
