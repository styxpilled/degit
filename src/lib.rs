use flate2::read::GzDecoder;
use indicatif::{ProgressBar, ProgressStyle};
use regex::Regex;
use reqwest::blocking;
use std::{error::Error, path::PathBuf};
use tar::Archive;

#[derive(Debug, Clone)]
pub struct Repository {
    host: String,
    name: String,
    owner: String,
    src: Option<String>,
}

pub enum Host {
    Github,
    Gitlab(String),
    Custom(String),
}

pub fn degit(src: &str) {
    // download(src).unwrap();
    let repo = dbg!(parse(src)).unwrap().unwrap();
    let name = repo.name.clone();
    download(repo, PathBuf::from(format!("./{}", name))).unwrap();
}

fn parse(src: &str) -> Result<Option<Repository>, Box<dyn Error>> {
    let longhand = Regex::new(
        r"(?x)
    (?P<protocol>(git@|https://))
    (?P<host>([\w\.@]+))
    (/|:)
    (?P<owner>[\w,\-,_]+)/(?P<name>[\w,\-,_]+)(.git)?/?",
    )?;
    if let Some(captures) = longhand.captures(src) {
        return Ok(Some(Repository {
            owner: captures.name("owner").unwrap().as_str().to_string(),
            name: captures.name("name").unwrap().as_str().to_string(),
            host: captures.name("host").unwrap().as_str().to_string(),
            src: Some(captures.get(0).unwrap().as_str().to_string()),
        }));
    }
    let shorthand = Regex::new(
        r"(?x)(?P<host>(github|gitlab|bitbucket)?)
            (?P<colon>(:))?
            (?P<owner>[\w,\-,_]+)/(?P<name>[\w,\-,_]+)",
    )?;
    if let Some(captures) = shorthand.captures(src) {
        return Ok(Some(Repository {
            owner: captures.name("owner").unwrap().as_str().to_string(),
            name: captures.name("name").unwrap().as_str().to_string(),
            host: match captures.name("host") {
                Some(host) => match host.as_str() {
                    "" => "github".to_string(),
                    _ => host.as_str().to_string(),
                },
                None => "github".to_string(),
            },
            src: None,
        }));
    }

    Ok(None)
}

pub fn download(repo: Repository, dest: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let src = if repo.src.is_some() {
        repo.src.unwrap()
    } else {
        dbg!("https://{}/{}/{}", &repo.host, &repo.owner, &repo.name);
        match repo.host.as_str() {
            "github" => format!(
                "https://github.com/{}/{}/archive/HEAD.tar.gz",
                repo.owner, repo.name
            ),
            _ => format!("https://{}/{}/{}", repo.host, repo.owner, repo.name),
        }
    };
    let client = blocking::Client::new();
    let response = client.get(src).send()?;
    let total_size = response.content_length();

    let pb = match total_size {
        Some(x) => ProgressBar::with_style(
            ProgressBar::new(x),
            ProgressStyle::default_bar()
                .template(
                    "[{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta}) {msg}",
                )?
                .progress_chars("#>-"),
        ),
        None => ProgressBar::new_spinner(),
    };
    // The header is probably 512B but it can be larger so I'm not hardcoding it in.
    let tar = GzDecoder::new(pb.wrap_read(response));
    let mut archive = Archive::new(pb.wrap_read(tar));
    pb.wrap_iter(
        archive
            .entries()?
            .filter_map(|e| e.ok())
            .map(|mut entry| -> Result<PathBuf, Box<dyn Error>> {
                let path = entry.path()?;
                let path = path
                    .strip_prefix(path.components().next().unwrap())?
                    .to_owned();
                entry.unpack(dest.join(&path))?;
                Ok(path)
            })
            .filter_map(|e| e.ok()),
    )
    .for_each(drop);
    // while let Some(mut item) = response.chunk().await? {
    //     file.write_all_buf(item.borrow_mut()).await?;
    // }
    pb.finish_with_message("\nDone...");
    // pb.finish();
    Ok(())
}
