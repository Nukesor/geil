use std::path::PathBuf;

use strum::Display;

#[derive(Display)]
pub enum RepositoryState {
    Unknown,
    /// The current git HEAD is detached.
    Detached,
    /// Repo has been fetched, merged and is up-to-date.
    UpToDate,
    /// The repo looks fine during a `Check` run.
    Ok,
    /// We just fetched the newest info from the default remote.
    Fetched,
    /// The repository has been successfully updated.
    Updated,
    /// There's no way to fast-forward merge.
    NoFastForward,
    /// There're some local filesystem changes.
    LocalChanges,
    /// There're unpushed commits in this repo.
    NotPushed,
}

pub struct RepositoryInfo {
    pub path: PathBuf,
    pub name: String,
    pub state: RepositoryState,
    pub stashed: usize,
    /// The time (ms) it took to check the repo.
    pub check_time: Option<usize>,
    pub hook: Option<String>,
}

impl RepositoryInfo {
    pub fn new(path: PathBuf, hook: Option<String>) -> RepositoryInfo {
        // Get the repository name from the path for the progress bar
        let name = path.file_name().map_or("no_name?".to_string(), |name| {
            name.to_string_lossy().to_string()
        });

        RepositoryInfo {
            path,
            name,
            state: RepositoryState::Unknown,
            stashed: 0,
            check_time: None,
            hook,
        }
    }
}
