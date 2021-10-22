use std::path::PathBuf;
use std::str::FromStr;

use mass_file_transmuter::run_transformations;

use mass_file_transmuter::filters::{FilterAction, FilterSet};
use mass_file_transmuter::input_files::*;
use mass_file_transmuter::transformer::{CopyTransformer, TransformerInstance};

fn main() {
    // Manual setup of everything until we can get this info from
    // configuration file(s)
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
        },
    ];
    let output_dir_path = PathBuf::from("/home/ociaw/Music (processed)");

    let mut copy_jpg_transformer =
        TransformerInstance::new(100, "Copy JPGs".to_string(), Box::new(CopyTransformer));
    let mut copy_mp3_transformer =
        TransformerInstance::new(50, "Copy MP3s".to_string(), Box::new(CopyTransformer));
    copy_jpg_transformer
        .filter
        .append_regex(jpg_regex.clone(), FilterAction::Accept);
    copy_mp3_transformer
        .filter
        .append_glob(mp3_glob.clone(), FilterAction::Accept);

    let mut transformers = vec![copy_jpg_transformer, copy_mp3_transformer];

    run_transformations(&mut transformers, &input_dirs, output_dir_path);
}
