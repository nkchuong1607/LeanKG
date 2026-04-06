use super::entropy::EntropyAnalyzer;
use super::modes::{parse_lines_spec, LinesRange, ReadMode};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

pub struct FileReader {
    entropy_analyzer: EntropyAnalyzer,
    cache: HashSet<String>,
}

impl FileReader {
    pub fn new() -> Self {
        Self {
            entropy_analyzer: EntropyAnalyzer::default(),
            cache: HashSet::new(),
        }
    }

    pub fn read(
        &mut self,
        path: &str,
        mode: ReadMode,
        lines_spec: Option<&str>,
    ) -> Result<ReadResult, String> {
        let content =
            fs::read_to_string(path).map_err(|e| format!("Failed to read file {}: {}", path, e))?;

        let lines: Vec<&str> = content.lines().collect();
        let total_lines = lines.len();
        let total_tokens = self.estimate_tokens(&content);
        let hash = self.compute_hash(&content);

        let is_cached = self.cache.contains(&hash);
        self.cache.insert(hash);

        let result = match mode {
            ReadMode::Adaptive => {
                return Err("Adaptive mode should be resolved before calling read()".into());
            }
            ReadMode::Full => self.read_full(&content, &lines),
            ReadMode::Map => self.read_map(path, &content, &lines)?,
            ReadMode::Signatures => self.read_signatures(path, &content, &lines)?,
            ReadMode::Diff => self.read_diff(&content, &lines),
            ReadMode::Aggressive => self.read_aggressive(&content, &lines),
            ReadMode::Entropy => self.read_entropy(&content, &lines),
            ReadMode::Lines => {
                let ranges = lines_spec.map(parse_lines_spec).unwrap_or_default();
                self.read_lines(&lines, &ranges)
            }
        };

        let tokens = self.estimate_tokens(&result.content);
        let savings_percent = if total_tokens > 0 {
            ((total_tokens - tokens) as f64 / total_tokens as f64 * 100.0)
        } else {
            0.0
        };
        let output_lines = result.content.lines().count();

        Ok(ReadResult {
            path: path.to_string(),
            mode,
            content: result.content,
            tokens,
            total_tokens,
            savings_percent,
            total_lines,
            output_lines,
            is_cached,
            lines_included: result.lines_included,
        })
    }

    fn read_full(&self, _content: &str, lines: &[&str]) -> ReadResult {
        ReadResult {
            path: String::new(),
            mode: ReadMode::Full,
            content: lines.join("\n"),
            tokens: 0,
            total_tokens: 0,
            savings_percent: 0.0,
            total_lines: lines.len(),
            output_lines: lines.len(),
            is_cached: false,
            lines_included: Some(lines.len()),
        }
    }

    fn read_map(
        &mut self,
        path: &str,
        _content: &str,
        lines: &[&str],
    ) -> Result<ReadResult, String> {
        let mut result_lines = Vec::new();
        let mut imports = Vec::new();
        let mut exports = Vec::new();
        let mut signatures = Vec::new();
        let mut current_function = String::new();
        let mut in_function = false;
        let mut brace_count = 0;

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            if is_import_line(trimmed) {
                imports.push(format!("L{}: {}", i + 1, trimmed));
            }

            if is_export_line(trimmed) {
                exports.push(format!("L{}: {}", i + 1, trimmed));
            }

            if is_function_signature(trimmed) {
                if in_function && !current_function.is_empty() {
                    signatures.push(current_function);
                }
                current_function = format!("L{}: {}", i + 1, trimmed);
                in_function = true;
                brace_count = count_braces(trimmed);
            } else if in_function {
                brace_count += count_braces(trimmed);
                if brace_count <= 0 {
                    signatures.push(current_function.clone());
                    current_function.clear();
                    in_function = false;
                } else {
                    current_function.push_str(&format!("\n  {}", trimmed));
                }
            }
        }

        if !current_function.is_empty() {
            signatures.push(current_function);
        }

        let signatures_count = signatures.len();

        result_lines.push(format!(
            "# {} [{}L]",
            Path::new(path)
                .file_name()
                .unwrap_or_default()
                .to_string_lossy(),
            lines.len()
        ));
        result_lines.push(String::new());

        if !imports.is_empty() {
            result_lines.push(format!("deps: {}", imports.join(", ")));
        }

        if !exports.is_empty() {
            result_lines.push(format!("exports: {}", exports.join(", ")));
        }

        result_lines.push(String::new());
        result_lines.push("API:".to_string());

        for sig in signatures {
            result_lines.push(format!("  {}", sig));
        }

