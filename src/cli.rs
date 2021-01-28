use std::str::FromStr;

use super::permafrust;
use clap::{App, Arg, ArgMatches, SubCommand};
use sqlx::PgPool;
use uuid::Uuid;

pub fn watch_subcommand<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("watch")
        .about("Watch a root directory")
        .arg(Arg::with_name("PATH").help("Path to watch").required(true))
        .arg(
            Arg::with_name("DEPTH")
                .short("d")
                .long("depth")
                .help("Watch all subdirectories at the specified depth")
                .required(false)
                .takes_value(true)
                .default_value("0"),
        )
}

pub fn unwatch_subcommand<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("unwatch")
        .about("Unwatch root directories")
        .arg(
            Arg::with_name("PATHS")
                .help("Path to unwatch")
                .required(true)
                .multiple(true),
        )
}

pub fn scan_subcommand<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("scan").about("Scan all root directories")
}

pub fn init_subcommand<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("init")
        .about("Init a borg repository")
        .arg(
            Arg::with_name("REPO")
                .short("r")
                .long("repo")
                .help("Borg repo")
                .required(false)
                .env("BORG_REPO"),
        )
        .arg(
            Arg::with_name("ENCRYPTION")
                .short("e")
                .long("encryption")
                .help("Borg encryption")
                .required(false)
                .env("BORG_ENCRYPTION"),
        )
}

pub fn create_subcommand<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("create")
        .about("Archive directories")
        .arg(
            Arg::with_name("UUIDS")
                .help("UUIDs of directories to backup")
                .required(true)
                .multiple(true),
        )
        .arg(
            Arg::with_name("REPO")
                .short("r")
                .long("repo")
                .help("Borg repo")
                .required(false)
                .env("BORG_REPO"),
        )
        .arg(
            Arg::with_name("COMPRESSION")
                .short("c")
                .long("compression")
                .help("Borg compression")
                .required(false)
                .env("BORG_COMPRESSION"),
        )
        .arg(
            Arg::with_name("DRYRUN")
                .short("n")
                .long("dry-run")
                .help("Dry run")
                .required(false),
        )
}

pub fn update_subcommand<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("update")
        .about("Update backups")
        .arg(
            Arg::with_name("UUIDS")
                .help("UUIDs of archives to update")
                .required(true)
                .multiple(true),
        )
        .arg(
            Arg::with_name("REPO")
                .short("r")
                .long("repo")
                .help("Borg repo")
                .required(false)
                .env("BORG_REPO"),
        )
        .arg(
            Arg::with_name("COMPRESSION")
                .short("c")
                .long("compression")
                .help("Borg compression")
                .required(false)
                .env("BORG_COMPRESSION"),
        )
        .arg(
            Arg::with_name("DRYRUN")
                .short("n")
                .long("dry-run")
                .help("Dry run")
                .required(false),
        )
}

pub fn delete_subcommand<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("delete")
        .about("Delete archives")
        .arg(
            Arg::with_name("UUIDS")
                .help("UUIDs of archives to delete")
                .required(true)
                .multiple(true),
        )
        .arg(
            Arg::with_name("REPO")
                .short("r")
                .long("repo")
                .help("Borg repo")
                .required(false)
                .env("BORG_REPO"),
        )
        .arg(
            Arg::with_name("DRYRUN")
                .short("n")
                .long("dry-run")
                .help("Dry run")
                .required(false),
        )
}

pub fn extract_subcommand<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("extract")
        .about("Extract archives")
        .arg(
            Arg::with_name("UUIDS")
                .help("UUIDs of archives to extract")
                .required(true)
                .multiple(true),
        )
        .arg(
            Arg::with_name("REPO")
                .short("r")
                .long("repo")
                .help("Borg repo")
                .required(false)
                .env("BORG_REPO"),
        )
        .arg(
            Arg::with_name("DRYRUN")
                .short("n")
                .long("dry-run")
                .help("Dry run")
                .required(false),
        )
}

