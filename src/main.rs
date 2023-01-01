use std::env::vars;
use std::time::Instant;
use std::{collections::HashMap, path::PathBuf};

use anyhow::{Context, Result};
use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
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
        SubCommand::Watch { directories } => watch(state, directories),
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

fn watch(mut state: State, directories: Vec<PathBuf>) -> Result<()> {
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

    let repo_infos = if parallel {
        // Set up the styling for the progress bar.
        let style = ProgressStyle::default_bar()
            .template("{msg}: {wide_bar} {pos}/{len}")
            .context("Wrong context indicatif style.")?;
        let bar = ProgressBar::new(repo_infos.len() as u64);

        // Set the amount of threads, if specified.
        if let Some(threads) = threads {
            rayon::ThreadPoolBuilder::new()
                .num_threads(threads)
                .build_global()
                .unwrap();
        }

        bar.set_style(style);
        bar.set_message("Checking repositories");
        let results: Result<Vec<RepositoryInfo>> = repo_infos
            .into_par_iter()
            .map(|repo_info| {
                bar.inc(1);

                // Handle the repository and track execution time.
                let start = Instant::now();
                let mut repo_info = handle_repo(repo_info, &envs)?;
                repo_info.check_time = Some(start.elapsed().as_millis() as usize);

                Ok(repo_info)
            })
            .collect();

        bar.finish_with_message("All done: ");

        results?
    } else {
        let mut results = Vec::new();
        for repo_info in repo_infos.into_iter() {
            // Handle the repository and track execution time.
            let start = Instant::now();
            let mut repo_info = handle_repo(repo_info, &envs)?;
            repo_info.check_time = Some(start.elapsed().as_millis() as usize);

            debug!("Check took {}ms", start.elapsed().as_millis());
            results.push(repo_info);
        }

        results
    };

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
