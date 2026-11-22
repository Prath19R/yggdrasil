use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileKind {
    Text,
    Image,
    Video,
    Document,
    Binary,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FileItem {
    pub id: String,
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub kind: FileKind,
    pub content: String,
    pub size_bytes: u64,
    pub is_expanded: bool,
    pub children: Vec<FileItem>,
}

impl FileItem {
    pub fn new_file(name: &str, path: &str, content: &str) -> Self {
        let kind = Self::detect_kind(name);
        Self {
            id: path.to_string(),
            name: name.to_string(),
            path: path.to_string(),
            is_dir: false,
            kind,
            content: content.to_string(),
            size_bytes: content.len() as u64,
            is_expanded: false,
            children: Vec::new(),
        }
    }

    pub fn new_dir(name: &str, path: &str, children: Vec<FileItem>) -> Self {
        Self {
            id: path.to_string(),
            name: name.to_string(),
            path: path.to_string(),
            is_dir: true,
            kind: FileKind::Document,
            content: String::new(),
            size_bytes: 0,
            is_expanded: true,
            children,
        }
    }

    pub fn detect_kind(name: &str) -> FileKind {
        let lower = name.to_lowercase();
        if lower.ends_with(".png") || lower.ends_with(".jpg") || lower.ends_with(".jpeg") || lower.ends_with(".gif") || lower.ends_with(".webp") || lower.ends_with(".svg") {
            FileKind::Image
        } else if lower.ends_with(".mp4") || lower.ends_with(".webm") || lower.ends_with(".mov") || lower.ends_with(".mkv") {
            FileKind::Video
        } else if lower.ends_with(".pdf") || lower.ends_with(".doc") || lower.ends_with(".docx") {
            FileKind::Document
        } else if lower.ends_with(".zip") || lower.ends_with(".wasm") || lower.ends_with(".exe") {
            FileKind::Binary
        } else {
            FileKind::Text
        }
    }

    pub fn icon(&self) -> &'static str {
        if self.is_dir {
            if self.is_expanded { "📂" } else { "📁" }
        } else {
            let lower = self.name.to_lowercase();
            if lower.ends_with(".rs") { "🦀" }
            else if lower.ends_with(".ts") || lower.ends_with(".tsx") || lower.ends_with(".js") || lower.ends_with(".jsx") { "📦" }
            else if lower.ends_with(".py") { "🐍" }
            else if lower.ends_with(".toml") || lower.ends_with(".json") || lower.ends_with(".yaml") || lower.ends_with(".yml") { "⚙️" }
            else if lower.ends_with(".md") { "📝" }
            else if lower.ends_with(".png") || lower.ends_with(".jpg") || lower.ends_with(".jpeg") { "🖼️" }
            else if lower.ends_with(".mp4") || lower.ends_with(".webm") { "🎬" }
            else if lower.ends_with(".pdf") { "📑" }
            else { "📄" }
        }
    }
}
