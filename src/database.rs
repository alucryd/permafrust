use super::model::*;
use chrono::NaiveDateTime;
use sqlx::migrate::Migrator;
use sqlx::postgres::PgPoolOptions;
use sqlx::{PgConnection, PgPool};
use uuid::Uuid;

static MIGRATOR: Migrator = sqlx::migrate!();

pub async fn establish_connection(url: &str) -> PgPool {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(url)
        .await
        .expect(&format!("Error connecting to {}", url));
    MIGRATOR
        .run(&pool)
        .await
        .expect("Failed to run database migrations");
    pool
}

pub async fn create_root_directory(conn: &mut PgConnection, path: &str, depth: i16) {
    sqlx::query!(
        "
        INSERT INTO root_directories (id, path, depth)
        VALUES ($1, $2, $3)
        ",
        Uuid::new_v4(),
        path,
        depth,
    )
    .execute(conn)
    .await
    .expect("Error while creating root directory");
}

pub async fn find_root_directories(conn: &mut PgConnection) -> Vec<RootDirectory> {
    sqlx::query_as!(
        RootDirectory,
        "
        SELECT *
        FROM root_directories
        ORDER BY path
        ",
    )
    .fetch_all(conn)
    .await
    .expect("Error while finding root directories")
}

pub async fn find_root_directory_by_path(
    conn: &mut PgConnection,
    path: &str,
) -> Option<RootDirectory> {
    sqlx::query_as!(
        RootDirectory,
        "
        SELECT *
        FROM root_directories
        WHERE path = $1
        ",
        path,
    )
    .fetch_optional(conn)
    .await
    .expect(&format!(
        "Error while finding root directory with path {}",
        path
    ))
}

pub async fn delete_root_directory(conn: &mut PgConnection, id: &Uuid) {
    sqlx::query!(
        "
        DELETE FROM root_directories
        WHERE id = $1
        ",
        id,
    )
    .execute(conn)
    .await
    .expect(&format!(
        "Error while deleting root directory with id {}",
        id
    ));
}

pub async fn create_directory(
    conn: &mut PgConnection,
    path: &str,
    modified_date: &NaiveDateTime,
    root_directory_id: &Uuid,
) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query!(
        "
        INSERT INTO directories (id, path, modified_date, root_directory_id)
        VALUES ($1, $2, $3, $4)
        ",
        &id,
        path,
        modified_date,
        root_directory_id,
    )
    .execute(conn)
    .await
    .expect("Error while creating directory");
    id
}

pub async fn update_directory(conn: &mut PgConnection, id: &Uuid, modified_date: &NaiveDateTime) {
    sqlx::query!(
        "
        UPDATE directories
        SET modified_date = $2
        WHERE id = $1
        ",
        id,
        modified_date,
    )
    .execute(conn)
    .await
    .expect("Error while updating directory");
}

pub async fn find_directories(conn: &mut PgConnection) -> Vec<Directory> {
    sqlx::query_as!(
        Directory,
        "
        SELECT *
        FROM directories
        ORDER BY path
        ",
    )
    .fetch_all(conn)
    .await
    .expect("Error while finding directories")
}

pub async fn find_directory_by_id(conn: &mut PgConnection, id: &Uuid) -> Directory {
    sqlx::query_as!(
        Directory,
        "
        SELECT *
        FROM directories
        WHERE id = $1
        ",
        id,
    )
    .fetch_one(conn)
    .await
    .expect(&format!("Error while finding directory with id {}", id))
}

pub async fn find_directory_by_path(conn: &mut PgConnection, path: &str) -> Option<Directory> {
    sqlx::query_as!(
        Directory,
        "
        SELECT *
        FROM directories
        WHERE path = $1
        ",
        path,
    )
    .fetch_optional(conn)
    .await
    .expect(&format!("Error while finding directory with path {}", path))
}

pub async fn delete_directory(conn: &mut PgConnection, id: &Uuid) {
    sqlx::query!(
        "
        DELETE FROM directories
        WHERE id = $1
        ",
        id,
    )
    .execute(conn)
    .await
    .expect(&format!("Error while deleting directory with id {}", id));
}

pub async fn create_archive(
    conn: &mut PgConnection,
    name: &str,
    repo_id: &str,
    created_date: &NaiveDateTime,
    directory_id: &Uuid,
) {
    sqlx::query!(
        "
        INSERT INTO archives (id, name, repo_id, created_date, directory_id)
        VALUES ($1, $2, $3, $4, $5)
        ",
        Uuid::new_v4(),
        name,
        repo_id,
        created_date,
        directory_id,
    )
    .execute(conn)
    .await
    .expect("Error while creating archive");
}

pub async fn update_archive(conn: &mut PgConnection, id: &Uuid, created_date: &NaiveDateTime) {
    sqlx::query!(
        "
        UPDATE archives
        SET created_date = $2
        WHERE id = $1
        ",
        id,
        created_date,
    )
    .execute(conn)
    .await
    .expect("Error while updating archive");
}

pub async fn find_archive_by_id(conn: &mut PgConnection, id: &Uuid) -> Archive {
    sqlx::query_as!(
        Archive,
        "
        SELECT *
        FROM archives
        WHERE id = $1
        ",
        id,
    )
    .fetch_one(conn)
    .await
    .expect(&format!("Error while finding archive with id {}", id))
}

pub async fn find_archive_by_directory_id(
    conn: &mut PgConnection,
    directory_id: &Uuid,
) -> Option<Archive> {
    sqlx::query_as!(
        Archive,
        "
        SELECT *
        FROM archives
        WHERE directory_id = $1
        ",
        directory_id,
    )
    .fetch_optional(conn)
    .await
    .expect(&format!(
        "Error while finding archive with directory_id {}",
        directory_id
    ))
}

pub async fn delete_archive(conn: &mut PgConnection, id: &Uuid) {
    sqlx::query!(
        "
        DELETE FROM archives
        WHERE id = $1
        ",
        id,
    )
    .execute(conn)
    .await
    .expect(&format!("Error while deleting archive with id {}", id));
}
