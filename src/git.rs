use std::collections::HashMap;

use anyhow::Result;
use log::{debug, info};

use crate::{cmd, process::*, repository_info::*};

pub fn get_stashed_entries(
    repo_info: &mut RepositoryInfo,
    envs: &HashMap<String, String>,
) -> Result<()> {
    let name = repo_info.name.clone();

    let merge = cmd!("git rev-list --walk-reflogs --count refs/stash")
        .cwd(repo_info.path.clone())
        .env(envs.clone());
    let capture_data = merge.run()?;
    let stdout = String::from_utf8_lossy(&capture_data.stdout);
    // There are now stashes
    if stdout.contains("unknown revision or path not in the working tree") {
        info!("{name}: No stashed changes");
        return Ok(());
    }
    let number = stdout
        .lines()
        .next()
        .expect("At least two lines of output")
        .trim();

    repo_info.stashed = number.parse::<usize>().expect("Couldn't get stash amount");
    info!("{name}: Found {} stashed entries!", repo_info.stashed);

    Ok(())
}

pub fn check_local_changes(
    repo_info: &mut RepositoryInfo,
    envs: &HashMap<String, String>,
) -> Result<()> {
    let name = repo_info.name.clone();

    let merge = cmd!("git status")
        .cwd(repo_info.path.clone())
        .env(envs.clone());
    let capture_data = merge.run()?;
    let stdout = String::from_utf8_lossy(&capture_data.stdout);

    // No local changes, everything seems clean.
    if stdout.contains("nothing to commit, working tree clean") {
        info!("{name}: No local changes");
        return Ok(());
    }

    repo_info.state = RepositoryState::LocalChanges;
    info!("{name}: Found local changes!");

    Ok(())
}

pub fn fetch(repo_info: &mut RepositoryInfo, envs: &HashMap<String, String>) -> Result<()> {
    let name = repo_info.name.clone();

    let fetch = cmd!("git fetch --all")
        .cwd(repo_info.path.clone())
        .env(envs.clone());
    let capture_data = fetch.run()?;

    if String::from_utf8_lossy(&capture_data.stdout).contains("Receiving objects: 100%") {
        info!("{name}: Got new changes from remote!");
        repo_info.state = RepositoryState::Fetched;
    } else {
        info!("{name}: Everything is up to date");
        repo_info.state = RepositoryState::UpToDate;
    }

    Ok(())
}

pub fn merge(repo_info: &mut RepositoryInfo, envs: &HashMap<String, String>) -> Result<()> {
    let name = repo_info.name.clone();

    let merge = cmd!("git merge --ff-only")
        .cwd(repo_info.path.clone())
        .env(envs.clone());
    let capture_data = merge.run()?;
    let stdout = String::from_utf8_lossy(&capture_data.stdout);

    if stdout.contains("Updating") {
        info!("{name}: Fast forward succeeded");
        repo_info.state = RepositoryState::Updated;

        // Update any submodules if this worked out.
        let submodule_update = cmd!("git submodule update --init --recursive")
            .cwd(repo_info.path.clone())
            .env(envs.clone());
        submodule_update.run()?;
    } else if stdout.contains("up to date") {
        info!("{name}: Already up to date");
        repo_info.state = RepositoryState::UpToDate;
    } else if stdout.contains("fatal:") {
        info!("{name}: Fast forward not possible!");
        repo_info.state = RepositoryState::NoFastForward;
    } else {
        info!("{name}: Couldn't get state from output: {stdout}");
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
    let name = repo_info.name.clone();

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
    debug!("{name}: Checking origin/{current_branch}");
    let capture_data = cmd!("git rev-parse origin/{current_branch}")
        .cwd(repo_info.path.clone())
        .env(envs.clone())
        .run()?;
    let remote_hash = String::from_utf8_lossy(&capture_data.stdout);
    let remote_hash = remote_hash.trim();

    // The hashes differ. Since the branch is already UpToDate at this state,
    // this (most likely) means the rpeository has unpushed changes.
    debug!("{name}: Local hash: {local_hash}, Remote: {remote_hash}");
    if local_hash != remote_hash {
        info!("Found unpushed commits!");
        repo_info.state = RepositoryState::NotPushed;
        return Ok(());
    }

    info!("No unpushed commits");
    Ok(())
}
