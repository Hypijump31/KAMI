//! Text-transform KAMI plugin — case conversion, slugify, truncate, and text utilities.

#[cfg(target_arch = "wasm32")] mod wasm;
use kami_guest::kami_tool;
use serde::{Deserialize, Serialize};

kami_tool! {
    name: "dev.kami.text-transform",
    version: "0.1.0",
    description: "Transform text: case conversion, slugify, truncate, word count, and more",
    handler: handle,
}

/// Input schema for the text-transform plugin.
#[derive(Deserialize)]
struct Input {
    action: String,
    text: String,
    max_length: Option<usize>,
}

/// Output schema for the text-transform plugin.
#[derive(Serialize)]
struct Output {
    result: String,
}

fn handle(input: &str) -> Result<String, String> {
    let args: Input = kami_guest::parse_input(input)?;
    let result = match args.action.as_str() {
        "uppercase" => args.text.to_uppercase(),
        "lowercase" => args.text.to_lowercase(),
        "titlecase" => to_titlecase(&args.text),
        "snake_case" => to_snake_case(&args.text),
        "camel_case" => to_camel_case(&args.text),
        "pascal_case" => to_pascal_case(&args.text),
        "kebab_case" => to_kebab_case(&args.text),
        "slugify" => slugify(&args.text),
        "truncate" => {
            let max = args.max_length.ok_or("truncate requires max_length")?;
            truncate(&args.text, max)
        }
        "trim" => args.text.trim().to_string(),
        "reverse" => args.text.chars().rev().collect(),
        "word_count" => args.text.split_whitespace().count().to_string(),
        other => return Err(format!("unknown action: {other}")),
    };
    kami_guest::to_output(&Output { result })
}

fn to_titlecase(text: &str) -> String {
    text.split_whitespace()
        .map(|w| {
            let mut c = w.chars();
            c.next().map_or(String::new(), |f| {
                f.to_uppercase().to_string() + c.as_str()
            })
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn split_words(text: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut current = String::new();
    let chars: Vec<char> = text.chars().collect();
    for (i, &ch) in chars.iter().enumerate() {
        if ch == ' ' || ch == '_' || ch == '-' {
            if !current.is_empty() { words.push(current.clone()); current.clear(); }
        } else if ch.is_uppercase() && i > 0 && chars[i - 1].is_lowercase() {
            if !current.is_empty() { words.push(current.clone()); current.clear(); }
            current.push(ch);
        } else {
            current.push(ch);
        }
    }
    if !current.is_empty() { words.push(current); }
    words
}

fn to_snake_case(text: &str) -> String {
    split_words(text).iter().map(|w| w.to_lowercase()).collect::<Vec<_>>().join("_")
}

fn to_camel_case(text: &str) -> String {
    let words = split_words(text);
    words.iter().enumerate().map(|(i, w)| {
        if i == 0 { w.to_lowercase() }
        else {
            let mut c = w.chars();
            c.next().map_or(String::new(), |f| f.to_uppercase().to_string() + &c.as_str().to_lowercase())
        }
    }).collect()
}

fn to_pascal_case(text: &str) -> String {
    split_words(text).iter().map(|w| {
        let mut c = w.chars();
        c.next().map_or(String::new(), |f| f.to_uppercase().to_string() + &c.as_str().to_lowercase())
    }).collect()
}

fn to_kebab_case(text: &str) -> String {
    split_words(text).iter().map(|w| w.to_lowercase()).collect::<Vec<_>>().join("-")
}

fn slugify(text: &str) -> String {
    let ascii: String = text.chars().map(|c| match c {
        'à'|'á'|'â'|'ã'|'ä'|'å' => 'a',
        'è'|'é'|'ê'|'ë' => 'e',
        'ì'|'í'|'î'|'ï' => 'i',
        'ò'|'ó'|'ô'|'õ'|'ö' => 'o',
        'ù'|'ú'|'û'|'ü' => 'u',
        'ç' => 'c', 'ñ' => 'n',
        c if c.is_alphanumeric() => c,
        _ => '-',
    }).collect();
    let lower = ascii.to_lowercase();
    lower.split('-').filter(|s| !s.is_empty()).collect::<Vec<_>>().join("-")
}

fn truncate(text: &str, max: usize) -> String {
    let chars: Vec<char> = text.chars().collect();
    if chars.len() <= max { text.to_string() }
    else { chars[..max.saturating_sub(1)].iter().collect::<String>() + "…" }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
