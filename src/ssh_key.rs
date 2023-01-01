//! This module handles all ssh key related logic.
use std::path::PathBuf;

use anyhow::{bail, Result};
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
