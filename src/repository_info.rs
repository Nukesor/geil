use std::path::PathBuf;

pub enum GeilRepositoryState {
    NotChecked,
    RemoteUnknown,
    RemoteNotFound,
    InvalidRemoteName,
    NoShorthand,
}

pub struct RepositoryInfo {
    pub path: PathBuf,
    pub state: GeilRepositoryState,
    pub error: Option<String>,
    pub stashed: usize,
}

impl RepositoryInfo {
    pub fn new(path: PathBuf) -> RepositoryInfo {
        RepositoryInfo {
            path,
            state: GeilRepositoryState::NotChecked,
            error: None,
            stashed: 0,
        }
    }
}
