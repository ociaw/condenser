use std::path::PathBuf;
use std::str::FromStr;

use condenser::run_transformations;

use condenser::{CopyTransformer, FilterAction, FilterSet, InputDirectory, TransformerInstance};

fn main() {
    env_logger::Builder::new().filter_level(log::LevelFilter::max()).init();

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

    let mut copy_jpg_transformer = TransformerInstance::new(
        100,
        condenser::OverwriteBehavior::IfNewer,
        "Copy JPGs".to_string(),
        Box::new(CopyTransformer),
    );

    let compress_flac_transformer = {
        use transformers::{CommandArgument, CommandTransformer, FullCommand};
        let ffmpeg_program = "/usr/bin/ffmpeg".into();
        let mut command = FullCommand::new(ffmpeg_program);

        command.args = vec![
            CommandArgument::Arg("-i".into()),
            CommandArgument::InputPath,
            // Quiet down
            CommandArgument::Arg("-hide_banner".into()),
            CommandArgument::Arg("-nostats".into()),
            CommandArgument::Arg("-loglevel".into()),
            CommandArgument::Arg("error".into()),
            // Always overwrite existing files - this is handled at a higher level
            CommandArgument::Arg("-y".into()),
            // Ignore video
            CommandArgument::Arg("-vn".into()),
            // Audio codec is opus
            CommandArgument::Arg("-c:a".into()),
            CommandArgument::Arg("libopus".into()),
            // Bitrate is 96 kbps
            CommandArgument::Arg("-b:a".into()),
            CommandArgument::Arg("96K".into()),
            CommandArgument::OutputPath,
        ];

        let transformer = CommandTransformer {
            transform_command: command,
            output_file_extension: Some("ogg".into()),
            check_command: None,
        };

        let mut instance = TransformerInstance::new(
            75,
            condenser::OverwriteBehavior::IfNewer,
            "Compress to opus".to_string(),
            Box::new(transformer),
        );
        instance
            .filter
            .append_glob(flac_glob.clone(), FilterAction::Accept);
        instance
            .filter
            .append_glob(mp3_glob.clone(), FilterAction::Accept);
        instance
    };

    copy_jpg_transformer
        .filter
        .append_regex(jpg_regex.clone(), FilterAction::Accept);

    let mut transformers = vec![
        copy_jpg_transformer,
        compress_flac_transformer,
    ];

    run_transformations(&mut transformers, &input_dirs, output_dir_path);
}
