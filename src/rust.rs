use crate::compilation_service::{CompileService, DockerOutput};
use rmcp::schemars;
use serde::Deserialize;
use std::fmt;
use std::io;
use std::path::{Path, PathBuf};
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
            ExecutionType::Run => write!(f, "run"),
            ExecutionType::Test => write!(f, "test"),
        }
    }
}

fn build_docker_args(work_dir_str: &str, execution_type: &ExecutionType) -> Vec<String> {
    vec![
        "run".to_string(),
        "--rm".to_string(),
        "-v".to_string(),
        format!("{}:/usr/src/app", work_dir_str),
        "-w".to_string(),
        "/usr/src/app".to_string(),
        "rust-no-root".to_string(),
        "cargo".to_string(),
        execution_type.to_string(),
    ]
}

fn compile_rust_project(
    work_dir: &Path,
    execution_type: &ExecutionType,
) -> io::Result<DockerOutput> {
    let work_dir_str = work_dir
        .to_str()
        .ok_or_else(|| io::Error::other("Supplied working directory is not an actual path"))?;

    //assumes that external to this we already have run `docker build -t rust-no-root -f docker/rust.Dockerfile ./docker`
    let args = build_docker_args(work_dir_str, execution_type);
    let mut command = Command::new("docker");
    command.args(args);

    crate::compilation_service::run_docker_command(command)
}

impl CompileService for RustService {
    fn compile_project(
        &self,
        path: &Path,
        _main_file: &Option<PathBuf>,
    ) -> io::Result<DockerOutput> {
        compile_rust_project(path, &self.execution_type)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_docker_args_run() {
        let args = build_docker_args("/mock/path", &ExecutionType::Run);
        assert_eq!(args[3], "/mock/path:/usr/src/app");
        assert_eq!(args[8], "run");
        assert_eq!(args.len(), 9);
    }

    #[test]
    fn test_build_docker_args_test() {
        let args = build_docker_args("/mock/path", &ExecutionType::Test);
        assert_eq!(args[3], "/mock/path:/usr/src/app");
        assert_eq!(args[8], "test");
        assert_eq!(args.len(), 9);
    }
}
