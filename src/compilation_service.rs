use crate::constraints::CLIError;
use serde::Serialize;
use std::io;
use std::process::Output;
pub const LANGUAGE: &[&str] = &["rust", "javascript", "python"];

#[derive(Serialize)]
pub struct DockerOutput {
    stdout: String,
    stderr: String,
    pub is_error: bool,
}

impl DockerOutput {
    pub fn new(stdout: String, stderr: String, is_error: bool) -> Self {
        Self {
            stdout,
            stderr,
            is_error,
        }
    }
}

pub fn handle_command_output(output: Output) -> Result<DockerOutput, CLIError> {
    let stdout = String::from_utf8(output.stdout).map_err(|_e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "Unable to convert stdout to string",
        )
    })?;
    let stderr = String::from_utf8(output.stderr).map_err(|_e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "Unable to convert stderr to string",
        )
    })?;
    Ok(DockerOutput::new(stdout, stderr, !output.status.success()))
}
