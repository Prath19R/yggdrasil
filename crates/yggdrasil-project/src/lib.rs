pub mod detector;
pub mod tree;
pub mod git;
pub mod manager;

pub use detector::{ProjectDetector, ProjectType};
pub use tree::FileNode;
pub use git::GitStatus;
pub use manager::ProjectManager;
