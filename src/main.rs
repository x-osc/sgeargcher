use std::{path::PathBuf, sync::OnceLock};

use clap::Parser;
use shadow_rs::shadow;

use crate::config::get_config_or_create;

mod config;
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
