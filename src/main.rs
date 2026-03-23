mod cli;
mod config;
mod db;
mod doc;
mod graph;
mod indexer;
mod mcp;
mod watcher;
mod web;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "leankg")]
#[command(about = "Lightweight knowledge graph for AI-assisted development")]
pub struct Args {
    #[command(subcommand)]
    pub command: cli::CLICommand,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    match args.command {
        cli::CLICommand::Init { path } => {
            init_project(&path)?;
        }
        cli::CLICommand::Index { path, incremental } => {
            let project_path = find_project_root()?;
            let db_path = project_path.join(".leankg");
            tokio::fs::create_dir_all(&db_path).await?;
            if incremental {
                incremental_index_codebase(path.as_deref().unwrap_or("."), &db_path).await?;
            } else {
                index_codebase(path.as_deref().unwrap_or("."), &db_path).await?;
            }
        }
        cli::CLICommand::Serve { mcp_port: _, web_port } => {
            let project_path = find_project_root()?;
            let db_path = project_path.join(".leankg");
            
            tokio::fs::create_dir_all(&db_path).await.ok();
            
            println!("Starting LeanKG server...");
            println!("Web UI: http://127.0.0.1:{}", web_port);
            
            if let Err(e) = web::start_server(web_port).await {
                eprintln!("Web server error: {}", e);
            }
        }
        cli::CLICommand::Impact { file, depth } => {
            let project_path = find_project_root()?;
            let db_path = project_path.join(".leankg");
            let result = calculate_impact(&file, depth, &db_path).await?;
            println!("Impact radius for {} (depth={}):", file, depth);
            if result.affected_elements.is_empty() {
                println!("  No affected elements found");
            } else {
                for elem in result.affected_elements.iter().take(20) {
                    println!("  - {}", elem.qualified_name);
                }
                if result.affected_elements.len() > 20 {
                    println!("  ... and {} more", result.affected_elements.len() - 20);
                }
            }
        }
        cli::CLICommand::Generate { template: _ } => {
            let project_path = find_project_root()?;
            let db_path = project_path.join(".leankg");
            generate_docs(&db_path).await?;
        }
        cli::CLICommand::Query { query: _ } => {
            println!("Query functionality ready for implementation");
        }
        cli::CLICommand::Install => {
            install_mcp_config()?;
        }
        cli::CLICommand::Status => {
            let project_path = find_project_root()?;
            let db_path = project_path.join(".leankg");
            show_status(&db_path).await?;
        }
        cli::CLICommand::Watch => {
            let project_path = find_project_root()?;
            println!("Starting file watcher for {}...", project_path.display());
            println!("Watch functionality ready for implementation");
        }
        cli::CLICommand::Quality => {
            println!("Quality metrics ready for implementation");
        }
        cli::CLICommand::Export { output: _ } => {
            println!("Export functionality ready for implementation");
        }
        cli::CLICommand::Annotate { element, description, user_story, feature } => {
            let project_path = find_project_root()?;
            let db_path = project_path.join(".leankg");
            annotate_element(&element, &description, user_story.as_deref(), feature.as_deref(), &db_path).await?;
        }
        cli::CLICommand::Link { element, id, kind } => {
            let project_path = find_project_root()?;
            let db_path = project_path.join(".leankg");
            link_element(&element, &id, &kind, &db_path).await?;
        }
        cli::CLICommand::SearchAnnotations { query } => {
            let project_path = find_project_root()?;
            let db_path = project_path.join(".leankg");
            search_annotations(&query, &db_path).await?;
        }
        cli::CLICommand::ShowAnnotations { element } => {
            let project_path = find_project_root()?;
            let db_path = project_path.join(".leankg");
            show_annotations(&element, &db_path).await?;
        }
    }

    Ok(())
}

fn find_project_root() -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    let current_dir = std::env::current_dir()?;
    if current_dir.join(".leankg").exists() || current_dir.join("leankg.yaml").exists() {
        return Ok(current_dir);
    }
    for parent in current_dir.ancestors() {
        if parent.join(".leankg").exists() || parent.join("leankg.yaml").exists() {
            return Ok(parent.to_path_buf());
        }
    }
    Ok(current_dir)
}

