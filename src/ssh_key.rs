//! This module handles all ssh key related logic.
use std::{
    fs::read_to_string,
    path::PathBuf,
    process::{Command, Stdio},
};

use anyhow::{Context, Result, bail};
use comfy_table::{ContentArrangement, Table};

use crate::{
    cli::KeysCmd,
    state::{SshKey, State},
};

pub fn handle_key_command(state: State, cmd: KeysCmd) -> Result<()> {
    match cmd {
        KeysCmd::Add { name, path } => add_key(state, name, path)?,
        KeysCmd::List => list_keys(state),
        KeysCmd::Remove { name } => remove_key(state, name)?,
    }

    Ok(())
}

pub fn add_key(mut state: State, name: String, path: PathBuf) -> Result<()> {
    // Check if we already have a key with this name or path.
    for key in state.keys.iter() {
        if name == key.name {
            bail!("There already exists a key with this name.");
        } else if path == key.path {
            bail!("There already exists a key for this path.");
        }
    }

    if !path.exists() {
        bail!("Couldn't find a key file at path {path:?}.");
    }

    // Add the key and save the state.
    state.keys.push(SshKey { name, path });
    state.save()?;

    Ok(())
}

pub fn list_keys(state: State) {
    if state.keys.is_empty() {
        println!("There have been no keys added yet.");
        return;
    }

    let mut table = Table::new();
    table.set_content_arrangement(ContentArrangement::Dynamic);
    table.load_preset(comfy_table::presets::UTF8_FULL);

    table.set_header(vec!["Name", "Path"]);
    for key in state.keys {
        table.add_row([key.name, key.path.to_string_lossy().to_string()]);
    }

    println!("{table}")
}

pub fn remove_key(mut state: State, name: String) -> Result<()> {
    // Check if we have a key with this name
    let mut key_index = None;
    for (index, key) in state.keys.iter().enumerate() {
        if name == key.name {
            key_index = Some(index);
            break;
        }
    }

    let Some(key_index) = key_index else {
        bail!("There's no key with name name '{name}'");
    };

    // Remove the key
    state.keys.remove(key_index);
    state.save()?;

    Ok(())
}

/// Pre-load all keys that have been added by the user.
pub fn load_keys(state: &State) -> Result<()> {
    if state.keys.is_empty() {
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
    for key in state.keys.iter() {
        let mut pub_key_path = key.path.clone();
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
            .arg(&key.path)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .output()
            .context(format!("Failed to add key '{}' to ssh-add.", &key.name))?;
    }

    Ok(())
}
