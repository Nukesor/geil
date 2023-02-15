use std::fs::{read_dir, File};
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use log::debug;
use serde_derive::{Deserialize, Serialize};
use serde_with::{serde_as, DefaultOnError};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Repository {
    /// The path to the repository
    pub path: PathBuf,
    /// The time it took to check this repository in the last run.
    pub check_time: Option<usize>,
}

impl Repository {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            check_time: None,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SshKey {
    /// The name of the key
    pub name: String,
    /// The path to the private key.
    pub path: PathBuf,
}

#[serde_as]
#[derive(Deserialize, Serialize)]
pub struct State {
    pub watched: Vec<PathBuf>,
    #[serde_as(deserialize_as = "DefaultOnError")]
    pub repositories: Vec<Repository>,
    #[serde(default = "Default::default")]
    pub keys: Vec<SshKey>,
}

impl State {
    pub fn new() -> State {
        State {
            watched: Vec::new(),
            repositories: Vec::new(),
            keys: Vec::new(),
        }
    }
}

impl State {
    /// Save a state to the disk.
    pub fn save(&self) -> Result<()> {
        let path = default_cache_path()?;
        let file = File::create(path)?;

        serde_cbor::to_writer(file, &self).context("Failed to write state to disk:")?;

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
        let state = serde_cbor::from_reader(file)?;

        Ok(state)
    }

    pub fn scan(&mut self) -> Result<()> {
        // Go through all watched folder and check if they still exist
        for key in (0..self.watched.len()).rev() {
            if !self.watched[key].exists() || !self.watched[key].is_dir() {
                println!(
                    "Watched folder does no longer exist: {:?}",
                    &self.watched[key]
                );
                self.watched.remove(key);
            }
        }

        // Go through all repositories and check if they still exist
        for key in (0..self.repositories.len()).rev() {
            if !self.repositories[key].path.exists()
                || !self.repositories[key].path.is_dir()
                || !self.repositories[key].path.join(".git").exists()
                || !self.repositories[key].path.join(".git").is_dir()
            {
                println!(
                    "Repository does no longer exist: {:?}",
                    &self.repositories[key].path
                );
                self.repositories.remove(key);
            }
        }

        // Do a full repository discovery on all watched repositories
        for watched in &self.watched.clone() {
            self.discover(watched, 0);
        }

        self.save()?;

        Ok(())
    }

    pub fn has_repo_at_path(&self, path: &Path) -> bool {
        self.repositories.iter().any(|repo| repo.path == path)
    }

    pub fn discover(&mut self, path: &Path, depths: usize) {
        // Check if a .git directory exists.
        // If it does, always stop searching.
        let git_dir = path.join(".git");
        debug!("{} Looking at folder {:?}", depths, path);
        if git_dir.exists() {
            debug!("Found .git folder");
            // Add the repository, if we don't know it yet.
            let path = path.to_path_buf();
            if !self.has_repo_at_path(&path) {
                println!("Found new repository: {path:?}");
                self.repositories.push(Repository::new(path));
            }
            return;
        }

        // Recursion stop. Only check up to a dephts of 5
        if depths == 5 {
            debug!("Max depth reached");
            return;
        }

        let current_dir = match read_dir(path) {
            Ok(current_dir) => current_dir,
            Err(err) => {
                debug!(
                    "Couldn't read directory at {:?} with error: {:?}",
                    path, err
                );
                return;
            }
        };

        // The current path is no repository, search it's subdirectories
        for entry_result in current_dir {
            match entry_result {
                Ok(entry) => {
                    let path = entry.path();
                    if !path.is_dir() {
                        continue;
                    }

                    self.discover(&path, depths + 1);
                }
                Err(err) => {
                    debug!(
                        "Couldn't read directory path {:?} with error: {:?}",
                        path, err
                    );
                    continue;
                }
            }
        }
    }
}

fn default_cache_path() -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| anyhow!("Couldn't resolve home dir"))?;
    let path = home.join(".local/share/geil");
    Ok(path)
}
