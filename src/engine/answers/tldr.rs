use std::{collections::HashMap, io::Cursor, path::PathBuf};

use crate::regex;
use anyhow::Context;
use async_trait::async_trait;
use maud::{Markup, PreEscaped, html};
use regex::Captures;
use tokio::fs;
use wreq::redirect;
use zip::ZipArchive;

use crate::engine::{answers::AnswerEngine, scrapers::SearchContext};

pub struct TldrAnswer {
    /// map of cmd_name to path (relative to tldr dir)
    index: HashMap<String, Vec<TldrEntry>>,
    dir: PathBuf,
}

#[expect(dead_code)]
pub struct TldrEntry {
    name: String,
    platform: String,
    path: PathBuf,
}

impl TldrAnswer {
    pub async fn new(dir: PathBuf) -> anyhow::Result<Self> {
        if !(dir.join("LICENSE.md").is_file() && dir.join("common").is_dir()) {
            redownload_cache(dir.clone()).await?;
        } else {
            println!("tldr cache already exists, skipping...");
        }

        println!("building tldr index..");
        let mut index = HashMap::new();

        // TODO: platform order
        let mut platforms = fs::read_dir(&dir).await?;
        while let Some(platform) = &platforms.next_entry().await? {
            if !platform.path().is_dir() {
                continue;
            }

            let platform_name = platform.file_name();

            let mut files = fs::read_dir(platform.path()).await?;
            while let Some(file) = files.next_entry().await? {
                let path = PathBuf::from(platform.file_name()).join(file.file_name());

                let Some(name) = file
                    .file_name()
                    .to_string_lossy()
                    .to_string()
                    .strip_suffix(".md")
                    .map(|s| s.to_owned())
                else {
                    continue;
                };

                let page = TldrEntry {
                    name: name.to_owned(),
                    platform: platform_name.to_string_lossy().into(),
                    path: path,
                };

                index
                    .entry(name.to_owned())
                    .or_insert(Vec::new())
                    .push(page);
            }
        }

        Ok(Self { dir, index })
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
        let query = search.query.trim();

        let regex1 = regex!(r"^(.+)\s+(?:cmd|command)$");
        let regex2 = regex!(r"^linux\s+(.+)$");
        let regex3 = regex!(r"^(.+)\s+linux");

        let word = if let Some(caps) = regex1.captures(query) {
            caps.get(1).map(|m| m.as_str())
        } else if let Some(caps) = regex2.captures(query) {
            caps.get(1).map(|m| m.as_str())
        } else if let Some(caps) = regex3.captures(query) {
            caps.get(1).map(|m| m.as_str())
        } else {
            None
        }?
        .trim()
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("-");

        let pages = self.index.get(&word)?;
        let page = pages.first()?;

        let content = match fs::read_to_string(self.dir.join(&page.path)).await {
            Ok(c) => c,
            Err(e) => {
                println!("{}", e);
                return None;
            }
        };
        let page = parse(&content).ok()?;

        Some(page_to_html(&page).into_string())
    }
}

fn page_to_html(page: &TldrPage) -> Markup {
    html! {
        h2.cmd-name { (page.title) }
        p.cmd-description {
            (page.description)
        }
        ul.cmd-examples {
            @for example in page.examples.iter() {
                li {
                    p.cmd-ex-description {
                        (example.description)
                    }
                    pre.command {
                        code { (example.command) }
                    }
                }
            }
        }
    }
}

#[derive(Debug)]
struct TldrPage {
    title: String,
    description: PreEscaped<String>,
    examples: Vec<TldrExample>,
}

#[derive(Debug)]
struct TldrExample {
    description: String,
    command: String,
}

fn parse(markdown: &str) -> anyhow::Result<TldrPage> {
    let mut lines = markdown.lines().peekable();

    let title = lines
        .find(|l| l.starts_with("#"))
        .context("missing title")?
        .trim_start_matches("#")
        .to_string()
        .trim()
        .to_string();

    let mut description_parts = Vec::new();
    for line in lines.by_ref() {
        if line.starts_with("-") {
            break;
        }

        if let Some(part) = line.strip_prefix(">") {
            description_parts.push(part.trim().to_string());
        }
    }
    let description = description_parts.join(" ");
    let desc_links = regex::Regex::new(r"<(https?://[^>]+)>").unwrap();
    let description = desc_links
        .replace_all(&description, r#"<a href="$1">$1</a>"#)
        .into_owned();

    let mut examples = Vec::new();
    for line in lines.by_ref() {
        let line = line.trim();
        match line {
            l if l.starts_with('-') => {
                examples.push(TldrExample {
                    description: line
                        .trim_start_matches("-")
                        .trim_end_matches(":")
                        .trim()
                        .to_string(),
                    command: "".into(),
                });
            }
            l if l.starts_with('`') && l.ends_with('`') => {
                if let Some(ex) = examples.last_mut().filter(|e| e.command.is_empty()) {
                    let command = l.trim_matches('`').to_string();
                    ex.command = render_command(&command, OptionMode::Both);
                }
            }
            _ => {}
        }
    }

    examples.retain(|e| !e.command.is_empty());
    Ok(TldrPage {
        title: title,
        description: PreEscaped(description),
        examples: examples,
    })
}

#[derive(Debug, Default)]
#[allow(dead_code)]
pub enum OptionMode {
    #[default]
    Both,
    Short,
    Long,
}

pub fn render_command(input: &str, mode: OptionMode) -> String {
    let regex = regex!(
        r"(?x)
        (?P<esc_open>\\\{\\\{)
        |(?P<esc_close>\\\}\\\})
        |(?P<option>\{\{\[(?P<short>[^\|]+)\|(?P<long>[^\]]+)\]\}\})
        |(?P<placeholder>\{\{(?P<value>.+?)\}\})",
    );

    regex
        .replace_all(input, |caps: &Captures| {
            if caps.name("esc_open").is_some() {
                "{{".to_owned()
            } else if caps.name("esc_close").is_some() {
                "}}".to_owned()
            } else if caps.name("option").is_some() {
                let short = caps.name("short").unwrap().as_str();
                let long = caps.name("long").unwrap().as_str();
                match mode {
                    OptionMode::Both => format!("[{}|{}]", short, long),
                    OptionMode::Short => short.to_owned(),
                    OptionMode::Long => long.to_owned(),
                }
            } else {
                caps.name("value").unwrap().as_str().to_owned()
            }
        })
        .into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_placeholder() {
        assert_eq!(
            render_command("ping {{example.com}}", OptionMode::Both),
            "ping example.com"
        );
    }

    #[test]
    fn escaped_braces() {
        assert_eq!(
            render_command(r"'\{\{range\}\}' {{container}}", OptionMode::Both),
            r"'{{range}}' container"
        );
    }

    #[test]
    fn partial_escape_works() {
        // \{{ is not a full escape
        assert_eq!(
            render_command(
                r"mount \\{{computer_name}}\{{share_name}} Z:",
                OptionMode::Both
            ),
            r"mount \\computer_name\share_name Z:"
        );
    }

    #[test]
    fn inner_braces_preserved() {
        assert_eq!(
            render_command("git stash show --patch {{stash@{0}}}", OptionMode::Both),
            "git stash show --patch stash@{0}"
        );
    }

    #[test]
    fn option_modes() {
        let cmd = "git add {{[-A|--all]}}";
        assert_eq!(render_command(cmd, OptionMode::Both), "git add [-A|--all]");
        assert_eq!(render_command(cmd, OptionMode::Short), "git add -A");
        assert_eq!(render_command(cmd, OptionMode::Long), "git add --all");
    }
}
