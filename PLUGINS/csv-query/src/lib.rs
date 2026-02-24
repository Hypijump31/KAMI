//! Csv-query KAMI plugin — parse CSV data and filter rows or select columns.

#[cfg(target_arch = "wasm32")] mod wasm;
use kami_guest::kami_tool;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::HashMap;

kami_tool! {
    name: "dev.kami.csv-query",
    version: "0.1.0",
    description: "Parse CSV data, filter rows, select columns, or count",
    handler: handle,
}

/// Input schema for the csv-query plugin.
#[derive(Deserialize)]
struct Input {
    action: String,
    csv: String,
    #[serde(default)]
    columns: Vec<String>,
    #[serde(rename = "where", default)]
    filter: HashMap<String, String>,
}

/// Output schema for the csv-query plugin.
#[derive(Serialize)]
struct Output {
    headers: Vec<String>,
    rows: Vec<Map<String, Value>>,
    count: usize,
}

fn handle(input: &str) -> Result<String, String> {
    let args: Input = kami_guest::parse_input(input)?;
    let (headers, rows) = parse_csv(&args.csv)?;
    match args.action.as_str() {
        "parse" => {
            let count = rows.len();
            kami_guest::to_output(&Output { headers, rows, count })
        }
        "count" => {
            let count = rows.len();
            kami_guest::to_output(&serde_json::json!({ "count": count }))
        }
        "filter" => {
            let filtered: Vec<_> = rows
                .into_iter()
                .filter(|row| {
                    args.filter.iter().all(|(k, v)| {
                        row.get(k).and_then(Value::as_str) == Some(v.as_str())
                    })
                })
                .collect();
            let count = filtered.len();
            kami_guest::to_output(&Output { headers, rows: filtered, count })
        }
        "select" => {
            let selected: Vec<Map<String, Value>> = rows
                .into_iter()
                .map(|row| {
                    args.columns
                        .iter()
                        .filter_map(|col| row.get(col).map(|v| (col.clone(), v.clone())))
                        .collect()
                })
                .collect();
            let count = selected.len();
            kami_guest::to_output(&Output { headers: args.columns, rows: selected, count })
        }
        other => Err(format!("unknown action: {other}")),
    }
}

type CsvResult = Result<(Vec<String>, Vec<Map<String, Value>>), String>;

fn parse_csv(csv: &str) -> CsvResult {
    let mut lines = csv.lines().filter(|l| !l.trim().is_empty());
    let header_line = lines.next().ok_or("empty CSV")?;
    let headers: Vec<String> = header_line.split(',').map(|s| s.trim().to_string()).collect();
    if headers.is_empty() {
        return Err("empty CSV".to_string());
    }
    let rows = lines
        .map(|line| {
            let values: Vec<&str> = line.split(',').collect();
            headers
                .iter()
                .enumerate()
                .map(|(i, h)| {
                    let val = values.get(i).copied().unwrap_or("").trim();
                    (h.clone(), Value::String(val.to_string()))
                })
                .collect::<Map<String, Value>>()
        })
        .collect();
    Ok((headers, rows))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_csv() -> &'static str {
        "name,age,city\nAlice,30,Paris\nBob,25,Lyon\nCarol,35,Paris"
    }

    #[test]
    fn parse_basic_csv() {
        let (headers, rows) = parse_csv(sample_csv()).expect("parse");
        assert_eq!(headers, vec!["name", "age", "city"]);
        assert_eq!(rows.len(), 3);
    }

    #[test]
    fn filter_by_column_value() {
        let input = format!(r#"{{"action":"filter","csv":"{}","where":{{"city":"Paris"}}}}"#, sample_csv().replace('\n', "\\n"));
        let v: serde_json::Value = serde_json::from_str(&handle(&input).expect("filter")).expect("json");
        assert_eq!(v["count"], 2);
    }

    #[test]
    fn select_specific_columns() {
        let input = format!(r#"{{"action":"select","csv":"{}","columns":["name","city"]}}"#, sample_csv().replace('\n', "\\n"));
        let v: serde_json::Value = serde_json::from_str(&handle(&input).expect("select")).expect("json");
        assert!(v["rows"][0].get("age").is_none());
        assert!(v["rows"][0].get("name").is_some());
    }

    #[test]
    fn count_rows() {
        let input = format!(r#"{{"action":"count","csv":"{}"}}"#, sample_csv().replace('\n', "\\n"));
        let v: serde_json::Value = serde_json::from_str(&handle(&input).expect("count")).expect("json");
        assert_eq!(v["count"], 3);
    }

    #[test]
    fn empty_csv_returns_error() {
        let (_, rows) = parse_csv("name,age\n").unwrap_or_default();
        assert_eq!(rows.len(), 0);
        assert!(parse_csv("").is_err());
    }

    #[test]
    fn unknown_action_returns_error() {
        let input = format!(r#"{{"action":"nope","csv":"{}"}}"#, sample_csv().replace('\n', "\\n"));
        assert!(handle(&input).is_err());
    }
}
