use anyhow::Result;

use crate::{config::GeilConfig, state::State};

pub fn print_info(config: &GeilConfig, state: &State) -> Result<()> {
    if !config.watched.is_empty() {
        println!("Watched folders:");
        for watched in config.watched() {
            println!("  - {watched:?}");
        }
        println!();
    }

    if !config.ignored.is_empty() || !state.ignored.is_empty() {
        println!("Ignored folders:");
        for ignored in config.ignored().chain(state.ignored.clone().into_iter()) {
            println!("  - {ignored:?}");
        }
        println!();
    }

    if !config.keys.is_empty() {
        println!("Known keys:");
        for key in &config.keys {
            println!("  - {} ({:?})", key.name, key.path());
        }
        println!();
    }

    if !state.repositories.is_empty() {
        println!("Known repositories:\n");
        for repo in &state.repositories {
            println!("  - {:?}", repo.path);
        }
    }

    Ok(())
}
