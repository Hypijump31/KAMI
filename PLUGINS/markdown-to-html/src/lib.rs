//! Markdown-to-html KAMI plugin — convert Markdown to HTML with GFM extensions.

#[cfg(target_arch = "wasm32")] mod wasm;
use kami_guest::kami_tool;
use pulldown_cmark::{html, Options, Parser};
use serde::{Deserialize, Serialize};

kami_tool! {
    name: "dev.kami.markdown-to-html",
    version: "0.1.0",
    description: "Convert Markdown to HTML with optional GFM extensions",
    handler: handle,
}

/// GFM extension options.
#[derive(Deserialize, Default)]
struct MdOptions {
    #[serde(default = "default_true")]
    tables: bool,
    #[serde(default = "default_true")]
    strikethrough: bool,
    #[serde(default = "default_true")]
    tasklists: bool,
    #[serde(default)]
    smart_punctuation: bool,
}

fn default_true() -> bool {
    true
}

/// Input schema for the markdown-to-html plugin.
#[derive(Deserialize)]
struct Input {
    markdown: String,
    #[serde(default)]
    options: MdOptions,
}

/// Output schema for the markdown-to-html plugin.
#[derive(Serialize)]
struct Output {
    html: String,
    word_count: usize,
    has_code_blocks: bool,
    has_tables: bool,
    heading_count: usize,
}

fn handle(input: &str) -> Result<String, String> {
    let args: Input = kami_guest::parse_input(input)?;
    let html_output = convert_markdown(&args.markdown, &args.options);
    let word_count = args.markdown.split_whitespace().count();
    let has_code_blocks = args.markdown.contains("```")
        || args.markdown.lines().any(|l| l.starts_with("    "));
    let has_tables = args.markdown.contains('|');
    let heading_count = args
        .markdown
        .lines()
        .filter(|l| l.starts_with('#'))
        .count();
    kami_guest::to_output(&Output {
        html: html_output,
        word_count,
        has_code_blocks,
        has_tables,
        heading_count,
    })
}

/// Convert Markdown source to HTML using pulldown-cmark.
fn convert_markdown(markdown: &str, opts: &MdOptions) -> String {
    let mut options = Options::empty();
    if opts.tables {
        options.insert(Options::ENABLE_TABLES);
    }
    if opts.strikethrough {
        options.insert(Options::ENABLE_STRIKETHROUGH);
    }
    if opts.tasklists {
        options.insert(Options::ENABLE_TASKLISTS);
    }
    if opts.smart_punctuation {
        options.insert(Options::ENABLE_SMART_PUNCTUATION);
    }
    let parser = Parser::new_ext(markdown, options);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    html_output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convert_heading_to_h1() {
        let html = convert_markdown("# Hello", &MdOptions::default());
        assert!(html.contains("<h1>Hello</h1>"));
    }

    #[test]
    fn convert_bold_italic() {
        let html = convert_markdown("**bold** and *italic*", &MdOptions::default());
        assert!(html.contains("<strong>bold</strong>"));
        assert!(html.contains("<em>italic</em>"));
    }

    #[test]
    fn convert_list() {
        let html = convert_markdown("- item1\n- item2", &MdOptions::default());
        assert!(html.contains("<ul>"));
        assert!(html.contains("<li>item1</li>"));
    }

    #[test]
    fn empty_markdown_returns_empty_html() {
        let result = handle(r#"{"markdown":""}"#).expect("handle");
        let v: serde_json::Value = serde_json::from_str(&result).expect("json");
        assert_eq!(v["html"], "");
        assert_eq!(v["word_count"], 0);
    }

    #[test]
    fn word_count_is_correct() {
        let result = handle(r##"{"markdown":"# Hello\n\nThis is **bold** text."}"##).expect("h");
        let v: serde_json::Value = serde_json::from_str(&result).expect("json");
        assert!(v["word_count"].as_u64().unwrap_or(0) > 0);
    }

    #[test]
    fn heading_count_is_correct() {
        let result =
            handle(r##"{"markdown":"# H1\n## H2\n### H3\nsome text"}"##).expect("handle");
        let v: serde_json::Value = serde_json::from_str(&result).expect("json");
        assert_eq!(v["heading_count"], 3);
    }

    #[test]
    fn has_code_blocks_detected() {
        let result = handle(r#"{"markdown":"```rust\nlet x = 1;\n```"}"#).expect("handle");
        let v: serde_json::Value = serde_json::from_str(&result).expect("json");
        assert_eq!(v["has_code_blocks"], true);
    }
}
