use super::borg;
use super::permafrust;
use async_std::task;
use clap::{App, SubCommand};
use serde::Deserialize;
use sqlx::postgres::Postgres;
use sqlx::{Acquire, PgPool};
use std::env;
use tide::{Body, Request, Response, StatusCode};
use tide_sqlx::{SQLxMiddleware, SQLxRequestExt};
use uuid::Uuid;

#[derive(Deserialize)]
struct InitRequest {
    #[serde(default = "default_repo")]
    repo: String,
    #[serde(default = "default_encryption")]
    encryption: String,
}

#[derive(Deserialize)]
struct ListRequest {
    #[serde(default = "default_repo")]
    repo: String,
}

#[derive(Deserialize)]
struct CreateRequest {
    #[serde(default = "default_repo")]
    repo: String,
    directory_id: Uuid,
    #[serde(default = "default_compression")]
    compression: String,
    dry_run: bool,
}

#[derive(Deserialize)]
struct UpdateRequest {
    #[serde(default = "default_repo")]
    repo: String,
    archive_id: Uuid,
    #[serde(default = "default_compression")]
    compression: String,
    dry_run: bool,
}

#[derive(Deserialize)]
struct DeleteRequest {
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
    app.with(SQLxMiddleware::from(pool));
    app.at("/api/init").post(init);
    app.at("/api/archives").get(list);
    app.at("/api/archives").post(create);
    // todo use path variables when tide supports them
    app.at("/api/archives").put(update);
    app.at("/api/archives").delete(delete);
    app.listen("127.0.0.1:8080").await?;
    Ok(())
}

async fn init(mut req: Request<()>) -> tide::Result {
    let init_req: InitRequest = req.body_json().await?;
    task::spawn(async move {
        permafrust::init(&init_req.repo, &init_req.encryption).await;
    });
    let res = Response::new(StatusCode::Accepted);
    Ok(res)
}

async fn list(mut req: Request<()>) -> tide::Result {
    let list_req: ListRequest = req.body_json().await?;
    let mut res = Response::new(StatusCode::Ok);
    res.set_body(Body::from_json(&borg::list(&list_req.repo).await?)?);
    Ok(res)
}

async fn create(mut req: Request<()>) -> tide::Result {
    let create_req: CreateRequest = req.body_json().await?;
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

async fn update(mut req: Request<()>) -> tide::Result {
    let recreate_req: UpdateRequest = req.body_json().await?;
    task::spawn(async move {
        permafrust::update(
            &mut req.sqlx_conn::<Postgres>().await.acquire().await.unwrap(),
            &recreate_req.repo,
            &recreate_req.archive_id,
            &recreate_req.compression,
            recreate_req.dry_run,
        )
        .await;
    });
    let res = Response::new(StatusCode::Accepted);
    Ok(res)
}

async fn delete(mut req: Request<()>) -> tide::Result {
    let delete_req: DeleteRequest = req.body_json().await?;
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
