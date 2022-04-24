use crate::load_all_benchmarks;
use anyhow::Result;
use cli_table::{print_stdout, WithTitle};

pub fn show() -> Result<()> {
    let benchmarks = load_all_benchmarks()?;

    print_stdout(benchmarks.with_title()).expect("Cannot show benchmarks");
    Ok(())
}
