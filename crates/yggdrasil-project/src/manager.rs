use crate::detector::{ProjectDetector, ProjectType};
use crate::git::GitStatus;
use crate::tree::FileNode;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct ProjectManager {
    pub root_path: Option<PathBuf>,
    pub project_type: ProjectType,
    pub root_node: Option<FileNode>,
    pub git_status: GitStatus,
    pub recent_projects: Vec<PathBuf>,
}

impl Default for ProjectManager {
    fn default() -> Self {
        Self {
            root_path: None,
            project_type: ProjectType::Generic,
            root_node: None,
            git_status: GitStatus::default(),
            recent_projects: Vec::new(),
        }
    }
}

impl ProjectManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn open_project<P: AsRef<Path>>(&mut self, path: P) -> Result<(), String> {
        let path = path.as_ref().to_path_buf();
        if !path.exists() || !path.is_dir() {
            return Err("Target path does not exist or is not a directory".into());
        }

        self.project_type = ProjectDetector::detect(&path);
        self.git_status = GitStatus::check(&path);
        self.root_node = Some(FileNode::from_root(&path));
        
        if !self.recent_projects.contains(&path) {
            self.recent_projects.insert(0, path.clone());
            if self.recent_projects.len() > 10 {
                self.recent_projects.pop();
            }
        }

        self.root_path = Some(path);
        Ok(())
    }

    pub fn open_via_dialog(&mut self) -> Result<Option<PathBuf>, String> {
        if let Some(folder) = rfd::FileDialog::new().pick_folder() {
            self.open_project(&folder)?;
            Ok(Some(folder))
        } else {
            Ok(None)
        }
    }

    pub fn read_file<P: AsRef<Path>>(&self, relative_or_abs: P) -> std::io::Result<String> {
        let full_path = if relative_or_abs.as_ref().is_absolute() {
            relative_or_abs.as_ref().to_path_buf()
        } else if let Some(root) = &self.root_path {
            root.join(relative_or_abs)
        } else {
            relative_or_abs.as_ref().to_path_buf()
        };
        std::fs::read_to_string(full_path)
    }

    pub fn write_file<P: AsRef<Path>>(&self, relative_or_abs: P, content: &str) -> std::io::Result<()> {
        let full_path = if relative_or_abs.as_ref().is_absolute() {
            relative_or_abs.as_ref().to_path_buf()
        } else if let Some(root) = &self.root_path {
            root.join(relative_or_abs)
        } else {
            relative_or_abs.as_ref().to_path_buf()
        };

        if let Some(parent) = full_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(full_path, content)
    }
}
