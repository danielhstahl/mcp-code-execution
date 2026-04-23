use crate::compilation_service::{CompileService, DockerOutput, LANGUAGE};
use crate::javascript::{DependencyType as JsDependencyType, JSService};
use crate::python::{DependencyType as PyDependencyType, PythonService};
use crate::rust::{ExecutionType, RustService};
use rmcp::{
    ErrorData as McpError, RoleServer, ServerHandler,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
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
pub struct PythonInput {
    #[schemars(
        description = "If using uv or requirements.txt.  If no dependencies, don't specify"
    )]
    pub dependency_type: Option<PyDependencyType>,
    #[schemars(
        description = "Path to the project.  This should be a folder/directory, not a file, and should be relative to the current working directory."
    )]
    pub project_dir: PathBuf,
    #[schemars(description = "File name within the project to execute.  Eg, `main.py`")]
    pub entry_file: PathBuf,
}

#[derive(Debug, schemars::JsonSchema, Deserialize)]
pub struct JavascriptInput {
    #[schemars(description = "If using npm or yarn.  If no dependencies, don't specify")]
    pub dependency_type: Option<JsDependencyType>,
    #[schemars(
        description = "Path to the project.  This should be a folder/directory, not a file, and should be relative to the current working directory."
    )]
    pub project_dir: PathBuf,
    #[schemars(description = "File name within the project to execute.  Eg, `index.js`")]
    pub entry_file: PathBuf,
}

#[derive(Debug, schemars::JsonSchema, Deserialize)]
pub struct RustInput {
    #[schemars(
        description = "If running tests or running the app.  Use \"run\" for executing code and \"test\" for testing code."
    )]
    pub execution_type: ExecutionType,
    #[schemars(
        description = "Path to the project.  This should be a folder/directory, not a file, and should be relative to the current working directory."
    )]
    pub project_dir: PathBuf,
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
    tool_router: ToolRouter<Self>,
    #[cfg(feature = "docker")]
    path: PathBuf,
}

#[cfg(feature = "docker")]
#[tool_router]
impl CodeCompiler {
    pub fn new(project_location: &str) -> Self {
        Self {
            tool_router: Self::tool_router(),
            path: PathBuf::from(project_location),
        }
    }

    #[tool(description = "run Python code")]
    pub async fn run_python(
        &self,
        Parameters(PythonInput {
            dependency_type,
            project_dir,
            entry_file,
        }): Parameters<PythonInput>,
    ) -> Result<CallToolResult, McpError> {
        let service = PythonService::new(dependency_type);
        let mut path = self.path.clone();
        path.push(project_dir);
        let result = service.compile_project(&path, &Some(entry_file));
        result
            .map(convert_docker_output_to_tool_result)
            .map_err(|e| McpError::internal_error(e.to_string(), None))
    }

    #[tool(description = "run Javascript code")]
    pub async fn run_javascript(
        &self,
        Parameters(JavascriptInput {
            dependency_type,
            project_dir,
            entry_file,
        }): Parameters<JavascriptInput>,
    ) -> Result<CallToolResult, McpError> {
        let service = JSService::new(dependency_type);
        let mut path = self.path.clone();
        path.push(project_dir);
        let result = service.compile_project(&path, &Some(entry_file));
        result
            .map(convert_docker_output_to_tool_result)
            .map_err(|e| McpError::internal_error(e.to_string(), None))
    }
    #[tool(description = "run Rust code")]
    pub async fn run_rust(
        &self,
        Parameters(RustInput {
            project_dir,
            execution_type,
        }): Parameters<RustInput>,
    ) -> Result<CallToolResult, McpError> {
        let service = RustService::new(execution_type);
        let mut path = self.path.clone();
        path.push(project_dir);
        let result = service.compile_project(&path, &None);
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
            tool_router: Self::tool_router(),
            #[cfg(feature = "docker")]
            path: PathBuf::from(project_location),
        }
    }

    #[tool(description = "run Python code")]
    pub async fn run_python(
        &self,
        Parameters(PythonInput {
            dependency_type,
            project_dir,
            entry_file,
        }): Parameters<PythonInput>,
    ) -> Result<CallToolResult, McpError> {
        let service = PythonService::new(dependency_type);
        let result = service.compile_project(&project_dir, &Some(entry_file));
        result
            .map(convert_docker_output_to_tool_result)
            .map_err(|e| McpError::internal_error(e.to_string(), None))
    }

    #[tool(description = "run Javascript code")]
    pub async fn run_javascript(
        &self,
        Parameters(JavascriptInput {
            dependency_type,
            #[cfg(not(feature = "docker"))]
            project_dir,
            entry_file,
        }): Parameters<JavascriptInput>,
    ) -> Result<CallToolResult, McpError> {
        let service = JSService::new(dependency_type);
        let result = service.compile_project(&project_dir, &Some(entry_file));
        result
            .map(convert_docker_output_to_tool_result)
            .map_err(|e| McpError::internal_error(e.to_string(), None))
    }
    #[tool(description = "run Rust code")]
    pub async fn run_rust(
        &self,
        Parameters(RustInput {
            execution_type,
            #[cfg(not(feature = "docker"))]
            project_dir,
        }): Parameters<RustInput>,
    ) -> Result<CallToolResult, McpError> {
        let service = RustService::new(execution_type);
        let result = service.compile_project(&project_dir, &None);
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
