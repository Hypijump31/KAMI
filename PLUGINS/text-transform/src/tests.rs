use super::*;

#[test]
fn uppercase() {
    let r = handle(r#"{"action":"uppercase","text":"hello"}"#).expect("handle");
    let v: serde_json::Value = serde_json::from_str(&r).expect("json");
    assert_eq!(v["result"].as_str().expect("str"), "HELLO");
}

#[test]
fn snake_case_from_camel() {
    assert_eq!(to_snake_case("helloWorld"), "hello_world");
}

#[test]
fn camel_case_from_words() {
    assert_eq!(to_camel_case("hello world"), "helloWorld");
}

#[test]
fn pascal_case_from_words() {
    assert_eq!(to_pascal_case("hello world"), "HelloWorld");
}

#[test]
fn kebab_case_from_words() {
    assert_eq!(to_kebab_case("Hello World"), "hello-world");
}

#[test]
fn slugify_removes_accents() {
    assert_eq!(slugify("Héllo Wörld"), "hello-world");
}

#[test]
fn slugify_removes_special_chars() {
    assert_eq!(slugify("Hello, World! 2024"), "hello-world-2024");
}

#[test]
fn truncate_long_text() {
    let result = truncate("Hello World", 6);
    assert_eq!(result, "Hello…");
}

#[test]
fn truncate_short_text_unchanged() {
    assert_eq!(truncate("Hi", 10), "Hi");
}

#[test]
fn truncate_without_max_length_returns_error() {
    let result = handle(r#"{"action":"truncate","text":"hello"}"#);
    assert!(result.is_err());
}

#[test]
fn word_count() {
    let result = handle(r#"{"action":"word_count","text":"hello world foo"}"#).expect("h");
    let v: serde_json::Value = serde_json::from_str(&result).expect("json");
    assert_eq!(v["result"], "3");
}

#[test]
fn unknown_action_returns_error() {
    assert!(handle(r#"{"action":"explode","text":"x"}"#).is_err());
}
