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
/// assert_exists!(program, jet_staking::state::StakeAccount, &stake_account);
/// ```
///
/// You can also provide a fallback block of code to execute in-place
/// of throwing an error on a bad assertion:
///
/// ```
/// let program = program_client!(config, jet_staking::ID);
/// let stake_account = find_staking_address(&pool, &owner);
/// assert_exists!(
///     program,
///     jet_staking::state::StakeAccount,
///     &stake_account,
///     {
///         println!("my fallback code block");
///     },
/// );
/// ```
macro_rules! assert_exists {
    ($program:expr, $acc_type:ty, $pubkey:expr $(,)?) => {{
        if !crate::accounts::account_exists($program, $pubkey)? {
            return Err(anyhow::anyhow!(
                "{} {} does not exist",
                std::any::type_name::<$acc_type>(),
                $pubkey
            ));
        }
    }};

    ($program:expr, $acc_type:ty, $pubkey:expr, $fallback:block $(,)?) => {{
        if !crate::accounts::account_exists($program, $pubkey)? {
            eprintln!(
                "{} {} does not exist",
                std::any::type_name::<$acc_type>(),
                $pubkey,
            );
            $fallback
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
/// assert_not_exists!(program, jet_staking::state::StakeAccount, &stake_account);
/// ```
///
/// You can also provide a fallback block of code to execute in-place
/// of throwing an error on a bad assertion:
///
/// ```
/// let program = program_client!(config, jet_staking::ID);
/// let stake_account = find_staking_address(&pool, &owner);
/// assert_not_exists!(
///     program,
///     jet_staking::state::StakeAccount,
///     &stake_account,
///     {
///         println!("my fallback code block");
///     },
/// );
/// ```
macro_rules! assert_not_exists {
    ($program:expr, $acc_type:ty, $pubkey:expr $(,)?) => {{
        if crate::accounts::account_exists($program, $pubkey)? {
            return Err(anyhow::anyhow!(
                "{} {} already exists",
                std::any::type_name::<$acc_type>(),
                $pubkey
            ));
        }
    }};

    ($program:expr, $acc_type:ty, $pubkey:expr, $fallback:block $(,)?) => {{
        if crate::accounts::account_exists($program, $pubkey)? {
            eprintln!(
                "{} {} already exists",
                std::any::type_name::<$acc_type>(),
                $pubkey,
            );

            $fallback
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
/// let realm_data = fetch_realm!(program, &jet_staking::spl_governance::ID, &realm_pubkey);
/// ```
macro_rules! fetch_realm {
    ($program:ident, $gov_program_id:expr, $pk:expr $(,)?) => {{
        let __client = $program.rpc();
        let mut __realm_account =
            __client.get_account_with_commitment($pk, __client.commitment())?;

        if __realm_account.value.is_none() {
            return Err(anyhow::anyhow!(
                "Error: realm {} not found for program {}",
                $pk,
                $gov_program_id,
            ));
        }

        get_realm_data(
            $gov_program_id,
            &anchor_client::solana_sdk::account_info::AccountInfo::new(
                $pk,
                false,
                false,
                &mut __realm_account.value.as_ref().unwrap().lamports.clone(),
                &mut __realm_account.value.as_ref().unwrap().data.clone(),
                &__realm_account.value.as_ref().unwrap().owner,
                false,
                __realm_account.value.as_ref().unwrap().rent_epoch,
            ),
        )?
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

/// Macro to wrap a sendable transaction expression to be
/// sent, confirmed and log the signature hash based on the
/// detected verbosity setting in the exposed configuration.
///
/// # Example
///
/// ```
/// send_tx! { |config|
///     program
///         .request()
///         .accounts(accounts::Init {})
///         .args(instruction::Init {})
///         .signer(signer.as_ref())
/// };
/// ```
macro_rules! send_tx {
    (|$cfg:ident| $exec:expr) => {{
        let __sp = spinners::Spinner::new(spinners::Spinners::Dots, "Sending transaction".into());
        let __signature = $exec.send()?;
        __sp.stop_with_message("✅ Transaction confirmed!\n".into());

        if $cfg.verbose {
            println!("Signature: {}", __signature);
        }
    }};
}
pub(crate) use send_tx;

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
