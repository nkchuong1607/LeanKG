pub mod data;

use std::path::PathBuf;

pub struct BenchmarkRunner {
    opencode_path: PathBuf,
    output_dir: PathBuf,
}

impl BenchmarkRunner {
    pub fn new(output_dir: PathBuf) -> Self {
        Self {
            opencode_path: PathBuf::from("opencode"),
            output_dir,
        }
    }

    pub fn run_with_leankg(&self, _prompt: &str) -> data::BenchmarkResult {
        data::BenchmarkResult {
            total_tokens: 0,
            token_percent: 0.0,
            build_time_seconds: 0.0,
            success: false,
        }
    }

    pub fn run_without_leankg(&self, _prompt: &str) -> data::BenchmarkResult {
        data::BenchmarkResult {
            total_tokens: 0,
            token_percent: 0.0,
            build_time_seconds: 0.0,
            success: false,
        }
    }
}
