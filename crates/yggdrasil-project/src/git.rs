use std::path::Path;

#[derive(Debug, Clone, Default)]
pub struct GitStatus {
    pub is_git_repo: bool,
    pub current_branch: Option<String>,
    pub dirty_files_count: usize,
}

impl GitStatus {
    pub fn check<P: AsRef<Path>>(root: P) -> Self {
        let root = root.as_ref();
        let git_dir = root.join(".git");
        if !git_dir.exists() {
            return Self::default();
        }

        let mut branch = None;
        let head_file = git_dir.join("HEAD");
        if let Ok(content) = std::fs::read_to_string(head_file) {
            let trimmed = content.trim();
            if let Some(stripped) = trimmed.strip_prefix("ref: refs/heads/") {
                branch = Some(stripped.to_string());
            } else {
                branch = Some("detached".to_string());
            }
        }

        Self {
            is_git_repo: true,
            current_branch: branch,
            dirty_files_count: 0,
        }
    }
}
