use crate::compilation_service::{CompileService, DockerOutput};
use rmcp::schemars;
use serde::Deserialize;
use std::fmt;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone)]
pub struct JSService {
    dependency_type: Option<DependencyType>,
}

impl JSService {
    pub fn new(dependency_type: Option<DependencyType>) -> Self {
        Self { dependency_type }
    }
}

#[derive(Deserialize, Debug, Clone, schemars::JsonSchema)]
pub enum DependencyType {
    Npm,
    Yarn,
    Default,
}

impl fmt::Display for DependencyType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DependencyType::Npm => write!(f, "npm"),
            DependencyType::Yarn => write!(f, "yarn"),
            DependencyType::Default => write!(f, "default"),
        }
    }
}

fn build_docker_args(
    work_dir_str: &str,
    file_name_str: &str,
    dependency_type: &Option<DependencyType>,
) -> Vec<String> {
    vec![
        "run".to_string(),
        "--rm".to_string(),
        "-v".to_string(),
        format!("{}:/usr/src/app", work_dir_str),
        "-e".to_string(),
        format!(
            "TYPE={}",
            dependency_type.as_ref().unwrap_or(&DependencyType::Default)
        ),
        "-w".to_string(),
        "/usr/src/app".to_string(),
        "js-no-root".to_string(),
        file_name_str.to_string(),
    ]
}

fn compile_javascript_project(
    work_dir: &Path,
    file_name: &Path,
    dependency_type: &Option<DependencyType>,
) -> io::Result<DockerOutput> {
    let work_dir_str = work_dir
        .to_str()
        .ok_or_else(|| io::Error::other("Supplied working directory is not an actual path"))?;
    let file_name_str = file_name
        .to_str()
        .ok_or_else(|| io::Error::other("Supplied file name is not an actual path"))?;
    //assumes that external to this we already have run `docker build -t js-no-root -f docker/javascript.Dockerfile ./docker`
    let args = build_docker_args(work_dir_str, file_name_str, dependency_type);
    let mut command = Command::new("docker");
    command.args(args);

    crate::compilation_service::run_docker_command(command)
}

impl CompileService for JSService {
    fn compile_project(
        &self,
        path: &Path,
        main_file: &Option<PathBuf>,
    ) -> io::Result<DockerOutput> {
        let main_file = main_file.as_ref().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidData, "main_file needs to be defined")
        })?;
        compile_javascript_project(path, main_file, &self.dependency_type)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_docker_args_npm() {
        let args = build_docker_args("/mock/path", "index.js", &Some(DependencyType::Npm));
        assert_eq!(args[3], "/mock/path:/usr/src/app");
        assert_eq!(args[5], "TYPE=npm");
        assert_eq!(args[9], "index.js");
        assert_eq!(args.len(), 10);
    }

    #[test]
    fn test_build_docker_args_yarn() {
        let args = build_docker_args("/mock/path", "index.js", &Some(DependencyType::Yarn));
        assert_eq!(args[3], "/mock/path:/usr/src/app");
        assert_eq!(args[5], "TYPE=yarn");
        assert_eq!(args[9], "index.js");
        assert_eq!(args.len(), 10);
    }

    #[test]
    fn test_build_docker_args_default() {
        let args = build_docker_args("/mock/path", "index.js", &None);
        assert_eq!(args[3], "/mock/path:/usr/src/app");
        assert_eq!(args[5], "TYPE=default");
        assert_eq!(args[9], "index.js");
        assert_eq!(args.len(), 10);
    }
}
