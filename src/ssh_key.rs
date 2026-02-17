//! This module handles all ssh key related logic.
use std::{
    fs::read_to_string,
    process::{Command, Stdio},
};

use anyhow::{Context, Result, bail};

use crate::config::GeilConfig;

/// Pre-load all keys that have been added by the user.
pub fn load_keys(config: &GeilConfig) -> Result<()> {
    if config.keys.is_empty() {
        return Ok(());
    }

    // Get the list of keys that're already added to ssh-agent.
    let output = Command::new("ssh-add")
        .arg("-L")
        .output()
        .context("Failed to get list of already added keys.")?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let known_keys: Vec<&str> = stdout.lines().collect();

    // Add all keys that aren't in there.
    for key in config.keys.iter() {
        let mut pub_key_path = key.path();
        pub_key_path.set_extension("pub");

        if !pub_key_path.exists() {
            bail!(
                "Couldn't find public key for key '{}' at path {pub_key_path:?}",
                key.name
            )
        }

        // Read the public key.
        let pub_key = read_to_string(pub_key_path).context("Couldn't read public key file.")?;

        // The key is already added, check the next one.
        if known_keys.contains(&pub_key.trim()) {
            continue;
        }

        // The key isn't loaded yet. Load it via ssh-add.
        Command::new("ssh-add")
            .arg(key.path())
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .output()
            .context(format!("Failed to add key '{}' to ssh-add.", &key.name))?;
    }

    Ok(())
}
