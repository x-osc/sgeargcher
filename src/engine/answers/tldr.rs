use std::{io::Cursor, path::PathBuf};

use async_trait::async_trait;
use tokio::fs;
use wreq::redirect;
use zip::ZipArchive;

use crate::engine::{answers::AnswerEngine, scrapers::SearchContext};

pub struct TldrAnswer {
    dir: PathBuf,
}

impl TldrAnswer {
    pub async fn new(dir: PathBuf) -> anyhow::Result<Self> {
        if !(dir.join("LICENSE.md").is_file() && dir.join("common").is_dir()) {
            redownload_cache(dir.clone()).await?;
        } else {
            println!("tldr cache already exists, skipping...");
        }

        Ok(Self { dir })
    }
}

async fn redownload_cache(dir: PathBuf) -> anyhow::Result<()> {
    let url = "https://github.com/tldr-pages/tldr/releases/latest/download/tldr-pages.zip";
    println!("downloading tldr zip from {}", url);
    let bytes = wreq::Client::new()
        .get(url)
        .redirect(redirect::Policy::limited(6))
        .send()
        .await?
        .bytes()
        .await?;
    println!("download complete");

    if dir.exists() {
        println!("replacing tldr cache dir {}", dir.display());
        fs::remove_dir_all(&dir).await?;
    }
    fs::create_dir_all(&dir).await?;

    tokio::task::spawn_blocking(move || -> anyhow::Result<()> {
        let mut archive = ZipArchive::new(Cursor::new(bytes))?;
        archive.extract(&dir)?;

        Ok(())
    })
    .await??;

    println!("tldr archive created");

    Ok(())
}

#[async_trait]
impl AnswerEngine for TldrAnswer {
    async fn query(&self, search: SearchContext) -> Option<String> {
        None
    }
}
