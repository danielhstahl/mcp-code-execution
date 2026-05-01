use crate::constraints::{AllowedCommand, ArgConstraint, CLIError};
#[cfg(feature = "docker")]
use std::env;
use std::path::PathBuf;
pub struct CodeCommand {
    pub bin: &'static str,
    pub args: Vec<String>,
    pub working_dir: PathBuf,
}

impl CodeCommand {
    /// Construct a validated command. `args` are the caller-supplied values
    /// matched positionally against `spec.arg_constraints`.
    pub fn new(
        spec: &AllowedCommand,
        args: &[String],
        working_dir: PathBuf,
    ) -> Result<Self, CLIError> {
        // Validate arg count: required slots must be filled,
        // optional slots may be absent.
        let required = spec
            .arg_constraints
            .iter()
            .filter(|c| !matches!(c, ArgConstraint::Optional(_)))
            .count();

        if args.len() < required || args.len() > spec.arg_constraints.len() {
            return Err(CLIError::InvalidArgument(
                format!("{} args supplied", args.len()),
                format!("expected {required}..{}", spec.arg_constraints.len()),
            ));
        }

        for (constraint, value) in spec.arg_constraints.iter().zip(args.iter()) {
            constraint.validate(value, &working_dir)?;
        }

        Ok(Self {
            bin: spec.bin,
            args: args.iter().map(|s| s.to_string()).collect(),
            working_dir,
        })
    }

    /// Execute — always via execv, never through a shell.
    #[cfg(feature = "docker")]
    pub async fn execute(&self) -> Result<std::process::Output, std::io::Error> {
        let path =
            env::var("PATH").unwrap_or_else(|_| String::from("/usr/local/bin:/usr/bin:/bin"));
        let home = env::home_dir().unwrap_or_else(|| PathBuf::from("/app"));
        let mut command = tokio::process::Command::new(&self.bin);
        command
            .args(&self.args)
            .current_dir(&self.working_dir)
            .env_clear() // no inherited env leakage
            .env("PATH", path)
            .env("HOME", home);
        //conditionally get this to pass the permission issues in integration testing
        if let Ok(target) = env::var("CARGO_TARGET_DIR") {
            command.env("CARGO_TARGET_DIR", target);
        }
        command.output().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::REGISTRY;
    use std::env;
    #[test]
    fn test_python_code_command_no_error() {
        let curr_wd = env::current_dir().unwrap();
        let args_as_vec: Vec<String> = vec!["integration-tests/helloworld.py".to_string()];
        let _result = CodeCommand::new(&REGISTRY.python, &args_as_vec, curr_wd).unwrap();
    }
    #[test]
    fn test_node_command_no_error() {
        let curr_wd = env::current_dir().unwrap();
        let args_as_vec: Vec<String> = vec!["integration-tests/helloworld.js".to_string()];
        let _result = CodeCommand::new(&REGISTRY.node, &args_as_vec, curr_wd).unwrap();
    }
    #[test]
    fn test_code_command_error() {
        let args_as_vec: Vec<String> = vec!["-m".to_string(), "venv".to_string()];
        let result = CodeCommand::new(&REGISTRY.python, &args_as_vec, PathBuf::from("/tmp"));
        assert!(result.is_err());
    }
    #[test]
    fn test_code_command_error_with_workspace() {
        let curr_wd = env::current_dir().unwrap().join("integration-tests");
        let args_as_vec: Vec<String> = vec!["../README.md".to_string()];
        let result = CodeCommand::new(&REGISTRY.python, &args_as_vec, curr_wd);
        assert!(result.is_err());
    }
}
