use serde::Serialize;
use std::io;
use std::path::PathBuf;
pub const LANGUAGE: &'static [&str] = &["rust", "javascript", "python"];

//#[allow(dead_code)]
pub trait CompileService: Send + Sync + 'static {
    //fn set_language(&self, lang: Language) -> String;
    //fn compile(&self, data: String); //intended for single "bits" of code rather than full projects
    fn compile_project(
        &self,
        path: &PathBuf,
        main_file: &Option<PathBuf>,
    ) -> io::Result<DockerOutput>; //for running this mcp server locally
}

#[derive(Serialize)]
pub struct DockerOutput {
    stdout: String,
    stderr: String,
    pub is_error: bool,
}

impl DockerOutput {
    pub fn new(stdout: String, stderr: String, is_error: bool) -> Self {
        Self {
            stdout,
            stderr,
            is_error,
        }
    }
}
