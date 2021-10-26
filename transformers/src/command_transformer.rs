use std::{
    ffi::OsString,
    fmt::Display,
    path::{Path, PathBuf},
    process::ExitStatus,
};

use condenser::{InputId, OutputId, Transformer};

/// An argument to pass to the command transformer.
pub enum CommandArgument {
    /// An arbitary string to be an argument.
    Arg(OsString),

    /// Indicates that this argument should be the path to the input file.
    InputPath,

    /// Indicates that this argument should be the path to the output file.
    OutputPath,
}

#[derive(Debug)]
pub enum CommandError {
    /// A command has exited with a non-zero status code.
    ExitStatus(ExitStatus),

    /// An error occurred executing the command.
    Error(Box<dyn std::error::Error>),
}

impl CommandError {
    /// Creates a new CommandError from an exit status.
    pub fn from_status_code(exit_status: ExitStatus) -> CommandError {
        CommandError::ExitStatus(exit_status)
    }

    /// Creates a new Command error from an Error.
    pub fn from_error(error: Box<dyn std::error::Error>) -> CommandError {
        CommandError::Error(error)
    }
}

impl Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CommandError::ExitStatus(status) => write!(f, "Exit status: {}", status),
            CommandError::Error(err) => write!(f, "Error: {}", err),
        }
    }
}

impl std::error::Error for CommandError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            CommandError::ExitStatus(_) => None,
            CommandError::Error(err) => Some(err.as_ref()),
        }
    }

    fn description(&self) -> &str {
        "description() is deprecated; use Display"
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        self.source()
    }
}

/// A full command with an executable path and arguments.
pub struct FullCommand {
    /// The path to an executable.
    pub program: OsString,

    /// An ordered list of arguments to pass to the executable.
    pub args: Vec<CommandArgument>,
}

impl FullCommand {
    /// Creates a new instance of FullCommand with the provided program and no arguments.
    pub fn new(program: OsString) -> FullCommand {
        FullCommand {
            program,
            args: Vec::new(),
        }
    }

    /// Executes the program with the specifed arguments, substituting input and output
    /// for any arguments equal to InputPath or OutputPath.
    pub fn execute(&self, input: &Path, output: &Path) -> Result<i32, Box<dyn std::error::Error>> {
        use std::process::Command;

        let status = Command::new(&self.program)
            .args(self.args.iter().map(|arg| match arg {
                CommandArgument::Arg(arg) => arg,
                CommandArgument::InputPath => input.as_os_str(),
                CommandArgument::OutputPath => output.as_os_str(),
            }))
            .status()?;

        // TODO: Check return status and maybe output?
        // TODO: More robust status handling.
        let code = status.code().unwrap_or_default();
        Ok(code)
    }
}

/// A transformer that executes an external command.
pub struct CommandTransformer {
    /// The command to execute the transformation.
    pub transform_command: FullCommand,

    /// The command to check the ability of the transformer to handle the input
    /// file. If None, it is assumed that the transformer can always handle
    /// the input file.
    pub check_command: Option<FullCommand>,

    /// The output file extension - uses the input file name if None.
    pub output_file_extension: Option<OsString>,
}

impl Transformer for CommandTransformer {
    fn can_handle(&self, _input: &InputId) -> bool {
        true
    }

    fn determine_output_id(&self, input: &InputId) -> OutputId {
        OutputId(input.file_path().with_extension("").into_os_string())
    }

    fn determine_output_path(&self, input: &InputId) -> PathBuf {
        match &self.output_file_extension {
            Some(ext) => input.file_path().with_extension(ext),
            None => input.file_path().to_path_buf(),
        }
    }

    fn transform(&self, input: &Path, output: &Path) -> Result<(), Box<dyn std::error::Error>> {
        // Create the parent directory if it doesn't exist
        if let Some(output_parent) = output.parent() {
            std::fs::create_dir_all(output_parent).map_err(Box::new)?;
        }

        self.transform_command.execute(input, output)?;
        Ok(())
    }
}
