use std::path::PathBuf;

use strum_macros::Display;

#[derive(Display)]
pub enum RepositoryState {
    UpToDate,
    Fetched,
    Updated,
    NoFastForward,
}

pub struct RepositoryInfo {
    pub path: PathBuf,
    pub state: RepositoryState,
    pub stashed: usize,
    pub local_changes: bool,
}

impl RepositoryInfo {
    pub fn new(path: PathBuf) -> RepositoryInfo {
        RepositoryInfo {
            path,
            state: RepositoryState::UpToDate,
            stashed: 0,
            local_changes: false,
        }
    }
}
