use std::env::vars;
use std::time::Instant;
use std::{collections::HashMap, path::PathBuf};

use anyhow::{Context, Result};
use clap::Parser;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use log::{debug, error};
use rayon::prelude::*;
use simplelog::{Config, LevelFilter, SimpleLogger};

mod cli;
mod display;
mod git;
mod process;
mod repository_info;
mod ssh_key;
mod state;

use cli::*;
use display::*;
use git::*;
use repository_info::*;
use ssh_key::load_keys;
use state::*;

fn main() -> Result<()> {
    // Parse commandline options.
    let opt = CliArguments::parse();

    // Set the verbosity level of the logger.
    let level = match opt.verbose {
        0 => LevelFilter::Error,
        1 => LevelFilter::Warn,
        2 => LevelFilter::Info,
        _ => LevelFilter::Debug,
    };
    SimpleLogger::init(level, Config::default()).unwrap();

    let mut state = State::load()?;

    match opt.cmd {
        SubCommand::Add { repos } => add(state, repos),
        SubCommand::Remove { repos } => remove(state, repos),
        SubCommand::Watch { directories } => watch(state, directories),
        SubCommand::Unwatch { directories } => unwatch(state, directories),
        SubCommand::Update {
            all,
            not_parallel,
            threads,
        } => {
            state.scan()?;
            load_keys(&state)?;
            update(state, all, !not_parallel, threads)
        }
        SubCommand::Keys { cmd } => ssh_key::handle_key_command(state, cmd),
    }
}

fn add(mut state: State, repos: Vec<PathBuf>) -> Result<()> {
    // Just print the known repositories, if no arguments have been supplied.
    if repos.is_empty() {
        println!("Watched repositories:");
        for repo in state.repositories {
            println!("  - {:?}", repo.path);
        }
        return Ok(());
    }

    for path in repos {
        // Check if the directory to add actually exists
        if !path.exists() || !path.is_dir() {
            error!("Cannot find repository at {:?}", path);
        }

        // Store the absolute path.
        let real_path = std::fs::canonicalize(&path)?;
        if !state.has_repo_at_path(&real_path) {
            println!("Added repository: {:?}", &real_path);
            state.repositories.push(Repository::new(real_path));
        }
    }
    state.save()
}

fn remove(mut state: State, repos: Vec<PathBuf>) -> Result<()> {
    for path in repos {
        // Check if the directory to add actually exists
        if !path.exists() || !path.is_dir() {
            error!("Cannot find repository at {:?}", path);
        }

        // Store the absolute path.
        let real_path = std::fs::canonicalize(&path)?;
        if !state.has_repo_at_path(&real_path) {
            error!(
                "The repository at {:?} hasn't been added to geil yet.",
                path
            );
        } else {
            println!("Forgetting about repository: {:?}", &real_path);
            state.repositories.retain(|repo| repo.path != real_path);
        }
    }
    state.save()
}

fn watch(mut state: State, directories: Vec<PathBuf>) -> Result<()> {
    // Just print the watched folders, if no arguments have been supplied.
    if directories.is_empty() {
        println!("Watched folders");
        for dir in state.watched {
            println!("  - {dir:?}");
        }
        return Ok(());
    }

    for path in directories {
        // Check if the directory to add actually exists
        if !path.exists() || !path.is_dir() {
            error!("Cannot find directory at {:?}", path);
        }

        // Store the absolute path.
        let real_path = std::fs::canonicalize(&path)?;
        if !state.watched.contains(&real_path) {
            println!("Watching folder: {:?}", &real_path);
            state.watched.push(real_path);
        }
    }

    state.scan()
}

fn unwatch(mut state: State, directories: Vec<PathBuf>) -> Result<()> {
    for path in directories {
        // Check if the directory to add actually exists
        if !path.exists() || !path.is_dir() {
            error!("Cannot find directory at {:?}", path);
        }

        // Get the absolute path
        let real_path = std::fs::canonicalize(&path)?;
        if !state.watched.contains(&real_path) {
            error!("The folder hasn't been watched: {:?}", &real_path);
        } else {
            println!("Unwatching path : {:?}", &real_path);
            state.watched.retain(|path| path != &real_path);

            // Scan the watched path for repositories, so we can forget about them
            let mut repos = Vec::new();
            discover(&real_path, 0, &mut repos);

            for repo_to_remove in repos {
                println!("Forgetting about repository: {:?}", repo_to_remove.path);
                state
                    .repositories
                    .retain(|repo| repo.path != repo_to_remove.path);
            }
        }
    }

    state.save()
}

fn update(mut state: State, show_all: bool, parallel: bool, threads: Option<usize>) -> Result<()> {
    // First we order the repositories by check time (from the last run).
    // Repositories with long running checks will be executed first.
    let mut repos = state.repositories.clone();
    repos.sort_by(|a, b| b.check_time.cmp(&a.check_time));

    // We create a struct for our internal representation for each repository
    let mut repo_infos: Vec<RepositoryInfo> = Vec::new();
    for repo in repos {
        let repository_info = RepositoryInfo::new(repo.path.clone());
        repo_infos.push(repository_info);
    }

    // Save all environment variables for later injection into git
    let mut envs = HashMap::new();
    for (key, value) in vars() {
        envs.insert(key, value);
    }

    let multi_bar = MultiProgress::new();

    // Get the power of the repository count.
    // format the progress count based on that count, otherwise we get unwanted line breaks.
    let power = repo_infos.len().checked_ilog10().unwrap_or(1) + 1;

    // Set up the styling for the "main" progress bar.
    let template = &format!("Checking repositories: {{wide_bar}} {{pos:>{power}}}/{{len:{power}}}");
    let style = ProgressStyle::default_bar()
        .template(template)
        .context("Wrong context indicatif style.")?;
    let mut main_bar = ProgressBar::new(repo_infos.len() as u64);
    main_bar.set_style(style);

    // Add the main bar to the multi_bar at the last possible position.
    main_bar = multi_bar.insert(0, main_bar);
    // Tick once to immediately show it.
    main_bar.tick();

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
                repo_info = match handle_repo(&multi_bar, repo_info, &envs) {
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
            let mut repo_info = handle_repo(&multi_bar, repo_info, &envs)?;
            repo_info.check_time = Some(start.elapsed().as_millis() as usize);

            debug!("Check took {}ms", start.elapsed().as_millis());
            results.push(repo_info);
        }

        results
    };

    main_bar.finish();
    let _ = multi_bar.clear();

    for info in repo_infos.iter() {
        let repo = state
            .repositories
            .iter_mut()
            .find(|r| r.path == info.path)
            .context("Expect repository to be there")?;

        repo.check_time = info.check_time;
    }
    state.save()?;

    print_status(repo_infos, show_all)?;

    Ok(())
}
