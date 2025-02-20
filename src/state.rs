use std::{
    fs::{File, read_dir},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, anyhow};
use log::debug;
use serde::{Deserialize, Serialize};
use serde_with::{DefaultOnError, serde_as};

use crate::repository_info::RepositoryInfo;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Repository {
    /// The path to the repository
    pub path: PathBuf,
    /// The time it took to check this repository in the last run.
    pub check_time: Option<usize>,
    /// A command that will be executed after a successful update.
    pub hook: Option<String>,
}

impl Repository {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            check_time: None,
            hook: None,
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
    /// All paths that're actively watched for new repositories
    pub watched: Vec<PathBuf>,
    /// All paths that're explicitly ignored.
    #[serde(default = "Default::default")]
    pub ignored: Vec<PathBuf>,
    #[serde_as(deserialize_as = "DefaultOnError")]
    pub repositories: Vec<Repository>,
    #[serde(default = "Default::default")]
    pub keys: Vec<SshKey>,
}

impl State {
    pub fn new() -> State {
        State {
            ignored: Vec::new(),
            watched: Vec::new(),
            repositories: Vec::new(),
            keys: Vec::new(),
        }
    }
}

impl State {
    /// Save a state to the disk.
    pub fn save(&mut self) -> Result<()> {
        self.repositories.sort_by(|a, b| a.path.cmp(&b.path));
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
            let mut new_repos = Vec::new();
            discover(&self.ignored, watched, 0, &mut new_repos);
            for repo in new_repos {
                if !self.has_repo_at_path(&repo.path) {
                    println!("Found new repository: {:?}", repo.path);
                    self.repositories.push(repo);
                }
            }
        }

        self.save()?;

        Ok(())
    }

    pub fn repo_at_path(&mut self, path: &Path) -> Option<&mut Repository> {
        self.repositories.iter_mut().find(|repo| repo.path == path)
    }

    pub fn has_repo_at_path(&self, path: &Path) -> bool {
        self.repositories.iter().any(|repo| repo.path == path)
    }

    /// Create a list of [RepositoryInfo]s for internal processing, based on the list
    /// of known Git repositories.
    ///
    /// Order the repositories by check wall time from the last run.
    /// Repositories with long running checks will be at the top of the vector.
    /// That way, we try to minimize wall execution time, by doing smarter scheduling.
    pub fn repo_infos_by_wall_time(&self) -> Vec<RepositoryInfo> {
        let mut repos = self.repositories.clone();
        repos.sort_by(|a, b| b.check_time.cmp(&a.check_time));

        // We create a struct for our internal representation for each repository
        let mut repo_infos: Vec<RepositoryInfo> = Vec::new();
        for repo in repos {
            let repository_info = RepositoryInfo::new(repo.path.clone(), repo.hook);
            repo_infos.push(repository_info);
        }

        repo_infos
    }

    pub fn update_check_times(&mut self, repo_infos: &[RepositoryInfo]) -> Result<()> {
        for info in repo_infos.iter() {
            let repo = self
                .repositories
                .iter_mut()
                .find(|r| r.path == info.path)
                .context("Expect repository to be there")?;

            repo.check_time = info.check_time;
        }
        self.save()?;

        Ok(())
    }
}

/// Discover repositories inside a given folder.
pub fn discover(
    ignored_paths: &[PathBuf],
    path: &Path,
    depths: usize,
    new_repos: &mut Vec<Repository>,
) {
    if ignored_paths.contains(&path.to_path_buf()) {
        return;
    }

    // Check if a .git directory exists.
    // If it does, always stop searching.
    let git_dir = path.join(".git");
    debug!("{} Looking at folder {:?}", depths, path);
    if git_dir.exists() {
        debug!("Found .git folder");
        // Add the repository, if we don't know it yet.
        new_repos.push(Repository::new(path.to_owned()));
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

                discover(ignored_paths, &path, depths + 1, new_repos);
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

fn default_cache_path() -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| anyhow!("Couldn't resolve home dir"))?;
    let path = home.join(".local/share/geil");
    Ok(path)
}
