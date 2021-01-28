use std::collections::HashMap;
use std::env::vars;

use anyhow::Result;
use clap::Clap;
use indicatif::{ProgressBar, ProgressStyle};
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
            state.save()?;
            return Ok(());
        }
        SubCommand::Watch { directory: path } => {
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
            state.scan()?;
            return Ok(());
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

    // Set up the styling for the progress bar.
    let style = ProgressStyle::default_bar().template("{msg}: {wide_bar} {pos}/{len}");
    let bar = ProgressBar::new(repo_infos.len() as u64);
    bar.set_style(style);
    bar.set_message("Checking repositories");
    let results: Vec<Result<RepositoryInfo>> = repo_infos
        .into_par_iter()
        // Commend above and uncomment below for debug
        //.into_iter()
        .map(|repo_info| {
            bar.inc(1);
            handle_repo(repo_info, &envs)
        })
        .collect();

    bar.finish_with_message("All done: ");

    let mut repo_infos = Vec::new();
    for result in results {
        repo_infos.push(result?);
    }

    print_status(repo_infos, show_all)?;

    Ok(())
}
