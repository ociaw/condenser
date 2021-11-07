mod config;
use std::path::PathBuf;

use condenser::run_transformations;

use condenser::TransformerInstance;
use log::error;

fn main() {
    env_logger::Builder::new().filter_level(log::LevelFilter::max()).init();

    // TODO: This should be able to be specified from the command line.
    let config = std::fs::read_to_string("./config.toml").unwrap();
    let config: config::Config = toml::from_str(&config).unwrap();
    
    let mut input_dirs = Vec::new();
    let output_dir_path: PathBuf = config.output_dir.into();
    for dir in config.input_dirs {
        match dir.try_into() {
            Ok(dir) => input_dirs.push(dir),
            Err(err) => error!("Failed to read config file: {}", err),
        }
    }

    let mut ts: Vec<TransformerInstance> = config.transformers
        .into_iter().map(|t| t.try_into().expect("Invalid transformer instance config."))
        .collect();

    run_transformations(&mut ts, &input_dirs, output_dir_path);
}
