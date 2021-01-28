use async_std::fs;
use async_std::path::Path;
use async_std::process::{Command, Stdio};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use tide::Result;

mod datetime_format {
    use chrono::NaiveDateTime;
    use serde::{Deserialize, Deserializer, Serializer};

    const DATETIME_FORMAT: &'static str = "%Y-%m-%dT%H:%M:%S%.6f";

    pub fn serialize<S>(date: &NaiveDateTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", date.format(DATETIME_FORMAT));
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDateTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        NaiveDateTime::parse_from_str(&s, DATETIME_FORMAT).map_err(serde::de::Error::custom)
    }
}

#[derive(Deserialize, Serialize)]
pub struct Archive {
    id: String,
    name: String,
    #[serde(with = "datetime_format")]
    pub start: NaiveDateTime,
}

#[derive(Deserialize, Serialize)]
pub struct Encryption {
    mode: String,
}

#[derive(Deserialize, Serialize)]
pub struct Repository {
    pub id: String,
    #[serde(with = "datetime_format")]
    last_modified: NaiveDateTime,
    location: String,
}

#[derive(Deserialize, Serialize)]
pub struct ListOutput {
    archives: Vec<Archive>,
    encryption: Encryption,
    repository: Repository,
}

#[derive(Deserialize, Serialize)]
pub struct CreateOutput {
    pub archive: Archive,
    pub repository: Repository,
}

pub async fn init(repo: &str, encryption: &str) -> Result<()> {
    if !Path::new(repo).is_dir().await {
        fs::create_dir_all(repo).await?;
    }
    let mut args: Vec<&str> = Vec::new();
    args.push("init");
    args.push("--progress");
    args.push("--encryption");
    args.push(encryption);
    args.push(repo);
    Command::new("borg")
        .args(&args)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .await?;
    Ok(())
}

pub async fn list(repo: &str) -> Result<ListOutput> {
    let mut args: Vec<&str> = Vec::new();
    args.push("list");
    args.push("--json");
    args.push(repo);
    let output = Command::new("borg").args(&args).output().await?;
    let list_output = serde_json::from_slice(output.stdout.as_slice())?;
    Ok(list_output)
}

pub async fn create(
    repo: &str,
    name: &str,
    path: &str,
    compression: &str,
    dry_run: bool,
) -> Result<CreateOutput> {
    let repo_name = format!("{}::{}", repo, name);
    let mut args: Vec<&str> = Vec::new();
    args.push("create");
    if dry_run {
        args.push("--dry-run");
    }
    args.push("--json");
    args.push("--progress");
    args.push("--compression");
    args.push(compression);
    args.push("--noatime");
    args.push("--nobsdflags");
    args.push(&repo_name);
    args.push(".");
    let output = Command::new("borg")
        .current_dir(path)
        .args(&args)
        .stderr(Stdio::inherit())
        .output()
        .await?;
    let create_output = serde_json::from_slice(output.stdout.as_slice())?;
    Ok(create_output)
}

pub async fn delete(repo: &str, name: &str, dry_run: bool) -> Result<()> {
    let repo_name = format!("{}::{}", repo, name);
    let mut args: Vec<&str> = Vec::new();
    args.push("delete");
    if dry_run {
        args.push("--dry-run");
    }
    args.push("--progress");
    args.push(&repo_name);
    Command::new("borg")
        .args(&args)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .await?;
    Ok(())
}

pub async fn rename(repo: &str, name: &str, new_name: &str, dry_run: bool) -> Result<()> {
    let repo_name = format!("{}::{}", repo, name);
    let mut args: Vec<&str> = Vec::new();
    args.push("rename");
    if dry_run {
        args.push("--dry-run");
    }
    args.push(&repo_name);
    args.push(new_name);
    Command::new("borg")
        .args(&args)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .await?;
    Ok(())
}

pub async fn extract(repo: &str, name: &str, path: &str, dry_run: bool) -> Result<()> {
    if !Path::new(path).is_dir().await {
        fs::create_dir_all(path).await?;
    }
    let repo_name = format!("{}::{}", repo, name);
    let mut args: Vec<&str> = Vec::new();
    args.push("extract");
    if dry_run {
        args.push("--dry-run");
    }
    args.push(&repo_name);
    Command::new("borg")
        .current_dir(path)
        .args(&args)
        .stderr(Stdio::inherit())
        .status()
        .await?;
    Ok(())
}

pub async fn check(repo: &str, name: &str, repair: bool) -> Result<()> {
    let repo_name = format!("{}::{}", repo, name);
    let mut args: Vec<&str> = Vec::new();
    args.push("check");
    if repair {
        args.push("--repair");
    }
    args.push(&repo_name);
    Command::new("borg")
        .args(&args)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .await?;
    Ok(())
}
