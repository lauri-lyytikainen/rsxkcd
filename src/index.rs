use crate::Connection;
use crate::comic::XkcdComic;
use crate::database::{load_comics_with_no_terms, save_entries};
use porter_stemmer::stem;
use regex::Regex;
use std::collections::HashMap;

pub fn update_index(connection: &Connection) {
    // TODO: return all comics if -1 is passed as the amount
    let comics = match load_comics_with_no_terms(connection) {
        Ok(comics) => comics,
        Err(e) => panic!("PANIC: Failed to load comics from database: {e}"),
    };
    for comic in comics {
        let terms = comic_to_terms(&comic);
        match save_entries(connection, comic.num, &terms) {
            Ok(()) => println!(
                "INFO: Saved {} entries from comic {}",
                terms.len(),
                comic.num
            ),
            Err(e) => println!("ERROR: Failed to save entries for comic {}: {e}", comic.num),
        };
    }
}

pub fn comic_to_terms(comic: &XkcdComic) -> HashMap<String, i32> {
    let split_regex = Regex::new(r"[ \n]+").unwrap();
    let ignored_words = [
        "a", "an", "or", "and", "the", "is", "to", "of", "in", "that", "it", "for", "on", "with",
    ];
    let mut searchable_terms = HashMap::new();
    let all_searchable_content = format!("{} {}", &comic.title, &comic.transcript);
    for word in split_regex.split(&all_searchable_content) {
        let cleaned = word
            .chars()
            .filter(|c| c.is_alphanumeric())
            .collect::<String>()
            .trim()
            .to_lowercase()
            .to_string();
        let stemmed = stem(&cleaned);
        if stemmed.is_empty() {
            println!(
                "WARN: Found empty stemmed word in comic {}: {}",
                comic.num, cleaned
            );
            continue;
        }
        if !stemmed.is_empty() && !ignored_words.contains(&stemmed.as_str()) {
            *searchable_terms.entry(stemmed).or_insert(0) += 1;
        }
    }
    searchable_terms
}
