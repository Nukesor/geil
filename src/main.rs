use anyhow::Result;
use clap::Parser;
use cli::{CliArguments, SubCommand};
use simplelog::{Config, LevelFilter, SimpleLogger};

mod cli;
mod commands;
mod display;
mod git;
mod process;
mod repository_info;
mod ssh_key;
mod state;

use ssh_key::load_keys;
use state::State;

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
        SubCommand::Add { repos } => commands::add(&mut state, repos),
        SubCommand::Remove { repos } => commands::remove(&mut state, repos),
        SubCommand::Watch { directories } => commands::watch(&mut state, &directories),
        SubCommand::Unwatch { directories } => commands::unwatch(&mut state, &directories),
        SubCommand::Ignore { directories } => commands::ignore(&mut state, &directories),
        SubCommand::Info => commands::print_info(&mut state),
        SubCommand::Update {
            all,
            not_parallel,
            threads,
        } => {
            state.scan()?;
            load_keys(&state)?;
            commands::update(&mut state, all, !not_parallel, threads)
        }
        SubCommand::Check {
            all,
            not_parallel,
            threads,
        } => {
            state.scan()?;
            load_keys(&state)?;
            commands::check(&mut state, all, !not_parallel, threads)
        }
        SubCommand::Keys { cmd } => ssh_key::handle_key_command(state, cmd),
    }
}
