use anyhow::Result;
use comfy_table::*;

use crate::repository_info::{RepositoryInfo, RepositoryState};

pub fn print_status(mut repo_infos: Vec<RepositoryInfo>, show_all: bool) -> Result<()> {
    // Filter all repos that don't need attention.
    if !show_all {
        repo_infos.retain(|info| {
            !matches!(info.state, RepositoryState::UpToDate | RepositoryState::Ok)
                || info.stashed != 0
        });
    }

    if repo_infos.is_empty() {
        println!("Nothing to do here, everything looks perfectly fine.");
        return Ok(());
    }

    let mut table = Table::new();
    table.set_content_arrangement(ContentArrangement::Dynamic);
    table.load_preset(comfy_table::presets::UTF8_FULL);

    table.set_header(vec!["Path", "State", "Stash size"]);
    for info in repo_infos.iter() {
        table.add_row(vec![
            Cell::new(info.path.to_string_lossy().into_owned()),
            format_state(&info.state),
            format_number(info.stashed),
        ]);
    }

    println!("{table}");

    Ok(())
}

pub fn format_state(state: &RepositoryState) -> Cell {
    match state {
        RepositoryState::Unknown => Cell::new("Unknown").fg(Color::Red),
        RepositoryState::Detached => Cell::new("Detached HEAD").fg(Color::Yellow),
        RepositoryState::Ok => Cell::new("Ok").fg(Color::Green),
        RepositoryState::Updated => Cell::new("Updated").fg(Color::Green),
        RepositoryState::UpToDate => Cell::new("Up to date").fg(Color::DarkGreen),
        RepositoryState::Fetched => Cell::new("Fetched").fg(Color::Yellow),
        RepositoryState::NoFastForward => Cell::new("No fast forward").fg(Color::Red),
        RepositoryState::LocalChanges => Cell::new("Local changes").fg(Color::Red),
        RepositoryState::NotPushed => Cell::new("Unpushed commits").fg(Color::Yellow),
    }
}

pub fn format_number(number: usize) -> Cell {
    match number {
        0 => Cell::new("0").fg(Color::Green),
        _ => Cell::new(number.to_string()).fg(Color::Red),
    }
}
