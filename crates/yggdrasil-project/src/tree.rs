use ignore::WalkBuilder;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileNode {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub children: Vec<FileNode>,
}

impl FileNode {
    pub fn from_root<P: AsRef<Path>>(root: P) -> Self {
        let root = root.as_ref();
        let name = root
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "workspace".to_string());

        let mut root_node = FileNode {
            name,
            path: root.to_path_buf(),
            is_dir: true,
            children: Vec::new(),
        };

        // Populate direct children first for high-performance lazy expansion
        Self::populate_children(&mut root_node);
        root_node
    }

    pub fn populate_children(&mut self) {
        if !self.is_dir {
            return;
        }

        let mut children = Vec::new();
        let walker = WalkBuilder::new(&self.path)
            .max_depth(Some(1))
            .git_ignore(true)
            .hidden(true)
            .build();

        for result in walker.flatten() {
            let path = result.into_path();
            if path == self.path {
                continue;
            }

            let is_dir = path.is_dir();
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();

            children.push(FileNode {
                name,
                path,
                is_dir,
                children: Vec::new(),
            });
        }

        // Sort: directories first, then alphabetical
        children.sort_by(|a, b| {
            if a.is_dir == b.is_dir {
                a.name.to_lowercase().cmp(&b.name.to_lowercase())
            } else if a.is_dir {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Greater
            }
        });

        self.children = children;
    }
}