fn init_project(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let config = config::ProjectConfig::default();
    let config_yaml = serde_yaml::to_string(&config)?;
    
    std::fs::create_dir_all(path)?;
    std::fs::write(std::path::Path::new(path).join("leankg.yaml"), config_yaml)?;
    
    let readme = r#"# Project

This project uses LeanKG for code intelligence.

## Setup

```bash
leankg init
leankg index ./src
```

## Commands

- `leankg index ./src` - Index codebase
- `leankg serve` - Start server
- `leankg impact <file> --depth 3` - Calculate impact radius
"#;
    std::fs::write(std::path::Path::new(path).join("README.md"), readme)?;
    
    println!("Initialized LeanKG project at {}", path);
    Ok(())
}

async fn index_codebase(path: &str, db_path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    let db = db::init_db(db_path).await?;
    let graph_engine = graph::GraphEngine::new(db);
    let mut parser_manager = indexer::ParserManager::new();
    parser_manager.init_parsers()?;
    
    println!("Indexing codebase at {}...", path);
    
    let files = indexer::find_files(path).await?;
    println!("Found {} files to index", files.len());
    
    let mut indexed = 0;
    for file_path in files {
        match indexer::index_file(&graph_engine, &mut parser_manager, &file_path).await {
            Ok(count) => {
                if count > 0 {
                    indexed += 1;
                    println!("  Indexed {} ({} elements)", file_path, count);
                }
            }
            Err(e) => {
                println!("  Warning: Failed to index {}: {}", file_path, e);
            }
        }
    }
    
    println!("Indexed {} files", indexed);
    Ok(())
}

async fn incremental_index_codebase(path: &str, db_path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    let db = db::init_db(db_path).await?;
    let graph_engine = graph::GraphEngine::new(db);
    let mut parser_manager = indexer::ParserManager::new();
    parser_manager.init_parsers()?;
    
    println!("Performing incremental indexing for {}...", path);
    
    match indexer::incremental_index(&graph_engine, &mut parser_manager, path).await {
        Ok(result) => {
            if result.changed_files.is_empty() && result.dependent_files.is_empty() {
                println!("No changes detected since last index.");
            } else {
                println!("Changed files: {}", result.changed_files.len());
                for f in &result.changed_files {
                    println!("  Modified: {}", f);
                }
                
                println!("Dependent files re-indexed: {}", result.dependent_files.len());
                for f in &result.dependent_files {
                    println!("  Dependent: {}", f);
                }
                
                println!("Total files processed: {}", result.total_files_processed);
                println!("Total elements indexed: {}", result.elements_indexed);
            }
        }
        Err(e) => {
            println!("Incremental index failed: {}. Falling back to full index.", e);
            index_codebase(path, db_path).await?;
        }
    }
    
    Ok(())
}

async fn calculate_impact(file: &str, depth: u32, db_path: &std::path::Path) -> Result<graph::ImpactResult, Box<dyn std::error::Error>> {
    let db = db::init_db(db_path).await?;
    let graph_engine = graph::GraphEngine::new(db);
    let analyzer = graph::ImpactAnalyzer::new(&graph_engine);
    
    let result = analyzer.calculate_impact_radius(file, depth).await?;
    Ok(result)
}

async fn generate_docs(db_path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    let db = db::init_db(db_path).await?;
    let graph_engine = graph::GraphEngine::new(db);
    let generator = doc::DocGenerator::new(graph_engine, std::path::PathBuf::from("./docs"));
    
    let content = generator.generate_agents_md().await?;
    println!("Generated documentation:\n{}", content);
    
    std::fs::create_dir_all("./docs")?;
    std::fs::write("./docs/AGENTS.md", &content)?;
    println!("\nSaved to docs/AGENTS.md");
    
    Ok(())
}

fn install_mcp_config() -> Result<(), Box<dyn std::error::Error>> {
    let mcp_config = serde_json::json!({
        "mcpServers": {
            "leankg": {
                "command": "leankg",
                "args": ["serve"]
            }
        }
    });
    
    std::fs::write(".mcp.json", serde_json::to_string_pretty(&mcp_config)?)?;
    println!("Installed MCP config to .mcp.json");
    
    Ok(())
}

