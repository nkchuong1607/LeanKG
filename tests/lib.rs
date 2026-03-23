// Integration tests for LeanKG

mod config_tests {
    use leankg::config::ProjectConfig;

    #[test]
    fn test_config_default() {
        let config = ProjectConfig::default();
        assert_eq!(config.project.name, "my-project");
    }

    #[test]
    fn test_config_default_mcp() {
        let config = ProjectConfig::default();
        assert!(config.mcp.enabled);
        assert_eq!(config.mcp.port, 3000);
    }
}

mod parser_tests {
    use leankg::indexer::ParserManager;

    #[test]
    fn test_parser_manager_creation() {
        let _pm = ParserManager::new();
    }

    #[test]
    fn test_parser_manager_init_and_get() {
        let mut pm = ParserManager::new();
        if pm.init_parsers().is_ok() {
            assert!(pm.get_parser_for_language("go").is_some());
        }
    }
}

mod mcp_tools_tests {
    use leankg::mcp::tools::ToolRegistry;

    #[test]
    fn test_mcp_tools_registry() {
        let tools = ToolRegistry::list_tools();
        assert!(!tools.is_empty());
    }
}
