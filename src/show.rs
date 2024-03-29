use crate::{load_all_benchmarks, BenchmarkRaw, Value, Values};
use anyhow::Result;
use cli_table::{format::Justify, Cell, Style, Table, TableStruct};
use itertools::Itertools;
use std::collections::HashMap;

pub fn show(filter: &HashMap<String, String>) -> Result<()> {
    const EXAMPLE_VALUES_DISPLAYED: usize = 3;
    let benchmarks = load_all_benchmarks()?;
    let key_infos = compute_key_infos(
        benchmarks.iter().filter(|b| apply_filter(b, filter)),
        EXAMPLE_VALUES_DISPLAYED,
    );

    let rows: Vec<_> = key_infos
        .iter()
        .sorted_by(|a, b| (*a).0.cmp((*b).0))
        .map(|(key, info)| {
            vec![
                key.cell(),
                info.occurrences.cell(),
                display_example_values(&info.example_values).cell(),
            ]
        })
        .collect();

    // convert table to TableStruct and set title
    let table = rows
        .table()
        .title(vec![
            "key".cell().bold(true),
            "occurrences".cell().bold(true),
            "example values".cell().bold(true),
        ])
        .bold(true);

    println!(
        "Basic information about all your {} saved benchmarks:",
        benchmarks.len()
    );
    println!("{}", table.display()?);

    Ok(())
}

fn display_example_values(values: &[Value]) -> String {
    match values.len() {
        0 => String::from(""),
        1 => format!("{}", values.get(0).expect("checked")),
        2 => format!(
            "{}, {}",
            values.get(0).expect("checked"),
            values.get(1).expect("checked")
        ),
        _ => format!(
            "{}, {}, {},...",
            values.get(0).expect("checked"),
            values.get(1).expect("checked"),
            values.get(2).expect("checked")
        ),
    }
}

struct KeyInfo {
    occurrences: u64,
    example_values: Vec<Value>,
    example_values_str_len: usize,
}

fn compute_key_infos<'a, It>(benchmarks: It, max_example_values: usize) -> HashMap<String, KeyInfo>
where
    It: IntoIterator<Item = &'a BenchmarkRaw>,
    <It as IntoIterator>::IntoIter: DoubleEndedIterator,
{
    const MAX_EXAMPLE_VALUE_LENGTH: usize = 120;
    let mut info_per_key = HashMap::<String, KeyInfo>::new();

    for benchmark in benchmarks.into_iter().rev() {
        for (key, value) in benchmark.data.iter() {
            let serialized_len = value.to_string().len();

            if let Some(info) = info_per_key.get_mut(key) {
                if info.example_values.len() < max_example_values
                    && info.example_values_str_len + serialized_len <= MAX_EXAMPLE_VALUE_LENGTH
                    && !info.example_values.iter().any(|v| v == value)
                {
                    info.example_values.push(value.clone());
                    info.example_values_str_len += serialized_len
                }
                info.occurrences += 1;
            } else {
                info_per_key.insert(
                    key.clone(),
                    KeyInfo {
                        occurrences: 1,
                        example_values: if serialized_len <= MAX_EXAMPLE_VALUE_LENGTH {
                            vec![value.clone()]
                        } else {
                            vec![]
                        },
                        example_values_str_len: serialized_len,
                    },
                );
            }
        }
    }

    info_per_key
}

fn apply_filter(benchmark: &BenchmarkRaw, filter: &HashMap<String, String>) -> bool {
    filter.iter().all(|(key, value)| {
        benchmark
            .data
            .get(key)
            .map_or(false, |other| &other.to_string() == value)
    })
}

type Row1d = (String, String);

fn benchmark_to_row(row: &str, metric: &str, benchmark: &BenchmarkRaw) -> Option<Row1d> {
    let row_value = benchmark.data.get(row);
    let metric_value = benchmark.data.get(metric);

    match (row_value, metric_value) {
        (Some(row_value), Some(metric_value)) => {
            let row_value = format!("{}", row_value);
            let metric_value = format!("{}", metric_value);

            // add row to table
            Some((row_value, metric_value))
        }
        _ => None,
    }
}

