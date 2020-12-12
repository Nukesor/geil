use git2::{Cred, Error};

pub fn get_credentials() -> Result<Cred, Error> {
    let credentials = Cred::ssh_key_from_agent("nuke");

    if let Ok(credentials) = credentials {
        return Ok(credentials);
    }

    let cred = Cred::ssh_key(
        "nuke",
        None,
        std::path::Path::new("/home/nuke/.ssh/id_rsa"),
        None,
    )?;

    Ok(cred)
}
