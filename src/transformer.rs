use std::{ffi::OsString, path::{Path, PathBuf}};

use crate::filters::FilterSet;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
/// An ID uniquely identifying an input.
pub struct InputId<'a, 'b> {
    pub dir_path: &'a Path,
    pub file_path: &'b Path,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
/// An ID uniquely identifying an output.
pub struct OutputId(pub OsString);

/// Transforms one file to another
pub trait Transformer {
    /// Determines the output id of a transformation.
    fn determine_output_id(&self, input: &InputId) -> OutputId;

    /// Determines the relative path of the output of a transformation.
    fn determine_output_path(&self, input: &InputId) -> PathBuf;

    /// Transforms the file at input into a new file at output. The input file is not modified, but
    /// any file existing at output is overwritten.
    fn transform(&self, input: &Path, output: &Path) -> Result<(), Box<dyn std::error::Error>> ;
}

/// Metadata about an individual instance of a transformer.
pub struct TransformerInstance {
    /// The priority - transformers with higher priorities are run first.
    pub priority: u32,

    /// The filter that files should pass before being transformed.
    pub filter: FilterSet,

    /// The tranformer itself.
    pub transformer: Box<dyn Transformer>,
}

/// A simple transformer that copies the entire file located at input to output.
pub struct CopyTransformer;

impl Transformer for CopyTransformer {
    fn determine_output_id(&self, input: &InputId) -> OutputId {
        OutputId(input.file_path.with_extension("").into_os_string())
    }

    fn determine_output_path(&self, input: &InputId) -> PathBuf {
        input.file_path.to_path_buf()
    }

    fn transform(&self, input: &Path, output: &Path) -> Result<(), Box<dyn std::error::Error>> {
        // Create the parent directory if it doesn't exist
        if let Some(output_parent) = output.parent() {
            std::fs::create_dir_all(output_parent).map_err(|err| Box::new(err))?;
        }
        std::fs::copy(input, output).map_err(|err| Box::new(err))?;
        Ok(())
    }
}