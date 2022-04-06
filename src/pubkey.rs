use anchor_client::solana_sdk::pubkey::Pubkey;
use jet_staking::spl_governance as jet_spl_governance;

#[derive(Debug)]
pub struct StakePoolAddresses {
    pub pool: Pubkey,
    pub vault: Pubkey,
    pub collateral_mint: Pubkey,
    pub vote_mint: Pubkey,
}

/// Derive the public key of a `jet_auth::UserAuthentication` program account.
pub fn derive_auth_account(owner: &Pubkey, program: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[owner.as_ref()], program).0
}

/// Derive the public key of a governance token vault program account.
pub fn derive_governance_vault(realm: &Pubkey, mint: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &["governance".as_ref(), realm.as_ref(), mint.as_ref()],
        &jet_spl_governance::ID,
    )
    .0
}

/// Derive the public key of a `jet_staking::state::StakeAccount` program account.
pub fn derive_stake_account(stake_pool: &Pubkey, owner: &Pubkey, program: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[stake_pool.as_ref(), owner.as_ref()], program).0
}

/// Derive all the necessary public keys for creating a new
/// `jet_staking::state::StakePool` program account.
pub fn derive_stake_pool(seed: &str, program: &Pubkey) -> StakePoolAddresses {
    StakePoolAddresses {
        pool: Pubkey::find_program_address(&[seed.as_ref()], program).0,
        vault: Pubkey::find_program_address(&[seed.as_ref(), b"vault"], program).0,
        collateral_mint: Pubkey::find_program_address(
            &[seed.as_ref(), b"collateral-mint"],
            program,
        )
        .0,
        vote_mint: Pubkey::find_program_address(&[seed.as_ref(), b"vote-mint"], program).0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derive_correct_auth_address() {
        let auth = derive_auth_account(&Pubkey::default(), &jet_auth::ID);
        assert_eq!(
            auth.to_string(),
            "L2QDXAsEpjW1kmyCJSgJnifrMLa5UiG19AUFa83hZND"
        );
    }

    #[test]
    fn derive_correct_governance_vault_address() {
        let vault = derive_governance_vault(&Pubkey::default(), &Pubkey::default());
        assert_eq!(
            vault.to_string(),
            "J3JXRJuUMRRASSVc6jrvGQL3UwnPDR6F6x42rCak4ex6"
        );
    }

    #[test]
    fn derive_correct_staking_address() {
        let staking =
            derive_stake_account(&Pubkey::default(), &Pubkey::default(), &jet_staking::ID);
        assert_eq!(
            staking.to_string(),
            "3c7McYaJYNGR5jNyxgudWejKMebRZL4AoFPSuNKp9Dsq"
        );
    }

    // TODO: test derive_stake_pool
}
