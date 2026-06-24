use crate::compilation_service::{DockerOutput, LANGUAGE};
use crate::javascript::compile_javascript_project;
use crate::python::compile_python_project;
use crate::rust::compile_rust_project;
use rmcp::{
    ErrorData as McpError, RoleServer, ServerHandler,
    handler::server::wrapper::Parameters,
    model::{
        CallToolResult, Content, Implementation, InitializeRequestParams, InitializeResult,
        ProtocolVersion, ServerCapabilities, ServerInfo,
    },
    schemars,
    service::RequestContext,
    tool, tool_handler, tool_router,
};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, schemars::JsonSchema, Deserialize)]
#[schemars(rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum PythonCommand {
    Python,
    Pip,
    Uv,
    Pytest,
}

#[derive(Debug, schemars::JsonSchema, Deserialize)]
#[schemars(rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum NodeCommand {
    Npm,
    Yarn,
    Node,
}

#[derive(Debug, schemars::JsonSchema, Deserialize)]
#[schemars(rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum RustCommand {
    Cargo,
}

#[derive(Debug, schemars::JsonSchema, Deserialize)]
pub struct PythonInput {
    #[schemars(
        description = "Path to the project.  This should be a relative folder/directory, not a file.  Must be relative to the current working directory.  Example: 'my_project' or './src/my_project'."
    )]
    pub project_dir: PathBuf,
    #[schemars(description = "Command to run.  One of \"python\", \"pip\", \"uv\", \"pytest\"")]
    pub command: PythonCommand,
    #[schemars(
        description = "CLI arguments to pass to the command.  Examples: \"['main.py']\" or \"['install', 'numpy']\""
    )]
    pub args: Vec<String>,
}

#[derive(Debug, schemars::JsonSchema, Deserialize)]
pub struct JavascriptInput {
    #[schemars(
        description = "Path to the project.  This should be a relative folder/directory, not a file.  Must be relative to the current working directory. Example: 'my_project' or './src/my_project'."
    )]
    pub project_dir: PathBuf,
    #[schemars(description = "Command to run. One of \"node\", \"npm\", \"yarn\"")]
    pub command: NodeCommand,
    #[schemars(
        description = "CLI arguments to pass to the command.  Examples: \"['index.ts']\" or \"['install', 'axios']\""
    )]
    pub args: Vec<String>,
}

#[derive(Debug, schemars::JsonSchema, Deserialize)]
pub struct RustInput {
    #[schemars(
        description = "Path to the project.  This should be a relative folder/directory, not a file.  Must be relative to the current working directory.  Example: 'my_project' or './src/my_project'."
    )]
    pub project_dir: PathBuf,
    #[schemars(description = "Command to run.  \"cargo\" is the only valid command.")]
    pub command: RustCommand,
    #[schemars(
        description = "CLI arguments to pass to the command.  Examples: \"['test']\" or \"['build']"
    )]
    pub args: Vec<String>,
}

fn convert_docker_output_to_tool_result(docker_output: DockerOutput) -> CallToolResult {
    let is_error = docker_output.is_error;
    let json_content = Content::json(docker_output).unwrap();
    if is_error {
        CallToolResult::error(vec![json_content])
    } else {
        CallToolResult::success(vec![json_content])
    }
}

pub struct CodeCompiler {
    #[cfg(feature = "docker")]
    path: PathBuf,
}

fn sanitize_path(canonical_base_path: &PathBuf, sub_folder: &PathBuf) -> Result<PathBuf, McpError> {
    if sub_folder.is_absolute() {
        return Err(McpError::invalid_params(
            "project_dir must be a relative path",
            None,
        ));
    }
    let base_path = canonical_base_path.join(&sub_folder); //();
    // Canonicalize and verify the final path is a child of the allowed base path
    let canonical_full = base_path
        .canonicalize()
        .map_err(|e| McpError::invalid_params(e.to_string(), None))?;
    if !canonical_full.starts_with(&canonical_base_path) {
        return Err(McpError::invalid_params(
            "project_dir escapes the workspace",
            None,
        ));
    }
    Ok(canonical_full)
}

