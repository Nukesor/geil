use std::collections::HashMap;

use anyhow::Result;
use log::{debug, info};

use crate::cmd;
use crate::process::*;
use crate::repository_info::*;

pub fn handle_repo(
    mut repo_info: RepositoryInfo,
    envs: &HashMap<String, String>,
) -> Result<RepositoryInfo> {
    info!("Checking repo: {}", repo_info.path.to_string_lossy());
    get_stashed_entries(&mut repo_info, envs)?;
    fetch_repo(&mut repo_info, envs)?;
    check_local_changes(&mut repo_info, envs)?;

    // Skip update
    // We cannot merge with local changes anyway.
    if matches!(repo_info.state, RepositoryState::LocalChanges) {
        return Ok(repo_info);
    }
    update_repo(&mut repo_info, envs)?;

    if matches!(repo_info.state, RepositoryState::UpToDate) {
        check_unpushed_commits(&mut repo_info, envs)?;
    }

    Ok(repo_info)
}

pub fn get_stashed_entries(
    repo_info: &mut RepositoryInfo,
    envs: &HashMap<String, String>,
) -> Result<()> {
    let merge = cmd!("git rev-list --walk-reflogs --count refs/stash")
        .cwd(repo_info.path.clone())
        .env(envs.clone());
    let capture_data = merge.run()?;
    let stdout = String::from_utf8_lossy(&capture_data.stdout);
    // There are now stashes
    if stdout.contains("unknown revision or path not in the working tree") {
        info!("No stashed changes");
        return Ok(());
    }
    let number = stdout
        .lines()
        .next()
        .expect("At least two lines of output")
        .trim();

    repo_info.stashed = number.parse::<usize>().expect("Couldn't get stash amount");
    info!("Found {} stashed entries!", repo_info.stashed);

    Ok(())
}

pub fn check_local_changes(
    repo_info: &mut RepositoryInfo,
    envs: &HashMap<String, String>,
) -> Result<()> {
    let merge = cmd!("git status")
        .cwd(repo_info.path.clone())
        .env(envs.clone());
    let capture_data = merge.run()?;
    let stdout = String::from_utf8_lossy(&capture_data.stdout);

    // No local changes, everything seems clean.
    if stdout.contains("nothing to commit, working tree clean") {
        info!("No local changes");
        return Ok(());
    }

    repo_info.state = RepositoryState::LocalChanges;
    info!("Found local changes!");

    Ok(())
}

pub fn fetch_repo(repo_info: &mut RepositoryInfo, envs: &HashMap<String, String>) -> Result<()> {
    let fetch = cmd!("git fetch --all")
        .cwd(repo_info.path.clone())
        .env(envs.clone());
    let capture_data = fetch.run()?;
    if String::from_utf8_lossy(&capture_data.stdout).contains("Receiving objects: 100%") {
        info!("Got new changes from remote!");
        repo_info.state = RepositoryState::Fetched;
    } else {
        info!("Everything is up to date");
        repo_info.state = RepositoryState::UpToDate;
    }

    Ok(())
}

pub fn update_repo(repo_info: &mut RepositoryInfo, envs: &HashMap<String, String>) -> Result<()> {
    let merge = cmd!("git merge --ff-only")
        .cwd(repo_info.path.clone())
        .env(envs.clone());
    let capture_data = merge.run()?;
    let stdout = String::from_utf8_lossy(&capture_data.stdout);

    if stdout.contains("Updating") {
        info!("Fast forward succeeded");
        repo_info.state = RepositoryState::Updated;
    } else if stdout.contains("up to date") {
        info!("Already up to date");
        repo_info.state = RepositoryState::UpToDate;
    } else if stdout.contains("fatal:") {
        info!("Fast forward not possible!");
        repo_info.state = RepositoryState::NoFastForward;
    } else {
        info!("Couldn't get state from output: {}", stdout);
        repo_info.state = RepositoryState::Unknown;
    }

    Ok(())
}

/// Check whether the current branch has some commits that're newer than the remotes.
/// If the current HEAD isn't on a branch, the repository enters the `Detached` state.
pub fn check_unpushed_commits(
    repo_info: &mut RepositoryInfo,
    envs: &HashMap<String, String>,
) -> Result<()> {
    // Get all remotes for this repository
    let capture_data = cmd!("git remote")
        .cwd(repo_info.path.clone())
        .env(envs.clone())
        .run()?;
    let remotes = String::from_utf8_lossy(&capture_data.stdout);
    let remotes = remotes.trim();

    let capture_data = cmd!("git rev-parse --abbrev-ref HEAD")
        .cwd(repo_info.path.clone())
        .env(envs.clone())
        .run()?;
    let current_branch = String::from_utf8_lossy(&capture_data.stdout);
    let current_branch = current_branch.trim();

    // The repository is in a detached state. Return early.
    if current_branch == "HEAD" {
        repo_info.state = RepositoryState::Detached;
        return Ok(());
    }

    // Get the hash of the local commit
    let capture_data = cmd!("git rev-parse HEAD")
        .cwd(repo_info.path.clone())
        .env(envs.clone())
        .run()?;
    let local_hash = String::from_utf8_lossy(&capture_data.stdout);
    let local_hash = local_hash.trim();

    // Check if all remotes have been pushed.
    for remote in remotes.lines() {
        debug!("Checking {remote}/{current_branch}");
        let capture_data = cmd!("git rev-parse {remote}/{current_branch}")
            .cwd(repo_info.path.clone())
            .env(envs.clone())
            .run()?;
        let remote_hash = String::from_utf8_lossy(&capture_data.stdout);
        let remote_hash = remote_hash.trim();

        // The hashes differ. Since the branch is already UpToDate at this state,
        // this (most likely) means the rpeository has unpushed changes.
        debug!("Local hash: {local_hash}, Remote: {remote_hash}");
        if local_hash != remote_hash {
            info!("Found unpushed commits!");
            repo_info.state = RepositoryState::NotPushed;
            return Ok(());
        }
    }

    info!("No unpushed commits");
    Ok(())
}
