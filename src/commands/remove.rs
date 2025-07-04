use std::path::PathBuf;

use anyhow::Result;
use log::error;

use crate::state::State;

pub fn remove(state: &mut State, repos: Vec<PathBuf>) -> Result<()> {
    for path in repos {
        // Check if the directory to add actually exists
        if !path.exists() || !path.is_dir() {
            error!("Cannot find repository at {path:?}");
            continue;
        }

        // Store the absolute path.
        let real_path = std::fs::canonicalize(&path)?;
        if !state.has_repo_at_path(&real_path) {
            error!("The repository at {path:?} hasn't been added to geil yet.");
        } else {
            println!("Forgetting about repository: {:?}", &real_path);
            state.repositories.retain(|repo| repo.path != real_path);
        }
    }
    state.save()
}
