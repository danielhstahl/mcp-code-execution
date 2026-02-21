use crate::compilation_service::{CompileService, DockerOutput};
use rmcp::schemars;
use serde::Deserialize;
use std::fmt;
use std::io;
use std::path::PathBuf;
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
            DependencyType::RequirementsTxt => write!(f, "{}", "requirements.txt"),
            DependencyType::Uv => write!(f, "{}", "uv"),
            DependencyType::Default => write!(f, "{}", "default"),
        }
    }
}

//docker run -it --rm --name my-python-script -v "$PWD":/usr/src/app -w /usr/src/app python:3 python your_script.py
fn compile_python_project(
    work_dir: &PathBuf,
    file_name: &PathBuf,
    dependency_type: &Option<DependencyType>,
) -> io::Result<DockerOutput> {
    let work_dir_str = work_dir.to_str().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::Other,
            "Supplied working directory is not an actual path",
        )
    })?;
    let file_name_str = file_name.to_str().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::Other,
            "Supplied file name is not an actual path",
        )
    })?;
    //assumes that external to this we already have run `docker build -t python-no-root -f docker/python.Dockerfile ./docker`
    let output = Command::new("docker")
        .args([
            "run",
            "--rm",
            "-v",
            format!("{}:/usr/src/app", &work_dir_str).as_str(),
            "-e",
            format!(
                "TYPE={}",
                dependency_type
                    .as_ref()
                    .unwrap_or_else(|| &DependencyType::Default)
            )
            .as_str(),
            "-w",
            "/usr/src/app",
            "python-no-root",
            file_name_str,
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

impl CompileService for PythonService {
    /*fn compile(&self, _data: String) {
        //self.data.clone()
    }*/

    fn compile_project(
        &self,
        path: &PathBuf,
        main_file: &Option<PathBuf>,
    ) -> io::Result<DockerOutput> {
        let main_file = main_file.as_ref().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidData, "main_file needs to be defined")
        })?;
        compile_python_project(&path, &main_file, &self.dependency_type)
    }
}
