use std::collections::HashMap;
use std::env::vars;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use log::debug;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::display::multi_progress_bar;
use crate::git::{check_local_changes, check_unpushed_commits, fetch, get_stashed_entries, merge};
use crate::repository_info::RepositoryState;
use crate::{display::print_status, repository_info::RepositoryInfo, state::State};

pub fn update(
    state: &mut State,
    show_all: bool,
    parallel: bool,
    threads: Option<usize>,
) -> Result<()> {
    let repo_infos = state.repo_infos_by_wall_time();

    // Save all environment variables for later injection into git
    let mut envs = HashMap::new();
    for (key, value) in vars() {
        envs.insert(key, value);
    }

    let (multi_progress, main_bar) = multi_progress_bar(repo_infos.len() as u64)?;

    let repo_infos = if parallel {
        // Set the amount of threads, if specified.
        if let Some(threads) = threads {
            rayon::ThreadPoolBuilder::new()
                .num_threads(threads)
                .build_global()
                .unwrap();
        }

        let results: Result<Vec<RepositoryInfo>> = repo_infos
            .into_par_iter()
            .map(|mut repo_info| {
                // Handle the repository and track execution time.
                let start = Instant::now();
                let repo_path = repo_info.path.clone();
                repo_info = match update_repo(&multi_progress, repo_info, &envs)
                    .context(format!("Error while updating repo: {:?}", repo_path))
                {
                    Ok(repo_info) => repo_info,
                    Err(err) => {
                        // Make sure the bar gets incremented even if we get an error.
                        main_bar.inc(1);
                        return Err(err);
                    }
                };
                repo_info.check_time = Some(start.elapsed().as_millis() as usize);

                main_bar.inc(1);
                Ok(repo_info)
            })
            .collect();

        main_bar.finish_with_message("All done: ");

        results?
    } else {
        let mut results = Vec::new();
        for repo_info in repo_infos.into_iter() {
            // Handle the repository and track execution time.
            let start = Instant::now();
            let mut repo_info = update_repo(&multi_progress, repo_info, &envs)?;
            repo_info.check_time = Some(start.elapsed().as_millis() as usize);

            debug!("Check took {}ms", start.elapsed().as_millis());
            results.push(repo_info);
        }

        results
    };

    // Finish and clean up the progress bar
    main_bar.finish();
    let _ = multi_progress.clear();

    state.update_check_times(&repo_infos)?;

    print_status(repo_infos, show_all)?;

    Ok(())
}

/// This is a simple wrapper around the actual repo handling function
/// for easier progress bar handling.
pub fn update_repo(
    multi_bar: &MultiProgress,
    repo_info: RepositoryInfo,
    envs: &HashMap<String, String>,
) -> Result<RepositoryInfo> {
    let mut bar = ProgressBar::new(5);
    let spinner_style =
        ProgressStyle::with_template("{duration} {spinner} {prefix:.bold.white.dim} - {wide_msg}")
            .unwrap()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ");
    bar.set_style(spinner_style);

    // Add the bar to the end of the multi_bar.
    bar = multi_bar.add(bar);

    // Enable a steady tick after adding it to the bar, to ensure correct position rendering.
    bar.enable_steady_tick(Duration::from_millis(125));

    // Run the actual repo handling logic.
    let result = update_repo_inner(&bar, repo_info, envs);

    // Clean up this repo's progress bar.
    bar.disable_steady_tick();
    bar.finish();
    multi_bar.remove(&bar);

    result
}

pub fn update_repo_inner(
    bar: &ProgressBar,
    mut repo_info: RepositoryInfo,
    envs: &HashMap<String, String>,
) -> Result<RepositoryInfo> {
    let name = repo_info.name.clone();

    bar.set_prefix(format!("[1/5] - {name}"));
    bar.set_message(format!("{name}: Checking stash"));
    get_stashed_entries(&mut repo_info, envs)?;

    bar.set_prefix(format!("[2/5] - {name}"));
    bar.set_message(format!("{name}: Fetch from remote"));
    fetch(&mut repo_info, envs)?;

    bar.set_prefix(format!("[3/5] - {name}"));
    bar.set_message(format!("{name}: Check for local changes"));
    check_local_changes(&mut repo_info, envs)?;

    // Skip update
    // We cannot merge with local changes anyway.
    if matches!(repo_info.state, RepositoryState::LocalChanges) {
        return Ok(repo_info);
    }

    bar.set_prefix(format!("[4/5] - {name}"));
    bar.set_message(format!("{name}: Try to fast forward"));
    merge(&mut repo_info, envs)?;

    bar.set_prefix(format!("[5/5] - {name}"));
    bar.set_message(format!("{name}: Check for unpushed commits"));
    if matches!(repo_info.state, RepositoryState::UpToDate) {
        check_unpushed_commits(&mut repo_info, envs)?;
    }

    Ok(repo_info)
}
