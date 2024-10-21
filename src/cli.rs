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

    /// Remove a repository from the list of known repositories.
    /// Note: The repository will be re-added if it's inside a watched folder!
    Remove {
        /// The repositories that should be removed
        repos: Vec<PathBuf>,
    },

    /// Add a directory which should be searched for repositories.
    /// The maximum depths for this search is 5 subdirectories.
    /// This repository will be searched for repos every time you run `update` or `check`.
    Watch {
        /// The directory that should be watched
        directories: Vec<PathBuf>,
    },

    /// Unwatch folders and remove all repositories that can be found inside
    Unwatch {
        /// The directory that should be watched
        directories: Vec<PathBuf>,
    },

    /// Ignore a specific repository.
    /// This can be useful, if you want to ignore a specific directory inside a watched directory.
    Ignore {
        /// The directory that should be ignored
        directories: Vec<PathBuf>,
    },

    /// Print information about the current configuration of geil.
    Info,

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

    /// Only check for local changes
    /// - Check stash sizes
    /// - Check for local changes
    Check {
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

    /// The nested key subcommand
    Keys {
        #[clap(subcommand)]
        cmd: KeysCmd,
    },

    /// The nested hooks subcommand
    Hooks {
        #[clap(subcommand)]
        cmd: HooksCmd,
    },
}

#[derive(Parser, Debug)]
pub enum KeysCmd {
    /// Add one or more repositories to your watchlist
    Add {
        /// The name you want to give the key.
        name: String,
        /// The absolute path to the private key.
        path: PathBuf,
    },

    /// List all known keys.
    List,

    /// Remove a string by name.
    Remove { name: String },
}

#[derive(Parser, Debug)]
pub enum HooksCmd {
    /// Add one or more repositories to your watchlist
    Add {
        /// The path of the repository
        repo_path: PathBuf,

        /// The command to execute
        command: String,
    },

    /// List all known keys.
    List,

    /// Remove a command by repository.
    Remove { repo_path: PathBuf },
}
