use std::collections::HashMap;

pub struct EntropyAnalyzer {
    window_size: usize,
    #[allow(dead_code)]
    jaccard_threshold: f64,
}

impl EntropyAnalyzer {
    pub fn new(window_size: usize, jaccard_threshold: f64) -> Self {
        Self {
            window_size,
            jaccard_threshold,
        }
    }

    pub fn shannon_entropy(&self, text: &str) -> f64 {
        if text.is_empty() {
            return 0.0;
        }

        let mut char_counts: HashMap<char, usize> = HashMap::new();
        for c in text.chars() {
            *char_counts.entry(c).or_insert(0) += 1;
        }

        let len = text.len() as f64;
        let mut entropy = 0.0;

        for count in char_counts.values() {
            let p = *count as f64 / len;
            if p > 0.0 {
                entropy -= p * p.log2();
            }
        }

        entropy
    }

    pub fn normalized_entropy(&self, text: &str) -> f64 {
        let entropy = self.shannon_entropy(text);
        let len = text.len();
        if len <= 1 {
            return entropy;
        }
        let max_entropy = (len as f64).log2();
        if max_entropy > 0.0 {
            entropy / max_entropy
        } else {
            0.0
        }
    }

    pub fn line_entropies(&self, lines: &[&str]) -> Vec<f64> {
        lines
            .iter()
            .map(|line| self.normalized_entropy(line))
            .collect()
    }

    pub fn jaccard_similarity(set1: &[&str], set2: &[&str]) -> f64 {
        if set1.is_empty() && set2.is_empty() {
            return 1.0;
        }
        if set1.is_empty() || set2.is_empty() {
            return 0.0;
        }

        let set1: std::collections::HashSet<_> = set1.iter().collect();
        let set2: std::collections::HashSet<_> = set2.iter().collect();

        let intersection = set1.intersection(&set2).count();
        let union = set1.union(&set2).count();

        if union == 0 {
            return 0.0;
        }

        intersection as f64 / union as f64
    }

    pub fn filter_low_entropy_lines<'a>(&self, lines: &[&'a str], threshold: f64) -> Vec<&'a str> {
        let entropies = self.line_entropies(lines);
        lines
            .iter()
            .zip(entropies.iter())
            .filter(|(_, &entropy)| entropy >= threshold)
            .map(|(line, _)| *line)
            .collect()
    }

    pub fn find_repetitive_patterns(&self, lines: &[&str]) -> Vec<(usize, usize)> {
        let mut patterns = Vec::new();
        let n = lines.len();

        if n < 2 {
            return patterns;
        }

        for window_size in 2..=(n / 2).min(10) {
            for i in 0..=(n - 2 * window_size) {
                let pattern = &lines[i..i + window_size];
                let mut count = 1;

                for j in (i + window_size..=(n - window_size)).step_by(window_size) {
                    if lines[j..j + window_size] == *pattern {
                        count += 1;
                    }
                }

                if count >= 3 {
                    patterns.push((i, window_size));
                }
            }
        }

        patterns
    }

    pub fn kolmogorov_adjustment(&self, entropy: f64, complexity: usize) -> f64 {
        let complexity_factor = 1.0 / (1.0 + (complexity as f64).ln());
        entropy * complexity_factor
    }
}

impl Default for EntropyAnalyzer {
    fn default() -> Self {
        Self::new(256, 0.7)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shannon_entropy() {
        let analyzer = EntropyAnalyzer::default();

        let uniform = "abcdefghijklmnop";
        let repeated = "aaaaaaaa";
        let mixed = "aZ4!@9#";

        let entropy_uniform = analyzer.shannon_entropy(uniform);
        let entropy_repeated = analyzer.shannon_entropy(repeated);
        let entropy_mixed = analyzer.shannon_entropy(mixed);

        assert!(entropy_repeated < entropy_uniform);
        assert!(entropy_mixed < entropy_uniform);
    }

    #[test]
    fn test_filter_low_entropy() {
        let analyzer = EntropyAnalyzer::default();
        let lines = vec!["aaaaa", "xxxxx", "abcde", "fghij"];
        let filtered = analyzer.filter_low_entropy_lines(&lines, 0.5);
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_jaccard_similarity() {
        let a = vec!["x", "y", "z"];
        let b = vec!["y", "z", "w"];
        let similarity = EntropyAnalyzer::jaccard_similarity(&a, &b);
        assert!((similarity - 0.5).abs() < 0.01);
    }
}
