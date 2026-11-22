use ropey::Rope;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct EditorBuffer {
    pub id: String,
    pub path: Option<PathBuf>,
    pub file_name: String,
    pub rope: Rope,
    pub is_dirty: bool,
    pub cursor_line: usize,
    pub cursor_col: usize,
}

impl EditorBuffer {
    pub fn new_empty(id: String, name: String) -> Self {
        Self {
            id,
            path: None,
            file_name: name,
            rope: Rope::new(),
            is_dirty: false,
            cursor_line: 0,
            cursor_col: 0,
        }
    }

    pub fn from_file<P: AsRef<Path>>(id: String, path: P) -> std::io::Result<Self> {
        let content = std::fs::read_to_string(&path)?;
        let file_name = path
            .as_ref()
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| "untitled".to_string());

        Ok(Self {
            id,
            path: Some(path.as_ref().to_path_buf()),
            file_name,
            rope: Rope::from_str(&content),
            is_dirty: false,
            cursor_line: 0,
            cursor_col: 0,
        })
    }

    pub fn text(&self) -> String {
        self.rope.to_string()
    }

    pub fn set_text(&mut self, text: &str) {
        self.rope = Rope::from_str(text);
        self.is_dirty = true;
    }

    pub fn line_count(&self) -> usize {
        self.rope.len_lines()
    }

    pub fn save(&mut self) -> std::io::Result<()> {
        if let Some(path) = &self.path {
            std::fs::write(path, self.text())?;
            self.is_dirty = false;
        }
        Ok(())
    }
}
