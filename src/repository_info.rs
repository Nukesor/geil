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
    pub state: RepositoryState,
    pub stashed: usize,
    /// The time (ms) it took to check the repo.
    pub check_time: Option<usize>,
}

impl RepositoryInfo {
    pub fn new(path: PathBuf) -> RepositoryInfo {
        RepositoryInfo {
            path,
            state: RepositoryState::Unknown,
            stashed: 0,
            check_time: None,
        }
    }
}
