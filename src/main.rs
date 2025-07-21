use reqwest::Error;
use sqlite::{self, Connection, State};
use thiserror::Error;
mod comic;
use crate::comic::XkcdComic;
mod request;
use crate::request::fetch_comic;
mod database;
use crate::database::{initialize_db, populate_db};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let connection = initialize_db(&"db.db".to_string());
    populate_db(&connection).await?;
    Ok(())
}
