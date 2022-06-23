#![allow(dead_code)]

use git2::Repository;
use snailquote::unescape;
use std::env::{current_dir, set_current_dir};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use std::{env, fs};
use tempfile::{tempdir, TempDir};

/// CARE: this function modifies the current directory
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

pub fn with_temp_data_dir<F, R>(f: F) -> R
where
    F: FnOnce(Arc<TempDir>) -> R,
{
    let temp_dir = Arc::new(tempdir().unwrap());

    build_git_repo(temp_dir.path());

    fs::create_dir(temp_dir.path().join(".benchie"))
        .expect("should be able to create benchie dir in new temp directory");

    fs::copy(
        current_dir().unwrap().join("tests").join("data.json"),
        temp_dir.path().join(".benchie").join("data.json"),
    )
    .expect("should be able to copy data");

    f(temp_dir)
}

pub struct Benchie {
    executable: PathBuf,
}

impl Benchie {
    pub fn new() -> Self {
        fn cargo_dir() -> PathBuf {
            let mut c = Command::new("cargo");
            c.arg("build").output().expect("build should work");

            env::var_os("CARGO_BIN_PATH")
                .map(PathBuf::from)
                .or_else(|| {
                    env::current_exe().ok().map(|mut path| {
                        path.pop();
                        if path.ends_with("deps") {
                            path.pop();
                        }
                        path
                    })
                })
                .unwrap_or_else(|| {
                    panic!("CARGO_BIN_PATH wasn't set. Cannot continue running test")
                })
        }

        fn cargo_exe() -> PathBuf {
            cargo_dir().join(format!("benchie{}", env::consts::EXE_SUFFIX))
        }

        Self {
            executable: cargo_exe(),
        }
    }

    pub fn run(&self, args: &[&str]) -> String {
        self.run_in_dir(args, current_dir().unwrap())
    }

    pub fn run_in_dir<P: AsRef<Path>>(&self, args: &[&str], path: P) -> String {
        let mut command = Command::new(&self.executable);
        let output = command
            .current_dir(&path)
            .args(args)
            .output()
            .expect("show command should have succeeded");

        let stdout = strip_ansi_escapes::strip(&output.stdout).unwrap();
        let stdout = String::from_utf8(stdout).expect("should be utf8");
        unescape(&stdout).expect("should not have unknown escape codes")
    }
}

pub fn build_git_repo<P: AsRef<Path>>(path: P) {
    let _ = Repository::init(&path).unwrap();

    let _ = fs::write(path.as_ref().join("README.md"), "# Header");

    commit_at_path(&["README.md"], "initial commit", path);
}

pub fn commit(files: &[&str], msg: &str) {
    commit_at_path(files, msg, current_dir().unwrap())
}

pub fn commit_at_path<P: AsRef<Path>>(files: &[&str], msg: &str, path: P) {
    let _ = Command::new("git")
        .args([&["add"], files].concat())
        .current_dir(&path)
        .output()
        .expect("failed to execute process");

    let _ = Command::new("git")
        .env("GIT_AUTHOR_NAME", "benchie")
        .env("GIT_AUTHOR_EMAIL", "benchie@benchie.io")
        .env("GIT_COMMITTER_NAME", "benchie")
        .env("GIT_COMMITTER_EMAIL", "benchie@benchie.io")
        .args(["commit", "-m", msg, "--no-gpg-sign", "--no-verify"])
        .current_dir(&path)
        .output()
        .expect("failed to execute process");
}
