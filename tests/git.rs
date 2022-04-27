mod common;

use benchie::{read_git_info, GitError};
use common::with_temp_dir;
use git2::Repository;
use serial_test::serial;
use std::fs;
use std::path::Path;
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

        assert!(matches!(result, Err(GitError::NotFound)));
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
            info.commit_message, "update\n",
            "verify that we get the latest commit message"
        );
    });
}

fn build_git_repo<P: AsRef<Path>>(path: P) {
    let _ = Repository::init(path).unwrap();

    let _ = fs::write("./README.md", "# Header");

    commit(&["README.md"], "initial commit");
}

fn commit(files: &[&str], msg: &str) {
    let _ = Command::new("git")
        .args([&["add"], files].concat())
        .output()
        .expect("failed to execute process");

    let _ = Command::new("git")
        .env("GIT_AUTHOR_NAME", "benchie")
        .env("GIT_AUTHOR_EMAIL", "benchie@benchie.io")
        .env("GIT_COMMITTER_NAME", "benchie")
        .env("GIT_COMMITTER_EMAIL", "benchie@benchie.io")
        .args(["commit", "-m", msg, "--no-gpg-sign", "--no-verify"])
        .output()
        .expect("failed to execute process");
}
