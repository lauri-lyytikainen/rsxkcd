use reqwest::Error;
use sqlite::{self, Connection, State};
mod structs;
use crate::structs::XkcdComic;

#[tokio::main]
async fn main() -> Result<(), Error> {
    // let url = "https://xkcd.com/614/info.0.json";
    let connection = initialize_db(&"db.db".to_string());
    populate_db(&connection);
    // let response = get_json(url).await?;
    // let transcript = response.transcript;
    // println!("{:?}", response.title);
    Ok(())
}

// async fn get_json(url: &str) -> Result<XkcdComic, Error> {
//     let response = reqwest::get(url).await?.json::<XkcdComic>().await?;
//     Ok(response)
// }

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
fn populate_db(connection: &Connection) {
    let json = XkcdComic {
        title: "Test title".to_string(),
        alt: "test".to_string(),
        day: "test".to_string(),
        img: "test".to_string(),
        month: "test".to_string(),
        news: "test".to_string(),
        link: "test".to_string(),
        num: 1,
        safe_title: "test".to_string(),
        transcript: "test".to_string(),
        year: "test".to_string(),
    };
    save_comic(connection, json);
}

fn save_comic(connection: &Connection, comic: XkcdComic) {
    match connection.execute(
        format!(
            "INSERT INTO comics (title, alt, day, img, month, news, link, num, safe_title, transcript, year)
            VALUES ('{}', '{}', '{}', '{}', '{}', '{}', '{}', {}, '{}', '{}', '{}')",
            comic.title,
            comic.alt,
            comic.day,
            comic.img,
            comic.month,
            comic.news,
            comic.link,
            comic.num,
            comic.safe_title,
            comic.transcript,
            comic.year
        )
    ) {
        Ok(()) => (),
        Err(error) => eprintln!("ERROR: Failed to insert a comic to database: {error}")
    };
}
