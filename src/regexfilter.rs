use crate::errors::{error, Result};
use crate::filter::FilterFn;
use regex::Regex;

pub fn regex_direntry_filter(expressions: &Vec<String>) -> Result<Box<FilterFn>> {
    let regex_result: Result<Vec<Regex>> = expressions
        .iter()
        .map(|e| {
            Regex::new(&e).or_else(|e| error("Could not parse regular expression", Some(e.into())))
        })
        .collect();
    let regexes = regex_result?;
    Ok(Box::new(move |entry: &walkdir::DirEntry| {
        let path = entry.path();
        let path_str = path.to_str();
        if let Some(p) = path_str {
            regexes.iter().any(|e| e.is_match(p))
        } else {
            log::warn!(
                "Could not transform path into string {}.",
                path.to_string_lossy()
            );
            false
        }
    }))
}
