use serde::Serialize;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;
pub const LANGUAGE: &[&str] = &["rust", "javascript", "python"];

pub trait CompileService: Send + Sync + 'static {
    fn compile_project(&self, path: &Path, main_file: &Option<PathBuf>)
    -> io::Result<DockerOutput>; //for running this mcp server locally
}

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

pub fn run_docker_command(mut command: Command) -> io::Result<DockerOutput> {
    let output = command.output()?;
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
