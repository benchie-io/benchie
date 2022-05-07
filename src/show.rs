use crate::{load_all_benchmarks, Benchmark, Value};
use anyhow::Result;
use cli_table::{format::Justify, Cell, Style, Table};
use std::collections::HashMap;

pub fn show() -> Result<()> {
    const EXAMPLE_VALUES_DISPLAYED: usize = 3;
    let key_infos = compute_key_infos(&load_all_benchmarks()?, EXAMPLE_VALUES_DISPLAYED);

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

fn compute_key_infos(
    benchmarks: &[Benchmark],
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

pub fn show_1d_table(row: String, metric: String) -> Result<()> {
    let benchmarks = load_all_benchmarks()?;

    let mut table = vec![];
    let mut empty_matches = 0;

    for benchmark in benchmarks.iter() {
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

    // convert table to TableStruct and set title
    let table = table
        .table()
        .title(vec![
            row.clone().cell().bold(true),
            metric.clone().cell().bold(true),
        ])
        .bold(true);

    println!("{}", table.display()?);

    if empty_matches > 0 {
        println!(
            "\"{}\" together with \"{}\" was {}x not present in your benchmarks",
            row, metric, empty_matches
        );
    }

    Ok(())
}

pub fn show_2d_table(_row: String, _col: String, _metric: String) -> Result<()> {
    todo!("implement 2d table")
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{Benchmark, ExecutionResult};

    #[test]
    fn check_if_occurrences_are_accumulated_correctly() {
        let b1 = Benchmark::new(
            &[String::from("cmd")],
            &ExecutionResult::default(),
            &None,
            &HashMap::new(),
        );
        let mut tags = HashMap::new();
        tags.insert(String::from("key"), String::from("value"));
        let b2 = Benchmark::new(
            &[String::from("cmd")],
            &ExecutionResult::default(),
            &None,
            &tags,
        );

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
}
