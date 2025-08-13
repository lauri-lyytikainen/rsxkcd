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
            Ok(()) => log_info!("Saved {} entries from comic {}", terms.len(), comic.num),
            Err(e) => log_error!("Failed to save entries for comic {}: {}", comic.num, e),
        };
    }
}

pub fn comic_to_terms(comic: &XkcdComic) -> HashMap<String, i32> {
    let split_regex = Regex::new(r"[ \n]+").unwrap();
    let stop_words = [
        "i",
        "me",
        "my",
        "myself",
        "we",
        "our",
        "ours",
        "ourselves",
        "you",
        "your",
        "yours",
        "yourself",
        "yourselves",
        "he",
        "him",
        "his",
        "himself",
        "she",
        "her",
        "hers",
        "herself",
        "it",
        "its",
        "itself",
        "they",
        "them",
        "their",
        "theirs",
        "themselves",
        "what",
        "which",
        "who",
        "whom",
        "this",
        "that",
        "these",
        "those",
        "am",
        "is",
        "are",
        "was",
        "were",
        "be",
        "been",
        "being",
        "have",
        "has",
        "had",
        "having",
        "do",
        "does",
        "did",
        "doing",
        "a",
        "an",
        "the",
        "and",
        "but",
        "if",
        "or",
        "because",
        "as",
        "until",
        "while",
        "of",
        "at",
        "by",
        "for",
        "with",
        "about",
        "against",
        "between",
        "into",
        "through",
        "during",
        "before",
        "after",
        "above",
        "below",
        "to",
        "from",
        "up",
        "down",
        "in",
        "out",
        "on",
        "off",
        "over",
        "under",
        "again",
        "further",
        "then",
        "once",
        "here",
        "there",
        "when",
        "where",
        "why",
        "how",
        "all",
        "any",
        "both",
        "each",
        "few",
        "more",
        "most",
        "other",
        "some",
        "such",
        "no",
        "nor",
        "not",
        "only",
        "own",
        "same",
        "so",
        "than",
        "too",
        "very",
        "s",
        "t",
        "can",
        "will",
        "just",
        "don",
        "should",
        "now",
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
        // TODO: Handle special cases where there are only empty stemmed words eg title "a"
        if stemmed.is_empty() {
            log_warning!("Found empty stemmed word in comic {}: {}", comic.num, word);
            continue;
        }
        if !stemmed.is_empty() && !stop_words.contains(&stemmed.as_str()) {
            *searchable_terms.entry(stemmed).or_insert(0) += 1;
        }
    }
    searchable_terms
}
