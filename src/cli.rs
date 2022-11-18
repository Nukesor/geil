use std::path::PathBuf;

use clap::{ArgAction, Parser};

#[derive(Parser, Debug)]
#[clap(
    name = "Geil",
    about = "A git repository manager",
    author = env!("CARGO_PKG_AUTHORS"),
    version = env!("CARGO_PKG_VERSION")
)]
pub struct CliArguments {
    /// Verbose mode (-v, -vv, -vvv)
    #[clap(short, long, action = ArgAction::Count)]
    pub verbose: u8,

    #[clap(subcommand)]
    pub cmd: SubCommand,
}

#[derive(Parser, Debug)]
pub enum SubCommand {
    /// Add one or more repositories to your watchlist
    Add {
        /// The repository that should be watched
        repos: Vec<PathBuf>,
    },

    /// Add a directory which should be searched for repositories.
    /// The maximum depths for this search is 5 subdirectories.
    /// This repository will be searched for repos every time you run `update` or `check`.
    Watch {
        /// The directory that should be watched
        directories: Vec<PathBuf>,
    },

    /// This is the main command of `geil`. This will:
    /// - Fetch all branches from a remote
    /// - Check stash sizes
    /// - Check for local changes
    /// - Update via fast-forward if possible
    Update {
        /// Show all repositories and not only those that are somehow interesting
        #[clap(short, long)]
        all: bool,

        /// Don't run repository checks in parallel
        /// This is useful in combination with the verbose flag for debugging.
        #[clap(short, long)]
        not_parallel: bool,

        /// The amount of threads that should run in parallel for checking repositories.
        #[clap(short, long)]
        threads: Option<usize>,
    },
}
