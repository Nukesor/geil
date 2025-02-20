//! This module handles all ssh key related logic.

use std::{fs, path::PathBuf};

use anyhow::{Result, anyhow};

use crate::{cli::HooksCmd, state::State};

pub fn handle_hooks_command(state: State, cmd: HooksCmd) -> Result<()> {
    match cmd {
        HooksCmd::Add { repo_path, command } => add_hook(state, repo_path, command)?,
        HooksCmd::List => list_hooks(state),
        HooksCmd::Remove { repo_path } => remove_hook(state, repo_path)?,
    }

    Ok(())
}

fn add_hook(mut state: State, path: PathBuf, command: String) -> Result<()> {
    let path = fs::canonicalize(path)?;
    let repo = state
        .repo_at_path(&path)
        .ok_or(anyhow!("Unknown repository at path: {path:?}"))?;

    repo.hook = Some(command);
    state.save()?;

    Ok(())
}

fn list_hooks(state: State) {
    for repo in state.repositories {
        let Some(hook) = repo.hook else {
            continue;
        };
        println!("{:?}: {hook}\n", repo.path);
    }
}

fn remove_hook(mut state: State, path: PathBuf) -> Result<()> {
    let path = fs::canonicalize(path)?;
    let repo = state
        .repo_at_path(&path)
        .ok_or(anyhow!("Unknown repository at path: {path:?}"))?;

    repo.hook = None;
    state.save()?;

    Ok(())
}
