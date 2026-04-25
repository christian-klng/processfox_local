//! Best-effort recovery for tool-call argument JSON produced by smaller or
//! quantized models.
//!
//! Most providers stream tool arguments as well-formed JSON. Smaller local
//! models sometimes wrap them in fenced code blocks, add trailing commas,
//! single-quote keys, or emit prose around the JSON. We try the obvious
//! repair steps in order and fall back to a string payload if all fail —
//! letting the LLM see the raw text and decide what to do next.

use serde_json::Value as JsonValue;

/// Best-effort parse of a JSON object out of `raw`. Tries:
/// 1. Direct parse.
/// 2. Strip surrounding code fences and prose; parse what's between the
///    first `{` and matching `}`.
/// 3. Light syntax repair (single-quoted keys, trailing commas).
///
/// Returns the parsed value, or — on total failure — the raw string wrapped
/// as `JsonValue::String` so the model still gets something it can react to.
pub fn extract_json_value(raw: &str) -> JsonValue {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return serde_json::json!({});
    }
    if let Ok(v) = serde_json::from_str::<JsonValue>(trimmed) {
        return v;
    }
    if let Some(slice) = first_balanced_object(trimmed) {
        if let Ok(v) = serde_json::from_str::<JsonValue>(slice) {
            return v;
        }
        let repaired = light_repair(slice);
        if let Ok(v) = serde_json::from_str::<JsonValue>(&repaired) {
            return v;
        }
    }
    JsonValue::String(raw.to_string())
}

/// Extract the substring covering the first balanced `{ ... }` block,
/// respecting strings and escapes. Returns `None` if no balanced object
/// could be found.
fn first_balanced_object(s: &str) -> Option<&str> {
    let bytes = s.as_bytes();
    let start = bytes.iter().position(|&b| b == b'{')?;
    let mut depth: i32 = 0;
    let mut in_string = false;
    let mut escape = false;
    for (i, &b) in bytes.iter().enumerate().skip(start) {
        if in_string {
            if escape {
                escape = false;
            } else if b == b'\\' {
                escape = true;
            } else if b == b'"' {
                in_string = false;
            }
            continue;
        }
        match b {
            b'"' => in_string = true,
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(&s[start..=i]);
                }
            }
            _ => {}
        }
    }
    None
}

/// Cheap repairs that fix the most common quirks: single-quoted keys,
/// trailing commas before `}` or `]`. Strings with literal apostrophes
/// won't be affected because the regexes below only match keys.
fn light_repair(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    let mut prev = '\0';
    while let Some(c) = chars.next() {
        match c {
            ',' => {
                // Skip comma if the next non-whitespace is `}` or `]`.
                let mut look_ahead = String::new();
                while let Some(&n) = chars.peek() {
                    if n.is_whitespace() {
                        look_ahead.push(n);
                        chars.next();
                    } else {
                        break;
                    }
                }
                match chars.peek() {
                    Some('}') | Some(']') => {
                        // Drop the comma; flush whitespace.
                        out.push_str(&look_ahead);
                    }
                    _ => {
                        out.push(',');
                        out.push_str(&look_ahead);
                    }
                }
            }
            '\'' => {
                // Convert lone single quotes to double quotes only when they
                // sit in identifier-like contexts (start of key, end of key,
                // start of string value). We approximate: replace any single
                // quote that's adjacent to whitespace, `{`, `,`, `:`, `[`.
                if matches!(prev, '\0' | '{' | ',' | ':' | '[' | ' ' | '\t' | '\n') {
                    out.push('"');
                } else if let Some(&n) = chars.peek() {
                    if matches!(n, ':' | ',' | '}' | ']' | ' ' | '\t' | '\n') {
                        out.push('"');
                    } else {
                        out.push('\'');
                    }
                } else {
                    out.push('"');
                }
            }
            _ => out.push(c),
        }
        prev = c;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parses_clean_json() {
        assert_eq!(extract_json_value("{\"a\":1}"), json!({ "a": 1 }));
    }

    #[test]
    fn empty_returns_empty_object() {
        assert_eq!(extract_json_value(""), json!({}));
        assert_eq!(extract_json_value("   "), json!({}));
    }

    #[test]
    fn extracts_object_from_prose() {
        let raw = "Sure, here you go:\n```json\n{\"path\":\"\"}\n```\nThanks!";
        assert_eq!(extract_json_value(raw), json!({ "path": "" }));
    }

    #[test]
    fn repairs_trailing_comma() {
        let raw = "{ \"a\": 1, \"b\": 2, }";
        assert_eq!(extract_json_value(raw), json!({ "a": 1, "b": 2 }));
    }

    #[test]
    fn repairs_single_quoted_keys() {
        let raw = "{ 'path': 'reports' }";
        assert_eq!(extract_json_value(raw), json!({ "path": "reports" }));
    }

    #[test]
    fn falls_back_to_string_on_total_failure() {
        let raw = "definitely not json !!";
        assert_eq!(extract_json_value(raw), json!("definitely not json !!"));
    }

    #[test]
    fn balanced_object_handles_nested() {
        let raw = "prefix {\"outer\":{\"inner\":1}} suffix";
        assert_eq!(extract_json_value(raw), json!({ "outer": { "inner": 1 } }));
    }
}
