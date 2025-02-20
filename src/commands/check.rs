use std::{
    collections::HashMap,
    env::vars,
    time::{Duration, Instant},
};

use anyhow::Result;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use log::debug;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::{
    display::{multi_progress_bar, print_status},
    git::{check_local_changes, check_unpushed_commits, get_stashed_entries},
    repository_info::{RepositoryInfo, RepositoryState},
    state::State,
};

pub fn check(
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
                repo_info = match check_repo(&multi_progress, repo_info, &envs) {
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
            let mut repo_info = check_repo(&multi_progress, repo_info, &envs)?;
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

/// This is a simple wrapper around the actual repo check function
/// for easier progress bar handling.
pub fn check_repo(
    multi_progress: &MultiProgress,
    repo_info: RepositoryInfo,
    envs: &HashMap<String, String>,
) -> Result<RepositoryInfo> {
    let mut bar = ProgressBar::new(3);
    let spinner_style =
        ProgressStyle::with_template("{duration} {spinner} {prefix:.bold.white.dim} - {wide_msg}")
            .unwrap()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ");
    bar.set_style(spinner_style);

    // Add the bar to the end of the multi_bar.
    bar = multi_progress.add(bar);

    // Enable a steady tick after adding it to the bar, to ensure correct position rendering.
    bar.enable_steady_tick(Duration::from_millis(125));

    // Run the actual repo handling logic.
    let result = check_repo_inner(&bar, repo_info, envs);

    // Clean up this repo's progress bar.
    bar.disable_steady_tick();
    bar.finish();
    multi_progress.remove(&bar);

    result
}

pub fn check_repo_inner(
    bar: &ProgressBar,
    mut repo_info: RepositoryInfo,
    envs: &HashMap<String, String>,
) -> Result<RepositoryInfo> {
    let name = repo_info.name.clone();

    // Default to a `Ok` repo state.
    // If anything is not ok, it'll be set in the respective function.
    repo_info.state = RepositoryState::Ok;

    bar.set_prefix(format!("[1/3] - {name}"));
    bar.set_message(format!("{name}: Checking stash"));
    get_stashed_entries(&mut repo_info, envs)?;

    bar.set_prefix(format!("[2/3] - {name}"));
    bar.set_message(format!("{name}: Check for local changes"));
    check_local_changes(&mut repo_info, envs)?;

    bar.set_prefix(format!("[3/3] - {name}"));
    bar.set_message(format!("{name}: Check for unpushed commits"));
    check_unpushed_commits(&mut repo_info, envs)?;

    Ok(repo_info)
}
