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
        let info = $program
            .rpc()
            .get_account_with_commitment(&$pubkey, CommitmentConfig::confirmed())?;

        if info.value.is_none() {
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
        let info = $program
            .rpc()
            .get_account_with_commitment(&$pubkey, CommitmentConfig::confirmed())?;

        if info.value.is_some() {
            return Err(anyhow!("Program account {} already exists", $pubkey));
        }
    }};
}
pub(crate) use assert_not_exists;

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
        let payer = Rc::new($config.keypair);
        (
            Client::new_with_options(
                $config.cluster,
                payer.clone(),
                CommitmentConfig::confirmed(),
            )
            .program($program),
            payer,
        )
    }};
}
pub(crate) use program_client;

/// Macro to create and wrap transaction calls with automatic error
/// transformation and logging of the confirmation signature.
///
/// # Example
///
/// Basic example without an instruction arguments.
///
/// ```
/// send_and_log!(
///     program,
///     my_program::accounts::Init {
///         authority: program.payer(),
///         system_program: system_program::ID,
///     },
///     signer
/// );
/// ```
///
/// You can also provide instruction arguments as a third parameter.
///
/// ```
/// send_and_log!(
///     program,
///     my_program::accounts::Init {
///         authority: program.payer(),
///         system_program: system_program::ID,
///     },
///     my_program::instruction::Init { value: 25 },
///     signer
/// );
/// ```
macro_rules! send_and_log {
    ($program:ident, $accs:expr, $signer:ident) => {{
        $program
            .request()
            .accounts($accs)
            .signer($signer.as_ref())
            .send()
            .map(|sig| println!("Signature: {}", sig))
            .map_err(Into::into)
    }};

    ($program:ident, $accs:expr, $args:expr, $signer:ident) => {{
        $program
            .request()
            .accounts($accs)
            .args($args)
            .signer($signer.as_ref())
            .send()
            .map(|sig| println!("Signature: {}", sig))
            .map_err(Into::into)
    }};
}
pub(crate) use send_and_log;
