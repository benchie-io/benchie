mod benchmark;
mod storage;

pub use benchmark::benchmark;
pub use storage::{append_benchmark, load_all_benchmarks, Benchmark};
