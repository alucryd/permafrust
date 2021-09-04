#[macro_use]
extern crate lazy_static;
extern crate env_logger;
extern crate log;

use clap::App;
use dotenv::dotenv;
use std::env;

mod borg;
mod cli;
mod database;
mod df;
mod du;
mod model;
mod permafrust;

#[async_std::main]
async fn main() {
    env_logger::init();
    let matches = App::new(env!("CARGO_BIN_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .subcommands(vec![
            cli::watch_subcommand(),
            cli::unwatch_subcommand(),
            cli::scan_subcommand(),
            cli::init_subcommand(),
            cli::list_subcommand(),
            cli::create_subcommand(),
            cli::update_subcommand(),
            cli::delete_subcommand(),
            cli::extract_subcommand(),
            cli::check_subcommand(),
        ])
        .get_matches();

    if matches.subcommand.is_some() {
        dotenv().ok();
        let pool = database::establish_connection(&env::var("DATABASE_URL").unwrap()).await;
        match matches.subcommand_name() {
            Some("watch") => cli::watch(&pool, matches.subcommand_matches("watch").unwrap()).await,
            Some("unwatch") => {
                cli::unwatch(&pool, matches.subcommand_matches("unwatch").unwrap()).await
            }
            Some("scan") => cli::scan(&pool, matches.subcommand_matches("scan").unwrap()).await,
            Some("init") => cli::init(matches.subcommand_matches("init").unwrap()).await,
            Some("list") => cli::list(&pool, matches.subcommand_matches("list").unwrap()).await,
            Some("create") => {
                cli::create(&pool, matches.subcommand_matches("create").unwrap()).await
            }
            Some("update") => {
                cli::update(&pool, matches.subcommand_matches("update").unwrap()).await
            }
            Some("delete") => {
                cli::delete(&pool, matches.subcommand_matches("delete").unwrap()).await
            }
            Some("extract") => {
                cli::extract(&pool, matches.subcommand_matches("extract").unwrap()).await
            }
            Some("check") => cli::check(&pool, matches.subcommand_matches("check").unwrap()).await,
            _ => (),
        }
    }
}
