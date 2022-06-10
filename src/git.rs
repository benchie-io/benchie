use crate::value;
use anyhow::{anyhow, ensure, Context, Result};
use git2::{BranchType, Commit, Repository, StatusOptions, Statuses};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitInfo {
    #[serde(with = "value")]
    pub commit_id: String,

    #[serde(with = "value")]
    pub commit_message: String,

    #[serde(with = "value", skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,

    #[serde(with = "value")]
    pub is_dirty: bool,

    #[serde(skip)]
    pub path: PathBuf,
}

#[derive(Error, Debug)]
pub enum GitError {
    #[error("no Git repository found in current directory")]
    NotFound,
    #[error("Git repository does not have a commit yet")]
    NoCommit,
    #[error("unknown error occurred during Git repository discovery")]
    Unknown(#[from] anyhow::Error),
}

pub fn read_git_info() -> Result<GitInfo, GitError> {
    let repo = discover_repository()?;

    let path = repository_path(&repo)?;
    let branch = read_current_branch(&repo)?;
    let is_dirty = is_dirty(&repo)?;

    let (commit_id, commit_message) = read_head_commit(&repo).map(commit_to_details)?;

    Ok(GitInfo {
        commit_id,
        commit_message,
        branch,
        is_dirty,
        path,
    })
}

fn discover_repository() -> Result<Repository, GitError> {
    Repository::discover(".").map_err(|error| {
        if error.code() == git2::ErrorCode::NotFound {
            GitError::NotFound
        } else {
            GitError::Unknown(anyhow!(error))
        }
    })
}

fn repository_path(repo: &Repository) -> Result<PathBuf, GitError> {
    repo.path()
        .parent()
        .map(|p| p.to_path_buf())
        .ok_or(GitError::NotFound)
}

fn read_current_branch(repo: &Repository) -> Result<Option<String>, GitError> {
    let branches = repo
        .branches(Some(BranchType::Local))
        .context("failed to read local Git branches from repository")?;

    let branch_name = branches
        .into_iter()
        .filter_map(|r| match r {
            Ok((branch, _)) => {
                if branch.is_head() {
                    branch.name().ok().and_then(|v| v.map(|s| s.to_owned()))
                } else {
                    None
                }
            }
            Err(_) => None,
        })
        .next();

    Ok(branch_name)
}

fn read_head_commit(repo: &Repository) -> Result<Commit, GitError> {
    repo.head()
        .and_then(|h| h.peel_to_commit())
        .map_err(|_| GitError::NoCommit)
}

fn commit_to_details(commit: Commit) -> (String, String) {
    let commit_id = commit.id().to_string();
    let first_line = commit
        .message()
        .and_then(|msg| msg.split('\n').next())
        .unwrap_or("")
        .to_owned();

    (commit_id, first_line)
}

fn is_dirty(repo: &Repository) -> Result<bool> {
    ensure!(!repo.is_bare(), "cannot report status on bare repository");

    let mut opt = StatusOptions::new();
    opt.include_untracked(true);
    let statuses = repo.statuses(Some(&mut opt))?;

    let staged = staged_files(&statuses);
    let unstaged = unstaged_files(&statuses);

    Ok(!staged.is_empty() || !unstaged.is_empty())
}

fn staged_files(statuses: &Statuses) -> Vec<PathBuf> {
    statuses
        .iter()
        .filter(|entry| {
            let s = entry.status();

            s != git2::Status::CURRENT
                && (s.contains(git2::Status::INDEX_NEW)
                    || s.contains(git2::Status::INDEX_MODIFIED)
                    || s.contains(git2::Status::INDEX_DELETED)
                    || s.contains(git2::Status::INDEX_RENAMED)
                    || s.contains(git2::Status::INDEX_TYPECHANGE))
        })
        .filter_map(|entry| {
            entry
                .head_to_index()
                .and_then(|e| e.new_file().path())
                .map(|p| p.to_path_buf())
        })
        .collect()
}

fn unstaged_files(statuses: &Statuses) -> Vec<PathBuf> {
    statuses
        .iter()
        .filter(|entry| {
            let s = entry.status();
            // With `Status::OPT_INCLUDE_UNMODIFIED` (not used in this example)
            // `index_to_workdir` may not be `None` even if there are no differences,
            // in which case it will be a `Delta::Unmodified`.
            s != git2::Status::CURRENT
                && entry.index_to_workdir().is_some()
                && (s.contains(git2::Status::WT_MODIFIED)
                    || s.contains(git2::Status::WT_DELETED)
                    || s.contains(git2::Status::WT_RENAMED)
                    || s.contains(git2::Status::WT_TYPECHANGE))
        })
        .filter_map(|entry| {
            entry
                .index_to_workdir()
                .and_then(|p| p.new_file().path())
                .map(|p| p.to_path_buf())
        })
        .collect()
}
