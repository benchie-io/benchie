use std::env::{current_dir, set_current_dir};
use std::sync::Arc;
use tempfile::{tempdir, TempDir};

pub fn with_temp_dir<F, R>(f: F) -> R
where
    F: FnOnce(Arc<TempDir>) -> R,
{
    println!("bla");
    let temp_dir = Arc::new(tempdir().unwrap());

    println!("bla");
    println!("{:?}", temp_dir);
    let old_cd = current_dir().unwrap();
    println!("bla");
    let _ = set_current_dir(temp_dir.path());

    let result = f(temp_dir);

    let _ = set_current_dir(old_cd);

    result
}
