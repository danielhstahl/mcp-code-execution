use crate::commands::CodeCommand;
use crate::compilation_service::DockerOutput;
use crate::constraints::CLIError;
use crate::mcp::PythonCommand;
use crate::registry::REGISTRY;
#[cfg(not(feature = "docker"))]
use std::io;
use std::path::PathBuf;
#[cfg(not(feature = "docker"))]
use std::process::Command;

//confusingly named because if not feature docker, then spawns docker
#[cfg(not(feature = "docker"))]
fn build_docker_args(
    working_dir: PathBuf,
    command: &PythonCommand,
    args: &str,
) -> Result<Vec<String>, CLIError> {
    let registry = match command {
        PythonCommand::Python => &REGISTRY.python,
        PythonCommand::Uv => &REGISTRY.uv,
        PythonCommand::Pytest => &REGISTRY.pytest,
    };
    let working_dir_cln = working_dir.clone();
    let work_dir_str = working_dir_cln.to_str().ok_or_else(|| {
        CLIError::Io(io::Error::other(
            "Supplied working directory is not an actual path",
        ))
    })?;
    let args_as_vec: Vec<&str> = args.split(" ").collect();
    let result = CodeCommand::new(registry, &args_as_vec, working_dir)?;
    let mut docker_args = vec![
        "run".to_string(),
        "--rm".to_string(),
        "-v".to_string(),
        format!("{}:/usr/src/app", work_dir_str),
        "-w".to_string(),
        "/usr/src/app".to_string(),
        "python-no-root".to_string(),
        result.bin.to_string(),
    ];
    docker_args.extend_from_slice(&result.args);
    Ok(docker_args)
}

//docker run -it --rm --name my-python-script -v "$PWD":/usr/src/app -w /usr/src/app python:3 python your_script.py
#[cfg(not(feature = "docker"))]
pub fn compile_python_project(
    work_dir: PathBuf,
    command: &PythonCommand,
    args: &str,
) -> Result<DockerOutput, CLIError> {
    let docker_args = build_docker_args(work_dir, command, args)?;
    let mut command = Command::new("docker");
    command.args(docker_args);
    crate::compilation_service::handle_command_output(command.output()?)
}

#[cfg(feature = "docker")]
pub fn compile_python_project(
    work_dir: PathBuf,
    command: &PythonCommand,
    args: &str,
) -> Result<DockerOutput, CLIError> {
    let registry = match command {
        PythonCommand::Python => &REGISTRY.python,
        PythonCommand::Uv => &REGISTRY.uv,
        PythonCommand::Pytest => &REGISTRY.pytest,
    };
    let args_as_vec: Vec<&str> = args.split(" ").collect();
    let result = CodeCommand::new(registry, &args_as_vec, work_dir)?;
    let output = result.execute()?;
    crate::compilation_service::handle_command_output(output)
}

#[cfg(all(test, not(feature = "docker")))]
mod tests {
    use super::*;

    #[test]
    fn test_build_docker_args_python() {
        let args =
            build_docker_args(PathBuf::from("/tmp"), &PythonCommand::Python, "main.py").unwrap();
        assert_eq!(args[3], "/tmp:/usr/src/app");
        assert_eq!(args[8], "main.py");
        assert_eq!(args.len(), 9);
    }

    #[test]
    fn test_build_docker_args_uv() {
        let args =
            build_docker_args(PathBuf::from("/tmp"), &PythonCommand::Uv, "add numpy").unwrap();
        assert_eq!(args[3], "/tmp:/usr/src/app");
        assert_eq!(args[9], "numpy");
        assert_eq!(args.len(), 10);
    }

    #[test]
    fn test_build_docker_args_pytest() {
        let args =
            build_docker_args(PathBuf::from("/tmp"), &PythonCommand::Pytest, "main.py").unwrap();
        assert_eq!(args[3], "/tmp:/usr/src/app");
        assert_eq!(args[8], "main.py");
        assert_eq!(args.len(), 9);
    }
}
