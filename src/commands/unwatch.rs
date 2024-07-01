use std::path::PathBuf;

use anyhow::Result;
use log::error;

use crate::state::{discover, State};

pub fn unwatch(state: &mut State, directories: &[PathBuf]) -> Result<()> {
    for path in directories {
        // Check if the directory to add actually exists
        if !path.exists() || !path.is_dir() {
            error!("Cannot find directory at {:?}", path);
            continue;
        }

        // Get the absolute path
        let real_path = std::fs::canonicalize(path)?;
        if !state.watched.contains(&real_path) {
            error!("The folder hasn't been watched: {:?}", &real_path);
        } else {
            println!("Unwatching path : {:?}", &real_path);
            state.watched.retain(|path| path != &real_path);

            // Scan the watched path for repositories, so we can forget about them
            let mut repos = Vec::new();
            discover(&state.ignored, &real_path, 0, &mut repos);

            for repo_to_remove in repos {
                println!("Forgetting about repository: {:?}", repo_to_remove.path);
                state
                    .repositories
                    .retain(|repo| repo.path != repo_to_remove.path);
            }
        }
    }

    state.save()
}
