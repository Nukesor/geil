use std::path::PathBuf;

pub enum RepositoryState {
    NotChecked,
    RemoteUnknown,
}

pub struct RepositoryInfo {
    pub path: PathBuf,
    pub state: RepositoryState,
    pub stashed: usize,
}
