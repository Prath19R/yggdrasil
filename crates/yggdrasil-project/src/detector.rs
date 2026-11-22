use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectType {
    Rust,
    NodeTypeScript,
    Python,
    Go,
    Cpp,
    Generic,
}

impl ProjectType {
    pub fn display_badge(&self) -> &'static str {
        match self {
            ProjectType::Rust => "🦀 Rust (Cargo)",
            ProjectType::NodeTypeScript => "📦 TypeScript/Node",
            ProjectType::Python => "🐍 Python",
            ProjectType::Go => "🐹 Go",
            ProjectType::Cpp => "⚡ C/C++",
            ProjectType::Generic => "📁 Generic Project",
        }
    }

    pub fn default_check_command(&self) -> Option<&'static str> {
        match self {
            ProjectType::Rust => Some("cargo check"),
            ProjectType::NodeTypeScript => Some("npm test"),
            ProjectType::Python => Some("pytest"),
            ProjectType::Go => Some("go test ./..."),
            ProjectType::Cpp => Some("cmake --build build"),
            ProjectType::Generic => None,
        }
    }
}

pub struct ProjectDetector;

impl ProjectDetector {
    pub fn detect<P: AsRef<Path>>(root: P) -> ProjectType {
        let root = root.as_ref();
        if root.join("Cargo.toml").exists() {
            ProjectType::Rust
        } else if root.join("package.json").exists() || root.join("tsconfig.json").exists() {
            ProjectType::NodeTypeScript
        } else if root.join("pyproject.toml").exists() || root.join("requirements.txt").exists() {
            ProjectType::Python
        } else if root.join("go.mod").exists() {
            ProjectType::Go
        } else if root.join("CMakeLists.txt").exists() || root.join("Makefile").exists() {
            ProjectType::Cpp
        } else {
            ProjectType::Generic
        }
    }
}
