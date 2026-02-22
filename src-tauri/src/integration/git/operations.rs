//! Git Operations Module
//!
//! Actual Git operations using git2-rs.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

/// Get list of conflicted files
#[cfg(feature = "git")]
fn get_conflicts(repo: &git2::Repository) -> Result<Vec<String>, String> {
    let mut conflicts = Vec::new();

    // Use repo.statuses to detect conflicts instead
    let mut status_opts = git2::StatusOptions::new();
    status_opts.include_untracked(true);

    let statuses = repo.statuses(Some(&mut status_opts))
        .map_err(|e| format!("Failed to get status: {}", e))?;

    for entry in statuses.iter() {
        if entry.status().contains(git2::Status::CONFLICTED) {
            if let Some(path) = entry.path() {
                conflicts.push(path.to_string());
            }
        }
    }

    Ok(conflicts)
}

/// Perform a merge with conflict detection
#[cfg(feature = "git")]
fn perform_merge(
    repo: &git2::Repository,
    fetch_commit: &git2::Commit,
) -> Result<GitOperationResult, String> {
    use std::time::Instant;

    let start = Instant::now();

    let fetch_annotated = repo.find_annotated_commit(fetch_commit.id())
        .map_err(|e| format!("Failed to annotate fetched commit: {}", e))?;

    let head_commit = repo.head()
        .and_then(|h| h.peel_to_commit())
        .map_err(|e| format!("Failed to get HEAD commit: {}", e))?;

    let signature = repo.signature()
        .map_err(|e| format!("Failed to get signature: {}", e))?;

    let msg = format!("Merge commit {}", fetch_commit.id());

    let mut merge_options = git2::MergeOptions::new();
    merge_options.fail_on_conflict(true);

    let mut checkout_builder = git2::build::CheckoutBuilder::new();

    match repo.merge(
        &[&fetch_annotated],
        Some(&mut merge_options),
        Some(&mut checkout_builder),
    ) {
        Ok(_) => {
            let mut index = repo.index()
                .map_err(|e| format!("Failed to get index: {}", e))?;

            if index.has_conflicts() {
                // Clean up merge state
                repo.cleanup_state()
                    .map_err(|e| format!("Failed to cleanup merge: {}", e))?;

                let conflicts = get_conflicts(repo)?;

                return Ok(GitOperationResult {
                    success: false,
                    result: Some("Merge conflicts detected".to_string()),
                    error: Some(format!("{} files have conflicts:\n{}", conflicts.len(), conflicts.join("\n"))),
                    execution_time_ms: start.elapsed().as_millis() as u64,
                });
            }

            let tree_id = index.write_tree()
                .map_err(|e| format!("Failed to write tree: {}", e))?;

            let tree = repo.find_tree(tree_id)
                .map_err(|e| format!("Failed to find tree: {}", e))?;

            let oid = repo.commit(
                Some("HEAD"),
                &signature,
                &signature,
                &msg,
                &tree,
                &[&head_commit, fetch_commit],
            )
                .map_err(|e| format!("Failed to create merge commit: {}", e))?;

            // Clean up merge state
            repo.cleanup_state()
                .map_err(|e| format!("Failed to cleanup merge: {}", e))?;

            Ok(GitOperationResult {
                success: true,
                result: Some(format!("Merged commit {} successfully", oid)),
                error: None,
                execution_time_ms: start.elapsed().as_millis() as u64,
            })
        }
        Err(e) => {
            if e.class() == git2::ErrorClass::Merge {
                // Clean up merge state
                repo.cleanup_state()
                    .map_err(|e2| format!("Failed to cleanup merge: {}", e2))?;

                Ok(GitOperationResult {
                    success: false,
                    result: Some("Merge conflicts detected".to_string()),
                    error: Some(format!("Automatic merge failed: {}", e)),
                    execution_time_ms: start.elapsed().as_millis() as u64,
                })
            } else {
                Err(format!("Merge failed: {}", e))
            }
        }
    }
}

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
        use std::path::Path;

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
        let branch = head.as_ref().and_then(|h| h.shorthand()).map(|s| s.to_string());

        // Count changes
        let mut status_opts = git2::StatusOptions::new();
        status_opts.include_untracked(true);

        let statuses = repo.statuses(Some(&mut status_opts))
            .map_err(|e| format!("Failed to get status: {}", e))?;

        let (staged, unstaged, untracked, conflicted) = statuses.iter().fold(
            (0, 0, 0, 0),
            |(s, u, ut, c), entry| {
                let status = entry.status();
                let s = s + if status.contains(git2::Status::INDEX_NEW) || status.contains(git2::Status::INDEX_MODIFIED) { 1 } else { 0 };
                let u = u + if status.contains(git2::Status::WT_NEW) || status.contains(git2::Status::WT_MODIFIED) { 1 } else { 0 };
                let ut = ut + if status.contains(git2::Status::WT_NEW) { 1 } else { 0 };
                let c = c + if status.contains(git2::Status::CONFLICTED) { 1 } else { 0 };
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
        let pathspec = ".";
        index.update_all([pathspec], None)
            .map_err(|e| format!("Failed to stage files: {}", e))?;

        let tree_id = index.write_tree()
            .map_err(|e| format!("Failed to write tree: {}", e))?;

        let tree = repo.find_tree(tree_id)
            .map_err(|e| format!("Failed to find tree: {}", e))?;

        let sig = repo.signature()
            .map_err(|e| format!("Failed to get signature: {}", e))?;

        // Get parent commit if exists
        let parent_commit = repo.head()
            .ok()
            .and_then(|h| h.peel_to_commit().ok());

        // git2 0.19 requires 6 arguments: update_ref, author, committer, message, tree, parents
        let oid = repo.commit(
            Some("HEAD"),
            &sig,
            &sig,
            message,
            &tree,
            parent_commit.as_ref().into_iter().collect::<Vec<_>>().as_slice(),
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
        let mut repo_remote = repo.find_remote(remote)
            .map_err(|_| format!("Remote '{}' not found", remote))?;

        // Get the refspec for the branch
        let refspec = format!("refs/heads/{}:refs/heads/{}", branch, branch);

        // Configure authentication callbacks
        let _config = repo.config().ok();
        let mut callbacks = git2::RemoteCallbacks::new();

        // Try to use credentials from config or ssh agent
        callbacks.credentials(|_url, username_from_url, _allowed| {
            git2::Cred::ssh_key_from_agent(
                username_from_url.unwrap_or("git"),
            )
        });

        // Also try helper-based authentication
        callbacks.credentials(|_url, username_from_url, allowed| {
            if allowed.contains(git2::CredentialType::SSH_KEY) {
                git2::Cred::ssh_key_from_agent(
                    username_from_url.unwrap_or("git"),
                )
            } else if allowed.contains(git2::CredentialType::USER_PASS_PLAINTEXT) {
                // For HTTPS, try credential helper
                git2::Cred::default()
            } else {
                Err(git2::Error::from_str("Unsupported authentication type"))
            }
        });

        let mut push_options = git2::PushOptions::new();
        push_options.remote_callbacks(callbacks);

        // Push
        match repo_remote.push(&[refspec.as_str()], Some(&mut push_options)) {
            Ok(_) => Ok(GitOperationResult {
                success: true,
                result: Some(format!("Pushed to {}/{}", remote, branch)),
                error: None,
                execution_time_ms: start.elapsed().as_millis() as u64,
            }),
            Err(e) => Ok(GitOperationResult {
                success: false,
                result: None,
                error: Some(format!("Push failed: {}", e)),
                execution_time_ms: start.elapsed().as_millis() as u64,
            }),
        }
    }

    /// Pull changes with merge support
    #[cfg(feature = "git")]
    pub fn pull(&self, remote: &str, branch: &str) -> Result<GitOperationResult, String> {
        use std::time::Instant;

        let start = Instant::now();

        let repo = git2::Repository::open(&self.repo_path)
            .map_err(|e| format!("Failed to open repo: {}", e))?;

        // Find the remote
        let mut repo_remote = repo.find_remote(remote)
            .map_err(|_| format!("Remote '{}' not found", remote))?;

        // Configure authentication callbacks
        let mut callbacks = git2::RemoteCallbacks::new();

        callbacks.credentials(|_url, username_from_url, allowed| {
            if allowed.contains(git2::CredentialType::SSH_KEY) {
                git2::Cred::ssh_key_from_agent(
                    username_from_url.unwrap_or("git"),
                )
            } else if allowed.contains(git2::CredentialType::USER_PASS_PLAINTEXT) {
                git2::Cred::default()
            } else {
                Err(git2::Error::from_str("Unsupported authentication type"))
            }
        });

        let mut fetch_options = git2::FetchOptions::new();
        fetch_options.remote_callbacks(callbacks);

        // Fetch from remote
        let refspec = format!("refs/heads/{}:refs/heads/{}", branch, branch);
        match repo_remote.fetch(&[&refspec], Some(&mut fetch_options), None) {
            Ok(_) => {
                // Try to fast-forward merge
                let fetch_commit = repo.find_reference("FETCH_HEAD")
                    .and_then(|r| r.peel_to_commit());

                let head_commit = repo.head()
                    .ok()
                    .and_then(|h| h.peel_to_commit().ok());

                match (head_commit, fetch_commit) {
                    (_head, Ok(fetch)) => {
                        // Try fast-forward merge
                        let annotated_fetch = repo.find_annotated_commit(fetch.id())
                            .map_err(|e| format!("Failed to annotate commit: {}", e))?;

                        match repo.merge_analysis(&[&annotated_fetch]) {
                            Ok((analysis, _)) => {
                                if analysis.is_up_to_date() {
                                    return Ok(GitOperationResult {
                                        success: true,
                                        result: Some(format!("Already up to date with {}/{}", remote, branch)),
                                        error: None,
                                        execution_time_ms: start.elapsed().as_millis() as u64,
                                    });
                                }

                                if analysis.is_fast_forward() {
                                    let refname = format!("refs/heads/{}", branch);
                                    repo.reference(
                                        &refname,
                                        fetch.id(),
                                        true,
                                        "Fast-forward pull",
                                    )
                                        .map_err(|e| format!("Failed to update reference: {}", e))?;

                                    repo.checkout_head(Some(git2::build::CheckoutBuilder::new().force()))
                                        .map_err(|e| format!("Failed to checkout: {}", e))?;

                                    Ok(GitOperationResult {
                                        success: true,
                                        result: Some(format!("Fast-forward pull from {}/{}", remote, branch)),
                                        error: None,
                                        execution_time_ms: start.elapsed().as_millis() as u64,
                                    })
                                } else {
                                    // Normal merge required
                                    match perform_merge(&repo, &fetch) {
                                        Ok(merge_result) => Ok(merge_result),
                                        Err(e) => Ok(GitOperationResult {
                                            success: false,
                                            result: Some(format!("Fetched from {}/{}", remote, branch)),
                                            error: Some(format!("Merge failed: {}", e)),
                                            execution_time_ms: start.elapsed().as_millis() as u64,
                                        }),
                                    }
                                }
                            }
                            Err(e) => Ok(GitOperationResult {
                                success: false,
                                result: None,
                                error: Some(format!("Merge analysis failed: {}", e)),
                                execution_time_ms: start.elapsed().as_millis() as u64,
                            }),
                        }
                    }
                    _ => Ok(GitOperationResult {
                        success: true,
                        result: Some(format!("Fetched from {}/{}", remote, branch)),
                        error: None,
                        execution_time_ms: start.elapsed().as_millis() as u64,
                    }),
                }
            }
            Err(e) => Ok(GitOperationResult {
                success: false,
                result: None,
                error: Some(format!("Fetch failed: {}", e)),
                execution_time_ms: start.elapsed().as_millis() as u64,
            }),
        }
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
