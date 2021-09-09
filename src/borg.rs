use async_std::fs;
use async_std::io::Error;
use async_std::path::Path;
use async_std::process::{Command, Stdio};
use chrono::NaiveDateTime;
use log::debug;
use serde::{Deserialize, Serialize};

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
    pub id: String,
    pub name: String,
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
pub struct CreateOutput {
    pub archive: Archive,
    pub repository: Repository,
}

#[derive(Deserialize, Serialize)]
pub struct InfoOutput {
    pub repository: Repository,
}

#[derive(Deserialize, Serialize)]
pub struct ListOutput {
    pub archives: Vec<Archive>,
    encryption: Encryption,
    pub repository: Repository,
}

pub async fn init(repo: &str, encryption: &str) -> Result<(), Error> {
    if !Path::new(repo).is_dir().await {
        fs::create_dir_all(repo).await?;
    }
    let mut args: Vec<&str> = Vec::new();
    args.push("init");
    args.push("--progress");
    args.push("--encryption");
    args.push(encryption);
    args.push(repo);
    let mut command = Command::new("borg");
    command
        .args(&args)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    debug!("{:?}", command);
    command.status().await?;
    Ok(())
}

pub async fn info(repo: &str) -> Result<InfoOutput, Error> {
    let mut args: Vec<&str> = Vec::new();
    args.push("info");
    args.push("--json");
    args.push(repo);
    let mut command = Command::new("borg");
    command.args(&args);
    debug!("{:?}", command);
    let output = command.output().await?;
    let info_output = serde_json::from_slice(output.stdout.as_slice())?;
    Ok(info_output)
}

pub async fn list(repo: &str) -> Result<ListOutput, Error> {
    let mut args: Vec<&str> = Vec::new();
    args.push("list");
    args.push("--json");
    args.push(repo);
    let mut command = Command::new("borg");
    command.args(&args);
    debug!("{:?}", command);
    let output = command.output().await?;
    let list_output = serde_json::from_slice(output.stdout.as_slice())?;
    Ok(list_output)
}

pub async fn create(
    repo: &str,
    prefix: &str,
    path: &str,
    compression: &str,
    dry_run: bool,
) -> Result<CreateOutput, Error> {
    let repo_name = format!("{}::{}-{{utcnow:%Y%m%d-%H%M%S}}", repo, prefix);
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
    args.push("--noacls");
    args.push("--nobsdflags");
    args.push("--noxattrs");
    args.push(&repo_name);
    args.push(".");
    let mut command = Command::new("borg");
    command
        .current_dir(path)
        .args(&args)
        .stderr(Stdio::inherit());
    debug!("{:?}", command);
    let output = command.output().await?;
    let create_output = serde_json::from_slice(output.stdout.as_slice())?;
    Ok(create_output)
}

pub async fn delete(repo: &str, prefix: &str, dry_run: bool) -> Result<(), Error> {
    let mut args: Vec<&str> = Vec::new();
    args.push("delete");
    if dry_run {
        args.push("--dry-run");
    }
    args.push("--progress");
    args.push("--prefix");
    args.push(&prefix);
    args.push(&repo);
    let mut command = Command::new("borg");
    command
        .args(&args)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    debug!("{:?}", command);
    command.status().await?;
    Ok(())
}

pub async fn prune(repo: &str, prefix: &str, dry_run: bool) -> Result<(), Error> {
    let mut args: Vec<&str> = Vec::new();
    args.push("prune");
    if dry_run {
        args.push("--dry-run");
    }
    args.push("--keep-last");
    args.push("1");
    args.push("--prefix");
    args.push(prefix);
    args.push(&repo);
    let mut command = Command::new("borg");
    command
        .args(&args)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    debug!("{:?}", command);
    command.status().await?;
    Ok(())
}

pub async fn extract(repo: &str, name: &str, path: &str, dry_run: bool) -> Result<(), Error> {
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
    let mut command = Command::new("borg");
    command
        .current_dir(path)
        .args(&args)
        .stderr(Stdio::inherit());
    debug!("{:?}", command);
    command.status().await?;
    Ok(())
}

pub async fn check(repo: &str, name: &str, repair: bool) -> Result<(), Error> {
    let repo_name = format!("{}::{}", repo, name);
    let mut args: Vec<&str> = Vec::new();
    args.push("check");
    if repair {
        args.push("--repair");
    }
    args.push(&repo_name);
    let mut command = Command::new("borg");
    command
        .args(&args)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    debug!("{:?}", command);
    command.status().await?;
    Ok(())
}
