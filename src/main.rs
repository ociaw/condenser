mod filters;

use std::path::PathBuf;
use std::str::FromStr;

use crate::filters::{FilterAction, FilterSet};

fn main() {
    let regex = regex::Regex::from_str(".*").unwrap();
    let glob = glob::Pattern::from_str("*").unwrap();

    let mut filter_set = FilterSet::new();
    filter_set.append_regex(regex, FilterAction::Accept);
    filter_set.append_glob(glob, FilterAction::Accept);

    let input_dir = InputDirectory {
        priority: 100,
        filters: filter_set,
        path: "/home/ociaw/Music/input1".into(),
    };
    println!("Hello, world!");
}

pub struct InputDirectory {
    /// The priority of this input - higher priorities are favored in conflict resolution.
    pub priority: u32,

    /// The filters used when enumerating the files in this directory.
    pub filters: FilterSet,

    /// The absolute path to the directory.
    pub path: PathBuf,
}
