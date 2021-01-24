use std::collections::HashMap;
use std::env::vars;

use anyhow::Result;
use log::info;

use crate::cmd;
use crate::process::*;
use crate::repository_info::*;

pub fn update_repos(repo_infos: &mut Vec<RepositoryInfo>) -> Result<()> {
    // Save all environment variables for later injection into git
    let mut envs = HashMap::new();
    for (key, value) in vars() {
        envs.insert(key, value);
    }
    for repo_info in repo_infos.iter_mut() {
        info!("Looking at: {}", repo_info.path.clone().to_string_lossy());
        get_stashed_entries(repo_info, &envs)?;
        update_repo(repo_info, &envs)?;
    }

    Ok(())
}

pub fn get_stashed_entries(
    repo_info: &mut RepositoryInfo,
    envs: &HashMap<String, String>,
) -> Result<()> {
    info!("Check for stashed entries");
    let merge = cmd!("git rev-list --walk-reflogs --count refs/stash")
        .cwd(repo_info.path.clone())
        .env(envs.clone());
    let capture_data = merge.run()?;
    let stdout = String::from_utf8_lossy(&capture_data.stdout);
    // There are now stashes
    if stdout.contains("unknown revision or path not in the working tree") {
        return Ok(());
    }
    let number = stdout
        .lines()
        .next()
        .expect("At least two lines of output")
        .trim();

    repo_info.stashed = number.parse::<usize>().expect("Couldn't get stash amount");

    Ok(())
}

pub fn update_repo(repo_info: &mut RepositoryInfo, envs: &HashMap<String, String>) -> Result<()> {
    fetch_repo(repo_info, envs)?;

    info!("Merge");
    let merge = cmd!("git merge --ff-only")
        .cwd(repo_info.path.clone())
        .env(envs.clone());
    let capture_data = merge.run()?;
    let stdout = String::from_utf8_lossy(&capture_data.stdout);

    if stdout.contains("Updating") {
        repo_info.state = RepositoryState::Updated;
    } else if stdout.contains("Already up to date") {
        repo_info.state = RepositoryState::UpToDate;
    } else if stdout.contains("fatal:") {
        repo_info.state = RepositoryState::NoFastForward;
    } else {
        info!("Couldn't get state from output: {}", stdout);
    }

    Ok(())
}

pub fn fetch_repo(repo_info: &mut RepositoryInfo, envs: &HashMap<String, String>) -> Result<()> {
    info!("Fetch");
    let fetch = cmd!("git fetch --all")
        .cwd(repo_info.path.clone())
        .env(envs.clone());
    let capture_data = fetch.run()?;
    if String::from_utf8_lossy(&capture_data.stdout).contains("Receiving objects: 100%") {
        repo_info.state = RepositoryState::Fetched;
    }

    Ok(())
}
