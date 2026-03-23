use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeElement {
    pub qualified_name: String,
    pub element_type: String,
    pub name: String,
    pub file_path: String,
    pub line_start: u32,
    pub line_end: u32,
    pub language: String,
    pub parent_qualified: Option<String>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    pub id: Option<i64>,
    pub source_qualified: String,
    pub target_qualified: String,
    pub rel_type: String,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessLogic {
    pub id: Option<i64>,
    pub element_qualified: String,
    pub description: String,
    pub user_story_id: Option<String>,
    pub feature_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: Option<i64>,
    pub title: String,
    pub content: String,
    pub file_path: String,
    pub generated_from: Vec<String>,
    pub last_updated: String,
}
