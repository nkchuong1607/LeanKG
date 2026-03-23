pub mod parser;
pub mod extractor;

pub use parser::*;
pub use extractor::*;

use crate::graph::GraphEngine;
use std::path::Path;
use walkdir::WalkDir;

pub async fn find_files(root: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut files = Vec::new();
    let extensions = ["go", "ts", "js", "py"];
    
    for entry in WalkDir::new(root)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if extensions.contains(&ext.to_str().unwrap_or("")) {
                    if !path.to_string_lossy().contains("node_modules")
                        && !path.to_string_lossy().contains("vendor")
                        && !path.to_string_lossy().contains(".git")
                    {
                        files.push(path.to_string_lossy().to_string());
                    }
                }
            }
        }
    }
    
    Ok(files)
}

pub async fn index_file(
    graph: &GraphEngine,
    parser_manager: &mut ParserManager,
    file_path: &str,
) -> Result<usize, Box<dyn std::error::Error>> {
    use crate::db::models::{CodeElement, Relationship};
    use surrealdb::engine::local::Db;
    use surrealdb::Surreal;
    
    let content = tokio::fs::read(file_path).await?;
    let source = content.as_slice();
    
    let language = if file_path.ends_with(".go") {
        "go"
    } else if file_path.ends_with(".ts") || file_path.ends_with(".js") {
        "typescript"
    } else if file_path.ends_with(".py") {
        "python"
    } else {
        return Ok(0);
    };
    
    let parser = parser_manager.get_parser_for_language(language);
    let parser = match parser {
        Some(p) => p,
        None => return Ok(0),
    };
    
    let tree = parser.parse(source, None).ok_or("Failed to parse")?;
    
    let extractor = EntityExtractor::new(source, file_path, language);
    let (elements, relationships) = extractor.extract(&tree);
    
    if elements.is_empty() && relationships.is_empty() {
        return Ok(0);
    }
    
    // Get db from graph engine using reflection - for now just return count
    // In full implementation, this would insert into SurrealDB
    
    Ok(elements.len())
}