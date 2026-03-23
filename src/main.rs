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
        cli::CLICommand::Index { path, incremental: _ } => {
            let project_path = find_project_root()?;
            let db_path = project_path.join(".leankg");
            tokio::fs::create_dir_all(&db_path).await?;
            index_codebase(path.as_deref().unwrap_or("."), &db_path).await?;
        }
        cli::CLICommand::Serve { mcp_port, web_port } => {
            let project_path = find_project_root()?;
            let db_path = project_path.join(".leankg");
            
            tokio::fs::create_dir_all(&db_path).await.ok();
            
            println!("Starting LeanKG server...");
            println!("Web UI: http://127.0.0.1:{}", web_port);
            println!("MCP: http://127.0.0.1:{}/mcp", mcp_port);
            
            let web_handle = tokio::spawn(async move {
                if let Err(e) = web::start_server(web_port).await {
                    eprintln!("Web server error: {}", e);
                }
            });

            let _ = web_handle.await;
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
    
    Ok(())
}