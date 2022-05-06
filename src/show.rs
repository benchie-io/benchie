use crate::load_all_benchmarks;
use anyhow::Result;
use cli_table::{format::Justify, Cell, Style, Table};

pub fn show() -> Result<()> {
    todo!("implement default show")
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
