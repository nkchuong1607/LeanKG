use axum::{extract::Json, http::StatusCode, response::Html};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct QueryRequest {
    pub query: String,
}

#[derive(Serialize)]
pub struct QueryResponse {
    pub result: Vec<serde_json::Value>,
}

pub async fn index() -> Html<String> {
    Html(r#"
    <!DOCTYPE html>
    <html>
    <head>
        <title>LeanKG - Knowledge Graph</title>
        <style>
            body { font-family: system-ui; max-width: 1200px; margin: 0 auto; padding: 20px; }
            nav { margin-bottom: 20px; padding: 10px; background: #f5f5f5; }
            nav a { margin-right: 15px; text-decoration: none; color: #333; }
            nav a:hover { color: #0066cc; }
            h1 { color: #333; }
            .card { border: 1px solid #ddd; padding: 15px; margin: 10px 0; border-radius: 8px; }
        </style>
    </head>
    <body>
        <h1>LeanKG</h1>
        <nav>
            <a href="/">Dashboard</a>
            <a href="/graph">Graph</a>
            <a href="/browse">Browse</a>
            <a href="/docs">Docs</a>
            <a href="/quality">Quality</a>
        </nav>
        <div class="card">
            <h2>Welcome to LeanKG</h2>
            <p>Lightweight knowledge graph for AI-assisted development.</p>
            <p>Use the CLI to index your codebase:</p>
            <code>leankg init && leankg index ./src</code>
        </div>
    </body>
    </html>
    "#.to_string())
}

pub async fn graph() -> Html<String> {
    Html(r#"
    <!DOCTYPE html>
    <html>
    <head>
        <title>LeanKG - Graph View</title>
        <style>
            body { font-family: system-ui; max-width: 1200px; margin: 0 auto; padding: 20px; }
            nav { margin-bottom: 20px; padding: 10px; background: #f5f5f5; }
            nav a { margin-right: 15px; text-decoration: none; color: #333; }
        </style>
    </head>
    <body>
        <h1>Graph Visualization</h1>
        <nav>
            <a href="/">Dashboard</a>
            <a href="/graph">Graph</a>
            <a href="/browse">Browse</a>
        </nav>
        <div class="card">
            <p>Graph visualization coming soon...</p>
            <p>Use the CLI for full functionality:</p>
            <code>leankg serve</code>
        </div>
    </body>
    </html>
    "#.to_string())
}

pub async fn browse() -> Html<String> {
    Html(r#"
    <!DOCTYPE html>
    <html>
    <head>
        <title>LeanKG - Browse</title>
        <style>
            body { font-family: system-ui; max-width: 1200px; margin: 0 auto; padding: 20px; }
            nav { margin-bottom: 20px; padding: 10px; background: #f5f5f5; }
            nav a { margin-right: 15px; text-decoration: none; color: #333; }
        </style>
    </head>
    <body>
        <h1>Code Browser</h1>
        <nav>
            <a href="/">Dashboard</a>
            <a href="/graph">Graph</a>
            <a href="/browse">Browse</a>
        </nav>
        <div class="card">
            <p>Code browser coming soon...</p>
        </div>
    </body>
    </html>
    "#.to_string())
}

pub async fn docs() -> Html<String> {
    Html(r#"
    <!DOCTYPE html>
    <html>
    <head>
        <title>LeanKG - Docs</title>
        <style>
            body { font-family: system-ui; max-width: 1200px; margin: 0 auto; padding: 20px; }
            nav { margin-bottom: 20px; padding: 10px; background: #f5f5f5; }
            nav a { margin-right: 15px; text-decoration: none; color: #333; }
        </style>
    </head>
    <body>
        <h1>Documentation</h1>
        <nav>
            <a href="/">Dashboard</a>
            <a href="/graph">Graph</a>
            <a href="/browse">Browse</a>
            <a href="/docs">Docs</a>
        </nav>
        <div class="card">
            <p>Documentation viewer coming soon...</p>
        </div>
    </body>
    </html>
    "#.to_string())
}

pub async fn quality() -> Html<String> {
    Html(r#"
    <!DOCTYPE html>
    <html>
    <head>
        <title>LeanKG - Quality</title>
        <style>
            body { font-family: system-ui; max-width: 1200px; margin: 0 auto; padding: 20px; }
            nav { margin-bottom: 20px; padding: 10px; background: #f5f5f5; }
            nav a { margin-right: 15px; text-decoration: none; color: #333; }
        </style>
    </head>
    <body>
        <h1>Code Quality</h1>
        <nav>
            <a href="/">Dashboard</a>
            <a href="/graph">Graph</a>
            <a href="/quality">Quality</a>
        </nav>
        <div class="card">
            <p>Code quality metrics coming soon...</p>
        </div>
    </body>
    </html>
    "#.to_string())
}

pub async fn api_query(Json(req): Json<QueryRequest>) -> Result<Json<QueryResponse>, StatusCode> {
    Ok(Json(QueryResponse {
        result: vec![],
    }))
}
