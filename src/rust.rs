use crate::commands::CodeCommand;
use crate::compilation_service::DockerOutput;
use crate::constraints::CLIError;
use crate::mcp::RustCommand;
use crate::registry::REGISTRY;
use std::io;
use std::path::PathBuf;
use std::process::Command;

//confusingly named because if not feature docker, then spawns docker
#[cfg(not(feature = "docker"))]
fn build_docker_args(
    working_dir: PathBuf,
    command: &RustCommand,
    args: &str,
) -> Result<Vec<String>, CLIError> {
    let registry = match command {
        RustCommand::Cargo => &REGISTRY.cargo,
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
        "rust-no-root".to_string(),
        result.bin.to_string(),
    ];
    docker_args.extend_from_slice(&result.args);
    Ok(docker_args)
}

#[cfg(not(feature = "docker"))]
pub fn compile_rust_project(
    work_dir: PathBuf,
    command: &RustCommand,
    args: &str, //dependency_type: &Option<DependencyType>,
) -> Result<DockerOutput, CLIError> {
    //assumes that external to this we already have run `docker build -t rust-no-root -f docker/rust.Dockerfile ./docker`
    let docker_args = build_docker_args(work_dir, command, args)?; //, dependency_type);
    let mut command = Command::new("docker");
    command.args(docker_args);
    crate::compilation_service::run_docker_command(command)
}

#[cfg(feature = "docker")]
pub fn compile_rust_project(
    work_dir: PathBuf,
    command: &RustCommand,
    args: &str,
) -> io::Result<DockerOutput> {
    let registry = match command {
        RustCommand::Cargo => &REGISTRY.cargo,
    };
    let args_as_vec: Vec<&str> = args.split(" ").collect();
    let result = CodeCommand::new(registry, &args_as_vec, working_dir)?;
    let output = result.execute()?;
    crate::compilation_service::run_docker_command(output)
}

#[cfg(all(test, not(feature = "docker")))]
mod tests {
    use super::*;
    #[test]
    fn test_build_docker_args_run() {
        let args = build_docker_args(PathBuf::from("/tmp"), &RustCommand::Cargo, "build").unwrap();
        assert_eq!(args[3], "/tmp:/usr/src/app");
        assert_eq!(args[8], "build");
        assert_eq!(args.len(), 9);
    }
}
