use anyhow::Result;

use crate::state::State;

pub fn print_info(state: &mut State) -> Result<()> {
    if !state.watched.is_empty() {
        println!("Watched folders:");
        for watched in &state.watched {
            println!("  - {watched:?}");
        }
        println!();
    }

    if !state.ignored.is_empty() {
        println!("Ignored folders:");
        for ignored in &state.ignored {
            println!("  - {ignored:?}");
        }
        println!();
    }

    if !state.keys.is_empty() {
        println!("Known keys:");
        for key in &state.keys {
            println!("  - {} ({:?})", key.name, key.path);
        }
        println!();
    }

    if !state.keys.is_empty() {
        println!("Known repositories:\n");
        for repo in &state.repositories {
            println!("  - {:?}", repo.path);
        }
    }

    Ok(())
}
