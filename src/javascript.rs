use crate::commands::CodeCommand;
use crate::compilation_service::DockerOutput;
use crate::constraints::CLIError;
use crate::mcp::NodeCommand;
use crate::registry::REGISTRY;
#[cfg(not(feature = "docker"))]
use std::io;
use std::path::PathBuf;
#[cfg(not(feature = "docker"))]
use tokio::process::Command;

//confusingly named because if not feature docker, then spawns docker
#[cfg(not(feature = "docker"))]
fn build_docker_args(
    working_dir: PathBuf,
    command: &NodeCommand,
    args: &[String],
) -> Result<Vec<String>, CLIError> {
    let registry = match command {
        NodeCommand::Npm => &REGISTRY.npm,
        NodeCommand::Yarn => &REGISTRY.yarn,
        NodeCommand::Node => &REGISTRY.node,
    };
    let working_dir_cln = working_dir.clone();
    let work_dir_str = working_dir_cln.to_str().ok_or_else(|| {
        CLIError::Io(io::Error::other(
            "Supplied working directory is not an actual path",
        ))
    })?;
    let result = CodeCommand::new(registry, args, working_dir)?;
    let mut docker_args = vec![
        "run".to_string(),
        "--rm".to_string(),
        "-v".to_string(),
        format!("{}:/usr/src/app", work_dir_str),
        "-w".to_string(),
        "/usr/src/app".to_string(),
        "js-no-root".to_string(),
        result.bin.to_string(),
    ];
    docker_args.extend_from_slice(&result.args);
    Ok(docker_args)
}

#[cfg(not(feature = "docker"))]
pub async fn compile_javascript_project(
    work_dir: PathBuf,
    command: &NodeCommand,
    args: &[String],
) -> Result<DockerOutput, CLIError> {
    let docker_args = build_docker_args(work_dir, command, args)?;
    let mut command = Command::new("docker");
    command.args(docker_args);
    crate::compilation_service::handle_command_output(command.output().await?)
}

#[cfg(feature = "docker")]
pub async fn compile_javascript_project(
    work_dir: PathBuf,
    command: &NodeCommand,
    args: &[String],
) -> Result<DockerOutput, CLIError> {
    let registry = match command {
        NodeCommand::Npm => &REGISTRY.npm,
        NodeCommand::Yarn => &REGISTRY.yarn,
        NodeCommand::Node => &REGISTRY.node,
    };
    let result = CodeCommand::new(registry, args, work_dir)?;
    let output = result.execute().await?;
    crate::compilation_service::handle_command_output(output)
}

#[cfg(all(test, not(feature = "docker")))]
mod tests {
    use super::*;

    #[test]
    fn test_build_docker_args_npm() {
        let args =
            build_docker_args(PathBuf::from("/tmp"), &NodeCommand::Npm, "install axios").unwrap();
        assert_eq!(args[3], "/tmp:/usr/src/app");
        assert_eq!(args[9], "axios");
        assert_eq!(args.len(), 10);
    }

    #[test]
    fn test_build_docker_args_yarn() {
        let args =
            build_docker_args(PathBuf::from("/tmp"), &NodeCommand::Yarn, "install axios").unwrap();
        println!("{:?}", args);
        assert_eq!(args[3], "/tmp:/usr/src/app");
        assert_eq!(args[9], "axios");
        assert_eq!(args.len(), 10);
    }

    #[test]
    fn test_build_docker_args_node() {
        let args =
            build_docker_args(PathBuf::from("/tmp"), &NodeCommand::Node, "./index.ts").unwrap();
        assert_eq!(args[3], "/tmp:/usr/src/app");
        assert_eq!(args[8], "./index.ts");
        assert_eq!(args.len(), 9);
    }
}
