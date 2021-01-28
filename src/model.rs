use sqlx::types::chrono::NaiveDateTime;
use uuid::Uuid;

#[derive(sqlx::FromRow)]
pub struct RootDirectory {
    pub id: Uuid,
    pub path: String,
    pub depth: i16,
}

#[derive(sqlx::FromRow)]
pub struct Directory {
    pub id: Uuid,
    pub path: String,
    pub modified_date: NaiveDateTime,
    pub root_directory_id: Uuid,
}

#[derive(sqlx::FromRow)]
pub struct Archive {
    pub id: Uuid,
    pub name: String,
    pub repo_id: String,
    pub created_date: NaiveDateTime,
    pub directory_id: Option<Uuid>,
}
