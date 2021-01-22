use std::path::PathBuf;

pub enum RepositoryState {
    NotChecked,
    RemoteUnknown,
    InvalidRemoteName,
    NoShorthand,
}

pub struct RepositoryInfo {
    pub path: PathBuf,
    pub state: RepositoryState,
    pub error: Option<String>,
    pub stashed: usize,
}

impl RepositoryInfo {
    pub fn new(path: PathBuf) -> RepositoryInfo {
        RepositoryInfo {
            path,
            state: RepositoryState::NotChecked,
            error: None,
            stashed: 0,
        }
    }
}
