use std::path::PathBuf;

use anyhow::Result;
use log::error;

use super::unwatch;
use crate::state::{State, discover};

/// Explicitly ignore a given directory.
/// No repositories will be updated or discovered, even if it's inside a watched directory.
pub fn ignore(state: &mut State, directories: &[PathBuf]) -> Result<()> {
    // First, unwatch in case any of them have been actively watched.
    // This also explicitly removes any repositories that were tracked.
    unwatch(state, directories)?;

    for path in directories.iter() {
        // Check if the directory to add actually exists
        if !path.exists() || !path.is_dir() {
            error!("Cannot find directory at {:?}", path);
            continue;
        }

        // Get the absolute path
        let real_path = std::fs::canonicalize(path)?;

        if state.ignored.contains(&real_path) {
            error!("The folder is already ignored: {:?}", &real_path);
            continue;
        }

        // Scan the watched path for repositories, so we can forget about them
        let mut repos = Vec::new();
        discover(&state.ignored, &real_path, 0, &mut repos);

        for repo_to_remove in repos {
            println!("Forgetting about repository: {:?}", repo_to_remove.path);
            state
                .repositories
                .retain(|repo| repo.path != repo_to_remove.path);
        }

        println!("Ignoring directory: {:?}", real_path);
        state.ignored.push(real_path.to_owned())
    }

    state.save()
}