#[cfg(feature = "docker")]
#[tool_router]
impl CodeCompiler {
    pub fn new(project_location: &str) -> Self {
        Self {
            path: PathBuf::from(project_location).canonicalize().unwrap(),
        }
    }

    #[tool(
        description = "Execute Python code in an existing project directory relative to current working directory. Use \
                           command 'python', 'pytest', or 'uv' to run a script, 'pip' or 'uv' to install dependencies. \
                           Requires the project files to already exist in project_dir. \
                           Write the files first, then call this tool to run or test them. \
                           State does not persist between calls."
    )]
    pub async fn run_python(
        &self,
        Parameters(PythonInput {
            project_dir,
            command,
            args,
        }): Parameters<PythonInput>,
    ) -> Result<CallToolResult, McpError> {
        let canonical_full = sanitize_path(&self.path, &project_dir)?;
        let result = compile_python_project(canonical_full, &command, &args).await;
        result
            .map(convert_docker_output_to_tool_result)
            .map_err(|e| McpError::internal_error(e.to_string(), None))
    }

    #[tool(
        description = "Execute Javascript code in an existing project directory relative to current working directory. Use \
                       command 'node' to run a script, 'yarn' or 'npm' to install dependencies. \
                       Requires the project files to already exist in project_dir. \
                       Write the files first, then call this tool to run or test them. \
                       State does not persist between calls."
    )]
    pub async fn run_javascript(
        &self,
        Parameters(JavascriptInput {
            project_dir,
            command,
            args,
        }): Parameters<JavascriptInput>,
    ) -> Result<CallToolResult, McpError> {
        let canonical_full = sanitize_path(&self.path, &project_dir)?;
        let result = compile_javascript_project(canonical_full, &command, &args).await;
        result
            .map(convert_docker_output_to_tool_result)
            .map_err(|e| McpError::internal_error(e.to_string(), None))
    }
    #[tool(
        description = "Execute Rust code in an existing project directory relative to current working directory. Use \
                   command 'cargo'. \
                   Requires the project files to already exist in project_dir. \
                   Write the files first, then call this tool to run or test them. \
                   State does not persist between calls."
    )]
    pub async fn run_rust(
        &self,
        Parameters(RustInput {
            project_dir,
            command,
            args,
        }): Parameters<RustInput>,
    ) -> Result<CallToolResult, McpError> {
        let canonical_full = sanitize_path(&self.path, &project_dir)?;
        let result = compile_rust_project(canonical_full, &command, &args).await;
        result
            .map(convert_docker_output_to_tool_result)
            .map_err(|e| McpError::internal_error(e.to_string(), None))
    }

    #[tool(description = "get supported languages")]
    pub async fn get_supported_languages(&self) -> Result<CallToolResult, McpError> {
        let json_result = serde_json::to_string(&LANGUAGE).unwrap();
        Ok(CallToolResult::success(vec![Content::text(json_result)]))
    }
}

