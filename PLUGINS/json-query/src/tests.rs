use super::*;

#[test]
fn pick_selects_keys() {
    let result = handle(r#"{"action":"pick","data":{"a":1,"b":2,"c":3},"keys":["a","c"]}"#)
        .expect("pick");
    let v: serde_json::Value = serde_json::from_str(&result).expect("json");
    assert_eq!(v["a"], 1);
    assert!(v.get("b").is_none());
}

#[test]
fn pick_missing_key_ignored() {
    let result = handle(r#"{"action":"pick","data":{"a":1},"keys":["a","z"]}"#).expect("pick");
    let v: serde_json::Value = serde_json::from_str(&result).expect("json");
    assert_eq!(v["a"], 1);
    assert!(v.get("z").is_none());
}

#[test]
fn merge_two_objects() {
    let result =
        handle(r#"{"action":"merge","data":{"a":1},"extra":{"b":2}}"#).expect("merge");
    let v: serde_json::Value = serde_json::from_str(&result).expect("json");
    assert_eq!(v["a"], 1);
    assert_eq!(v["b"], 2);
}

#[test]
fn merge_extra_wins_on_conflict() {
    let result =
        handle(r#"{"action":"merge","data":{"a":1},"extra":{"a":99}}"#).expect("merge");
    let v: serde_json::Value = serde_json::from_str(&result).expect("json");
    assert_eq!(v["a"], 99);
}

#[test]
fn sort_keys_alphabetical() {
    let result = handle(r#"{"action":"sort_keys","data":{"c":3,"a":1,"b":2}}"#).expect("sort");
    let v: serde_json::Value = serde_json::from_str(&result).expect("json");
    let keys: Vec<&str> = v.as_object().expect("obj").keys().map(String::as_str).collect();
    assert_eq!(keys, vec!["a", "b", "c"]);
}

#[test]
fn flatten_nested_object() {
    let result = handle(r#"{"action":"flatten","data":{"a":{"b":1},"c":2}}"#).expect("flatten");
    let v: serde_json::Value = serde_json::from_str(&result).expect("json");
    assert_eq!(v["a.b"], 1);
    assert_eq!(v["c"], 2);
}

#[test]
fn unknown_action_returns_error() {
    let result = handle(r#"{"action":"nope","data":{}}"#);
    assert!(result.is_err());
}
