use anchor_client::solana_sdk::pubkey::Pubkey;

#[derive(Debug)]
pub struct StakePoolAddresses {
    pub pool: Pubkey,
    pub vault: Pubkey,
    pub collateral_mint: Pubkey,
}

/// Derive the public key of a `jet_auth::UserAuthentication` program account.
pub fn derive_auth_account(owner: &Pubkey, auth_program: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[owner.as_ref()], auth_program).0
}

/// Derive the public key of a `jet_margin::MarginAccount` program account.
pub fn derive_margin_account(owner: &Pubkey, seed: u16, margin_program: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &[owner.as_ref(), seed.to_le_bytes().as_ref()],
        margin_program,
    )
    .0
}

/// Derive the public key of a governance max vote weight record program account.
pub fn derive_max_voter_weight_record(realm: &Pubkey, staking_program: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &[realm.as_ref(), jet_staking::seeds::MAX_VOTE_WEIGHT_RECORD],
        staking_program,
    )
    .0
}

/// Derive the public key of a `jet_staking::state::StakeAccount` program account.
pub fn derive_stake_account(
    stake_pool: &Pubkey,
    owner: &Pubkey,
    staking_program: &Pubkey,
) -> Pubkey {
    Pubkey::find_program_address(&[stake_pool.as_ref(), owner.as_ref()], staking_program).0
}

/// Derive all the necessary public keys for creating a new
/// `jet_staking::state::StakePool` program account.
pub fn derive_stake_pool(seed: &str, staking_program: &Pubkey) -> StakePoolAddresses {
    StakePoolAddresses {
        pool: Pubkey::find_program_address(&[seed.as_ref()], staking_program).0,
        vault: Pubkey::find_program_address(
            &[seed.as_ref(), jet_staking::seeds::VAULT],
            staking_program,
        )
        .0,
        collateral_mint: Pubkey::find_program_address(
            &[seed.as_ref(), jet_staking::seeds::COLLATERAL_MINT],
            staking_program,
        )
        .0,
    }
}

/// Derive the public key of a governance voter weight record program account.
pub fn derive_voter_weight_record(stake_account: &Pubkey, stake_program: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &[
            jet_staking::seeds::VOTER_WEIGHT_RECORD,
            stake_account.as_ref(),
        ],
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

    #[test]
    fn derive_correct_margin_address() {
        let margin = derive_margin_account(&Pubkey::default(), 15, &jet_margin::ID);
        assert_eq!(
            margin.to_string(),
            "F8VbfbXdyeTonhEj2hs3mNb8VUpoYd78wPreLQSqRzj8"
        );
    }

    #[test]
    fn derive_correct_max_vote_weight_record() {
        let record = derive_max_voter_weight_record(&Pubkey::default(), &jet_staking::ID);
        assert_eq!(
            record.to_string(),
            "AwJBGLSw1ZKSjnc1o4eoibgUrNUrA6m1ucdRKoRPVmjK"
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

    #[test]
    fn derive_correct_stake_pool_addrs() {
        let addrs = derive_stake_pool("sample", &jet_staking::ID);

        assert_eq!(
            addrs.pool.to_string(),
            "8t8jY9M3jTaEWwWPwJ7CtTEZ7hHwRmojT1Smam93Yu3o"
        );

        assert_eq!(
            addrs.collateral_mint.to_string(),
            "3cmvamUuqAVVTyhuY6RbFnVm42ZHRYPuGn27yEmE1rut"
        );

        assert_eq!(
            addrs.vault.to_string(),
            "GoUSrowwjgV4ysBhDNTg44AkvaMdeJJC39e2qhMf21NY"
        );
    }

    #[test]
    fn derive_correct_voter_weight_record() {
        let record = derive_voter_weight_record(&Pubkey::default(), &jet_staking::ID);
        assert_eq!(
            record.to_string(),
            "HtqjKEAntMPicb7TD3UhTbTzu4iAJSHZpUfwdQQn5TvQ"
        );
    }
}
