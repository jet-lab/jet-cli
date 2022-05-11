use anyhow::{anyhow, Result};
use dialoguer::Confirm;
use indicatif::{ProgressBar, ProgressStyle};
use serde::ser::Serialize;
use std::borrow::Cow;
use std::fmt::Debug;

use super::config::Config;

/// Wrapper utility struct for housing arguments provided in a command
/// regarding the desired display serialization options for a printed
/// program account data struct.
pub struct DisplayOptions {
    json: bool,
    pretty: bool,
}

impl DisplayOptions {
    /// Instantiate based on the values of the argument options.
    pub fn from_args(json: bool, pretty: bool) -> Self {
        Self { json, pretty }
    }
}

/// Internal wrapper for the `indicatif::ProgressBar`.
#[derive(Debug)]
pub struct Spinner(ProgressBar);

impl Spinner {
    /// Create a new `indicatif::ProgressBar` spinner with
    /// a standardized style and ticker.
    pub fn new(msg: impl Into<Cow<'static, str>>) -> Self {
        let pb = ProgressBar::new_spinner();
        pb.enable_steady_tick(80);

        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.blue} {msg}")
                .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
        );

        pb.set_message(msg);
        Self(pb)
    }

    /// End the spinner with a new completion message.
    pub fn finish_with_message(&self, msg: impl Into<Cow<'static, str>>) {
        self.0
            .set_style(ProgressStyle::default_spinner().template("✅ {msg}"));
        self.0.finish_with_message(msg);
    }
}

/// Standardize function for printing structs that implement both `std::fmt::Debug`
/// and `serde::ser::Serialize` (JSON) to be printed to the terminal is either format
/// with the option to be pretty printed.
pub fn print_serialized(s: impl Debug + Serialize, opts: &DisplayOptions) -> Result<()> {
    if opts.json {
        println!(
            "{}",
            if opts.pretty {
                serde_json::to_string_pretty(&s)?
            } else {
                serde_json::to_string(&s)?
            }
        );
        return Ok(());
    }

    if opts.pretty {
        println!("{:#?}", s);
    } else {
        println!("{:?}", s);
    }

    Ok(())
}

/// Provides the user a confirmation `(y/N)` option in their terminal
/// to request approval to sign and send the compiled transaction(s)
/// using the configured keypair that was discovered or pointed to
/// based on the auto-approval flag set/unset in the command.
///
/// This should be called prior to sending any transactions on
/// behalf of the end user.
pub fn request_approval(config: &Config, ixs: Option<Vec<&str>>) -> Result<()> {
    if let Some(names) = ixs {
        println!("Instructions to be processed:");
        names
            .iter()
            .enumerate()
            .for_each(|(i, ix)| println!("[{}] {}", i + 1, *ix));
        println!();
    }

    if config.auto_approved {
        return Ok(());
    }

    let approved = Confirm::new()
        .with_prompt("Do you want to approve this transaction?")
        .default(false)
        .interact()?;

    if !approved {
        return Err(anyhow!("Transaction aborted"));
    }

    Ok(())
}
