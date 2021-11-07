use std::str::FromStr;

use condenser::{FilterAction, FilterPattern, FilterSet, InputDirectory};
use serde::Deserialize;

#[derive(Deserialize, Debug, PartialEq)]
pub struct Config {
    pub output_dir: String,
    pub input_dirs: Vec<CfgInputDirectory>,
    pub transformers: Vec<CfgTransformerInstance>,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct CfgInputDirectory {
    pub priority: u32,
    pub path: String,
    pub filters: Vec<CfgFilter>,
}

#[derive(Deserialize, Debug, PartialEq)]
pub enum CfgFilter {
    Glob(String),
    Regex(String),
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct CfgTransformerInstance {
    pub name: String,
    pub priority: u32,
    pub overwrite: CfgOverwriteBehavior,
    pub filters: Vec<CfgFilter>,
    pub transformer: CfgTranformerSelection,
}

#[derive(Deserialize, Debug, PartialEq)]
pub enum CfgOverwriteBehavior {
    Always,
    Never,
    IfNewer,
}

#[derive(Deserialize, Debug, PartialEq)]
pub enum CfgTranformerSelection {
    CopyTransformer,
    CommandTransformer(CfgCommandTransformer)
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct CfgCommandTransformer {
    pub transform_command: CfgCommand,
    pub check_command: Option<CfgCommand>,
    pub output_file_ext: Option<String>,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct CfgCommand {
    pub program: String,
    pub args: Vec<String>,
}

impl TryFrom<CfgFilter> for condenser::FilterPattern {
    // TODO: Proper error type for this
    type Error = Box<dyn std::error::Error>;
    
    fn try_from(f: CfgFilter) -> Result<Self, Self::Error> {
        match f {
            CfgFilter::Glob(str) => {
                let glob = match glob::Pattern::from_str(&str) {
                    Ok(glob) => Ok(glob),
                    // PatternError doesn't implement try, so 
                    Err(err) => Err(Box::new(err)),
                }?;
                Ok(FilterPattern::Glob(glob))
            },
            CfgFilter::Regex(str) => {
                Ok(FilterPattern::Regex(regex::Regex::from_str(&str)?))
            }
        }
    }
}

impl TryFrom<CfgInputDirectory> for condenser::InputDirectory {
    // TODO: Proper error type for this
    type Error = Box<dyn std::error::Error>;
 
    fn try_from(dir: CfgInputDirectory) -> Result<Self, Self::Error> {
        let mut filters = FilterSet::new();
        for filter in dir.filters {
            let filter = filter.try_into()?;
            filters.append(filter, FilterAction::Accept);
        }
        Ok(InputDirectory {
            priority: dir.priority,
            path: dir.path.into(),
            filters
        })
    }
}

impl TryFrom<CfgTransformerInstance> for condenser::TransformerInstance {
    // TODO: Proper error type for this
    type Error = Box<dyn std::error::Error>;

    fn try_from(cfg: CfgTransformerInstance) -> Result<Self, Self::Error> {
        use condenser::*;

        let transformer: Box<dyn Transformer> = match cfg.transformer {
            CfgTranformerSelection::CopyTransformer => Box::new(CopyTransformer),
            CfgTranformerSelection::CommandTransformer(cfg) => {
                let transform_command = cfg.transform_command.try_into().expect("Invalid command definition");
                let check_command = cfg.check_command.map(|cmd| cmd.try_into().expect("Invalid command definition"));
                let transfomer = transformers::CommandTransformer {
                    transform_command,
                    check_command,
                    output_file_extension: cfg.output_file_ext.map(|s| s.into())
                };
                Box::new(transfomer)
            }
        };

        let mut inst = TransformerInstance::new(cfg.priority, cfg.overwrite.into(), cfg.name, transformer);
        for filter in cfg.filters {
            let filter = filter.try_into()?;
            inst.filter.append(filter, FilterAction::Accept);
        }

        Ok(inst)
    }
}

impl TryFrom<CfgCommand> for transformers::FullCommand {
    // TODO: Proper error type for this
    type Error = Box<dyn std::error::Error>;

    fn try_from(cfg: CfgCommand) -> Result<Self, Self::Error> {
        use transformers::{CommandArgument, FullCommand};

        let mut cmd = FullCommand::new(cfg.program.into());
        for arg in cfg.args {
            let arg = match arg.as_str() {
                "!INPUTPATH!" => CommandArgument::InputPath,
                "!OUTPUTPATH!" => CommandArgument::OutputPath,
                str => CommandArgument::Arg(str.into())
            };
            cmd.args.push(arg);
        }

        Ok(cmd)
    }
}

impl From<CfgOverwriteBehavior> for condenser::OverwriteBehavior {
    fn from(cfg: CfgOverwriteBehavior) -> Self {
        use condenser::OverwriteBehavior::*;
        match cfg {
            CfgOverwriteBehavior::Always => Always,
            CfgOverwriteBehavior::Never => Never,
            CfgOverwriteBehavior::IfNewer => IfNewer,
        }
    }
}