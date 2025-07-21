use crate::Connection;
use crate::Error;
use crate::State;
use crate::XkcdComic;
use crate::fetch_comic;

pub fn initialize_db(db_name: &String) -> Connection {
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

pub async fn populate_db(connection: &Connection) -> Result<(), Error> {
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
            println!("INFO: Comic {i} already in database, skipping...");
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
