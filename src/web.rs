use super::borg;
use super::database::*;
use super::permafrust;
use async_std::task;
use clap::{App, SubCommand};
use http_types::headers::HeaderValue;
use serde::Deserialize;
use sqlx::postgres::Postgres;
use sqlx::{Acquire, PgPool};
use std::env;
use tide::security::{CorsMiddleware, Origin};
use tide::{Body, Request, Response, StatusCode};
use tide_sqlx::{SQLxMiddleware, SQLxRequestExt};
use uuid::Uuid;

#[derive(Deserialize)]
struct InitRepositoryRequest {
    #[serde(default = "default_repo")]
    repo: String,
    #[serde(default = "default_encryption")]
    encryption: String,
}

#[derive(Deserialize)]
struct ListArchivesRequest {
    #[serde(default = "default_repo")]
    repo: String,
}

#[derive(Deserialize)]
struct CreateArchiveRequest {
    #[serde(default = "default_repo")]
    repo: String,
    directory_id: Uuid,
    #[serde(default = "default_compression")]
    compression: String,
    dry_run: bool,
}

#[derive(Deserialize)]
struct ReplaceArchiveRequest {
    #[serde(default = "default_repo")]
    repo: String,
    archive_id: Uuid,
    #[serde(default = "default_compression")]
    compression: String,
    dry_run: bool,
}

#[derive(Deserialize)]
struct DeleteArchiveRequest {
    #[serde(default = "default_repo")]
    repo: String,
    archive_id: Uuid,
    dry_run: bool,
}

fn default_repo() -> String {
    env::var("BORG_REPO").unwrap()
}

fn default_encryption() -> String {
    env::var("BORG_ENCRYPTION").unwrap()
}

fn default_compression() -> String {
    env::var("BORG_COMPRESSION").unwrap()
}

pub fn subcommand<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("web").about("Run the webserver")
}

pub async fn main(pool: PgPool) -> tide::Result<()> {
    let mut app = tide::new();
    app.with(
        CorsMiddleware::new()
            .allow_methods("GET, POST, PUT, DELETE".parse::<HeaderValue>().unwrap())
            .allow_origin(Origin::from("*"))
            .allow_credentials(false),
    );
    app.with(SQLxMiddleware::from(pool));
    app.at("/api/repository/init").post(init_repository);
    app.at("/api/root-directories").get(list_root_directories);
    app.at("/api/directories").get(list_directories);
    app.at("/api/archives").get(list_archives);
    app.at("/api/archives").post(create_archive);
    // todo use path variables when tide supports them
    app.at("/api/archives").put(replace_archive);
    app.at("/api/archives").delete(delete_archive);
    app.listen("127.0.0.1:8080").await?;
    Ok(())
}

async fn init_repository(mut req: Request<()>) -> tide::Result {
    let init_req: InitRepositoryRequest = req.body_json().await?;
    task::spawn(async move {
        permafrust::init(&init_req.repo, &init_req.encryption).await;
    });
    let res = Response::new(StatusCode::Accepted);
    Ok(res)
}

async fn list_root_directories(req: Request<()>) -> tide::Result {
    let mut res = Response::new(StatusCode::Ok);
    res.set_body(Body::from_json(
        &find_root_directories(&mut req.sqlx_conn::<Postgres>().await.acquire().await.unwrap())
            .await,
    )?);
    Ok(res)
}

async fn list_directories(req: Request<()>) -> tide::Result {
    let mut res = Response::new(StatusCode::Ok);
    res.set_body(Body::from_json(
        &find_directories(&mut req.sqlx_conn::<Postgres>().await.acquire().await.unwrap()).await,
    )?);
    Ok(res)
}
async fn list_archives(mut req: Request<()>) -> tide::Result {
    let list_req: ListArchivesRequest = req.body_json().await?;
    let mut res = Response::new(StatusCode::Ok);
    res.set_body(Body::from_json(&borg::list(&list_req.repo).await?)?);
    Ok(res)
}

async fn create_archive(mut req: Request<()>) -> tide::Result {
    let create_req: CreateArchiveRequest = req.body_json().await?;
    task::spawn(async move {
        permafrust::create(
            &mut req.sqlx_conn::<Postgres>().await.acquire().await.unwrap(),
            &create_req.repo,
            &create_req.directory_id,
            &create_req.compression,
            create_req.dry_run,
        )
        .await;
    });
    let res = Response::new(StatusCode::Accepted);
    Ok(res)
}

async fn replace_archive(mut req: Request<()>) -> tide::Result {
    let replace_req: ReplaceArchiveRequest = req.body_json().await?;
    task::spawn(async move {
        permafrust::update(
            &mut req.sqlx_conn::<Postgres>().await.acquire().await.unwrap(),
            &replace_req.repo,
            &replace_req.archive_id,
            &replace_req.compression,
            replace_req.dry_run,
        )
        .await;
    });
    let res = Response::new(StatusCode::Accepted);
    Ok(res)
}

async fn delete_archive(mut req: Request<()>) -> tide::Result {
    let delete_req: DeleteArchiveRequest = req.body_json().await?;
    task::spawn(async move {
        permafrust::delete(
            &mut req.sqlx_conn::<Postgres>().await.acquire().await.unwrap(),
            &delete_req.repo,
            &delete_req.archive_id,
            delete_req.dry_run,
        )
        .await;
    });
    let res = Response::new(StatusCode::Accepted);
    Ok(res)
}
