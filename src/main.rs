use anyhow::{Context, Result};
use clap::Parser;
use cli::{CliArguments, SubCommand};
use simplelog::{Config, LevelFilter, SimpleLogger};

mod cli;
mod commands;
mod config;
mod display;
mod git;
mod process;
mod repository_info;
mod ssh_key;
mod state;

use ssh_key::load_keys;
use state::State;

use crate::config::GeilConfig;

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

    let mut state = State::load().context("Failed to parse state")?;
    let config = GeilConfig::load().context("Failed to parse config")?;

    match opt.cmd {
        SubCommand::Add { repos } => commands::add(&mut state, repos),
        SubCommand::Remove { repos } => commands::remove(&mut state, repos),
        SubCommand::Ignore { directories } => commands::ignore(&mut state, &directories),
        SubCommand::Info => commands::print_info(&config, &state),
        SubCommand::Update {
            all,
            not_parallel,
            threads,
        } => {
            state.scan(&config)?;
            load_keys(&config)?;
            commands::update(&mut state, &config, all, !not_parallel, threads)
        }
        SubCommand::Check {
            all,
            not_parallel,
            threads,
        } => {
            state.scan(&config)?;
            load_keys(&config)?;
            commands::check(&mut state, &config, all, !not_parallel, threads)
        }
    }
}
