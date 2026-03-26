use serde::{Deserialize, Serialize};
use std::error::Error;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    pub total_tokens: u32,
    pub token_percent: f32,
    pub build_time_seconds: f32,
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptTask {
    pub id: String,
    pub prompt: String,
    pub expected: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptCategory {
    pub name: String,
    pub description: String,
    pub tasks: Vec<PromptTask>,
}

impl PromptCategory {
    pub fn from_yaml(path: &Path) -> Result<Self, Box<dyn Error>> {
        let content = std::fs::read_to_string(path)?;
        let category: PromptCategory = serde_yaml::from_str(&content)?;
        Ok(category)
    }

    pub fn load_all(prompts_dir: &Path) -> Result<Vec<Self>, Box<dyn Error>> {
        let mut categories = Vec::new();
        for entry in std::fs::read_dir(prompts_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("yaml") {
                categories.push(Self::from_yaml(&path)?);
            }
        }
        Ok(categories)
    }
}
