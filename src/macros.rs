/// Macro to assert that the argued public key exists on chain.
///
/// Performs an `RpcClient::get_account_with_commitment` call for
/// confirmed accounts using the provided program and public key
/// and will return an error if the account is not found or there
/// is an RPC call error during the process.
///
/// # Example
///
/// ```
/// let program = program_client!(config, jet_staking::ID);
/// let stake_account = find_staking_address(&pool, &owner);
/// assert_exists!(program, stake_account);
/// ```
macro_rules! assert_exists {
    ($program:ident, $pubkey:expr) => {{
        let __acc_info = $program
            .rpc()
            .get_account_with_commitment($pubkey, CommitmentConfig::confirmed())?;

        if __acc_info.value.is_none() {
            return Err(anyhow!("Program account {} does not exist", $pubkey));
        }
    }};
}
pub(crate) use assert_exists;

/// Macro to assert that the argued public key does not exist on chain.
///
/// Performs an `RpcClient::get_account_with_commitment` call for confirmed
/// accounts using the provided public key and returns an error if
/// the account is found or there is an RPC call error.
///
/// # Example
///
/// ```
/// let program = program_client!(config, jet_staking::ID);
/// let stake_account = find_staking_address(&pool, &owner);
/// assert_not_exists!(program, stake_account);
/// ```
macro_rules! assert_not_exists {
    ($program:ident, $pubkey:expr) => {{
        let __acc_info = $program
            .rpc()
            .get_account_with_commitment($pubkey, CommitmentConfig::confirmed())?;

        if __acc_info.value.is_some() {
            return Err(anyhow!("Program account {} already exists", $pubkey));
        }
    }};
}
pub(crate) use assert_not_exists;

/// Macro to read the account of the argued governance realm
/// public key and deserialize the account data bytes into a usable
/// instance of the `spl_governance::state::realm::Realm` struct.
///
/// # Example
///
/// ```
/// let realm_data = fetch_realm!(program, &realm_pubkey)?;
/// ```
macro_rules! fetch_realm {
    ($program:ident, $pk:expr) => {{
        let mut __realm_account = $program.rpc().get_account($pk)?;
        get_realm_data(
            &jet_staking::spl_governance::ID,
            &anchor_client::solana_sdk::account_info::AccountInfo::new(
                $pk,
                false,
                false,
                &mut __realm_account.lamports,
                &mut __realm_account.data,
                &__realm_account.owner,
                false,
                __realm_account.rent_epoch,
            ),
        )
    }};
}
pub(crate) use fetch_realm;

/// Macro to handle the instantiation of a program client and
/// the designating signer keypair for the argued config and program ID.
///
/// # Example
///
/// ```
/// let program = program_client!(config, jet_staking::ID);
/// ```
macro_rules! program_client {
    ($config:ident, $program:expr) => {{
        let __payer = std::rc::Rc::new($config.keypair);
        (
            anchor_client::Client::new_with_options(
                $config.cluster,
                __payer.clone(),
                anchor_client::solana_sdk::commitment_config::CommitmentConfig::confirmed(),
            )
            .program($program),
            __payer,
        )
    }};
}
pub(crate) use program_client;

#[cfg(test)]
mod tests {
    use crate::config::Config;
    use anchor_client::solana_sdk::signer::Signer;

    #[test]
    fn program_client_creates_instance() {
        let config = Config::default();
        let signer_pubkey = config.keypair.pubkey();
        let p = program_client!(config, jet_staking::ID);

        assert_eq!(p.0.id(), jet_staking::ID);
        assert_eq!(p.0.payer(), signer_pubkey);
        assert_eq!(p.1.pubkey(), signer_pubkey);
    }
}