        Ok(ReadResult {
            path: path.to_string(),
            mode: ReadMode::Map,
            content: result_lines.join("\n"),
            tokens: 0,
            total_tokens: 0,
            savings_percent: 0.0,
            total_lines: lines.len(),
            output_lines: result_lines.len(),
            is_cached: false,
            lines_included: Some(signatures_count),
        })
    }

    fn read_signatures(
        &mut self,
        path: &str,
        _content: &str,
        lines: &[&str],
    ) -> Result<ReadResult, String> {
        let mut signatures = Vec::new();

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if is_function_signature(trimmed) {
                signatures.push(format!("L{}: {}", i + 1, trimmed));
            }
        }

        let result = format!(
            "{} [{}L]\nsignatures: {}\n",
            Path::new(path)
                .file_name()
                .unwrap_or_default()
                .to_string_lossy(),
            lines.len(),
            signatures.len()
        );

        Ok(ReadResult {
            path: path.to_string(),
            mode: ReadMode::Signatures,
            content: result,
            tokens: 0,
            total_tokens: 0,
            savings_percent: 0.0,
            total_lines: lines.len(),
            output_lines: signatures.len(),
            is_cached: false,
            lines_included: Some(signatures.len()),
        })
    }

    fn read_diff(&self, _content: &str, lines: &[&str]) -> ReadResult {
        ReadResult {
            path: String::new(),
            mode: ReadMode::Diff,
            content: lines[..lines.len().min(50)].join("\n"),
            tokens: 0,
            total_tokens: 0,
            savings_percent: 0.0,
            total_lines: lines.len(),
            output_lines: lines.len().min(50),
            is_cached: false,
            lines_included: Some(lines.len().min(50)),
        }
    }

    fn read_aggressive(&self, _content: &str, lines: &[&str]) -> ReadResult {
        let filtered: Vec<String> = lines
            .iter()
            .filter(|line| !is_noise_line(line.trim()))
            .map(|line| remove_syntax_noise(line))
            .collect();

        ReadResult {
            path: String::new(),
            mode: ReadMode::Aggressive,
            content: filtered.join("\n"),
            tokens: 0,
            total_tokens: 0,
            savings_percent: 0.0,
            total_lines: lines.len(),
            output_lines: filtered.len(),
            is_cached: false,
            lines_included: Some(filtered.len()),
        }
    }

    fn read_entropy(&self, _content: &str, lines: &[&str]) -> ReadResult {
        let threshold = 0.3;
        let filtered = self
            .entropy_analyzer
            .filter_low_entropy_lines(lines, threshold);

        ReadResult {
            path: String::new(),
            mode: ReadMode::Entropy,
            content: filtered.join("\n"),
            tokens: 0,
            total_tokens: 0,
            savings_percent: 0.0,
            total_lines: lines.len(),
            output_lines: filtered.len(),
            is_cached: false,
            lines_included: Some(filtered.len()),
        }
    }

    fn read_lines(&self, lines: &[&str], ranges: &[LinesRange]) -> ReadResult {
        let mut selected = Vec::new();

        for range in ranges {
            for i in (range.start.saturating_sub(1))..range.end.min(lines.len()) {
                selected.push(lines[i]);
            }
        }

        ReadResult {
            path: String::new(),
            mode: ReadMode::Lines,
            content: selected.join("\n"),
            tokens: 0,
            total_tokens: 0,
            savings_percent: 0.0,
            total_lines: lines.len(),
            output_lines: selected.len(),
            is_cached: false,
            lines_included: Some(selected.len()),
        }
    }

    fn estimate_tokens(&self, text: &str) -> usize {
        text.len() / 4
    }

    fn compute_hash(&self, content: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
}

impl Default for FileReader {
    fn default() -> Self {
        Self::new()
    }
}

pub struct ReadResult {
    pub path: String,
    pub mode: ReadMode,
    pub content: String,
    pub tokens: usize,
    pub total_tokens: usize,
    pub savings_percent: f64,
    pub total_lines: usize,
    pub output_lines: usize,
    pub is_cached: bool,
    pub lines_included: Option<usize>,
}

fn is_import_line(line: &str) -> bool {
    let imports = [
        "import ",
        "use ",
        "require(",
        "from ",
        "#include",
        "use crate::",
        "use self::",
    ];
    imports.iter().any(|p| line.starts_with(p))
}

fn is_export_line(line: &str) -> bool {
    line.starts_with("pub ") || line.starts_with("export ") || line.starts_with("module.exports")
}

fn is_function_signature(line: &str) -> bool {
    let markers = [
        "fn ",
        "func ",
        "function ",
        "def ",
        "async fn",
        "pub fn",
        "pub async fn",
        "impl ",
        "struct ",
        "enum ",
        "trait ",
    ];
    let trimmed = line.trim();
    (markers.iter().any(|p| trimmed.starts_with(p)) && trimmed.contains('('))
        || trimmed.starts_with("class ")
}

fn count_braces(line: &str) -> i32 {
    let mut count = 0i32;
    for c in line.chars() {
        match c {
            '{' => count += 1,
            '}' => count -= 1,
            _ => {}
        }
    }
    count
}

fn is_noise_line(line: &str) -> bool {
    let noise = [
        "// ", "/* ", "*/", "# ", "##", "---", "***", "<!--", "-->", "```", "\"\"\"",
    ];
    noise.iter().any(|p| line.starts_with(p)) || line.is_empty()
}

fn remove_syntax_noise(line: &str) -> String {
    let mut result = String::new();
    let mut in_string = false;
    let chars: Vec<char> = line.chars().collect();

    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];

        if c == '"' || c == '\'' {
            in_string = !in_string;
            result.push(c);
        } else if in_string {
            result.push(c);
        } else if c == '/' && i + 1 < chars.len() && chars[i + 1] == '/' {
            break;
        } else if c == '#' && i + 1 < chars.len() && chars[i + 1].is_numeric() {
            i += 1;
            while i < chars.len() && chars[i].is_numeric() {
                i += 1;
            }
            continue;
        } else {
            result.push(c);
        }
        i += 1;
    }

    result.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_import_line() {
        assert!(is_import_line("import foo from 'bar'"));
        assert!(is_import_line("use std::collections"));
        assert!(is_import_line("require('./module')"));
        assert!(!is_import_line("let x = 1"));
    }

    #[test]
    fn test_is_function_signature() {
        assert!(is_function_signature("fn foo() {"));
        assert!(is_function_signature("pub async fn bar() -> Result"));
        assert!(is_function_signature("function test(x: number)"));
        assert!(!is_function_signature("let x = foo();"));
    }
}
