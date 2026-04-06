pub mod entropy;
pub mod modes;
pub mod reader;
pub mod shell;

pub use entropy::EntropyAnalyzer;
#[allow(unused_import)]
pub use modes::{parse_lines_spec, LinesRange, ReadMode};
#[allow(unused_import)]
pub use reader::{FileReader, ReadResult};

pub const CHARS_PER_TOKEN: usize = 4;

pub fn estimate_tokens(text: &str) -> usize {
    text.len() / CHARS_PER_TOKEN
}

pub fn estimate_tokens_precise(text: &str) -> usize {
    let mut token_count = 0;
    let mut in_whitespace = true;

    for c in text.chars() {
        if c.is_whitespace() {
            in_whitespace = true;
        } else if in_whitespace {
            token_count += 1;
            in_whitespace = false;
        }
    }

    if !text.is_empty()
        && !text
            .chars()
            .last()
            .map(|c| c.is_whitespace())
            .unwrap_or(false)
    {
        token_count += 1;
    }

    token_count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_estimate_tokens() {
        assert_eq!(estimate_tokens("hello world"), 2);
        assert_eq!(estimate_tokens("fn foo()"), 2);
    }

    #[test]
    fn test_estimate_tokens_precise() {
        let text = "fn main() {\n    println!(\"hello\");\n}";
        let tokens = estimate_tokens_precise(&text);
        assert!(tokens > 0);
    }
}