pub fn show_1d_table(row: &str, metric: &str, filter: &HashMap<String, String>) -> Result<()> {
    let benchmarks = load_all_benchmarks()?;

    let TableData1d {
        rows,
        empty_matches,
    } = compute_table_data_1d(&benchmarks, row, metric, filter);

    println!("Showing 1-dimensional table with:");
    println!("row: {}, metric: {}\n", row, metric);

    if rows.is_empty() {
        println!("Result is empty");
    } else {
        println!("{}", build_1d_table(row, metric, &rows).display()?);
    }

    if empty_matches > 0 {
        println!(
            "\"{row}\" together with \"{metric}\" was {empty_matches}x not present in your benchmarks"
        );
    }

    Ok(())
}

struct TableData1d {
    rows: Vec<Row1d>,
    empty_matches: usize,
}

fn compute_table_data_1d(
    benchmarks: &[BenchmarkRaw],
    row: &str,
    metric: &str,
    filter: &HashMap<String, String>,
) -> TableData1d {
    let rows = benchmarks
        .iter()
        .filter(|benchmark| apply_filter(benchmark, filter))
        .filter_map(|benchmark| benchmark_to_row(row, metric, benchmark))
        .sorted_by(|(l, _), (r, _)| l.cmp(r))
        .collect_vec();

    let empty_matches = benchmarks.len() - rows.len();

    TableData1d {
        rows,
        empty_matches,
    }
}

fn build_1d_table(col1_title: &str, col2_title: &str, rows: &[Row1d]) -> TableStruct {
    rows.iter()
        .map(|(row, metric)| vec![row.cell(), metric.cell().justify(Justify::Right)])
        .table()
        .title(vec![
            col1_title.to_string().cell().bold(true),
            col2_title.to_string().cell().bold(true),
        ])
        .bold(true)
}

pub fn show_2d_table(
    row: &str,
    col: &str,
    metric: &str,
    filter: &HashMap<String, String>,
) -> Result<()> {
    let benchmarks = load_all_benchmarks()?;

    let data = compute_2d_table_data(&benchmarks, row, col, metric, filter);

    println!("Showing 2-dimensional table with:");
    println!("row: {}, col: {}, metric: {}\n", row, col, metric);

    if data.matrix.is_empty() {
        println!("Result is empty");
    } else {
        println!("{}", build_2d_table(&data).display()?);
    }

    Ok(())
}

fn build_2d_table(data: &TableData2d) -> TableStruct {
    data.matrix
        .iter()
        .sorted_by(|a, b| (*a).0.cmp((*b).0))
        .map(|(row, col_to_metrics)| {
            let mut table_row = vec![row.clone().cell()];
            for _ in 1..data.table_headers.len() {
                table_row.push("".cell());
            }

            for (col, metrics) in col_to_metrics.iter() {
                // TODO: here we can aggregate 'metrics'
                let metric_value = format!("{}", metrics);
                table_row[data.col_to_pos[col]] = metric_value.cell();
            }

            table_row
        })
        .table()
        .title(&data.table_headers)
}

struct TableData2d {
    table_headers: Vec<String>,
    col_to_pos: HashMap<String, usize>,
    matrix: HashMap<String, HashMap<String, Values>>,
}

