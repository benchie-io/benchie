mod common;

use crate::common::{build_git_repo, commit};
use benchie::{read_git_info, GitError};
use common::with_temp_dir;
use git2::Repository;
use serial_test::serial;
use std::fs;
use std::process::Command;

#[test]
#[serial]
fn git_info_without_git_repo() {
    with_temp_dir(|_| {
        let result = read_git_info();

        assert!(matches!(result, Err(GitError::NotFound)));
    });
}

#[test]
#[serial]
fn git_info_of_fresh_git_repo() {
    with_temp_dir(|dir| {
        let _ = Repository::init(dir.path()).unwrap();

        let result = read_git_info();

        assert!(matches!(result, Err(GitError::NoCommit)));
    });
}

#[test]
#[serial]
fn git_info_of_repo_with_head() {
    with_temp_dir(|dir| {
        build_git_repo(dir.path());

        let result = read_git_info();

        assert!(result.is_ok());

        let info = result.unwrap();
        assert!(!info.is_dirty);
    });
}

#[test]
#[serial]
fn git_info_of_dirty_repository() {
    with_temp_dir(|dir| {
        build_git_repo(dir.path());
        let _ = fs::write("./README.md", "# Header and new content");

        let result = read_git_info();

        assert!(result.is_ok());

        let info = result.unwrap();
        assert!(info.is_dirty);
    });
}

#[test]
#[serial]
fn git_info_of_repo_with_commit_history() {
    with_temp_dir(|dir| {
        build_git_repo(dir.path());
        let _ = fs::write("./README.md", "# Header and new content");

        commit(&["README.md"], "update");

        let result = read_git_info();

        assert!(result.is_ok());

        let info = result.unwrap();
        assert!(!info.is_dirty);
        assert_eq!(
            info.commit_id.len(),
            40,
            "valid commit hash is 40 chars long"
        );
        assert_eq!(
            info.commit_message, "update",
            "verify that we get the latest commit message"
        );
    });
}

#[test]
#[serial]
fn git_info_if_repo_is_not_at_head_commit() {
    with_temp_dir(|dir| {
        build_git_repo(dir.path());
        let _ = fs::write("./README.md", "# Header and new content");

        commit(&["README.md"], "update");

        let _ = Command::new("git")
            .args(["checkout", "HEAD~1"])
            .output()
            .expect("failed to execute process");

        let result = read_git_info();

        assert!(result.is_ok());

        let info = result.unwrap();
        assert!(!info.is_dirty);
        assert_eq!(
            info.commit_id.len(),
            40,
            "valid commit hash is 40 chars long"
        );
        assert_eq!(
            info.commit_message, "initial commit",
            "verify that we get the latest commit message"
        );
    });
}
