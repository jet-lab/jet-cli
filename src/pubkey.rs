use anchor_client::solana_sdk::pubkey::Pubkey;

#[derive(Debug)]
pub struct StakePoolAddresses {
    pub pool: Pubkey,
    pub vault: Pubkey,
    pub collateral_mint: Pubkey,
}

/// Derive the public key of a `jet_auth::UserAuthentication` program account.
pub fn derive_auth_account(owner: &Pubkey, program: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[owner.as_ref()], program).0
}

/// Derive the public key of a governance max vote weight record program account.
pub fn derive_max_voter_weight_record(realm: &Pubkey, stake_program: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[realm.as_ref(), b"max-vote-weight-record"], stake_program).0
}

/// Derive the public key of a `jet_staking::state::StakeAccount` program account.
pub fn derive_stake_account(stake_pool: &Pubkey, owner: &Pubkey, stake_program: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[stake_pool.as_ref(), owner.as_ref()], stake_program).0
}

/// Derive all the necessary public keys for creating a new
/// `jet_staking::state::StakePool` program account.
pub fn derive_stake_pool(seed: &str, stake_program: &Pubkey) -> StakePoolAddresses {
    StakePoolAddresses {
        pool: Pubkey::find_program_address(&[seed.as_ref()], stake_program).0,
        vault: Pubkey::find_program_address(&[seed.as_ref(), b"vault"], stake_program).0,
        collateral_mint: Pubkey::find_program_address(
            &[seed.as_ref(), b"collateral-mint"],
            stake_program,
        )
        .0,
    }
}

/// Derive the public key of a governance voter weight record program account.
pub fn derive_voter_weight_record(stake_account: &Pubkey, stake_program: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &[b"voter-weight-record", stake_account.as_ref()],
        stake_program,
    )
    .0
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

    // TODO: test derive_max_voter_weight_record

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

    // TODO: test derive_voter_weight_record
}
