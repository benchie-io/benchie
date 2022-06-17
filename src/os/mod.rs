#[cfg(unix)]
mod unix;
#[cfg(windows)]
mod windows;

#[cfg(windows)]
pub use crate::os::windows::execute_and_measure;
#[cfg(unix)]
pub use unix::execute_and_measure;
