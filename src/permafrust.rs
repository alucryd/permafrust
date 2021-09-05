use super::borg;
use super::database::*;
use super::df;
use super::du;
use async_std::path::Path;
use blake3::{Hash, Hasher};
use chrono::{DateTime, Local};
use log::info;
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
            let blake3_hash = compute_directory_hash(path);
            let directory = find_directory_by_path(conn, path).await;
            match directory {
                Some(directory) => {
                    if blake3_hash != Hash::from_hex(directory.blake3_hash).unwrap() {
                        update_directory(conn, &directory.id, &blake3_hash.to_hex()).await;
                    }
                }
                None => {
                    create_directory(conn, path, &blake3_hash.to_hex(), &root_directory.id).await;
                }
            }
        }
    }
    let directories = find_directories(conn).await;
    for directory in directories {
        let path = Path::new(&directory.path);
        if !path.is_dir().await {
            delete_directory(conn, &directory.id).await;
        }
        let archive = find_archive_by_directory_id(conn, &directory.id).await;
        match archive {
            Some(archive) => {
                if archive.blake3_hash != directory.blake3_hash {
                    println!("Out of date: {} [{}:{}]", &directory.path, &directory.root_directory_id, &directory.id);
                }
            }
            None => println!("Not backed up: {} [{}:{}]", &directory.path, &directory.root_directory_id, &directory.id),
        }
    }
}

pub async fn init(repo: &str, encryption: &str) {
    borg::init(repo, encryption)
        .await
        .expect("Failed to init repo");
}

pub async fn list(conn: &mut PgConnection, repo: &str) {
    let list_output = borg::list(repo).await.expect("Failed to list repo");
    for archive in list_output.archives {
        let archive =
            find_archive_by_repo_id_and_archive_id(conn, &list_output.repository.id, &archive.id)
                .await
                .unwrap();
        println!("{} {} [{}]", archive.name, archive.created_date, archive.id);
    }
}

pub async fn create(
    conn: &mut PgConnection,
    repo: &str,
    directory_id: &Uuid,
    compression: &str,
    dry_run: bool,
    root_directories: bool,
) {
    let directories = if root_directories {
        find_directories_without_archives_by_root_directory_id(conn, directory_id).await
    } else {
        vec![find_directory_by_id(conn, directory_id).await]
    };
    for directory in directories {
        if let Some(archive) = find_archive_by_directory_id(conn, &directory.id).await {
            panic!("Archive {} already exists", &archive.name);
        };
        let df_output = df::main(repo).await;
        let du_output = du::main(&directory.path).await;
        let remaining_space_after =
            f64::from(df_output.avail - du_output.size) / f64::from(df_output.size);
        info!("Remaining space after: {}", remaining_space_after);
        if remaining_space_after < 0.05 {
            panic!("Not enough space");
        }
        let prefix = get_archive_prefix(&directory.path);
        let create_output = borg::create(repo, &prefix, &directory.path, compression, dry_run)
            .await
            .expect("Failed to create archive");
        create_archive(
            conn,
            &prefix,
            &create_output.repository.id,
            &create_output.archive.id,
            &Local::now().naive_local(),
            &directory.blake3_hash,
            directory_id,
        )
        .await;
    }
}

pub async fn update(
    conn: &mut PgConnection,
    repo: &str,
    directory_id: &Uuid,
    compression: &str,
    dry_run: bool,
    root_directories: bool,
) {
    let directories = if root_directories {
        find_directories_with_archives_by_root_directory_id(conn, directory_id).await
    } else {
        vec![find_directory_by_id(conn, directory_id).await]
    };
    for directory in directories {
        let archive = find_archive_by_directory_id(conn, directory_id)
            .await
            .unwrap();
        let df_output = df::main(repo).await;
        let du_output = du::main(&directory.path).await;
        let remaining_space_after =
            f64::from(df_output.avail - du_output.size) / f64::from(df_output.size);
        info!("Remaining space after: {}", remaining_space_after);
        if remaining_space_after < 0.05 {
            panic!("Not enough space");
        }
        let prefix = get_archive_prefix(&directory.path);
        let create_output = borg::create(repo, &prefix, &directory.path, compression, dry_run)
            .await
            .expect("Failed to create new archive");
        borg::prune(repo, &prefix, dry_run)
            .await
            .expect("Failed to prune old archive(s)");
        update_archive(
            conn,
            &archive.id,
            &create_output.archive.id,
            &Local::now().naive_local(),
            &directory.blake3_hash,
        )
        .await;
    }
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

fn get_archive_prefix(path: &str) -> String {
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

fn compute_directory_hash(path: &str) -> Hash {
    let mut hasher = Hasher::new();
    WalkDir::new(path)
        .into_iter()
        .filter_map(|v| v.ok())
        .filter(|e| e.path().is_file())
        .for_each(|e| {
            hasher.update_rayon(e.path().as_os_str().to_str().unwrap().as_bytes());
            hasher.update_rayon(
                DateTime::<Local>::from(e.metadata().unwrap().modified().unwrap())
                    .naive_local()
                    .to_string()
                    .as_bytes(),
            );
        });
    hasher.finalize()
}
