use regex::Regex;
//use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CLIError {
    //#[error("executable '{0}' is not allowed")]
    //ForbiddenExecutable(String),
    //#[error("subcommand '{0}' is not allowed for '{1}'")]
    //ForbiddenSubcommand(String, String),
    #[error("argument '{0}' failed validation: {1}")]
    InvalidArgument(String, String),
    //#[error("path '{0}' escapes the workspace")]
    //PathEscape(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

/// One permitted "slot" in an argument list.
#[derive(Debug)]
pub enum ArgConstraint {
    /// Must exactly match one of these literals.
    Enum(&'static [&'static str]),
    /// Must match this regex (compiled once at startup).
    Pattern(Regex),
    /// Must be a valid path confined within the workspace root.
    //WorkspacePath,
    /// Optional; if absent the slot is simply skipped.
    Optional(Box<ArgConstraint>),
}

impl ArgConstraint {
    pub fn validate(&self, value: &str /* , workspace: &Path*/) -> Result<(), CLIError> {
        match self {
            ArgConstraint::Enum(variants) => {
                if variants.contains(&value) {
                    Ok(())
                } else {
                    Err(CLIError::InvalidArgument(
                        value.to_string(),
                        format!("must be one of: {}", variants.join(", ")),
                    ))
                }
            }
            ArgConstraint::Pattern(re) => {
                if re.is_match(value) {
                    Ok(())
                } else {
                    Err(CLIError::InvalidArgument(
                        value.to_string(),
                        format!("must match pattern {}", re.as_str()),
                    ))
                }
            }
            /*ArgConstraint::WorkspacePath => {
                let candidate = workspace.join(value);
                let canonical = candidate
                    .canonicalize()
                    .map_err(|_| CLIError::PathEscape(value.to_string()))?;
                if canonical.starts_with(workspace) {
                    Ok(())
                } else {
                    Err(CLIError::PathEscape(value.to_string()))
                }
            }*/
            ArgConstraint::Optional(inner) => {
                // Caller decides whether to pass this slot at all;
                // if they do, the inner constraint still applies.
                inner.validate(value /* , workspace*/)
            }
        }
    }
}

/// A single whitelisted command definition.
pub struct AllowedCommand {
    /// binary
    pub bin: &'static str,
    /// Ordered constraints for each positional argument slot.
    /// Trailing Optional slots may be omitted.
    pub arg_constraints: Vec<ArgConstraint>,
}
