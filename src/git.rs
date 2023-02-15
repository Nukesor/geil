use std::collections::HashMap;

use anyhow::Result;
use indicatif::MultiProgress;
use indicatif::ProgressBar;
use indicatif::ProgressStyle;
use log::{debug, info};

use crate::cmd;
use crate::process::*;
use crate::repository_info::*;

/// This is a simple wrapper around the actual repo handling function
/// for easier progress bar handling.
pub fn handle_repo(
    multi_bar: &MultiProgress,
    repo_info: RepositoryInfo,
    envs: &HashMap<String, String>,
) -> Result<RepositoryInfo> {
    let mut bar = ProgressBar::new(5);
    let spinner_style = ProgressStyle::with_template("{prefix:.bold.dim} {spinner} {wide_msg}")
        .unwrap()
        .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ");
    bar.set_style(spinner_style.clone());

    // Add the bar to the end of the multi_bar.
    bar = multi_bar.add(bar);

    // Run the actual repo handling logic.
    let result = handle_repo_inner(&bar, repo_info, envs);

    // Clean up this repo's progress bar.
    bar.finish();
    multi_bar.remove(&bar);

    result
}

pub fn handle_repo_inner(
    bar: &ProgressBar,
    mut repo_info: RepositoryInfo,
    envs: &HashMap<String, String>,
) -> Result<RepositoryInfo> {
    let name = repo_info.name.clone();

    bar.set_prefix("[1/5]");
    bar.set_message(format!("{name:?}: Checking stash"));
    get_stashed_entries(&mut repo_info, envs)?;

    bar.set_prefix("[2/5]");
    bar.set_message(format!("{name:?}: Fetch from remote"));
    fetch_repo(&mut repo_info, envs)?;

    bar.set_prefix("[3/5]");
    bar.set_message(format!("{name:?}: Check for local changes"));
    check_local_changes(&mut repo_info, envs)?;

    // Skip update
    // We cannot merge with local changes anyway.
    if matches!(repo_info.state, RepositoryState::LocalChanges) {
        return Ok(repo_info);
    }
    bar.set_prefix("[4/5]");
    bar.set_message(format!("{name:?}: Try to fast forward"));
    update_repo(&mut repo_info, envs)?;

    bar.set_prefix("[5/5]");
    bar.set_message(format!("{name:?}: Check for unpushed commits"));
    if matches!(repo_info.state, RepositoryState::UpToDate) {
        check_unpushed_commits(&mut repo_info, envs)?;
    }

    Ok(repo_info)
}

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

pub fn fetch_repo(repo_info: &mut RepositoryInfo, envs: &HashMap<String, String>) -> Result<()> {
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

pub fn update_repo(repo_info: &mut RepositoryInfo, envs: &HashMap<String, String>) -> Result<()> {
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
        info!("{name}: Couldn't get state from output: {}", stdout);
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
