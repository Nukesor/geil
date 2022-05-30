use std::env::vars;
use std::time::Instant;
use std::{collections::HashMap, path::PathBuf};

use anyhow::Result;
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
mod state;

use cli::*;
use display::*;
use git::*;
use repository_info::*;
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
            update(state, all, !not_parallel, threads)
        }
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
        if !state.repositories.contains(&real_path) {
            println!("Added repository: {:?}", &real_path);
            state.repositories.push(real_path);
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

fn update(state: State, show_all: bool, parallel: bool, threads: Option<usize>) -> Result<()> {
    // We create a struct for our internal representation for each repository
    let mut repo_infos: Vec<RepositoryInfo> = Vec::new();
    for path in state.repositories.iter() {
        let repository_info = RepositoryInfo::new(path.clone());
        repo_infos.push(repository_info);
    }

    // Save all environment variables for later injection into git
    let mut envs = HashMap::new();
    for (key, value) in vars() {
        envs.insert(key, value);
    }

    let mut results: Vec<Result<RepositoryInfo>>;
    if parallel {
        // Set up the styling for the progress bar.
        let style = ProgressStyle::default_bar().template("{msg}: {wide_bar} {pos}/{len}");
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
        results = repo_infos
            .into_par_iter()
            .map(|repo_info| {
                bar.inc(1);
                handle_repo(repo_info, &envs)
            })
            .collect();

        bar.finish_with_message("All done: ");
    } else {
        results = Vec::new();
        for repo_info in repo_infos.into_iter() {
            let start = Instant::now();
            results.push(handle_repo(repo_info, &envs));
            debug!("Check took {}ms", start.elapsed().as_millis());
        }
    }

    let mut repo_infos = Vec::new();
    for result in results {
        repo_infos.push(result?);
    }

    print_status(repo_infos, show_all)?;

    Ok(())
}
