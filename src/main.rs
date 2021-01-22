use anyhow::Result;
use clap::Clap;
use git2::{RemoteCallbacks, Repository};
use log::error;
use simplelog::{Config, LevelFilter, SimpleLogger};

mod cli;
mod display;
mod git;
#[macro_use]
mod macros;
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
            state.scan()?;
            return Ok(());
        }
        SubCommand::Check => {
            state.scan()?;
        }
        SubCommand::Update => {
            state.scan()?;
        }
    };

    // We create a struct for our internal representation for each repository
    let mut repo_infos: Vec<RepositoryInfo> = Vec::new();
    for path in state.repositories.iter() {
        let repository_info = RepositoryInfo {
            path: path.clone(),
            state: RepositoryState::NotChecked,
            stashed: 0,
        };
        repo_infos.push(repository_info);
    }

    // We don't necessarily need any callbacks, but they're needed for interaction with git2
    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(|_, _, _| credentials::get_credentials());

    for repo_info in repo_infos.iter_mut() {
        let repository = Repository::open(&repo_info.path)?;
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
    }

    print_status(repo_infos)?;

    Ok(())
}
