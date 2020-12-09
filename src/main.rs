use anyhow::Result;
use clap::Clap;
use log::error;
use simplelog::{Config, LevelFilter, SimpleLogger};

mod cli;
mod git;
mod state;

use cli::*;
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
        SubCommand::Add { repo: path } => {
            if !path.exists() {
                error!("Cannot find repository at {:?}", path);
            }
            if !state.repositories.contains(&path) {
                state.repositories.push(path);
            }
        }
        SubCommand::Watch { directory: path } => {
            if !path.exists() {
                error!("Cannot find directory at {:?}", path);
            }
            if !state.watched.contains(&path) {
                state.watched.push(path);
            }
        }
        SubCommand::Check => {
            scan(&mut state)?;
            state.save()?;
        }
        SubCommand::Update => {
            scan(&mut state)?;
            state.save()?;
        }
    }

    for repository in state.repositories {
        println!("{:?}", repository);
    }

    Ok(())
}
