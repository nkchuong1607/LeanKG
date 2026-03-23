use crate::db::models::{CodeElement, Relationship};
use std::path::Path;
use tree_sitter::{Node, Tree};

pub struct EntityExtractor<'a> {
    source: &'a [u8],
    file_path: &'a str,
    language: &'a str,
}

pub fn is_test_file(file_path: &str) -> bool {
    let path = Path::new(file_path);
    let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

    match path.extension().and_then(|e| e.to_str()).unwrap_or("") {
        "go" => file_name.ends_with("_test.go"),
        "py" => file_name.starts_with("test_") || file_name.ends_with("_test.py"),
        "rb" => file_name.ends_with("_spec.rb"),
        "ts" | "js" => {
            file_name.ends_with(".test.ts")
                || file_name.ends_with(".test.js")
                || file_name.ends_with(".spec.ts")
                || file_name.ends_with(".spec.js")
        }
        _ => false,
    }
}

pub fn get_tested_file_path(file_path: &str) -> Option<String> {
    let path = Path::new(file_path);
    let file_name = path.file_name()?.to_str()?;
    let parent = path.parent()?.to_string_lossy().to_string();

    let tested_name = match path.extension().and_then(|e| e.to_str()).unwrap_or("") {
        "go" => {
            if file_name.ends_with("_test.go") {
                Some(file_name.trim_end_matches("_test.go").to_string() + ".go")
            } else {
                None
            }
        }
        "py" => {
            if file_name.starts_with("test_") {
                Some(file_name.strip_prefix("test_").unwrap().to_string())
            } else if file_name.ends_with("_test.py") {
                Some(file_name.trim_end_matches("_test.py").to_string() + ".py")
            } else {
                None
            }
        }
        "rb" => {
            if file_name.ends_with("_spec.rb") {
                Some(file_name.trim_end_matches("_spec.rb").to_string() + ".rb")
            } else {
                None
            }
        }
        "ts" | "js" => {
            if file_name.ends_with(".test.ts") || file_name.ends_with(".test.js") {
                Some(file_name.replace(".test.", "."))
            } else if file_name.ends_with(".spec.ts") || file_name.ends_with(".spec.js") {
                Some(file_name.replace(".spec.", "."))
            } else {
                None
            }
        }
        _ => None,
    }?;

    if parent.is_empty() || parent == "." {
        Some(tested_name)
    } else {
        Some(format!("{}/{}", parent, tested_name))
    }
}

impl<'a> EntityExtractor<'a> {
    pub fn new(source: &'a [u8], file_path: &'a str, language: &'a str) -> Self {
        Self {
            source,
            file_path,
            language,
        }
    }

    pub fn extract(&self, tree: &Tree) -> (Vec<CodeElement>, Vec<Relationship>) {
        let mut elements = Vec::new();
        let mut relationships = Vec::new();
        self.visit_node(tree.root_node(), None, &mut elements, &mut relationships);

        if is_test_file(self.file_path) {
            if let Some(tested_path) = get_tested_file_path(self.file_path) {
                relationships.push(Relationship {
                    id: None,
                    source_qualified: tested_path,
                    target_qualified: self.file_path.to_string(),
                    rel_type: "tested_by".to_string(),
                    metadata: serde_json::json!({}),
                });
            }
        }

        (elements, relationships)
    }

