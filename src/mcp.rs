use crate::compilation_service::{DockerOutput, LANGUAGE};
use crate::javascript::compile_javascript_project;
use crate::python::compile_python_project;
use crate::rust::compile_rust_project;
use rmcp::{
    ErrorData as McpError, RoleServer, ServerHandler,
    handler::server::{/*router::tool::ToolRouter,*/ wrapper::Parameters},
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
        description = "Path to the project.  This should be a folder/directory, not a file, and should be relative to the current working directory."
    )]
    pub project_dir: PathBuf,
    #[schemars(description = "Command to run.")]
    pub command: PythonCommand,
    #[schemars(description = "CLI arguments to pass to the command")]
    pub args: String,
}

#[derive(Debug, schemars::JsonSchema, Deserialize)]
pub struct JavascriptInput {
    #[schemars(
        description = "Path to the project.  This should be a folder/directory, not a file, and should be relative to the current working directory."
    )]
    pub project_dir: PathBuf,
    #[schemars(description = "Command to run.")]
    pub command: NodeCommand,
    #[schemars(description = "CLI arguments to pass to the command")]
    pub args: String,
}

#[derive(Debug, schemars::JsonSchema, Deserialize)]
pub struct RustInput {
    #[schemars(
        description = "Path to the project.  This should be a folder/directory, not a file, and should be relative to the current working directory."
    )]
    pub project_dir: PathBuf,
    #[schemars(description = "Command to run.")]
    pub command: RustCommand,
    #[schemars(description = "CLI arguments to pass to the command")]
    pub args: String,
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
    //tool_router: ToolRouter<Self>,
    #[cfg(feature = "docker")]
    path: PathBuf,
}

#[cfg(feature = "docker")]
#[tool_router]
impl CodeCompiler {
    pub fn new(project_location: &str) -> Self {
        Self {
            //tool_router: Self::tool_router(),
            path: PathBuf::from(project_location),
        }
    }

    #[tool(description = "run Python code")]
    pub async fn run_python(
        &self,
        Parameters(PythonInput {
            project_dir,
            command,
            args,
        }): Parameters<PythonInput>,
    ) -> Result<CallToolResult, McpError> {
        let mut path = self.path.clone();
        path.push(project_dir);
        let result = compile_python_project(path, &command, &args);
        result
            .map(convert_docker_output_to_tool_result)
            .map_err(|e| McpError::internal_error(e.to_string(), None))
    }

    #[tool(description = "run Javascript code")]
    pub async fn run_javascript(
        &self,
        Parameters(JavascriptInput {
            project_dir,
            command,
            args,
        }): Parameters<JavascriptInput>,
    ) -> Result<CallToolResult, McpError> {
        let mut path = self.path.clone();
        path.push(project_dir);
        let result = compile_javascript_project(path, &command, &args);
        result
            .map(convert_docker_output_to_tool_result)
            .map_err(|e| McpError::internal_error(e.to_string(), None))
    }
    #[tool(description = "run Rust code")]
    pub async fn run_rust(
        &self,
        Parameters(RustInput {
            project_dir,
            command,
            args,
        }): Parameters<RustInput>,
    ) -> Result<CallToolResult, McpError> {
        let mut path = self.path.clone();
        path.push(project_dir);
        let result = compile_rust_project(path, &command, &args);
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
            //tool_router: Self::tool_router(),
            #[cfg(feature = "docker")]
            path: PathBuf::from(project_location),
        }
    }

    #[tool(description = "run Python code")]
    pub async fn run_python(
        &self,
        Parameters(PythonInput {
            project_dir,
            command,
            args,
        }): Parameters<PythonInput>,
    ) -> Result<CallToolResult, McpError> {
        let result = compile_python_project(project_dir, &command, &args);
        result
            .map(convert_docker_output_to_tool_result)
            .map_err(|e| McpError::internal_error(e.to_string(), None))
    }

    #[tool(description = "run Javascript code")]
    pub async fn run_javascript(
        &self,
        Parameters(JavascriptInput {
            project_dir,
            command,
            args,
        }): Parameters<JavascriptInput>,
    ) -> Result<CallToolResult, McpError> {
        let result = compile_javascript_project(project_dir, &command, &args);
        result
            .map(convert_docker_output_to_tool_result)
            .map_err(|e| McpError::internal_error(e.to_string(), None))
    }
    #[tool(description = "run Rust code")]
    pub async fn run_rust(
        &self,
        Parameters(RustInput {
            project_dir,
            command,
            args,
        }): Parameters<RustInput>,
    ) -> Result<CallToolResult, McpError> {
        let result = compile_rust_project(project_dir, &command, &args);
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
