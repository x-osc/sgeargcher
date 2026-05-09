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
    Ok(toml::from_str(&contents)?)
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct MetaSearchConfig {
    pub server: ServerConfig,
}

#[derive(Debug, Serialize, Deserialize)]
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
