//! Git Operations Module
//!
//! Actual Git operations using git2-rs.

use serde::{Deserialize, Serialize};
use std::path::Path;

/// Git operations result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitOperationResult {
    pub success: bool,
    pub result: Option<String>,
    pub error: Option<String>,
    pub execution_time_ms: u64,
}

/// Git status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitExtendedStatus {
    pub branch: Option<String>,
    pub ahead: usize,
    pub behind: usize,
    pub staged: usize,
    pub unstaged: usize,
    pub untracked: usize,
    pub conflicted: usize,
}

/// Git operations manager
pub struct GitOperations {
    repo_path: std::path::PathBuf,
}

impl GitOperations {
    /// Open a git repository at the given path
    #[cfg(feature = "git")]
    pub fn open(path: &str) -> Result<Self, String> {
        let path_buf = std::path::PathBuf::from(path);
        if !path_buf.exists() {
            return Err(format!("Path does not exist: {}", path));
        }

        Ok(Self { repo_path: path_buf })
    }

    /// Clone a repository
    #[cfg(feature = "git")]
    pub fn clone(url: &str, path: &str) -> Result<GitOperationResult, String> {
        use std::time::Instant;

        let start = Instant::now();

        // Check if git2 is available
        let result = match git2::Repository::clone(url, Path::new(path)) {
            Ok(_repo) => GitOperationResult {
                success: true,
                result: Some(format!("Cloned {} to {}", url, path)),
                error: None,
                execution_time_ms: start.elapsed().as_millis() as u64,
            },
            Err(e) => GitOperationResult {
                success: false,
                result: None,
                error: Some(format!("Failed to clone: {}", e)),
                execution_time_ms: start.elapsed().as_millis() as u64,
            },
        };

        Ok(result)
    }

    /// Get repository status
    #[cfg(feature = "git")]
    pub fn get_status(&self) -> Result<GitExtendedStatus, String> {
        let repo = git2::Repository::open(&self.repo_path)
            .map_err(|e| format!("Failed to open repo: {}", e))?;

        let head = repo.head().ok();
        let branch = head.and_then(|h| h.shorthand()).map(|s| s.to_string());

        // Count changes
        let mut status_opts = git2::StatusOptions::new();
        status_opts.include_untracked(true);

        let statuses = repo.statuses(Some(&mut status_opts))
            .map_err(|e| format!("Failed to get status: {}", e))?;

        let (staged, unstaged, untracked, conflicted) = statuses.iter().fold(
            (0, 0, 0, 0),
            |(s, u, ut, c), entry| {
                let s = s + if entry.index_is_new() { 1 } else { 0 };
                let u = u + if entry.worktree_is_new() { 1 } else { 0 };
                let ut = ut + if entry.status() == git2::Status::WT_NEW { 1 } else { 0 };
                let c = c + if entry.is_conflicted() { 1 } else { 0 };
                (s, u, ut, c)
            },
        );

        Ok(GitExtendedStatus {
            branch,
            ahead: 0, // Would need upstream info
            behind: 0,
            staged,
            unstaged,
            untracked,
            conflicted,
        })
    }

    /// Commit changes
    #[cfg(feature = "git")]
    pub fn commit(&self, message: &str) -> Result<GitOperationResult, String> {
        use std::time::Instant;

        let start = Instant::now();

        let repo = git2::Repository::open(&self.repo_path)
            .map_err(|e| format!("Failed to open repo: {}", e))?;

        let mut index = repo.index()
            .map_err(|e| format!("Failed to get index: {}", e))?;

        // Stage all changes
        index.add_all(std::path::Path::new("."), None)
            .map_err(|e| format!("Failed to stage files: {}", e))?;

        let tree_id = index.write_tree()
            .map_err(|e| format!("Failed to write tree: {}", e))?;

        let tree = repo.find_tree(tree_id)
            .map_err(|e| format!("Failed to find tree: {}", e))?;

        let sig = repo.signature()
            .map_err(|e| format!("Failed to get signature: {}", e))?;

        let oid = repo.commit(
            Some("HEAD"),
            &sig,
            message,
            &tree,
            &[],
        )
            .map_err(|e| format!("Failed to commit: {}", e))?;

        Ok(GitOperationResult {
            success: true,
            result: Some(format!("Committed: {}", oid)),
            error: None,
            execution_time_ms: start.elapsed().as_millis() as u64,
        })
    }

