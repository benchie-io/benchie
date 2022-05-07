use crate::Value;
use anyhow::{anyhow, ensure, Context, Result};
use git2::{BranchType, Commit, Repository, StatusOptions, Statuses};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitInfo {
    pub commit_id: String,
    pub commit_message: String,
    pub branch: String,
    pub is_dirty: bool,
}

impl<'a> IntoIterator for &'a GitInfo {
    type Item = (String, Value);
    type IntoIter = GitInfoIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        GitInfoIterator {
            git: self,
            index: 0,
        }
    }
}

pub struct GitInfoIterator<'a> {
    git: &'a GitInfo,
    index: usize,
}

impl<'a> Iterator for GitInfoIterator<'a> {
    type Item = (String, Value);

    fn next(&mut self) -> Option<Self::Item> {
        let result = match self.index {
            0 => (
                "git_commit_hash".to_string(),
                Value::String(self.git.commit_id.clone()),
            ),
            1 => (
                "git_commit_msg".to_string(),
                Value::String(self.git.commit_message.clone()),
            ),
            2 => (
                "git_branch".to_string(),
                Value::String(self.git.branch.clone()),
            ),
            3 => ("git_is_dirty".to_string(), Value::Bool(self.git.is_dirty)),
            _ => return None,
        };
        self.index += 1;
        Some(result)
    }
}

#[derive(Error, Debug)]
pub enum GitError {
    #[error("no Git repository found in current directory")]
    NotFound,
    #[error("unknown Git error")]
    Unknown(#[from] anyhow::Error),
}

pub fn read_git_info() -> Result<GitInfo, GitError> {
    let repo = discover_repository()?;

    let branch = read_current_branch(&repo)?;
    let commit = read_latest_commit(&repo, &branch)?;
    let is_dirty = is_dirty(&repo)?;

    let first_line = commit
        .message()
        .and_then(|msg| msg.split('\n').next())
        .unwrap_or("")
        .to_owned();

    Ok(GitInfo {
        commit_id: commit.id().to_string(),
        commit_message: first_line,
        branch,
        is_dirty,
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

fn read_current_branch(repo: &Repository) -> Result<String, GitError> {
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

    branch_name.ok_or(GitError::NotFound)
}

fn read_latest_commit<'a>(repo: &'a Repository, branch_name: &str) -> Result<Commit<'a>> {
    repo.revparse_single(branch_name)
        .and_then(|object| object.peel_to_commit())
        .context("failed to read latest commit")
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
