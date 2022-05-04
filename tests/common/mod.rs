use std::env::{current_dir, set_current_dir};
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
