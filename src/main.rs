mod filters;
mod input_files;

use std::str::FromStr;

use crate::filters::{FilterAction, FilterSet};
use crate::input_files::*;

fn main() {
    let regex = regex::Regex::from_str(".*jpg$").unwrap();
    let glob = glob::Pattern::from_str("*.flac").unwrap();

    let mut filter_set = FilterSet::new();
    filter_set.append_regex(regex, FilterAction::Accept);
    filter_set.append_glob(glob, FilterAction::Accept);

    let input_dir = InputDirectory {
        priority: 100,
        filters: filter_set,
        path: "/home/ociaw/Music".into(),
    };

    match input_dir.enumerate_files() {
        Err(err) => println!("Failed to enumerate directory: {}", err),
        Ok(vec) => {
            for file in vec {
                println!("{}", file.to_string_lossy())
            }
        },
    }
}

