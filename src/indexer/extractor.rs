use crate::db::models::{CodeElement, Relationship};
use tree_sitter::{Node, Tree};

pub struct EntityExtractor<'a> {
    source: &'a [u8],
    file_path: &'a str,
    language: &'a str,
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
