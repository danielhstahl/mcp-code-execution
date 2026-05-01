use crate::constraints::{AllowedCommand, ArgConstraint};
use regex::Regex;
use std::sync::LazyLock;

pub struct CommandRegistry {
    pub npm: AllowedCommand,
    pub yarn: AllowedCommand,
    pub node: AllowedCommand,
    pub python: AllowedCommand,
    pub uv: AllowedCommand,
    pub pytest: AllowedCommand,
    pub cargo: AllowedCommand,
}

// Regex constants — compiled exactly once.
static SCRIPT_NAME: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[a-zA-Z0-9_:.-]+$").unwrap());

static FILE_NAME: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-zA-Z0-9_/.-]+\.(py|js|ts|rs)$").unwrap());

pub static REGISTRY: LazyLock<CommandRegistry> = LazyLock::new(|| CommandRegistry {
    npm: AllowedCommand {
        bin: "npm",
        arg_constraints: vec![
            ArgConstraint::Enum(&["install", "run", "test", "build", "ci"]),
            ArgConstraint::Optional(Box::new(ArgConstraint::Pattern(SCRIPT_NAME.clone()))),
        ],
    },
    yarn: AllowedCommand {
        bin: "yarn",
        arg_constraints: vec![
            ArgConstraint::Enum(&["install", "run", "test", "build"]),
            ArgConstraint::Optional(Box::new(ArgConstraint::Pattern(SCRIPT_NAME.clone()))),
        ],
    },
    node: AllowedCommand {
        bin: "node",
        arg_constraints: vec![ArgConstraint::WorkspacePath(Box::new(
            ArgConstraint::Pattern(FILE_NAME.clone()),
        ))],
    },
    python: AllowedCommand {
        bin: "python",
        arg_constraints: vec![ArgConstraint::WorkspacePath(Box::new(
            ArgConstraint::Pattern(FILE_NAME.clone()),
        ))],
    },
    uv: AllowedCommand {
        bin: "uv",
        arg_constraints: vec![
            ArgConstraint::Enum(&["run", "sync", "add", "pip"]),
            ArgConstraint::Optional(Box::new(ArgConstraint::Pattern(SCRIPT_NAME.clone()))),
        ],
    },
    pytest: AllowedCommand {
        bin: "pytest",
        arg_constraints: vec![ArgConstraint::Optional(Box::new(ArgConstraint::Pattern(
            SCRIPT_NAME.clone(),
        )))],
    },
    cargo: AllowedCommand {
        bin: "cargo",
        arg_constraints: vec![ArgConstraint::Enum(&[
            "build", "test", "run", "check", "clippy", "fmt",
        ])],
    },
});
