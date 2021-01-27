use anyhow::Result;
use comfy_table::*;

use crate::repository_info::{RepositoryInfo, RepositoryState};

pub fn print_status(repo_infos: Vec<RepositoryInfo>, show_all: bool) -> Result<()> {
    let mut table = Table::new();
    table.set_content_arrangement(ContentArrangement::Dynamic);
    table.load_preset(comfy_table::presets::UTF8_FULL);

    table.set_header(vec!["Path", "State", "Local Changes", "Stash size"]);

    for info in repo_infos.iter() {
        if !show_all
            && matches!(info.state, RepositoryState::UpToDate)
            && info.stashed == 0
            && !info.local_changes
        {
            continue;
        }

        let mut row = Vec::new();
        row.push(Cell::new(info.path.to_string_lossy().into_owned()));
        row.push(format_state(&info.state));
        row.push(format_local_changes(info.local_changes));
        row.push(format_number(info.stashed));
        table.add_row(row);
    }

    println!("{}", table);

    Ok(())
}

pub fn format_state(state: &RepositoryState) -> Cell {
    match state {
        RepositoryState::Updated => Cell::new("Updated").fg(Color::Green),
        RepositoryState::UpToDate => Cell::new("Up to date").fg(Color::DarkGreen),
        RepositoryState::Fetched => Cell::new("Fetched").fg(Color::Yellow),
        RepositoryState::NoFastForward => Cell::new("No fast forward").fg(Color::Red),
    }
}

pub fn format_number(number: usize) -> Cell {
    match number {
        0 => Cell::new("0").fg(Color::Green),
        _ => Cell::new(number.to_string()).fg(Color::Red),
    }
}

pub fn format_local_changes(local_changes: bool) -> Cell {
    match local_changes {
        true => Cell::new("yes").fg(Color::Red),
        false => Cell::new("no").fg(Color::Green),
    }
}
