use std::{
    env::current_dir,
    fs::{self, create_dir},
    path::PathBuf,
    sync::OnceLock,
};

use clap::Parser;
use serde::{Deserialize, Serialize};
use shadow_rs::shadow;

mod engine;
mod utils;
mod web;

shadow!(build);

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let config_dir = args.config_dir;

    let config = get_config_or_create(config_dir.as_ref())?;

    web::run(config).await?;

    Ok(())
}

fn get_config_or_create(dir: Option<&PathBuf>) -> anyhow::Result<MetaSearchConfig> {
    let dir = match dir {
        Some(dir) => dir,
        None => {
            create_dir("config")?;
            &current_dir()?.join("config")
        }
    };

    if !dir.exists() || !dir.is_dir() {
        anyhow::bail!("directory {} does not exist", dir.display())
    }

    let config_path = dir.join("config.toml");

    if !config_path.exists() {
        let default = MetaSearchConfig::default();

        fs::write(config_path, toml::to_string_pretty(&default).unwrap())?;

        return Ok(default);
    }

    let contents = fs::read_to_string(config_path)?;
    Ok(toml::from_str(&contents)?)
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
struct MetaSearchConfig {
    server: ServerConfig,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
struct ServerConfig {
    bind: String,
    port: u16,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            bind: "127.0.0.1".into(),
            port: 7367,
        }
    }
}

pub fn short_version() -> &'static str {
    static VERSION: OnceLock<String> = OnceLock::new();
    VERSION.get_or_init(|| {
        format!(
            "{name} v{version}",
            name = build::PROJECT_NAME,
            version = build::PKG_VERSION
        )
    })
}

pub fn single_line_version() -> &'static str {
    static VERSION: OnceLock<String> = OnceLock::new();
    VERSION.get_or_init(|| {
        format!(
            "{name} v{version} ({git}{dirty} on {branch} @ {time} with {rust})",
            name = build::PROJECT_NAME,
            version = build::PKG_VERSION,
            git = build::SHORT_COMMIT,
            dirty = if build::GIT_CLEAN { "" } else { "+" },
            branch = build::BRANCH,
            time = build::BUILD_TIME,
            rust = build::RUST_VERSION
        )
    })
}

pub fn version_clap() -> &'static str {
    static VERSION: OnceLock<String> = OnceLock::new();
    VERSION.get_or_init(|| {
        format!(
            "v{version}\n{git}{dirty} on {branch},\ncompiled @ {time}\nwith {rust}",
            version = build::PKG_VERSION,
            git = build::SHORT_COMMIT,
            dirty = if build::GIT_CLEAN { "" } else { "+" },
            branch = build::BRANCH,
            time = build::BUILD_TIME,
            rust = build::RUST_VERSION
        )
    })
}

/// metasearcher enginer
#[derive(Parser, Debug)]
#[command(version = version_clap())]
struct Args {
    /// config directory
    #[arg(long)]
    config_dir: Option<PathBuf>,
}
