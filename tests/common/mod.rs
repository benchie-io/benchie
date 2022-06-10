use git2::Repository;
use std::env::{current_dir, set_current_dir};
use std::fs;
use std::path::Path;
use std::process::Command;
use std::sync::Arc;
use tempfile::{tempdir, TempDir};

pub fn with_temp_dir<F, R>(f: F) -> R
where
    F: FnOnce(Arc<TempDir>) -> R,
{
    let temp_dir = Arc::new(tempdir().unwrap());

    let old_cd = current_dir().unwrap();
    let _ = set_current_dir(temp_dir.path());

    let result = f(temp_dir);

    let _ = set_current_dir(old_cd);

    result
}

pub fn build_git_repo<P: AsRef<Path>>(path: P) {
    let _ = Repository::init(path).unwrap();

    let _ = fs::write("./README.md", "# Header");

    commit(&["README.md"], "initial commit");
}

pub fn commit(files: &[&str], msg: &str) {
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
