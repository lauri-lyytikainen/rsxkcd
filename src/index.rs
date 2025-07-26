use crate::Connection;
use crate::database::load_comics;

pub fn update_index(connection: &Connection) {
    let comics = match load_comics(connection, 3) {
        Ok(comics) => comics,
        Err(e) => panic!("PANIC: Failed to load comics from database: {e}"),
    };
    for comic in comics {
        println!("Comic loaded: {comic:?}");
    }
}
