use std::{fs::File, path::PathBuf};

use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize};
use serde_with::{DefaultOnError, serde_as};
use shellexpand::tilde;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SshKey {
    /// The name of the key
    pub name: String,
    /// The path to the private key.
    path: PathBuf,
    /// Whether this key is expected to exist.
    #[serde(default = "true_value")]
    pub optional: bool,
}

impl SshKey {
    pub fn path(&self) -> PathBuf {
        PathBuf::from(tilde(&self.path.to_string_lossy()).into_owned())
    }
}

pub fn true_value() -> bool {
    true
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Hook {
    path: PathBuf,
    pub command: String,
}

impl Hook {
    pub fn path(&self) -> PathBuf {
        PathBuf::from(tilde(&self.path.to_string_lossy()).into_owned())
    }
}

#[serde_as]
#[derive(Deserialize, Serialize)]
pub struct GeilConfig {
    /// All paths that're actively watched for new repositories
    pub watched: Vec<PathBuf>,
    /// All paths that're explicitly ignored.
    #[serde(default = "Default::default")]
    pub ignored: Vec<PathBuf>,
    #[serde_as(deserialize_as = "DefaultOnError")]
    pub repositories: Vec<PathBuf>,
    #[serde(default = "Default::default")]
    pub keys: Vec<SshKey>,

    #[serde(default = "Default::default")]
    pub hooks: Vec<Hook>,
}

impl GeilConfig {
    pub fn new() -> GeilConfig {
        GeilConfig {
            ignored: Vec::new(),
            watched: Vec::new(),
            repositories: Vec::new(),
            keys: Vec::new(),
            hooks: Vec::new(),
        }
    }

    pub fn ignored(&self) -> impl Iterator<Item = PathBuf> {
        self.ignored
            .iter()
            .map(|old_path| PathBuf::from(tilde(&old_path.to_string_lossy()).into_owned()))
    }

    pub fn watched(&self) -> impl Iterator<Item = PathBuf> {
        self.watched
            .iter()
            .map(|old_path| PathBuf::from(tilde(&old_path.to_string_lossy()).into_owned()))
    }

    pub fn repositories(&self) -> impl Iterator<Item = PathBuf> {
        self.repositories
            .iter()
            .map(|old_path| PathBuf::from(tilde(&old_path.to_string_lossy()).into_owned()))
    }
}

impl GeilConfig {
    /// Load an existing state from the disk or create an empty new one.
    pub fn load() -> Result<GeilConfig> {
        let path = config_path()?;
        // Return default path if it doesn't exist yet
        if !path.exists() {
            let default_config = GeilConfig::new();
            let file = File::create(&path)?;
            serde_yaml::to_writer(file, &default_config)
                .context("Failed to write state to disk:")?;

            println!("Default config file has been written to {path:?}");

            return Ok(default_config);
        }

        let file = File::open(path)?;
        let state = serde_yaml::from_reader(file)?;

        Ok(state)
    }
}

fn config_path() -> Result<PathBuf> {
    let config_dir = dirs::config_dir().ok_or_else(|| anyhow!("Couldn't resolve config dir"))?;
    Ok(config_dir.join("geil.yml"))
}
