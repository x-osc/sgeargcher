use std::{fs, path::PathBuf};

use rust_embed::RustEmbed;
use serde::{Deserialize, Serialize};

#[derive(RustEmbed)]
#[folder = "src/defaults"]
struct DefaultConfig;

pub fn get_config_or_create(dir: Option<&PathBuf>) -> anyhow::Result<MetaSearchConfig> {
    let dir = match dir {
        Some(dir) => dir,
        None => {
            fs::create_dir_all("config")?;
            &std::env::current_dir()?.join("config")
        }
    };

    if !dir.exists() || !dir.is_dir() {
        anyhow::bail!("directory {} does not exist", dir.display())
    }

    let config_path = dir.join("config.toml");

    if !config_path.exists() {
        for file in DefaultConfig::iter() {
            let embedded = DefaultConfig::get(&file).unwrap();
            let output_path = dir.join(file.as_ref());

            if let Some(parent) = output_path.parent() {
                fs::create_dir_all(parent)?;
            }

            if !output_path.exists() {
                fs::write(&output_path, &embedded.data)?;
            } else {
                println!(
                    "tried to create file which already exists: {}",
                    output_path.display()
                )
            }
        }
    }

    let contents = fs::read_to_string(config_path)?;
    let mut config = toml::from_str::<MetaSearchConfig>(&contents)?;
    config.config_dir = dir.to_owned();
    Ok(config)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct MetaSearchConfig {
    #[serde(skip)]
    pub config_dir: PathBuf,
    pub themes_dir: PathBuf,
    pub timeout: u64,
    pub server: ServerConfig,
}

impl Default for MetaSearchConfig {
    fn default() -> Self {
        Self {
            config_dir: "config".into(),
            themes_dir: "themes".into(),
            timeout: 3500,
            server: Default::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ServerConfig {
    pub bind: String,
    pub port: u16,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            bind: "127.0.0.1".into(),
            port: 7367,
        }
    }
}
