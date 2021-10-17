use std::{fs::read_dir, io::Error, path::{Path, PathBuf}};

use crate::filters::FilterSet;

pub struct InputDirectory {
    /// The priority of this input - higher priorities are favored in conflict resolution.
    pub priority: u32,

    /// The filters used when enumerating the files in this directory.
    pub filters: FilterSet,

    /// The absolute path to the directory.
    pub path: PathBuf,
}

impl InputDirectory {
    /// Enumerates all the files under this input directory and returns a Vec
    /// with all relative paths that match the filters.
    pub fn enumerate_files<'a>(&'a self) -> Result<Vec<PathBuf>, Error> {
        let mut vec = Vec::new();
        self.recurse_dir(&self.path, &mut vec)?;
        Ok(vec)
    }

    /// Recursively enumerates over this directory, pushing relative acceptable paths to vec.
    fn recurse_dir<'a>(&'a self, dir_path: &Path, vec: &mut Vec<PathBuf>) -> Result<(), Error> {
        let dir = read_dir(dir_path)?;
        for entry in dir {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                self.recurse_dir(&path, vec)?;
                continue
            } else if !path.is_file() {
                // Skip things that aren't files and aren't paths
                continue
            }

            let relative = path.strip_prefix(&self.path);

            // ReadDir should be returning paths that include the directory path, so it
            // should always return ok.
            let relative = relative.expect("Expected path to be relative to self.path");

            if self.filters.is_acceptable(relative) {
                vec.push(relative.to_path_buf())
            }
        }
        Ok(())
    }
}