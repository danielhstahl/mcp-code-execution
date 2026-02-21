use crate::compilation_service::{CompileService, DockerOutput};
use rmcp::schemars;
use serde::Deserialize;
use std::fmt;
use std::io;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct RustService {
    execution_type: ExecutionType,
}

impl RustService {
    pub fn new(execution_type: ExecutionType) -> Self {
        Self { execution_type }
    }
}

#[derive(Deserialize, Debug, Clone, schemars::JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ExecutionType {
    Run,
    Test,
}

impl fmt::Display for ExecutionType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ExecutionType::Run => write!(f, "{}", "run"),
            ExecutionType::Test => write!(f, "{}", "test"),
        }
    }
}

fn compile_rust_project(
    work_dir: &PathBuf,
    execution_type: &ExecutionType,
) -> io::Result<DockerOutput> {
    let work_dir_str = work_dir.to_str().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::Other,
            "Supplied working directory is not an actual path",
        )
    })?;

    //assumes that external to this we already have run `docker build -t rust-no-root -f docker/rust.Dockerfile ./docker`
    let output = Command::new("docker")
        .args([
            "run",
            "--rm",
            "-v",
            format!("{}:/usr/src/app", &work_dir_str).as_str(),
            "-w",
            "/usr/src/app",
            "rust-no-root",
            "cargo",
            format!("{}", execution_type).as_str(),
        ])
        .output()?;
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

impl CompileService for RustService {
    fn compile_project(
        &self,
        path: &PathBuf,
        _main_file: &Option<PathBuf>,
    ) -> io::Result<DockerOutput> {
        compile_rust_project(&path, &self.execution_type)
    }
}
