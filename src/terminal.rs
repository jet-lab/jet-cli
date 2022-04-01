use anyhow::{anyhow, Result};
use dialoguer::Confirm;

use crate::config::Config;

/// Provides the user a confirmation `(y/N)` option in their terminal
/// to request approval to sign and send the compiled transaction(s)
/// using the configured keypair that was discovered or pointed to
/// based on the auto-approval flag set/unset in the command.
///
/// This should be called prior to sending any transactions on
/// behalf of the end user.
pub fn request_approval(cfg: &Config) -> Result<()> {
    if cfg.auto_approved {
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
