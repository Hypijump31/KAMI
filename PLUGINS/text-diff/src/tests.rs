use super::*;

#[test]
fn lines_diff_adds_and_removes() {
    let old = vec!["hello".into(), "world".into()];
    let new = vec!["hello".into(), "earth".into()];
    let diff = compute_diff(&old, &new);
    assert!(diff.iter().any(|d| d.kind == "unchanged" && d.value == "hello"));
    assert!(diff.iter().any(|d| d.kind == "removed" && d.value == "world"));
    assert!(diff.iter().any(|d| d.kind == "added" && d.value == "earth"));
}

#[test]
fn identical_texts_all_unchanged() {
    let tokens = vec!["a".into(), "b".into()];
    let diff = compute_diff(&tokens, &tokens);
    assert!(diff.iter().all(|d| d.kind == "unchanged"));
}

#[test]
fn empty_old_all_added() {
    let diff = compute_diff(&[], &["x".into(), "y".into()]);
    assert!(diff.iter().all(|d| d.kind == "added"));
}

#[test]
fn empty_new_all_removed() {
    let diff = compute_diff(&["x".into()], &[]);
    assert!(diff.iter().all(|d| d.kind == "removed"));
}

#[test]
fn similarity_100_for_identical() {
    let result = handle(r#"{"old":"hello","new":"hello","mode":"words"}"#).expect("h");
    let v: serde_json::Value = serde_json::from_str(&result).expect("json");
    assert_eq!(v["similarity_percent"], 100);
}

#[test]
fn unknown_mode_returns_error() {
    let result = handle(r#"{"old":"a","new":"b","mode":"emoji"}"#);
    assert!(result.is_err());
}

#[test]
fn oversized_input_returns_error() {
    let big: Vec<String> = (0..=MAX_TOKENS).map(|i| i.to_string()).collect();
    let input = serde_json::json!({
        "old": big.join("\n"),
        "new": "x",
        "mode": "lines"
    });
    let result = handle(&input.to_string());
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("too large"));
}
