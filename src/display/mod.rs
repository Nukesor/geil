use anyhow::Result;
use comfy_table::*;

use crate::repository_info::RepositoryInfo;

pub fn print_status(repo_infos: Vec<RepositoryInfo>) -> Result<()> {
    let mut table = Table::new();
    table.set_content_arrangement(ContentArrangement::Dynamic);

    table.set_header(vec!["Path", "State", "Stash", "Local"]);

    for info in repo_infos.iter() {
        let mut row = Vec::new();
        row.push(info.path.to_string_lossy().into_owned());
        row.push(info.state.to_string());
        row.push(info.stashed.to_string());
        row.push(info.local_changes.to_string());
        table.add_row(row);
    }

    println!("{}", table);

    Ok(())
}
