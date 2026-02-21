// Git Integration - Git repository operations

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Git configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitConfig {
    pub name: String,
    pub repository_path: PathBuf,
    pub user_name: Option<String>,
    pub user_email: Option<String>,
    pub branch: Option<String>,
    pub remote_url: Option<String>,
}

impl GitConfig {
    /// Validate git configuration
    pub fn validate(&self) -> Result<(), String> {
        if !self.repository_path.exists() {
            return Err(format!("Repository path does not exist: {:?}", self.repository_path));
        }
        if !self.repository_path.join(".git").exists() {
            return Err("Not a valid git repository (missing .git folder)".to_string());
        }
        Ok(())
    }

    /// Get repository status
    pub fn get_status(&self) -> Result<GitStatus, String> {
        self.validate()?;

        // In production, this would use git2 or similar
        Ok(GitStatus {
            branch: self.branch.clone().unwrap_or("main".to_string()),
            commits_ahead: 0,
            commits_behind: 0,
            staged_files: vec![],
            modified_files: vec![],
            untracked_files: vec![],
        })
    }

    /// Get current commit
    pub fn get_current_commit(&self) -> Result<GitCommit, String> {
        self.validate()?;

        Ok(GitCommit {
            hash: "abc123".to_string(),
            author: self.user_name.clone().unwrap_or("Unknown".to_string()),
            message: "Latest commit".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        })
    }
}

/// Git repository status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitStatus {
    pub branch: String,
    pub commits_ahead: usize,
    pub commits_behind: usize,
    pub staged_files: Vec<String>,
    pub modified_files: Vec<String>,
    pub untracked_files: Vec<String>,
}

/// Git commit info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitCommit {
    pub hash: String,
    pub author: String,
    pub message: String,
    pub timestamp: String,
}

#[tauri::command]
pub fn validate_git_repository(path: String) -> Result<GitStatus, String> {
    let config = GitConfig {
        name: "temp".to_string(),
        repository_path: PathBuf::from(path),
        user_name: None,
        user_email: None,
        branch: None,
        remote_url: None,
    };
    config.get_status()
}

#[tauri::command]
pub fn get_git_status(config: GitConfig) -> Result<GitStatus, String> {
    config.get_status()
}

#[tauri::command]
pub fn get_git_current_commit(config: GitConfig) -> Result<GitCommit, String> {
    config.get_current_commit()
}
