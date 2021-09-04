use serde::Serialize;
use sqlx::types::chrono::NaiveDateTime;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(FromRow, Serialize)]
pub struct RootDirectory {
    pub id: Uuid,
    pub path: String,
    pub depth: i16,
}

#[derive(FromRow, Serialize)]
pub struct Directory {
    pub id: Uuid,
    pub path: String,
    pub modified_date: NaiveDateTime,
    pub root_directory_id: Uuid,
}

#[derive(FromRow)]
pub struct Archive {
    pub id: Uuid,
    pub name: String,
    pub repo_id: String,
    pub archive_id: String,
    pub created_date: NaiveDateTime,
    pub directory_id: Option<Uuid>,
}
