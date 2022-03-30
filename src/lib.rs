use anyhow::Result;
use clap::{AppSettings, Parser};

mod config;
mod staking;

use config::ConfigOverride;

/// The top level command line options parser for the binary.
#[derive(Debug, Parser)]
#[clap(version)]
#[clap(propagate_version = true)]
#[clap(global_setting(AppSettings::DeriveDisplayOrder))]
pub struct Opts {
    #[clap(flatten)]
    cfg: ConfigOverride,
    #[clap(subcommand)]
    command: Command,
}

/// The parser for the first level of commands that should
/// be based on the Jet Protocol programs that a user can
/// interact with via the command line tool.
#[derive(Debug, Parser)]
pub enum Command {
    Staking {
        #[clap(subcommand)]
        subcmd: staking::Command,
    },
}

/// Main handler function to root parse all commands and delegate
/// to the appropriate subcommand entrypoints.
pub fn run(opts: Opts) -> Result<()> {
    match opts.command {
        Command::Staking { subcmd } => staking::entry(&opts.cfg, &subcmd),
    }
}

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
