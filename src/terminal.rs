use anyhow::{anyhow, Result};
use dialoguer::Confirm;
use indicatif::{ProgressBar, ProgressStyle};
use std::borrow::Cow;

use super::config::Config;

/// Internal wrapper for the `indicatif::ProgressBar`.
#[derive(Debug)]
pub(crate) struct Spinner(ProgressBar);

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

/// Provides the user a confirmation `(y/N)` option in their terminal
/// to request approval to sign and send the compiled transaction(s)
/// using the configured keypair that was discovered or pointed to
/// based on the auto-approval flag set/unset in the command.
///
/// This should be called prior to sending any transactions on
/// behalf of the end user.
pub(crate) fn request_approval(config: &Config, ixs: Option<Vec<&str>>) -> Result<()> {
    if let Some(names) = ixs {
        println!("Instructions to be processed:");
        for (i, ix) in names.iter().enumerate() {
            println!("[{}] {}", i, *ix);
        }
        println!();
    }

    if config.auto_approved {
        return Ok(());
    }

    let approved = Confirm::new()
        .with_prompt("Do you want to approve this transaction?")
        .default(false)
        .interact()?;

    if approved {
        return Ok(());
    }

    Err(anyhow!("Transaction aborted."))
}
