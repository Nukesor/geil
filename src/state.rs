use std::fs::{read_dir, File};
use std::io::Write;
use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use serde_derive::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct State {
    pub watched: Vec<PathBuf>,
    pub repositories: Vec<PathBuf>,
}

impl State {
    pub fn new() -> State {
        State {
            watched: Vec::new(),
            repositories: Vec::new(),
        }
    }
}

impl State {
    /// Save a state to the disk.
    pub fn save(&self) -> Result<()> {
        let serialized = bincode::serialize(self)
            .context("Failed to serialize state. Please report this bug")?;

        let path = default_cache_path()?;
        let mut file = File::create(path)?;

        file.write_all(&serialized)?;

        Ok(())
    }

    /// Load an existing state from the disk or create an empty new one.
    pub fn load() -> Result<State> {
        let path = default_cache_path()?;
        // Return default path if it doesn't exist yet
        if !path.exists() {
            return Ok(State::new());
        }

        let file = File::open(path)?;
        let state = bincode::deserialize_from(&file)?;

        Ok(state)
    }

    pub fn scan(&mut self) -> Result<()> {
        // Go through all watched folder and check if they still exist
        for key in self.watched.len()..0 {
            if !self.watched[key].exists() || !self.watched[key].is_dir() {
                println!(
                    "Watched folder does no longer exist: {:?}",
                    &self.watched[key]
                );
                self.watched.remove(key);
            }
        }

        // Go through all repositories and check if they still exist
        for key in self.repositories.len()..0 {
            if !self.repositories[key].exists() || !self.repositories[key].is_dir() {
                println!(
                    "Repository does no longer exist: {:?}",
                    &self.repositories[key]
                );
                self.repositories.remove(key);
            }
        }

        // Do a full repository discovery on all watched repositories
        for watched in &self.watched.clone() {
            self.discover(watched, 0)?;
        }

        self.save()?;

        Ok(())
    }

    pub fn discover(&mut self, path: &PathBuf, depths: usize) -> Result<()> {
        // Check if a .git directory exists.
        // If it does, always stop searching.
        let git_dir = path.join(".git");
        if git_dir.exists() {
            // Add the repository, if we don't know it yet.
            if !self.repositories.contains(path) {
                self.repositories.push(path.clone());
            }
            return Ok(());
        }

        // Recursion stop. Only check up to a dephts of 5
        if depths == 5 {
            return Ok(());
        }

        // The current path is no repository, search it's subdirectories
        for dir_result in read_dir(path)? {
            let dir = dir_result?.path();
            if !dir.is_dir() {
                continue;
            }
            self.discover(&dir, depths + 1)?;
        }

        Ok(())
    }
}

fn default_cache_path() -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| anyhow!("Couldn't resolve home dir"))?;
    let path = home.join(".local/share/geil");
    Ok(path)
}
