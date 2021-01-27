use std::path::PathBuf;

use clap::Clap;

#[derive(Clap, Debug)]
#[clap(
    name = "Geil",
    about = "A git repository manager",
    author = env!("CARGO_PKG_AUTHORS"),
    version = env!("CARGO_PKG_VERSION")
)]
pub struct CliArguments {
    /// Verbose mode (-v, -vv, -vvv)
    #[clap(short, long, parse(from_occurrences))]
    pub verbose: u8,

    #[clap(subcommand)]
    pub cmd: SubCommand,
}

#[derive(Clap, Debug)]
pub enum SubCommand {
    /// Add a repository
    Add {
        /// The repository that should be watched
        repo: PathBuf,
    },

    /// Add a directory which should be searched for repositories.
    /// The maximum depths for this search is 5 subdirectories.
    Watch {
        /// The directory that should be watched
        directory: PathBuf,
    },

    /// Try a "git pull" for all repositories.
    Update {
        /// Show all repositories and not only those that are somehow interesting
        #[clap(short, long)]
        all: bool,
    },

    /// Do a quick check on the current status of all repositories.
    /// Doesn't alter anything. Only displays the current status.
    Check {
        /// Show all repositories and not only those that are somehow interesting
        #[clap(short, long)]
        all: bool,
    },
}
