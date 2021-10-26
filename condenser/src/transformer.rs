use std::{
    collections::{HashMap, HashSet},
    ffi::OsString,
    panic,
    path::{Path, PathBuf},
};

use crate::filters::FilterSet;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
/// An ID uniquely identifying an input.
pub struct InputId<'a, 'b> {
    dir_path: &'a Path,
    file_path: &'b Path,
}

impl<'a, 'b> InputId<'a, 'b> {
    /// Creates a new InputId. dir_path must be absolute, and file_path
    /// must be relative.
    pub fn new(dir_path: &'a Path, file_path: &'b Path) -> InputId<'a, 'b> {
        if dir_path.is_relative() {
            panic!(
                "dir_path may not be relative: {}",
                dir_path.to_string_lossy()
            )
        }
        if file_path.is_absolute() {
            panic!(
                "file_path may not be absolute: {}",
                file_path.to_string_lossy()
            )
        }

        InputId {
            dir_path,
            file_path,
        }
    }

    /// Returns the directory path.
    pub fn dir_path(&self) -> &'a Path {
        &self.dir_path
    }

    /// Returns the file path.
    pub fn file_path(&self) -> &'b Path {
        &self.file_path
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
/// An ID uniquely identifying an output.
pub struct OutputId(pub OsString);

/// Transforms one file to another
pub trait Transformer {
    /// Tests whether or not this transformer can handle the given input file.
    fn can_handle(&self, input: &InputId) -> bool;

    /// Determines the output id of a transformation.
    fn determine_output_id(&self, input: &InputId) -> OutputId;

    /// Determines the relative path of the output of a transformation.
    fn determine_output_path(&self, input: &InputId) -> PathBuf;

    /// Transforms the file at input into a new file at output. The input file is not modified, but
    /// any file existing at output is overwritten.
    fn transform(&self, input: &Path, output: &Path) -> Result<(), Box<dyn std::error::Error>>;
}

/// Metadata about an individual instance of a transformer.
pub struct TransformerInstance {
    /// The priority - transformers with higher priorities are run first.
    pub priority: u32,

    pub name: String,

    /// The filter that files should pass before being transformed.
    pub filter: FilterSet,

    /// The tranformer itself.
    pub transformer: Box<dyn Transformer>,

    /// Contains the files enqueued for transformation.
    pub input_queues: HashMap<PathBuf, Vec<PathBuf>>,
}

impl TransformerInstance {
    /// Creates a new instance of a transformer, with the specified priority, name, and transformer object.
    pub fn new(
        priority: u32,
        name: String,
        transformer: Box<dyn Transformer>,
    ) -> TransformerInstance {
        TransformerInstance {
            priority,
            name,
            transformer,
            filter: FilterSet::new(),
            input_queues: HashMap::new(),
        }
    }

    /// Finds acceptable input files within unprocessed_files, adds them to the input
    /// queue associated with input_dir_path and adds the output ids to claimed_outputs.
    pub fn claim_outputs(
        &mut self,
        input_dir_path: &Path,
        unprocessed_files: &mut Vec<PathBuf>,
        claimed_outputs: &mut HashSet<OutputId>,
    ) -> u64 {
        let mut claim_count = 0;
        let transformer = &mut self.transformer;
        unprocessed_files.retain(|path| {
            if !self.filter.is_acceptable(path) {
                // Skip this file since it doesn't pass the filter.
                return true;
            }

            let input_id = InputId {
                dir_path: input_dir_path,
                file_path: path,
            };
            if !transformer.can_handle(&input_id) {
                // Skip this file since the transformer says it can't handle it.
                return true;
            }

            let output_id = transformer.determine_output_id(&input_id);
            if !claimed_outputs.insert(output_id) {
                // Skip this file since a previous transformer has claimed the output.
                return true;
            }

            // Now we have successully claimed this file, so add it to our queue.
            self.input_queues
                .entry(input_dir_path.to_path_buf())
                .or_insert_with(Vec::new)
                .push(path.to_path_buf());
            claim_count += 1;
            false
        });
        claim_count
    }

    /// Processes all input queues, outputting to output_dir.
    pub fn process_queues(
        &mut self,
        output_dir: &Path,
    ) -> Vec<(PathBuf, Box<dyn std::error::Error>)> {
        let mut failed = Vec::new();
        for (parent_dir, file_paths) in &mut self.input_queues {
            for file_path in file_paths.drain(..) {
                let input_path: PathBuf = [parent_dir, &file_path].iter().collect();
                // TODO: Use a temporary file
                // TODO: Add handling of existing files
                let output_path: PathBuf = [
                    output_dir,
                    &self
                        .transformer
                        .determine_output_path(&InputId::new(&parent_dir, &file_path)),
                ]
                .iter()
                .collect();
                if let Err(err) = self.transformer.transform(&input_path, &output_path) {
                    failed.push((input_path, err));
                }
            }
        }
        failed
    }
}

/// A simple transformer that copies the entire file located at input to output.
pub struct CopyTransformer;

impl Transformer for CopyTransformer {
    fn can_handle(&self, _input: &InputId) -> bool {
        true
    }

    fn determine_output_id(&self, input: &InputId) -> OutputId {
        OutputId(input.file_path.with_extension("").into_os_string())
    }

    fn determine_output_path(&self, input: &InputId) -> PathBuf {
        input.file_path.to_path_buf()
    }

    fn transform(&self, input: &Path, output: &Path) -> Result<(), Box<dyn std::error::Error>> {
        // Create the parent directory if it doesn't exist
        if let Some(output_parent) = output.parent() {
            std::fs::create_dir_all(output_parent).map_err(Box::new)?;
        }
        std::fs::copy(input, output).map_err(Box::new)?;
        Ok(())
    }
}
