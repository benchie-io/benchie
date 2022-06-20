<div align="center">
  <h1>benchie</h1>
  <a href="https://github.com/benchie-io/benchie/actions"><img src="https://img.shields.io/github/workflow/status/benchie-io/benchie/CI" /></a>
  <a href="https://codecov.io/gh/benchie-io/benchie">
    <img src="https://codecov.io/gh/benchie-io/benchie/branch/main/graph/badge.svg?token=K4WQDNCN9F"/>
  </a>
  <a href="https://crates.io/crates/benchie"><img src="https://img.shields.io/crates/v/benchie.svg" /></a>
  <a href="https://docs.rs/benchie"><img src="https://docs.rs/benchie/badge.svg" /></a>
  <img src="https://img.shields.io/badge/Rust-v1.61.0-yellow" />
  <img src="https://img.shields.io/badge/platform-linux%20%7C%20macos%20%7C%20windows-brightgreen" />
  <a href="https://github.com/benchie-io/benchie/blob/master/LICENSE"><img src="https://img.shields.io/crates/l/benchie" /></a>
  <br />
  <br />
  <a href="https://www.benchie.io/">Website</a>
  <span>&nbsp;&nbsp;|&nbsp;&nbsp;</span>
  <a href="https://docs.rs/benchie">Docs</a>
  <span>&nbsp;&nbsp;|&nbsp;&nbsp;</span>
  <a href="#">Twitter</a>
  <span>&nbsp;&nbsp;|&nbsp;&nbsp;</span>
  <a href="mailto:hello@benchie.io">Mail</a>
  <br />
  <hr />
</div>

## What is benchie?

benchie is a simple-to-use CLI tool that benchmarks performance of your processes and organizes the benchmarking results for you.

## Installation
benchie ships as a single executable with no dependencies. You can install it using the installers below, or download a release binary from the releases page.

Shell (Mac, Linux):
```
curl -fsSL  https://raw.githubusercontent.com/benchie-io/benchie/main/scripts/install.sh | sh
```

PowerShell (Windows):
```
iwr https://raw.githubusercontent.com/benchie-io/benchie/main/scripts/install.ps1 -useb | iex
```

### Build from Source
benchie can be built and tested on all major platforms. First install Rust from https://rustup.rs and add it to your path.

#### Install via Cargo

Install the latest version of benchie using Rust's built-in package manager:

```bash
$ cargo install benchie --locked
```

#### Install via Github
* Clone this repository
* Test you toolchain setup by compiling benchie:
  ```bash
  $ cargo build --locked
  ```
* Run tests:
  ```bash
  $ cargo test --locked
  ```
  
  
## Usage

### Benchmark

In order to benchmark (measure runtime similar to the unix-tool `time`) a process, invocate benchie as follows:
```bash
$ benchie [OPTIONS] <COMMAND>
```
where `<COMMAND>` can be any command including command-line arguments. 
benchie stores additionally some meta information to the benchmark such as information of the commit, timestamps, exit status of the executable, etc.
Moreover, one can pass various options to the benchmarking process.

#### Tagging

A tag helps to identify benchmarking results. One can pass multiple tags as key-value pairs for each benchmark using `--tag key=value` as an option.
For instance,
```bash
$ benchie --tag algorithm=bubblesort ./bubblesort
```
runs the executable `./bubblesort` and tags the benchmark with `algorithm=bubblesort`.

#### Output Tagging

Tags can also be provided on stdout of the executable at runtime.
For instance, benchie will parse the output and if
```
@benchie key=value
```
is printed to stdout, the tag `key=value` is stored as tag to the benchmark.


### Display Benchmarking Results

To get an overview of all your benchmarks, simply type

```bash
$ benchie show
```

A sample output could be:
```
+-----------------+-------------+----------------------------------------------------------------+
| key             | occurrences | example values                                                 |
+-----------------+-------------+----------------------------------------------------------------+
| commit_id       | 2           | 014fbb4b2e5c5cc1de7266a708fa909df9915011                       |
+-----------------+-------------+----------------------------------------------------------------+
| user_time       | 2           | 567µs, 572µs                                                   |
+-----------------+-------------+----------------------------------------------------------------+
| algorithm       | 2           | mergesort, bubblesort                                          |
+-----------------+-------------+----------------------------------------------------------------+
| created_at      | 2           | 2022-05-11 09:11:54.315066 UTC, 2022-05-11 09:11:47.635744 UTC |
+-----------------+-------------+----------------------------------------------------------------+
| status_code     | 2           | 0                                                              |
+-----------------+-------------+----------------------------------------------------------------+
```
which means that we have 2 stored benchmarks, where we tagged one with `algorithm=mergesort` and the other with `algorithm=bubblesort`.

#### Table View
To usefully show benchmarking results, benchie provides a one-dimensional or two-dimensional table view.
For the one-dimensional view, we must provide the `--row` option and a *metric* that we want to display.
Both, the row and the metric must occur as a key in at least one benchmark. 
To identify occurring keys, use `benchie show`.

As an example, we use `algorithm` as row and `user_time` as metric:
```bash
$ benchie show --row algorithm user_time
```
which may give the following output:
```
+------------+-----------+
| algorithm  | user_time |
+------------+-----------+
| bubblesort |     572µs |
+------------+-----------+
| mergesort  |     567µs |
+------------+-----------+
```

For the two-dimensional table view, we need also to pass the `--col` option, which again must be an occuring key.
Let's assume we made 4 benchmarks: 2 for bubblesort and 2 for mergesort with 100 and 1000 elements, respectively.
Hence, we tagged each benchmark with two tags: `algorithm=bubblesort` and `elements=100`, etc.
To show a two-dimensional table view on `algorithm` and `elements` with `user_time` as metric, we can type
```bash
$ benchie show --row algorithm --col elements user_time
```
which may give the following output:
```
+------------+----------------+
|            |  100  |  1000  |
+------------+----------------+
| bubblesort | 572µs |  944µs |
+------------+----------------+
| mergesort  | 567µs |  598µs |
+------------+----------------+
```

#### Filtering

To filter the benchmark results, one can pass an equality filter as option.
The syntax for the filter option is `--filter key=value`, for instance
```bash
$ benchie show --filter algorithm=mergesort --row algorithm user_time
```
shows a one-dimensional table view, which lists only entries where `algorithm` is equal to `mergesort`.

## Contribution

TBA

## License

Licensed under the [MIT](LICENSE) license.
