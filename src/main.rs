use reqwest::Error;
use sqlite::{self, Connection, State};
use thiserror::Error;
mod comic;
use crate::comic::XkcdComic;
mod request;
use crate::request::fetch_comic;
mod database;
use crate::database::{initialize_db, populate_db};
mod index;
use crate::index::update_index;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let connection = initialize_db(&"db.db".to_string());
    populate_db(&connection).await.unwrap_or_else(|_| {
        panic!("PANIC: Failed to populate database");
    });
    update_index(&connection);
    Ok(())
}
