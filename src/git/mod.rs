use anyhow::Result;
use git2::{RemoteCallbacks, Repository};

use crate::repository_info::*;

pub mod credentials;
pub mod local;
pub mod update;

pub fn update_repos(repo_infos: &mut Vec<RepositoryInfo>) -> Result<()> {
    // We don't necessarily need any callbacks, but they're needed for interaction with git2
    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(|_, _, _| credentials::get_credentials());

    for repo_info in repo_infos.iter_mut() {
        let repository = Repository::open(&repo_info.path)?;
        let head = repository.head()?;
        if !head.is_branch() || head.name().is_none() {
            continue;
        }

        // Check if we can find a remote for the current branch.
        let remote = match repository.branch_remote_name(head.name().unwrap()) {
            Ok(remote) => remote,
            Err(err) => {
                repo_info.state = RepositoryState::InvalidRemoteName;
                repo_info.error = Some(err.to_string());
                continue;
            }
        };
        // Check if the remote is valid utf8.
        let remote = match remote.as_str() {
            Some(remote) => remote.clone(),
            None => {
                repo_info.state = RepositoryState::InvalidRemoteName;
                continue;
            }
        };
        // Check if the branch has a valid shorthand.
        let branch = match head.shorthand() {
            Some(branch) => branch,
            None => {
                repo_info.state = RepositoryState::NoShorthand;
                continue;
            }
        };

        update::update_repo(&repository, &remote, &branch)?;
    }

    Ok(())
}
