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
        Self::build_recursive(root, 8)
    }

    pub fn build_recursive<P: AsRef<Path>>(root: P, max_depth: usize) -> Self {
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

        Self::populate_recursive(&mut root_node, 1, max_depth);
        root_node
    }

    fn populate_recursive(node: &mut FileNode, current_depth: usize, max_depth: usize) {
        if !node.is_dir || current_depth > max_depth {
            return;
        }

        let mut children = Vec::new();
        let walker = WalkBuilder::new(&node.path)
            .max_depth(Some(1))
            .git_ignore(true)
            .hidden(true)
            .build();

        for result in walker.flatten() {
            let path = result.into_path();
            if path == node.path {
                continue;
            }

            let is_dir = path.is_dir();
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();

            // Skip heavy build/cache folders even if no .gitignore present
            if is_dir && (name == "target" || name == "node_modules" || name == ".git" || name == ".next") {
                continue;
            }

            let mut child = FileNode {
                name,
                path,
                is_dir,
                children: Vec::new(),
            };

            if is_dir && current_depth < max_depth {
                Self::populate_recursive(&mut child, current_depth + 1, max_depth);
            }

            children.push(child);
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

        node.children = children;
    }
}
