use crate::Connection;
use crate::State;
use crate::XkcdComic;
use crate::fetch_comic;
use crate::index::comic_to_terms;
use std::collections::HashMap;
use thiserror::Error;

pub fn initialize_db(db_name: &String) -> Connection {
    let connection = match sqlite::open(db_name) {
        Ok(c) => c,
        Err(error) => panic!("PANIC: Failed to open file {db_name}: {error}"),
    };
    println!("INFO: Database connection successfull: {db_name}");

    match connection.execute(
        "
    CREATE TABLE IF NOT EXISTS comics (
        month TEXT,
        num INTEGER PRIMARY KEY,
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

    CREATE TABLE IF NOT EXISTS terms (
        term TEXT,
        comicNum INTEGER,
        frequency INTEGER,
        PRIMARY KEY (comicNum, term),
        FOREIGN KEY (comicNum) REFERENCES comics(num)
    );
",
    ) {
        Ok(_) => connection,
        Err(e) => panic!("PANIC: Failed to initialize_db {e}"),
    }
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
        save_comic(connection, &comic);
        let terms = comic_to_terms(&comic);
        match save_entries(connection, comic.num, &terms) {
            Ok(()) => println!(
                "INFO: Saved {} entries from comic {}",
                terms.len(),
                comic.num
            ),
            Err(e) => println!("ERROR: Failed to save entries for comic {}: {e}", comic.num),
        };

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

pub fn save_comic(connection: &Connection, comic: &XkcdComic) {
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
    #[error("Failed to save terms to database")]
    SaveEntryError(String),
}

pub fn load_comics_with_no_terms(connection: &Connection) -> Result<Vec<XkcdComic>, DatabaseError> {
    let mut statement = connection
        .prepare("SELECT * FROM comics WHERE num NOT IN (SELECT comicNum FROM terms)")
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
pub fn load_comics(connection: &Connection) -> Result<Vec<XkcdComic>, DatabaseError> {
    let mut statement = connection
        .prepare("SELECT * FROM comics")
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

pub fn save_entries(
    connection: &Connection,
    comic_num: u32,
    terms: &HashMap<String, i32>,
) -> Result<(), DatabaseError> {
    //FIXME: Comic 1913 fails because it has no terms after removing 'a'

    if terms.is_empty() {
        return Err(DatabaseError::SaveEntryError(
            "No entries provided".to_owned(),
        ));
    }
    for term in terms.keys() {
        let query = format!(
            "INSERT INTO terms (term, comicNum, frequency) VALUES ('{}', '{}', '{}')",
            term,
            comic_num,
            terms.get(term).unwrap_or(&0)
        );
        match connection.execute(query) {
            Ok(_) => continue,
            Err(e) => {
                return Err(DatabaseError::SaveEntryError(
                    format!("Failed to save entry {e}").to_owned(),
                ));
            }
        };
    }
    Ok(())
}
