//! Json-query KAMI plugin — pick, merge, sort, flatten, and extract from JSON objects.

#[cfg(target_arch = "wasm32")] mod wasm;
use kami_guest::kami_tool;
use serde::Deserialize;
use serde_json::{Map, Value};

kami_tool! {
    name: "dev.kami.json-query",
    version: "0.1.0",
    description: "Transform JSON objects: pick, merge, sort_keys, keys, values, flatten",
    handler: handle,
}

/// Input schema for the json-query plugin.
#[derive(Deserialize)]
struct Input {
    action: String,
    data: Value,
    #[serde(default)]
    keys: Vec<String>,
    extra: Option<Value>,
}

fn handle(input: &str) -> Result<String, String> {
    let args: Input = kami_guest::parse_input(input)?;
    match args.action.as_str() {
        "pick" => pick(&args.data, &args.keys),
        "merge" => merge(
            &args.data,
            args.extra.as_ref().ok_or("merge requires extra object")?,
        ),
        "sort_keys" => sort_keys(&args.data),
        "keys" => extract_keys(&args.data),
        "values" => extract_values(&args.data),
        "flatten" => flatten_obj(&args.data),
        other => Err(format!("unknown action: {other}")),
    }
}

/// Extract only the specified keys from an object.
fn pick(data: &Value, keys: &[String]) -> Result<String, String> {
    let obj = data.as_object().ok_or("data must be an object")?;
    let picked: Map<String, Value> = keys
        .iter()
        .filter_map(|k| obj.get(k).map(|v| (k.clone(), v.clone())))
        .collect();
    kami_guest::to_output(&picked)
}

/// Merge two objects; extra keys overwrite data keys on conflict.
fn merge(data: &Value, extra: &Value) -> Result<String, String> {
    let base = data.as_object().ok_or("data must be an object")?;
    let ext = extra.as_object().ok_or("extra must be an object")?;
    let mut result = base.clone();
    for (k, v) in ext {
        result.insert(k.clone(), v.clone());
    }
    kami_guest::to_output(&result)
}

/// Return a new object with keys sorted alphabetically.
fn sort_keys(data: &Value) -> Result<String, String> {
    let obj = data.as_object().ok_or("data must be an object")?;
    let mut pairs: Vec<(String, Value)> = obj
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    pairs.sort_by(|a, b| a.0.cmp(&b.0));
    let sorted: Map<String, Value> = pairs.into_iter().collect();
    kami_guest::to_output(&sorted)
}

/// Return all top-level key names as a JSON array.
fn extract_keys(data: &Value) -> Result<String, String> {
    let obj = data.as_object().ok_or("data must be an object")?;
    let keys: Vec<&str> = obj.keys().map(String::as_str).collect();
    kami_guest::to_output(&keys)
}

/// Return all top-level values as a JSON array.
fn extract_values(data: &Value) -> Result<String, String> {
    let obj = data.as_object().ok_or("data must be an object")?;
    let vals: Vec<&Value> = obj.values().collect();
    kami_guest::to_output(&vals)
}

/// Flatten a nested object into dot-notation keys.
fn flatten_obj(data: &Value) -> Result<String, String> {
    let obj = data.as_object().ok_or("data must be an object")?;
    let mut result = Map::new();
    flatten_recursive(obj, String::new(), &mut result);
    kami_guest::to_output(&result)
}

fn flatten_recursive(obj: &Map<String, Value>, prefix: String, out: &mut Map<String, Value>) {
    for (key, value) in obj {
        let full = if prefix.is_empty() { key.clone() } else { format!("{prefix}.{key}") };
        match value {
            Value::Object(nested) => flatten_recursive(nested, full, out),
            _ => { out.insert(full, value.clone()); }
        }
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