async fn show_status(db_path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    if !db_path.exists() {
        println!("LeanKG not initialized. Run 'leankg init' first.");
        return Ok(());
    }
    
    let db = db::init_db(db_path).await?;
    let graph_engine = graph::GraphEngine::new(db);
    
    let elements = graph_engine.all_elements().await?;
    let relationships = graph_engine.all_relationships().await?;
    
    println!("LeanKG Status:");
    println!("  Database: {}", db_path.display());
    println!("  Elements: {}", elements.len());
    println!("  Relationships: {}", relationships.len());
    
    let files = elements.iter().filter(|e| e.element_type == "file").count();
    let functions = elements.iter().filter(|e| e.element_type == "function").count();
    let classes = elements.iter().filter(|e| e.element_type == "class").count();
    
    println!("  Files: {}", files);
    println!("  Functions: {}", functions);
    println!("  Classes: {}", classes);
    
    let annotations = db::all_business_logic(&db).await?;
    println!("  Annotations: {}", annotations.len());
    
    Ok(())
}

async fn annotate_element(
    element: &str,
    description: &str,
    user_story: Option<&str>,
    feature: Option<&str>,
    db_path: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let db = db::init_db(db_path).await?;
    
    let existing = db::get_business_logic(&db, element).await?;
    
    if existing.is_some() {
        db::update_business_logic(&db, element, description, user_story, feature).await?;
        println!("Updated annotation for '{}'", element);
    } else {
        db::create_business_logic(&db, element, description, user_story, feature).await?;
        println!("Created annotation for '{}'", element);
    }
    
    println!("  Description: {}", description);
    if let Some(story) = user_story {
        println!("  User Story: {}", story);
    }
    if let Some(feat) = feature {
        println!("  Feature: {}", feat);
    }
    
    Ok(())
}

async fn link_element(
    element: &str,
    id: &str,
    kind: &str,
    db_path: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let db = db::init_db(db_path).await?;
    
    let existing = db::get_business_logic(&db, element).await?;
    
    match existing {
        Some(bl) => {
            if kind == "story" {
                let new_desc = if bl.description.starts_with("Linked to") {
                    bl.description
                } else {
                    format!("{} | Linked to story {}", bl.description, id)
                };
                db::update_business_logic(
                    &db,
                    element,
                    &new_desc,
                    Some(id),
                    bl.feature_id.as_deref(),
                ).await?;
            } else {
                let new_desc = if bl.description.starts_with("Linked to") {
                    bl.description
                } else {
                    format!("{} | Linked to feature {}", bl.description, id)
                };
                db::update_business_logic(
                    &db,
                    element,
                    &new_desc,
                    bl.user_story_id.as_deref(),
                    Some(id),
                ).await?;
            }
        }
        None => {
            let description = format!("Linked to {} {}", kind, id);
            if kind == "story" {
                db::create_business_logic(&db, element, &description, Some(id), None).await?;
            } else {
                db::create_business_logic(&db, element, &description, None, Some(id)).await?;
            }
        }
    }
    
    println!("Linked '{}' to {} {}", element, kind, id);
    
    Ok(())
}

async fn search_annotations(query: &str, db_path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    let db = db::init_db(db_path).await?;
    
    let results = db::search_business_logic(&db, query).await?;
    
    if results.is_empty() {
        println!("No annotations found matching '{}'", query);
    } else {
        println!("Found {} annotation(s):", results.len());
        for bl in results {
            println!("\n  Element: {}", bl.element_qualified);
            println!("  Description: {}", bl.description);
            if let Some(story) = bl.user_story_id {
                println!("  User Story: {}", story);
            }
            if let Some(feature) = bl.feature_id {
                println!("  Feature: {}", feature);
            }
        }
    }
    
    Ok(())
}

async fn show_annotations(element: &str, db_path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    let db = db::init_db(db_path).await?;
    
    let result = db::get_business_logic(&db, element).await?;
    
    match result {
        Some(bl) => {
            println!("Annotations for '{}':", element);
            println!("  Description: {}", bl.description);
            if let Some(story) = bl.user_story_id {
                println!("  User Story: {}", story);
            }
            if let Some(feature) = bl.feature_id {
                println!("  Feature: {}", feature);
            }
        }
        None => {
            println!("No annotations found for '{}'", element);
        }
    }
    
    Ok(())
}
