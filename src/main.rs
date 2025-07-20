use reqwest::Error;
use sqlite::{self, Connection, State};
mod structs;
use crate::structs::XkcdComic;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let connection = initialize_db(&"db.db".to_string());
    populate_db(&connection).await?;
    // println!("{:?}", response.title);
    Ok(())
}

async fn get_json(url: &str) -> Result<XkcdComic, Error> {
    let response = reqwest::get(url).await?.json::<XkcdComic>().await?;
    Ok(response)
}

fn initialize_db(db_name: &String) -> Connection {
    let connection = match sqlite::open(db_name) {
        Ok(c) => c,
        Err(error) => panic!("PANIC: Failed to open file {db_name}: {error}"),
    };
    println!("INFO: Database connection successfull: {db_name}");

    let mut table_exists = false;
    // use table to encapsulate connection borrow
    {
        let mut statement = connection
            .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='comics';")
            .unwrap();

        while let Ok(State::Row) = statement.next() {
            let name: String = statement.read::<String, _>("name").unwrap();
            if name == "comics" {
                table_exists = true;
                break;
            }
        }
    }

    if !table_exists {
        println!("INFO: comics table not found, creating comics into {db_name}");
        match connection.execute(
            "
                CREATE TABLE comics (
                    month TEXT,
                    num NUMBER,
                    link TEXT,
                    year TEXT,
                    news TEXT,
                    safe_title TEXT,
                    transcript TEXT,
                    alt TEXT,
                    img TEXT,
                    title TEXT,
                    day TEXT
                );
                ",
        ) {
            Ok(()) => (),
            Err(error) => panic!("PANIC: Failed to create table comics: {error}"),
        }
        println!("INFO: comics table created");
    }
    connection
}

async fn populate_db(connection: &Connection) -> Result<(), Error> {
    //TODO: Check if comics is already populated
    let mut failed_attempts = 0;
    let max_attempts = 3;
    for i in 1..10 {
        println!("INFO: Fetching comic {i}");
        let url = format!("https://xkcd.com/{i}/info.0.json");
        let response = match get_json(&url).await {
            Ok(comic) => comic,
            Err(error) => {
                eprintln!("ERROR: Failed to fetch comic {i}: {error}");
                failed_attempts += 1;
                if failed_attempts >= max_attempts {
                    eprintln!(
                        "ERROR: Reached max failed attempts ({max_attempts}) while fetching comics. Stopping fetches"
                    );
                    return Ok(());
                }
                continue;
            }
        };
        save_comic(connection, response);
    }
    Ok(())
}

fn save_comic(connection: &Connection, comic: XkcdComic) {
    match connection.execute(
        format!(
            "INSERT INTO comics (title, alt, day, img, month, news, link, num, safe_title, transcript, year)
            VALUES ('{}', '{}', '{}', '{}', '{}', '{}', '{}', {}, '{}', '{}', '{}')",
            comic.title.replace("'", "''"),
            comic.alt.replace("'", "''"),
            comic.day.replace("'", "''"),
            comic.img.replace("'", "''"),
            comic.month.replace("'", "''"),
            comic.news.replace("'", "''"),
            comic.link.replace("'", "''"),
            comic.num,
            comic.safe_title.replace("'", "''"),
            comic.transcript.replace("'", "''"),
            comic.year.replace("'", "''")
        )
    ) {
        Ok(()) => (),
        Err(error) => eprintln!("ERROR: Failed to insert a comic to database: {error}")
    };
}
