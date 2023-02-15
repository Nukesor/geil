use std::path::PathBuf;

use strum_macros::Display;

#[derive(Display)]
pub enum RepositoryState {
    Unknown,
    Detached,
    UpToDate,
    Fetched,
    Updated,
    NoFastForward,
    LocalChanges,
    NotPushed,
}

pub struct RepositoryInfo {
    pub path: PathBuf,
    pub name: String,
    pub state: RepositoryState,
    pub stashed: usize,
    /// The time (ms) it took to check the repo.
    pub check_time: Option<usize>,
}

impl RepositoryInfo {
    pub fn new(path: PathBuf) -> RepositoryInfo {
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
        }
    }
}