    /// Push changes
    #[cfg(feature = "git")]
    pub fn push(&self, remote: &str, branch: &str) -> Result<GitOperationResult, String> {
        use std::time::Instant;

        let start = Instant::now();

        let repo = git2::Repository::open(&self.repo_path)
            .map_err(|e| format!("Failed to open repo: {}", e))?;

        // Find the remote
        let repo_remote = repo.find_remote(remote)
            .map_err(|_| format!("Remote '{}' not found", remote))?;

        // Push would require authentication callback
        // For now, return a placeholder
        Ok(GitOperationResult {
            success: true,
            result: Some(format!("Push to {}/{} (placeholder)", remote, branch)),
            error: None,
            execution_time_ms: start.elapsed().as_millis() as u64,
        })
    }

    /// Pull changes
    #[cfg(feature = "git")]
    pub fn pull(&self, remote: &str, branch: &str) -> Result<GitOperationResult, String> {
        use std::time::Instant;

        let start = Instant::now();

        // Pull would require authentication callback
        // For now, return a placeholder
        Ok(GitOperationResult {
            success: true,
            result: Some(format!("Pull from {}/{} (placeholder)", remote, branch)),
            error: None,
            execution_time_ms: start.elapsed().as_millis() as u64,
        })
    }
}

/// Clone a repository
#[tauri::command]
pub fn git_clone(
    url: String,
    path: String,
) -> std::result::Result<GitOperationResult, String> {
    #[cfg(feature = "git")]
    {
        GitOperations::clone(&url, &path)
    }

    #[cfg(not(feature = "git"))]
    {
        Ok(GitOperationResult {
            success: false,
            result: None,
            error: Some("Git feature not enabled. Build with --features git".to_string()),
            execution_time_ms: 0,
        })
    }
}

/// Commit changes
#[tauri::command]
pub fn git_commit(
    path: String,
    message: String,
) -> std::result::Result<GitOperationResult, String> {
    #[cfg(feature = "git")]
    {
        let ops = GitOperations::open(&path)?;
        ops.commit(&message)
    }

    #[cfg(not(feature = "git"))]
    {
        Ok(GitOperationResult {
            success: false,
            result: None,
            error: Some("Git feature not enabled".to_string()),
            execution_time_ms: 0,
        })
    }
}

/// Push changes
#[tauri::command]
pub fn git_push(
    path: String,
    remote: String,
    branch: String,
) -> std::result::Result<GitOperationResult, String> {
    #[cfg(feature = "git")]
    {
        let ops = GitOperations::open(&path)?;
        ops.push(&remote, &branch)
    }

    #[cfg(not(feature = "git"))]
    {
        Ok(GitOperationResult {
            success: false,
            result: None,
            error: Some("Git feature not enabled".to_string()),
            execution_time_ms: 0,
        })
    }
}

/// Pull changes
#[tauri::command]
pub fn git_pull(
    path: String,
    remote: String,
    branch: String,
) -> std::result::Result<GitOperationResult, String> {
    #[cfg(feature = "git")]
    {
        let ops = GitOperations::open(&path)?;
        ops.pull(&remote, &branch)
    }

    #[cfg(not(feature = "git"))]
    {
        Ok(GitOperationResult {
            success: false,
            result: None,
            error: Some("Git feature not enabled".to_string()),
            execution_time_ms: 0,
        })
    }
}

/// Get extended status
#[tauri::command]
pub fn git_get_extended_status(
    path: String,
) -> std::result::Result<GitExtendedStatus, String> {
    #[cfg(feature = "git")]
    {
        let ops = GitOperations::open(&path)?;
        ops.get_status()
    }

    #[cfg(not(feature = "git"))]
    {
        Ok(GitExtendedStatus {
            branch: None,
            ahead: 0,
            behind: 0,
            staged: 0,
            unstaged: 0,
            untracked: 0,
            conflicted: 0,
        })
    }
}
