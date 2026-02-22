//! Git Operations Module
//!
//! Public API for Git operations using git2-rs.

pub mod operations;

use serde::{Deserialize, Serialize};

/// Git repository configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitRepositoryConfig {
    pub name: String,
    pub path: String,
    pub remote_url: Option<String>,
    pub branch: Option<String>,
    pub username: Option<String>,
    pub email: Option<String>,
}

/// Git status for UI display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitStatus {
    pub path: String,
    pub branch: Option<String>,
    pub has_changes: bool,
    pub ahead: usize,
    pub behind: usize,
}

impl GitRepositoryConfig {
    /// Validate git repository configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.path.is_empty() {
            return Err("Repository path is required".to_string());
        }
        let path = std::path::Path::new(&self.path);
        if !path.exists() {
            return Err(format!("Path does not exist: {}", self.path));
        }
        Ok(())
    }
}

// ============================================================================
// Legacy Tauri Commands (for compatibility with existing UI)
// ============================================================================

/// Validate git repository (legacy command)
#[tauri::command]
pub fn validate_git_repository(path: String) -> Result<GitStatus, String> {
    let config = GitRepositoryConfig {
        name: "default".to_string(),
        path: path.clone(),
        remote_url: None,
        branch: None,
        username: None,
        email: None,
    };
    config.validate()?;

    #[cfg(feature = "git")]
    {
        let ops = GitOperations::open(&path)?;
        let status = ops.get_status()?;
        Ok(GitStatus {
            path,
            branch: status.branch,
            has_changes: status.staged > 0 || status.unstaged > 0 || status.untracked > 0,
            ahead: status.ahead,
            behind: status.behind,
        })
    }

    #[cfg(not(feature = "git"))]
    {
        Ok(GitStatus {
            path,
            branch: None,
            has_changes: false,
            ahead: 0,
            behind: 0,
        })
    }
}

/// Get git status (legacy command)
#[tauri::command]
pub fn get_git_status(path: String) -> Result<GitStatus, String> {
    validate_git_repository(path)
}

/// Get current git commit (legacy command)
#[tauri::command]
pub fn get_git_current_commit(path: String) -> Result<String, String> {
    #[cfg(feature = "git")]
    {
        let repo = git2::Repository::open(std::path::PathBuf::from(&path))
            .map_err(|e| format!("Failed to open repo: {}", e))?;
        let head = repo.head()
            .map_err(|e| format!("Failed to get HEAD: {}", e))?;
        let commit = head.peel_to_commit()
            .map_err(|e| format!("Failed to get commit: {}", e))?;
        Ok(commit.id().to_string())
    }

    #[cfg(not(feature = "git"))]
    {
        Ok("Git feature not enabled".to_string())
    }
}
