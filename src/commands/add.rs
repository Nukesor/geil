use std::path::PathBuf;

use anyhow::Result;
use log::error;

use crate::state::{Repository, State};

pub fn add(state: &mut State, repos: Vec<PathBuf>) -> Result<()> {
    // Just print the known repositories, if no arguments have been supplied.
    if repos.is_empty() {
        println!("Watched repositories:");
        for repo in state.repositories.iter() {
            println!("  - {:?}", repo.path);
        }
        return Ok(());
    }

    for path in repos {
        // Check if the directory to add actually exists
        if !path.exists() || !path.is_dir() {
            error!("Cannot find repository at {:?}", path);
        }

        // Store the absolute path.
        let real_path = std::fs::canonicalize(&path)?;
        if !state.has_repo_at_path(&real_path) {
            println!("Added repository: {:?}", &real_path);
            state.repositories.push(Repository::new(real_path));
        }
    }
    state.save()
}