#[cfg(not(feature = "docker"))]
#[tool_router]
impl CodeCompiler {
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "docker")]
            path: PathBuf::from(project_location),
        }
    }

    #[tool(
        description = "Execute Python code in an existing project directory relative to current working directory. Use \
                       command 'python', 'pytest', or 'uv' to run a script, 'pip' or 'uv' to install dependencies. \
                       Requires the project files to already exist in project_dir. \
                       Write the files first, then call this tool to run or test them. \
                       State does not persist between calls."
    )]
    pub async fn run_python(
        &self,
        Parameters(PythonInput {
            project_dir,
            command,
            args,
        }): Parameters<PythonInput>,
    ) -> Result<CallToolResult, McpError> {
        let result = compile_python_project(project_dir, &command, &args).await;
        result
            .map(convert_docker_output_to_tool_result)
            .map_err(|e| McpError::internal_error(e.to_string(), None))
    }

    #[tool(
        description = "Execute Javascript code in an existing project directory relative to current working directory. Use \
                       command 'node' to run a script, 'yarn' or 'npm' to install dependencies. \
                       Requires the project files to already exist in project_dir. \
                       Write the files first, then call this tool to run or test them. \
                       State does not persist between calls."
    )]
    pub async fn run_javascript(
        &self,
        Parameters(JavascriptInput {
            project_dir,
            command,
            args,
        }): Parameters<JavascriptInput>,
    ) -> Result<CallToolResult, McpError> {
        let result = compile_javascript_project(project_dir, &command, &args).await;
        result
            .map(convert_docker_output_to_tool_result)
            .map_err(|e| McpError::internal_error(e.to_string(), None))
    }
    #[tool(
        description = "Execute Rust code in an existing project directory relative to current working directory. Use \
                   command 'cargo'. \
                   Requires the project files to already exist in project_dir. \
                   Write the files first, then call this tool to run or test them. \
                   State does not persist between calls."
    )]
    pub async fn run_rust(
        &self,
        Parameters(RustInput {
            project_dir,
            command,
            args,
        }): Parameters<RustInput>,
    ) -> Result<CallToolResult, McpError> {
        let result = compile_rust_project(project_dir, &command, &args).await;
        result
            .map(convert_docker_output_to_tool_result)
            .map_err(|e| McpError::internal_error(e.to_string(), None))
    }

    #[tool(description = "Get languages supported by this MCP server")]
    pub async fn get_supported_languages(&self) -> Result<CallToolResult, McpError> {
        let json_result = serde_json::to_string(&LANGUAGE).unwrap();
        Ok(CallToolResult::success(vec![Content::text(json_result)]))
    }
}

#[tool_handler]
impl ServerHandler for CodeCompiler {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder()
            .enable_tools()
            .build()).with_instructions("This server provides compilation and code execution tools. Tools: run_python, run_javascript, run_rust, get_supported_languages.")
            .with_server_info(Implementation::from_build_env()).with_protocol_version(ProtocolVersion::V_2025_06_18)
    }

    async fn initialize(
        &self,
        _request: InitializeRequestParams,
        context: RequestContext<RoleServer>,
    ) -> Result<InitializeResult, McpError> {
        if let Some(http_request_part) = context.extensions.get::<axum::http::request::Parts>() {
            let initialize_headers = &http_request_part.headers;
            let initialize_uri = &http_request_part.uri;
            tracing::info!(?initialize_headers, %initialize_uri, "initialize from http server");
        }
        Ok(self.get_info())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    #[test]
    fn test_sanitize_if_good() {
        fs::create_dir_all("/tmp/hello/world").unwrap();
        let base = PathBuf::from("/tmp").canonicalize().unwrap();
        let sub = PathBuf::from("hello/world");
        let result = sanitize_path(&base, &sub).unwrap();
        fs::remove_dir_all("/tmp/hello").unwrap();
        assert!(result.to_str().unwrap().ends_with("/tmp/hello/world"));
    }
    #[test]
    fn test_error_if_full_dir() {
        fs::create_dir_all("/tmp/hello/world").unwrap();
        let base = PathBuf::from("/tmp").canonicalize().unwrap();
        let sub = PathBuf::from("/hello/world");
        let result = sanitize_path(&base, &sub);
        fs::remove_dir_all("/tmp/hello").unwrap();
        assert!(result.is_err());
    }
    #[test]
    fn test_error_if_escape() {
        fs::create_dir_all("/tmp/hello/world").unwrap();
        let base = PathBuf::from("/tmp").canonicalize().unwrap();
        let sub = PathBuf::from("../hello/world");
        let result = sanitize_path(&base, &sub);
        fs::remove_dir_all("/tmp/hello").unwrap();
        assert!(result.is_err());
    }
}
