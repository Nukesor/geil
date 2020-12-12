use anyhow::Result;
use clap::Clap;
use git2::{RemoteCallbacks, Repository};
use log::error;
use simplelog::{Config, LevelFilter, SimpleLogger};

mod cli;
mod git;
#[macro_use]
mod macros;
mod state;

use cli::*;
use git::*;
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
            return Ok(());
        }
        SubCommand::Watch { directory: path } => {
            if !path.exists() {
                error!("Cannot find directory at {:?}", path);
            }
            if !state.watched.contains(&path) {
                state.watched.push(path);
            }
            scan(&mut state)?;
            return Ok(());
        }
        SubCommand::Check => {
            scan(&mut state)?;
            state.save()?;
        }
        SubCommand::Update => {
            scan(&mut state)?;
            state.save()?;
        }
    };
    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(|_, _, _| credentials::get_credentials());

    for path in state.repositories.iter() {
        let repository = Repository::open(path)?;
        let head = repository.head()?;
        if !head.is_branch() || head.name().is_none() {
            continue;
        }

        // Check if we can find a remote for the current branch.
        let remote = continue_on_err!(repository.branch_remote_name(head.name().unwrap()));
        // Check if the remote is valid utf8.
        let remote = continue_on_none!(remote.as_str());
        // Check if the branch has a valid shorthand.
        let branch = continue_on_none!(head.shorthand());

        update_repo(&repository, &remote, &branch)?;
        show_stats(path)?;
    }

    Ok(())
}