pub fn check_subcommand<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("check")
        .about("Check archives")
        .arg(
            Arg::with_name("UUIDS")
                .help("UUIDs of archives to check")
                .required(true)
                .multiple(true),
        )
        .arg(
            Arg::with_name("REPO")
                .short("r")
                .long("repo")
                .help("Borg repo")
                .required(false)
                .env("BORG_REPO"),
        )
        .arg(
            Arg::with_name("REPAIR")
                .long("repair")
                .help("Try to repair inconsistencies")
                .required(false),
        )
}

pub async fn watch(pool: &PgPool, matches: &ArgMatches<'_>) {
    permafrust::watch(
        &mut pool.acquire().await.unwrap(),
        matches.value_of("PATH").unwrap(),
        i16::from_str_radix(matches.value_of("DEPTH").unwrap(), 10)
            .expect("Depth is not a valid number"),
    )
    .await;
}

pub async fn unwatch(pool: &PgPool, matches: &ArgMatches<'_>) {
    for path in matches.values_of("PATHS").unwrap() {
        permafrust::unwatch(&mut pool.acquire().await.unwrap(), path).await;
    }
}

pub async fn scan(pool: &PgPool, _: &ArgMatches<'_>) {
    permafrust::scan(&mut pool.acquire().await.unwrap()).await;
}

pub async fn init(matches: &ArgMatches<'_>) {
    permafrust::init(
        &matches.value_of("REPO").unwrap(),
        &matches.value_of("ENCRYPTION").unwrap(),
    )
    .await;
}

pub async fn create(pool: &PgPool, matches: &ArgMatches<'_>) {
    let uuids: Vec<Uuid> = matches
        .values_of("UUIDS")
        .unwrap()
        .into_iter()
        .map(|s| Uuid::from_str(s).unwrap())
        .collect();
    for uuid in &uuids {
        permafrust::create(
            &mut pool.acquire().await.unwrap(),
            matches.value_of("REPO").unwrap(),
            uuid,
            matches.value_of("COMPRESSION").unwrap(),
            matches.is_present("DRYRUN"),
        )
        .await;
    }
}

pub async fn update(pool: &PgPool, matches: &ArgMatches<'_>) {
    let uuids: Vec<Uuid> = matches
        .values_of("UUIDS")
        .unwrap()
        .into_iter()
        .map(|s| Uuid::from_str(s).unwrap())
        .collect();
    for uuid in &uuids {
        permafrust::update(
            &mut pool.acquire().await.unwrap(),
            matches.value_of("REPO").unwrap(),
            uuid,
            matches.value_of("COMPRESSION").unwrap(),
            matches.is_present("DRYRUN"),
        )
        .await;
    }
}

pub async fn delete(pool: &PgPool, matches: &ArgMatches<'_>) {
    let uuids: Vec<Uuid> = matches
        .values_of("UUIDS")
        .unwrap()
        .into_iter()
        .map(|s| Uuid::from_str(s).unwrap())
        .collect();
    for uuid in &uuids {
        permafrust::delete(
            &mut pool.acquire().await.unwrap(),
            matches.value_of("REPO").unwrap(),
            uuid,
            matches.is_present("DRYRUN"),
        )
        .await;
    }
}

pub async fn extract(pool: &PgPool, matches: &ArgMatches<'_>) {
    let uuids: Vec<Uuid> = matches
        .values_of("UUIDS")
        .unwrap()
        .into_iter()
        .map(|s| Uuid::from_str(s).unwrap())
        .collect();
    for uuid in &uuids {
        permafrust::extract(
            &mut pool.acquire().await.unwrap(),
            matches.value_of("REPO").unwrap(),
            uuid,
            matches.is_present("DRYRUN"),
        )
        .await;
    }
}

pub async fn check(pool: &PgPool, matches: &ArgMatches<'_>) {
    let uuids: Vec<Uuid> = matches
        .values_of("UUIDS")
        .unwrap()
        .into_iter()
        .map(|s| Uuid::from_str(s).unwrap())
        .collect();
    for uuid in &uuids {
        permafrust::check(
            &mut pool.acquire().await.unwrap(),
            matches.value_of("REPO").unwrap(),
            uuid,
            matches.is_present("REPAIR"),
        )
        .await;
    }
}
