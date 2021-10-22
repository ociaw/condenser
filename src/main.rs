mod filters;
mod input_files;
mod transformer;

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use crate::filters::{FilterAction, FilterSet};
use crate::input_files::*;
use crate::transformer::{CopyTransformer, TransformerInstance};

fn main() {
    let jpg_regex = regex::Regex::from_str(".*jpg$").unwrap();
    let flac_glob = glob::Pattern::from_str("*.flac").unwrap();
    let mp3_glob = glob::Pattern::from_str("*.mp3").unwrap();

    let mut filter_set = FilterSet::new();
    filter_set.accept_unmatched = true;
    filter_set.append_regex(jpg_regex.clone(), FilterAction::Accept);
    filter_set.append_glob(flac_glob.clone(), FilterAction::Accept);
    filter_set.append_glob(mp3_glob.clone(), FilterAction::Accept);

    let input_dirs = vec![
            InputDirectory {
            priority: 100,
            filters: filter_set.clone(),
            path: "/home/ociaw/Music".into(),
        },
        InputDirectory {
            priority: 50,
            filters: filter_set.clone(),
            path: "/home/ociaw/Music-test/cd-rips".into(),
        },
        InputDirectory {
            priority: 40,
            filters: filter_set.clone(),
            path: "/home/ociaw/Music-test/digital-media".into(),
        },
        InputDirectory {
            priority: 30,
            filters: filter_set.clone(),
            path: "/home/ociaw/Music-test/other".into(),
        }
    ];
    let output_dir_path = PathBuf::from("/home/ociaw/Music (processed)");

    let mut copy_jpg_transformer = TransformerInstance::new(100, "Copy JPGs".to_string(), Box::new(CopyTransformer));
    let mut copy_mp3_transformer = TransformerInstance::new(50, "Copy MP3s".to_string(), Box::new(CopyTransformer));
    copy_jpg_transformer.filter.append_regex(jpg_regex.clone(), FilterAction::Accept);
    copy_mp3_transformer.filter.append_glob(mp3_glob.clone(), FilterAction::Accept);
 
    let mut transformers = vec![copy_jpg_transformer, copy_mp3_transformer];

    run_transformations(&mut transformers, &input_dirs, output_dir_path);
}

pub fn run_transformations<'a, DirIter, P>(transformers: &mut Vec<TransformerInstance>, input_dirs: DirIter, output_dir_path: P) 
where DirIter : IntoIterator<Item = &'a InputDirectory>,
    P : AsRef<Path>
{
    transformers.sort_by(|t1, t2| t1.priority.cmp(&t2.priority));
    let output_dir_path = output_dir_path.as_ref();
    let mut claimed_outputs = HashSet::new();

    for input_dir in input_dirs {
        let input_path = &input_dir.path;
        let mut unprocessed_files = input_dir.enumerate_files().unwrap();

        println!("Beginning to claim {1} files in {0}", input_path.to_string_lossy(), unprocessed_files.len());
        for transformer in transformers.iter_mut() {
            let count = transformer.claim_outputs(input_path, &mut unprocessed_files, &mut claimed_outputs);
            println!("  Transformer '{}' claimed {} files - {} remaining ", &transformer.name, count, unprocessed_files.len());
        }
    }

    std::fs::create_dir_all(output_dir_path)
        .expect(&format!("Failed to create output directory: {}", output_dir_path.to_string_lossy()));

    println!("Running {} transformer(s)...", transformers.len());
    for transformer in transformers {
        let errors = transformer.process_queues(output_dir_path);
        println!("  Transformer '{}' processing completed - {} error(s)", &transformer.name, errors.len());
        for error in errors {
            println!("    `{}' - {}", error.0.to_string_lossy(), error.1)
        }
    }
}