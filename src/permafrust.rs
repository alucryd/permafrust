use super::borg;
use super::database::*;
use super::df;
use super::du;
use async_std::path::Path;
use chrono::{DateTime, Local, NaiveDateTime};
use sqlx::PgConnection;
use std::convert::TryFrom;
use uuid::Uuid;
use walkdir::{DirEntry, WalkDir};

pub async fn watch(conn: &mut PgConnection, path: &str, depth: i16) {
    let path = Path::new(path)
        .canonicalize()
        .await
        .expect("Failed to canonicalize path");
    let path = path.as_os_str().to_str().unwrap();
    let directory = find_root_directory_by_path(conn, path).await;
    match directory {
        Some(_) => println!("Directory {} is already watched", path),
        None => create_root_directory(conn, path, depth).await,
    }
}

pub async fn unwatch(conn: &mut PgConnection, path: &str) {
    let path = Path::new(path)
        .canonicalize()
        .await
        .expect("Failed to canonicalize path");
    let path = path.as_os_str().to_str().unwrap();
    let directory = find_root_directory_by_path(conn, path).await;
    match directory {
        Some(directory) => delete_root_directory(conn, &directory.id).await,
        None => println!("Directory does not exist"),
    }
}

pub async fn scan(conn: &mut PgConnection) {
    let directories = find_directories(conn).await;
    for directory in directories {
        let path = Path::new(&directory.path);
        if !path.is_dir().await {
            delete_directory(conn, &directory.id).await;
        }
    }

    let root_directories = find_root_directories(conn).await;
    for root_directory in root_directories {
        let directories: Vec<DirEntry> = WalkDir::new(&root_directory.path)
            .into_iter()
            .filter_entry(|e| {
                e.depth() <= usize::try_from(root_directory.depth).unwrap()
                    && e.path().is_dir()
                    && !e.file_name().to_str().unwrap().starts_with(".")
            })
            .filter_map(|v| v.ok())
            .filter(|e| e.depth() == usize::try_from(root_directory.depth).unwrap())
            .collect();
        for directory in directories {
            let path = directory.path().as_os_str().to_str().unwrap();
            let modified_date = get_most_recent_modified_date(path);
            let directory = find_directory_by_path(conn, path).await;
            match directory {
                Some(directory) => {
                    if modified_date > directory.modified_date {
                        update_directory(conn, &directory.id, &modified_date).await;
                    }
                }
                None => create_directory(conn, path, &modified_date, &root_directory.id).await,
            }
        }
    }
}

pub async fn init(repo: &str, encryption: &str) {
    borg::init(repo, encryption)
        .await
        .expect("Failed to init repo");
}

pub async fn create(
    conn: &mut PgConnection,
    repo: &str,
    directory_id: &Uuid,
    compression: &str,
    dry_run: bool,
) {
    let directory = find_directory_by_id(conn, directory_id).await;
    if let Some(archive) = find_archive_by_directory_id(conn, &directory.id).await {
        panic!("Archive {} already exists", &archive.name);
    };
    let df_output = df::main(repo).await;
    let du_output = du::main(&directory.path).await;
    println!(
        "{}",
        f64::from(df_output.avail - du_output.size) / f64::from(df_output.size)
    );
    if f64::from(df_output.avail - du_output.size) / f64::from(df_output.size) < 0.05 {
        panic!("Not enough space");
    }
    let name = get_archive_name(&directory.path);
    let create_output = borg::create(repo, &name, &directory.path, compression, dry_run)
        .await
        .expect("Failed to create archive");
    create_archive(
        conn,
        &name,
        &create_output.repository.id,
        &Local::now().naive_local(),
        directory_id,
    )
    .await;
}

pub async fn update(
    conn: &mut PgConnection,
    repo: &str,
    archive_id: &Uuid,
    compression: &str,
    dry_run: bool,
) {
    let archive = find_archive_by_id(conn, archive_id).await;
    let directory = find_directory_by_id(conn, &archive.directory_id.unwrap()).await;
    let df_output = df::main(repo).await;
    let du_output = du::main(&directory.path).await;
    println!(
        "{}",
        f64::from(df_output.avail - du_output.size) / f64::from(df_output.size)
    );
    if f64::from(df_output.avail - du_output.size) / f64::from(df_output.size) < 0.05 {
        panic!("Not enough space");
    }
    let new_name = format!("{}-new", &archive.name);
    borg::create(repo, &new_name, &directory.path, compression, dry_run)
        .await
        .expect("Failed to create new archive");
    borg::delete(repo, &archive.name, dry_run)
        .await
        .expect("Failed to delete old archive");
    borg::rename(repo, &new_name, &archive.name, dry_run)
        .await
        .expect("Failed to rename new archive");
    update_archive(conn, &archive.id, &Local::now().naive_local()).await;
}

pub async fn delete(conn: &mut PgConnection, repo: &str, archive_id: &Uuid, dry_run: bool) {
    let archive = find_archive_by_id(conn, archive_id).await;
    borg::delete(repo, &archive.name, dry_run)
        .await
        .expect("Failed to delete archive");
    delete_archive(conn, &archive.id).await;
}

pub async fn extract(conn: &mut PgConnection, repo: &str, archive_id: &Uuid, dry_run: bool) {
    let archive = find_archive_by_id(conn, archive_id).await;
    let directory = find_directory_by_id(conn, &archive.directory_id.unwrap()).await;
    borg::check(repo, &archive.name, false)
        .await
        .expect("Failed to check archive");
    borg::extract(repo, &archive.name, &directory.path, dry_run)
        .await
        .expect("Failed to extract archive");
}

pub async fn check(conn: &mut PgConnection, repo: &str, archive_id: &Uuid, repair: bool) {
    let archive = find_archive_by_id(conn, archive_id).await;
    borg::check(repo, &archive.name, repair)
        .await
        .expect("Failed to check archive");
}

fn get_archive_name(path: &str) -> String {
    path.split(|c| c == '/' || c == '_')
        .map(|component| {
            let mut component = component.to_owned();
            component.retain(|c| c.is_ascii_alphanumeric() || c == '_');
            component
        })
        .collect::<Vec<String>>()
        .join("-")
        .replace("--", "-")
        .trim_start_matches("-")
        .to_lowercase()
}

fn get_most_recent_modified_date(path: &str) -> NaiveDateTime {
    let modified_date = WalkDir::new(path)
        .into_iter()
        .filter_map(|v| v.ok())
        .filter(|e| e.path().is_file())
        .map(|e| e.metadata().unwrap().modified().unwrap())
        .max()
        .unwrap();
    DateTime::<Local>::from(modified_date).naive_local()
}
