use std::fmt::Write;

const DOC_LINE_COUNT_MAX: u32 = 32;
const DOC_PREFIX_WIDTH: usize = 4;
const DOC_WIDTH_MAX: usize = 100;
const INDENT_WIDTH_MAX: usize = 64;
const SCAN_ITERATIONS_MAX: u32 = 16 * 1024;
const TEXT_BYTES_MAX: usize = 8 * 1024;
const TRAILING_PUNCTUATION: [char; 7] = ['.', ',', ')', ';', ':', '!', '?'];
const WORD_COUNT_MAX: u32 = 4_096;

pub fn doc_emit(output: &mut String, indent: &str, text: &str) {
    assert!(
        indent.len() <= INDENT_WIDTH_MAX,
        "indent must be reasonable width"
    );

    let trimmed = text.trim();

    if trimmed.is_empty() {
        return;
    }

    assert!(
        trimmed.len() <= TEXT_BYTES_MAX,
        "doc text exceeds {TEXT_BYTES_MAX} bytes"
    );

    let sanitized = sanitize(trimmed);
    let mut lines: u32 = 0;

    for line in sanitized.lines() {
        lines += 1;

        if lines > DOC_LINE_COUNT_MAX {
            let _ = writeln!(output, "{indent}///");
            let _ = writeln!(output, "{indent}/// (truncated)");

            break;
        }

        let line = line.trim_end();

        if line.is_empty() {
            let _ = writeln!(output, "{indent}///");
        } else {
            line_emit_wrapped(output, indent, line);
        }
    }
}

fn line_emit_wrapped(output: &mut String, indent: &str, line: &str) {
    assert!(!line.is_empty(), "line must not be empty");
    assert!(
        indent.len() < DOC_WIDTH_MAX,
        "indent must leave room for content"
    );

    let width_content_max = DOC_WIDTH_MAX
        .saturating_sub(indent.len() + DOC_PREFIX_WIDTH)
        .max(1);

    let mut current = String::with_capacity(width_content_max);
    let mut words: u32 = 0;

    for word in line.split_whitespace() {
        words += 1;

        assert!(
            words <= WORD_COUNT_MAX,
            "word count exceeds {WORD_COUNT_MAX}"
        );

        if current.is_empty() {
            current.push_str(word);
        } else if current.len() + 1 + word.len() <= width_content_max {
            current.push(' ');
            current.push_str(word);
        } else {
            let _ = writeln!(output, "{indent}/// {current}");

            current.clear();
            current.push_str(word);
        }
    }

    assert!(!current.is_empty(), "wrapped output must not be empty");

    let _ = writeln!(output, "{indent}/// {current}");
}

pub fn doc_emit_or(output: &mut String, indent: &str, text: Option<&str>, fallback: &str) {
    assert!(!fallback.is_empty(), "fallback must not be empty");

    match text {
        Some(value) if !value.trim().is_empty() => doc_emit(output, indent, value),
        _ => doc_emit(output, indent, fallback),
    }
}

fn sanitize(text: &str) -> String {
    assert!(!text.is_empty(), "text must not be empty");

    let normalized = html_normalize(text);
    let escaped = brackets_escape(&normalized);

    urls_wrap(&escaped)
}

fn html_normalize(text: &str) -> String {
    let characters: Vec<char> = text.chars().collect();
    let mut result = String::with_capacity(text.len());
    let mut index: usize = 0;
    let mut iterations: u32 = 0;

    while index < characters.len() {
        iterations += 1;

        assert!(
            iterations <= SCAN_ITERATIONS_MAX,
            "html scan exceeded {SCAN_ITERATIONS_MAX} iterations"
        );

        if characters[index] != '<' {
            result.push(characters[index]);
            index += 1;

            continue;
        }

        let Some(close_index) = char_find(&characters, index + 1, '>') else {
            result.push_str("\\<");
            index += 1;

            continue;
        };

        let inner: String = characters[index + 1..close_index].iter().collect();
        let tag = inner.trim().trim_matches('/').trim().to_ascii_lowercase();

        if tag == "br" || tag.starts_with("br ") {
            result.push('\n');
        }

        index = close_index + 1;
    }

    result
}

fn brackets_escape(text: &str) -> String {
    let mut result = String::with_capacity(text.len() + 16);

    for character in text.chars() {
        match character {
            '[' => result.push_str("\\["),
            ']' => result.push_str("\\]"),
            _ => result.push(character),
        }
    }

    assert!(result.len() >= text.len(), "escaping only grows the text");

    result
}

