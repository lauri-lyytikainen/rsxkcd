use crate::Connection;
use crate::State;
use crate::XkcdComic;
use crate::fetch_comic;
use thiserror::Error;

pub fn initialize_db(db_name: &String) -> Connection {
    let connection = match sqlite::open(db_name) {
        Ok(c) => c,
        Err(error) => panic!("PANIC: Failed to open file {db_name}: {error}"),
    };
    println!("INFO: Database connection successfull: {db_name}");

    let mut table_exists = false;
    // Check if the comics table exists
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

    table_exists = false;
    // Check if terms table exists
    // use table to encapsulate connection borrow
    {
        let mut statement = connection
            .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='terms';")
            .unwrap();

        while let Ok(State::Row) = statement.next() {
            let name: String = statement.read::<String, _>("name").unwrap();
            if name == "terms" {
                table_exists = true;
                break;
            }
        }
    }
    if !table_exists {
        println!("INFO: terms table not found, creating terms into {db_name}");
        match connection.execute(
            "
                CREATE TABLE terms (
                    term TEXT,
                    comicNum NUMBER,
                    frequency TEXT
                );
                ",
        ) {
            Ok(()) => (),
            Err(error) => panic!("PANIC: Failed to create table terms: {error}"),
        }
        println!("INFO: terms table created");
    }
    connection
}

pub async fn populate_db(connection: &Connection) -> Result<(), DatabaseError> {
    let newest_comic_num = match fetch_comic(0).await {
        Ok(comic) => {
            println!("INFO: Found newest comic {}", comic.num);
            comic.num
        }
        Err(error) => {
            eprintln!("ERROR: Error fetching newest comic: {error}");
            9999
        }
    };
    let mut found_comics: Vec<u32> = Vec::new();
    // use table to encapsulate connection borrow
    {
        connection
            .iterate("SELECT DISTINCT num FROM comics", |values| {
                for value in values.iter() {
                    if let Some(num_str) = value.1 {
                        if let Ok(num) = num_str.parse::<u32>() {
                            found_comics.push(num);
                        }
                    }
                }
                true
            })
            .unwrap();
    }
    let mut fetched_comics_count = 0;
    for i in 1..=newest_comic_num {
        if found_comics.contains(&i) {
            // TODO: Make better logging for this
            // println!("INFO: Comic {i} already in database, skipping...");
            continue;
        }
        if i == 404 {
            println!("INFO: xkcd author played a prank on comic 404, skipping...");
            continue;
        }
        println!("INFO: Fetching comic {i}");
        let comic = match fetch_comic(i).await {
            Ok(comic) => comic,
            Err(error) => {
                eprintln!("ERROR: Error fetching comic {i}: {error}");
                return Ok(());
            }
        };
        save_comic(connection, comic);
        fetched_comics_count += 1;
    }
    println!(
        "INFO: Found {} comics in database, fetched and saved {fetched_comics_count} comics from https://xkcd.com",
        found_comics.len()
    );
    println!(
        "INFO: Total of {} comics in database",
        found_comics.len() + fetched_comics_count
    );
    Ok(())
}

pub fn save_comic(connection: &Connection, comic: XkcdComic) {
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

#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("Failed to load comic from database")]
    LoadComicError(String),
}

pub fn load_comics(connection: &Connection, amount: u32) -> Result<Vec<XkcdComic>, DatabaseError> {
    let mut statement = connection
        .prepare("SELECT * FROM comics LIMIT ?")
        .map_err(|e| DatabaseError::LoadComicError(e.to_string()))?;
    statement
        .bind((1, amount as i64))
        .map_err(|e| DatabaseError::LoadComicError(e.to_string()))?;

    let mut comics = Vec::new();
    while let Ok(State::Row) = statement.next() {
        let comic = XkcdComic {
            title: statement.read::<String, _>("title").unwrap_or_default(),
            alt: statement.read::<String, _>("alt").unwrap_or_default(),
            day: statement.read::<String, _>("day").unwrap_or_default(),
            img: statement.read::<String, _>("img").unwrap_or_default(),
            month: statement.read::<String, _>("month").unwrap_or_default(),
            news: statement.read::<String, _>("news").unwrap_or_default(),
            link: statement.read::<String, _>("link").unwrap_or_default(),
            num: statement.read::<i64, _>("num").unwrap_or(0) as u32,
            safe_title: statement
                .read::<String, _>("safe_title")
                .unwrap_or_default(),
            transcript: statement
                .read::<String, _>("transcript")
                .unwrap_or_default(),
            year: statement.read::<String, _>("year").unwrap_or_default(),
        };
        comics.push(comic);
    }

    Ok(comics)
}
