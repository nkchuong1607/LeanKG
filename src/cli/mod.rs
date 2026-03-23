use clap::Subcommand;

#[derive(Subcommand, Debug)]
pub enum CLICommand {
    /// Initialize a new LeanKG project
    Init {
        #[arg(long, default_value = ".leankg")]
        path: String,
    },
    /// Index the codebase
    Index {
        /// Path to index
        path: Option<String>,
        #[arg(long, short)]
        incremental: bool,
    },
    /// Query the knowledge graph
    Query {
        /// Query string
        query: String,
    },
    /// Generate documentation
    Generate {
        #[arg(long, short)]
        template: Option<String>,
    },
    /// Start MCP server
    Serve {
        #[arg(long, default_value = "3000")]
        mcp_port: u16,
        #[arg(long, default_value = "8080")]
        web_port: u16,
    },
    /// Calculate impact radius
    Impact {
        /// File to analyze
        file: String,
        /// Depth of analysis
        #[arg(long, default_value = "3")]
        depth: u32,
    },
    /// Auto-install MCP config
    Install,
    /// Show index status
    Status,
    /// Start file watcher
    Watch,
    /// Show code quality metrics
    Quality,
    /// Export graph as HTML
    Export {
        #[arg(long, default_value = "graph.html")]
        output: String,
    },
}