fn compute_2d_table_data(
    benchmarks: &[BenchmarkRaw],
    row: &str,
    col: &str,
    metric: &str,
    filter: &HashMap<String, String>,
) -> TableData2d {
    let mut matrix = HashMap::new();
    let mut table_headers = vec![String::from("")];

    let mut col_to_pos = HashMap::new();
    let mut pos = 1;

    for benchmark in benchmarks.iter().filter(|b| apply_filter(b, filter)) {
        if let (Some(row_value), Some(col_value), Some(metric_value)) = (
            benchmark.data.get(row),
            benchmark.data.get(col),
            benchmark.data.get(metric),
        ) {
            let row_value = format!("{}", row_value);
            let col_value = format!("{}", col_value);

            if !col_to_pos.contains_key(&col_value) {
                table_headers.push(col_value.clone()); //.cell().bold(true));
                col_to_pos.insert(col_value.clone(), pos);
                pos += 1;
            }

            // insert into 3d matrix:
            // first dimension does not exist
            if !matrix.contains_key(&row_value) {
                matrix.insert(row_value.clone(), HashMap::new());
                matrix
                    .get_mut(&row_value)
                    .expect("checked")
                    .insert(col_value.clone(), Values(vec![metric_value.clone()]));
            } else {
                // second dimension does not exist
                if !matrix[&row_value].contains_key(&col_value) {
                    matrix
                        .get_mut(&row_value)
                        .expect("checked")
                        .insert(col_value.clone(), Values(vec![metric_value.clone()]));

                // all dimensions already exist
                } else {
                    matrix
                        .get_mut(&row_value)
                        .expect("checked")
                        .get_mut(&col_value)
                        .expect("checked")
                        .push(metric_value.clone());
                }
            }
        }
    }

    TableData2d {
        table_headers,
        col_to_pos,
        matrix,
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn check_if_occurrences_are_accumulated_correctly() {
        let mut b1 = BenchmarkRaw::default();
        b1.data
            .insert("command".to_string(), Value::String("program".to_string()));

        let mut b2 = BenchmarkRaw::default();
        b2.data
            .insert("command".to_string(), Value::String("program".to_string()));
        b2.data
            .insert("key".to_string(), Value::String("value".to_string()));

        let infos = compute_key_infos(&[b1, b2], 1);

        assert_eq!(
            infos.get("command").unwrap().occurrences,
            2,
            "command key should be saved for every benchmark, therefore 2 occurrences"
        );
        assert_eq!(
            infos.get("key").unwrap().occurrences,
            1,
            "user provided tag was only present one time, therefore 1 occurrence"
        )
    }

    #[test]
    fn show_does_prefers_newer_example_values_over_older_ones() {
        let mut b1 = BenchmarkRaw::default();
        b1.data
            .insert("command".to_string(), Value::String("program".to_string()));

        let mut b2 = BenchmarkRaw::default();
        b2.data
            .insert("command".to_string(), Value::String("hello".to_string()));

        let infos = compute_key_infos(&[b1, b2], 1);
        let i = infos.get("command").expect("should work");

        assert_eq!(
            i.example_values.get(0),
            Some(&Value::String("hello".to_string())),
            "hello should come first because it is saved in the newer benchmark"
        );
    }

    #[test]
    fn show_does_not_produce_example_strings_which_are_too_long() {
        let mut b1 = BenchmarkRaw::default();
        b1.data.insert(
            "command".to_string(),
            Value::String("program steadfast important message message".to_string()),
        );

        let mut b2 = BenchmarkRaw::default();
        b2.data.insert(
            "command".to_string(),
            Value::String("hello important important important message message ls s".to_string()),
        );

        let infos = compute_key_infos(&[b1, b2], 1);
        let i = infos.get("command").expect("should work");

        assert!(
            display_example_values(&i.example_values).len() < 120,
            "one string should be omitted due to max string length constraint"
        );
    }

    #[test]
    fn remove_benchmark_with_missing_key_in_filter() {
        let benchmark = BenchmarkRaw::default();

        assert!(
            !apply_filter(
                &benchmark,
                &HashMap::from([("key".to_string(), "value".to_string())])
            ),
            "if one key value pair is missing, the benchmark should get filtered out"
        );
    }

    #[test]
    fn remove_benchmark_with_wrong_value_in_filter() {
        let mut benchmark = BenchmarkRaw::default();
        benchmark
            .data
            .insert("key".to_string(), Value::String("value".to_string()));

        let filter = HashMap::from([("key".to_string(), "value2".to_string())]);
        assert!(
            !apply_filter(&benchmark, &filter),
            "benchmark should get filtered out if the value of a filter doesn't match the value in a benchmark"
        );
    }

    #[test]
    fn benchmark_should_not_get_filtered_out_if_all_filters_do_match() {
        let filter = HashMap::from([
            ("key".to_string(), "value".to_string()),
            ("key2".to_string(), "value2".to_string()),
        ]);

        let benchmark = BenchmarkRaw {
            data: filter
                .iter()
                .map(|(k, v)| (k.to_owned(), Value::String(v.clone())))
                .collect(),
        };

        assert!(
            apply_filter(&benchmark, &filter),
            "benchmark should pass the filter if all key value pairs match the filter"
        );
    }
}
