use std::collections::HashMap;
use std::env::vars;

use anyhow::Result;
use clap::Clap;
use indicatif::ParallelProgressIterator;
use log::error;
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

    let mut show_all = false;
    match opt.cmd {
        SubCommand::Add { repos } => {
            for path in repos {
                if !path.exists() || !path.is_dir() {
                    error!("Cannot find repository at {:?}", path);
                }
                if !state.repositories.contains(&path) {
                    state.repositories.push(path);
                }
            }
            return Ok(());
        }
        SubCommand::Watch { directory: path } => {
            if !path.exists() || !path.is_dir() {
                error!("Cannot find directory at {:?}", path);
            }
            if !state.watched.contains(&path) {
                state.watched.push(path);
            }
            state.scan()?;
            return Ok(());
        }
        SubCommand::Check { all } => {
            state.scan()?;
            show_all = all;
        }
        SubCommand::Update { all } => {
            state.scan()?;
            show_all = all;
        }
    };

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

    let results: Vec<Result<RepositoryInfo>> = repo_infos
        .into_par_iter()
        .progress()
        .map(|repo_info| handle_repo(repo_info, &envs))
        .collect();

    let mut repo_infos = Vec::new();
    for result in results {
        repo_infos.push(result?);
    }

    print_status(repo_infos, show_all)?;

    Ok(())
}
