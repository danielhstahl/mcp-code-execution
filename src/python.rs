use crate::compilation_service::{CompileService, DockerOutput};
use rmcp::schemars;
use serde::Deserialize;
use std::fmt;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone)]
pub struct PythonService {
    dependency_type: Option<DependencyType>,
}

impl PythonService {
    pub fn new(dependency_type: Option<DependencyType>) -> Self {
        Self { dependency_type }
    }
}

#[derive(Deserialize, Debug, Clone, schemars::JsonSchema)]
pub enum DependencyType {
    RequirementsTxt,
    Uv,
    Default,
}

impl fmt::Display for DependencyType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DependencyType::RequirementsTxt => write!(f, "requirements.txt"),
            DependencyType::Uv => write!(f, "uv"),
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
        "python-no-root".to_string(),
        file_name_str.to_string(),
    ]
}

//docker run -it --rm --name my-python-script -v "$PWD":/usr/src/app -w /usr/src/app python:3 python your_script.py
fn compile_python_project(
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
    //assumes that external to this we already have run `docker build -t python-no-root -f docker/python.Dockerfile ./docker`
    let args = build_docker_args(work_dir_str, file_name_str, dependency_type);
    let mut command = Command::new("docker");
    command.args(args);

    crate::compilation_service::run_docker_command(command)
}

impl CompileService for PythonService {
    fn compile_project(
        &self,
        path: &Path,
        main_file: &Option<PathBuf>,
    ) -> io::Result<DockerOutput> {
        let main_file = main_file.as_ref().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidData, "main_file needs to be defined")
        })?;
        compile_python_project(path, main_file, &self.dependency_type)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_docker_args_requirements_txt() {
        let args = build_docker_args(
            "/mock/path",
            "main.py",
            &Some(DependencyType::RequirementsTxt),
        );
        assert_eq!(args[3], "/mock/path:/usr/src/app");
        assert_eq!(args[5], "TYPE=requirements.txt");
        assert_eq!(args[9], "main.py");
        assert_eq!(args.len(), 10);
    }

    #[test]
    fn test_build_docker_args_uv() {
        let args = build_docker_args("/mock/path", "main.py", &Some(DependencyType::Uv));
        assert_eq!(args[3], "/mock/path:/usr/src/app");
        assert_eq!(args[5], "TYPE=uv");
        assert_eq!(args[9], "main.py");
        assert_eq!(args.len(), 10);
    }

    #[test]
    fn test_build_docker_args_default() {
        let args = build_docker_args("/mock/path", "main.py", &None);
        assert_eq!(args[3], "/mock/path:/usr/src/app");
        assert_eq!(args[5], "TYPE=default");
        assert_eq!(args[9], "main.py");
        assert_eq!(args.len(), 10);
    }
}
