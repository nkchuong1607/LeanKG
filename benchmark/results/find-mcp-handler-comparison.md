# Benchmark Comparison: find-mcp-handler

## With LeanKG
- Total Tokens: 41316
- Input: 38747
- Cached: 2237
- Files Referenced: ["src/mcp_tools.rs", "src/mcp/tools.rs", "src/mcp/handler.rs", "src/mcp/watcher.rs", "src/mcp/server.rs", "src/mcp/auth.rs", "src/mcp/mod.rs", "src/auth.rs", "src/main.rs", "tests/mcp_tests.rs", "tests/benchmark/src/mcp_tools.rs"]

## Without LeanKG
- Total Tokens: 18731
- Input: 2086
- Cached: 16608
- Files Referenced: ["src/mcp/handler.rs", "src/graph/query.rs", "src/doc_indexer/mod.rs", "src/main.rs", "src/graph/context.rs", "src/graph/traversal.rs", "src/indexer/mod.rs", "src/indexer/extractor.rs", "src/web/handlers.rs", "src/db/models.rs", "src/mcp/server.rs", "src/mcp/tools.rs", "src/indexer/git.rs", "tests/batched_insert_tests.rs", "tests/doc_generation.rs", "tests/mcp_tests.rs", "tests/benchmark/prompts/queries.yaml"]

## Overhead
- Token Delta: 22585
