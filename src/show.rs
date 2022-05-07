use crate::{load_all_benchmarks, BenchmarkRaw, Value, Values};
use anyhow::Result;
use cli_table::{format::Justify, Cell, Style, Table};
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

    println!("Basic information about all your saved benchmarks:");
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
}

fn compute_key_infos<'a>(
    benchmarks: impl IntoIterator<Item = &'a BenchmarkRaw>,
    max_example_values: usize,
) -> HashMap<String, KeyInfo> {
    let mut info_per_key = HashMap::<String, KeyInfo>::new();

    for benchmark in benchmarks {
        for (key, value) in benchmark.data.iter() {
            if let Some(info) = info_per_key.get_mut(key) {
                if info.example_values.len() < max_example_values {
                    info.example_values.push(value.clone());
                }
                info.occurrences += 1;
            } else {
                info_per_key.insert(
                    key.clone(),
                    KeyInfo {
                        occurrences: 1,
                        example_values: vec![value.clone()],
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

pub fn show_1d_table(row: String, metric: String, filter: &HashMap<String, String>) -> Result<()> {
    let benchmarks = load_all_benchmarks()?;

    let mut table = vec![];
    let mut empty_matches = 0;

    for benchmark in benchmarks.iter().filter(|b| apply_filter(b, filter)) {
        let row_value = benchmark.data.get(&row);
        let metric_value = benchmark.data.get(&metric);

        match (row_value, metric_value) {
            (Some(row_value), Some(metric_value)) => {
                let row_value = format!("{}", row_value);
                let metric_value = format!("{}", metric_value);

                // add row to table
                table.push(vec![
                    row_value.cell(),
                    metric_value.cell().justify(Justify::Right),
                ]);
            }
            _ => empty_matches += 1,
        }
    }

    println!("Showing 1-dimensional table with:");
    println!("row: {}, metric: {}\n", row, metric);

    if table.is_empty() {
        println!("Result is empty");
    } else {
        // convert table to TableStruct and set title
        let table = table
            .table()
            .title(vec![
                row.clone().cell().bold(true),
                metric.clone().cell().bold(true),
            ])
            .bold(true);
        println!("{}", table.display()?);
    }

    if empty_matches > 0 {
        println!(
            "\"{}\" together with \"{}\" was {}x not present in your benchmarks",
            row, metric, empty_matches
        );
    }

    Ok(())
}

pub fn show_2d_table(
    row: String,
    col: String,
    metric: String,
    filter: &HashMap<String, String>,
) -> Result<()> {
    let benchmarks = load_all_benchmarks()?;

    let mut matrix: HashMap<String, HashMap<String, Values>> = HashMap::new();
    let mut table = vec![];
    let mut table_header = vec!["".cell()];

    let mut col_to_pos = HashMap::new();
    let mut pos = 1;

    for benchmark in benchmarks.iter().filter(|b| apply_filter(b, filter)) {
        if let (Some(row_value), Some(col_value), Some(metric_value)) = (
            benchmark.data.get(&row),
            benchmark.data.get(&col),
            benchmark.data.get(&metric),
        ) {
            let row_value = format!("{}", row_value);
            let col_value = format!("{}", col_value);

            if !col_to_pos.contains_key(&col_value) {
                table_header.push(col_value.clone().cell().bold(true));
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

    // build table
    for (row, col_to_metrics) in matrix.iter() {
        let mut table_row = vec![row.clone().cell()];
        for _ in 1..table_header.len() {
            table_row.push("".cell());
        }

        for (col, metrics) in col_to_metrics.iter() {
            // TODO: here we can aggregate 'metrics'
            let metric_value = format!("{}", metrics);
            table_row[col_to_pos[col]] = metric_value.cell();
        }
        table.push(table_row);
    }

    println!("Showing 2-dimensional table with:");
    println!("row: {}, col: {}, metric: {}\n", row, col, metric);

    if table.is_empty() {
        println!("Result is empty");
    } else {
        let table = table.table().title(table_header);
        println!("{}", table.display()?);
    }

    Ok(())
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