    fn visit_node(
        &self,
        node: Node,
        parent: Option<&str>,
        elements: &mut Vec<CodeElement>,
        relationships: &mut Vec<Relationship>,
    ) {
        let node_type = node.kind();

        match node_type {
            "function_declaration" | "function_definition" | "method_declaration" => {
                if let Some(name) = self.get_node_name(node) {
                    let qualified_name = format!("{}::{}", self.file_path, name);
                    elements.push(CodeElement {
                        qualified_name: qualified_name.clone(),
                        element_type: "function".to_string(),
                        name,
                        file_path: self.file_path.to_string(),
                        line_start: node.start_position().row as u32 + 1,
                        line_end: node.end_position().row as u32 + 1,
                        language: self.language.to_string(),
                        parent_qualified: parent.map(String::from),
                        metadata: serde_json::json!({}),
                    });
                }
            }
            "class_declaration" | "type_declaration" => {
                if let Some(name) = self.get_node_name(node) {
                    let qualified_name = format!("{}::{}", self.file_path, name);
                    elements.push(CodeElement {
                        qualified_name: qualified_name.clone(),
                        element_type: "class".to_string(),
                        name,
                        file_path: self.file_path.to_string(),
                        line_start: node.start_position().row as u32 + 1,
                        line_end: node.end_position().row as u32 + 1,
                        language: self.language.to_string(),
                        parent_qualified: parent.map(String::from),
                        metadata: serde_json::json!({}),
                    });
                }
            }
            "import_declaration" | "import_specifier" => {
                if let Some(source) = self.get_import_source(node) {
                    relationships.push(Relationship {
                        id: None,
                        source_qualified: self.file_path.to_string(),
                        target_qualified: source,
                        rel_type: "imports".to_string(),
                        metadata: serde_json::json!({}),
                    });
                }
            }
            _ => {}
        }

        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                let current_parent = if matches!(
                    node_type,
                    "function_declaration" | "function_definition" | "class_declaration"
                ) {
                    self.get_node_name(node)
                } else {
                    parent.map(String::from)
                };
                self.visit_node(child, current_parent.as_deref(), elements, relationships);
            }
        }
    }

    fn get_node_name(&self, node: Node) -> Option<String> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "identifier" {
                return std::str::from_utf8(self.source.get(child.byte_range())?)
                    .ok()
                    .map(String::from);
            }
        }
        None
    }

    fn get_import_source(&self, node: Node) -> Option<String> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "import_specifier" {
                if let Some(name_node) = child.child_by_field_name("name") {
                    return std::str::from_utf8(self.source.get(name_node.byte_range())?)
                        .ok()
                        .map(String::from);
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tree_sitter::Parser;

    fn parse_go(source: &[u8]) -> Option<tree_sitter::Tree> {
        let mut parser = Parser::new();
        let lang: tree_sitter::Language = tree_sitter_go::LANGUAGE.into();
        parser.set_language(&lang).ok()?;
        parser.parse(source, None)
    }

    fn parse_python(source: &[u8]) -> Option<tree_sitter::Tree> {
        let mut parser = Parser::new();
        let lang: tree_sitter::Language = tree_sitter_python::LANGUAGE.into();
        parser.set_language(&lang).ok()?;
        parser.parse(source, None)
    }

    #[test]
    fn test_extractor_new() {
        let source = b"func foo() {}";
        let extractor = EntityExtractor::new(source, "test.go", "go");
        assert_eq!(extractor.language, "go");
    }

    #[test]
    fn test_extract_go_function() {
        let source = b"package main\nfunc add(a int, b int) int { return a + b }";
        if let Some(tree) = parse_go(source) {
            let extractor = EntityExtractor::new(source, "pkg/math.go", "go");
            let (elements, _) = extractor.extract(&tree);
            assert!(!elements.is_empty());
            let funcs: Vec<_> = elements
                .iter()
                .filter(|e| e.element_type == "function")
                .collect();
            assert!(!funcs.is_empty());
            assert_eq!(funcs[0].name, "add");
        }
    }

    #[test]
    fn test_extract_python_function() {
        let source = b"def greet(name):\n    return f'Hello {name}'";
        if let Some(tree) = parse_python(source) {
            let extractor = EntityExtractor::new(source, "main.py", "python");
            let (elements, _) = extractor.extract(&tree);
            let funcs: Vec<_> = elements
                .iter()
                .filter(|e| e.element_type == "function")
                .collect();
            assert!(!funcs.is_empty());
            assert_eq!(funcs[0].name, "greet");
        }
    }

    #[test]
    fn test_extract_file_path_preserved() {
        let source = b"package p\nfunc f() {}";
        if let Some(tree) = parse_go(source) {
            let extractor = EntityExtractor::new(source, "src/pkg/f.go", "go");
            let (elements, _) = extractor.extract(&tree);
            assert!(!elements.is_empty());
            assert_eq!(elements[0].file_path, "src/pkg/f.go");
        }
    }

    #[test]
    fn test_is_test_file_go() {
        assert!(is_test_file("pkg/math_test.go"));
        assert!(is_test_file("math_test.go"));
        assert!(!is_test_file("pkg/math.go"));
        assert!(!is_test_file("pkg/math_wrong.go"));
    }

    #[test]
    fn test_is_test_file_python() {
        assert!(is_test_file("test_math.py"));
        assert!(is_test_file("math_test.py"));
        assert!(!is_test_file("math.py"));
        assert!(!is_test_file("testmath.py"));
    }

    #[test]
    fn test_is_test_file_ruby() {
        assert!(is_test_file("math_spec.rb"));
        assert!(!is_test_file("math.rb"));
    }

    #[test]
    fn test_is_test_file_typescript() {
        assert!(is_test_file("math.test.ts"));
        assert!(is_test_file("math.spec.ts"));
        assert!(is_test_file("math.test.js"));
        assert!(is_test_file("math.spec.js"));
        assert!(!is_test_file("math.ts"));
    }

    #[test]
    fn test_get_tested_file_path_go() {
        assert_eq!(
            get_tested_file_path("pkg/math_test.go"),
            Some("pkg/math.go".to_string())
        );
        assert_eq!(
            get_tested_file_path("math_test.go"),
            Some("math.go".to_string())
        );
        assert_eq!(get_tested_file_path("pkg/math.go"), None);
    }

    #[test]
    fn test_get_tested_file_path_python() {
        assert_eq!(
            get_tested_file_path("test_math.py"),
            Some("math.py".to_string())
        );
        assert_eq!(
            get_tested_file_path("math_test.py"),
            Some("math.py".to_string())
        );
        assert_eq!(get_tested_file_path("math.py"), None);
    }

    #[test]
    fn test_get_tested_file_path_ruby() {
        assert_eq!(
            get_tested_file_path("math_spec.rb"),
            Some("math.rb".to_string())
        );
        assert_eq!(get_tested_file_path("math.rb"), None);
    }

    #[test]
    fn test_get_tested_file_path_typescript() {
        assert_eq!(
            get_tested_file_path("math.test.ts"),
            Some("math.ts".to_string())
        );
        assert_eq!(
            get_tested_file_path("math.spec.ts"),
            Some("math.ts".to_string())
        );
        assert_eq!(
            get_tested_file_path("math.test.js"),
            Some("math.js".to_string())
        );
        assert_eq!(get_tested_file_path("math.ts"), None);
    }

    #[test]
    fn test_extract_creates_tested_by_relationship() {
        let source = b"package main\nfunc add(a int, b int) int { return a + b }";
        if let Some(tree) = parse_go(source) {
            let extractor = EntityExtractor::new(source, "pkg/math_test.go", "go");
            let (_elements, relationships) = extractor.extract(&tree);

            let tested_by: Vec<_> = relationships
                .iter()
                .filter(|r| r.rel_type == "tested_by")
                .collect();
            assert_eq!(tested_by.len(), 1);
            assert_eq!(tested_by[0].source_qualified, "pkg/math.go");
            assert_eq!(tested_by[0].target_qualified, "pkg/math_test.go");
        }
    }

    #[test]
    fn test_extract_non_test_file_no_tested_by() {
        let source = b"package main\nfunc add(a int, b int) int { return a + b }";
        if let Some(tree) = parse_go(source) {
            let extractor = EntityExtractor::new(source, "pkg/math.go", "go");
            let (_elements, relationships) = extractor.extract(&tree);

            let tested_by: Vec<_> = relationships
                .iter()
                .filter(|r| r.rel_type == "tested_by")
                .collect();
            assert!(tested_by.is_empty());
        }
    }
}
