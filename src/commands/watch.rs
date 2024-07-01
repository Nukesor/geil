use std::path::PathBuf;

use anyhow::Result;
use log::error;

use crate::state::State;

pub fn watch(state: &mut State, directories: &[PathBuf]) -> Result<()> {
    // Just print the watched folders, if no arguments have been supplied.
    if directories.is_empty() {
        println!("Watched folders");
        for dir in state.watched.iter() {
            println!("  - {dir:?}");
        }
        return Ok(());
    }

    for path in directories {
        // Check if the directory to add actually exists
        if !path.exists() || !path.is_dir() {
            error!("Cannot find directory at {:?}", path);
        }

        // Store the absolute path.
        let real_path = std::fs::canonicalize(path)?;
        if !state.watched.contains(&real_path) {
            println!("Watching folder: {:?}", &real_path);
            state.watched.push(real_path);
        }
    }

    state.scan()
}