fn urls_wrap(text: &str) -> String {
    let mut result = String::with_capacity(text.len() + 16);

    for (line_index, line) in text.split('\n').enumerate() {
        if line_index > 0 {
            result.push('\n');
        }

        for (token_index, token) in line.split(' ').enumerate() {
            if token_index > 0 {
                result.push(' ');
            }

            token_wrap(&mut result, token);
        }
    }

    assert!(
        result.len() >= text.len(),
        "url wrapping only grows the text"
    );

    result
}

fn token_wrap(result: &mut String, token: &str) {
    let bare_url = token.contains("://") && !token.contains('<') && !token.contains('`');

    if !bare_url {
        result.push_str(token);

        return;
    }

    let core = token.trim_end_matches(TRAILING_PUNCTUATION);

    assert!(core.len() <= token.len(), "core must not exceed the token");
    assert!(
        token.is_char_boundary(core.len()),
        "core boundary must be valid"
    );

    let suffix = &token[core.len()..];

    result.push('<');
    result.push_str(core);
    result.push('>');
    result.push_str(suffix);
}

fn char_find(characters: &[char], start: usize, target: char) -> Option<usize> {
    let mut index = start;
    let mut iterations: u32 = 0;

    while index < characters.len() {
        iterations += 1;

        assert!(
            iterations <= SCAN_ITERATIONS_MAX,
            "char scan exceeded {SCAN_ITERATIONS_MAX} iterations"
        );

        if characters[index] == target {
            return Some(index);
        }

        index += 1;
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn doc_emit_simple() {
        let mut output = String::new();

        doc_emit(&mut output, "    ", "Get a summoner by PUUID.");

        assert!(
            output == "    /// Get a summoner by PUUID.\n",
            "unexpected output: {output:?}"
        );
    }

    #[test]
    fn doc_emit_escapes_brackets() {
        let mut output = String::new();

        doc_emit(&mut output, "", "See [foo] and [bar].");

        assert!(
            output == "/// See \\[foo\\] and \\[bar\\].\n",
            "unexpected output: {output:?}"
        );
    }

    #[test]
    fn doc_emit_wraps_bare_url() {
        let mut output = String::new();

        doc_emit(&mut output, "", "https://example.com/issues/1");

        assert!(
            output == "/// <https://example.com/issues/1>\n",
            "unexpected output: {output:?}"
        );
    }

    #[test]
    fn doc_emit_wraps_url_with_trailing_period() {
        let mut output = String::new();

        doc_emit(&mut output, "", "See https://example.com/x.");

        assert!(
            output == "/// See <https://example.com/x>.\n",
            "unexpected output: {output:?}"
        );
    }

    #[test]
    fn doc_emit_preserves_inline_code() {
        let mut output = String::new();

        doc_emit(&mut output, "", "Use `puuid` instead.");

        assert!(
            output == "/// Use `puuid` instead.\n",
            "unexpected output: {output:?}"
        );
    }

    #[test]
    fn doc_emit_multiline_paragraphs() {
        let mut output = String::new();

        doc_emit(&mut output, "", "First line.\n\nSecond block.");

        assert!(
            output == "/// First line.\n///\n/// Second block.\n",
            "unexpected output: {output:?}"
        );
    }

    #[test]
    fn html_normalize_drops_tags() {
        let normalized = html_normalize("a <b>bold</b> word");

        assert!(normalized == "a bold word", "unexpected: {normalized:?}");
    }

    #[test]
    fn html_normalize_break_to_newline() {
        let normalized = html_normalize("line one<br>line two");

        assert!(
            normalized == "line one\nline two",
            "unexpected: {normalized:?}"
        );
    }

    #[test]
    fn doc_emit_bounds_line_count() {
        let mut output = String::new();
        let text = "x\n".repeat(64);

        doc_emit(&mut output, "", &text);

        let emitted = output.matches("///").count();

        assert!(
            emitted <= DOC_LINE_COUNT_MAX as usize + 2,
            "line count not bounded"
        );
        assert!(output.contains("(truncated)"), "must mark truncation");
    }

    #[test]
    fn doc_emit_wraps_long_line_within_columns() {
        let mut output = String::new();
        let text = "word ".repeat(40);

        doc_emit(&mut output, "    ", text.trim());

        let mut count: u32 = 0;

        for line in output.lines() {
            count += 1;

            assert!(
                line.chars().count() <= DOC_WIDTH_MAX,
                "line exceeds {DOC_WIDTH_MAX} columns: {line:?}"
            );
        }

        assert!(count > 1, "long text must wrap onto multiple lines");
    }
}
