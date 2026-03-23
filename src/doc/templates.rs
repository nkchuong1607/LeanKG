use std::collections::HashMap;
use std::path::PathBuf;

pub struct TemplateEngine {
    templates_dir: PathBuf,
}

impl TemplateEngine {
    pub fn new(templates_dir: PathBuf) -> Self {
        Self { templates_dir }
    }

    pub fn load_template(&self, name: &str) -> Result<String, Box<dyn std::error::Error>> {
        let template_path = self.templates_dir.join(format!("{}.tmpl", name));
        if template_path.exists() {
            Ok(std::fs::read_to_string(&template_path)?)
        } else {
            Err(format!(
                "Template '{}' not found at {}",
                name,
                template_path.display()
            )
            .into())
        }
    }

    pub fn render_template(template: &str, variables: &HashMap<String, String>) -> String {
        let mut output = template.to_string();
        for (key, value) in variables {
            let placeholder = format!("{{{{{}}}}}", key);
            output = output.replace(&placeholder, value);
        }
        output
    }

    pub fn render_element_template(
        &self,
        template_name: &str,
        qualified_name: &str,
        element_type: &str,
        relationships: &[String],
    ) -> Result<String, Box<dyn std::error::Error>> {
        let template = self.load_template(template_name)?;

        let rel_str = if relationships.is_empty() {
            String::new()
        } else {
            relationships.iter().map(|r| format!("- {}\n", r)).collect()
        };

        let mut variables = HashMap::new();
        variables.insert("qualified_name".to_string(), qualified_name.to_string());
        variables.insert("element_type".to_string(), element_type.to_string());
        variables.insert("relationships".to_string(), rel_str);

        Ok(Self::render_template(&template, &variables))
    }

    pub fn render_agents_template(elements: &[String]) -> String {
        let mut output = String::from("# AGENTS.md\n\n");
        output.push_str("```\n");
        output.push_str("## Codebase Structure\n\n");

        for element in elements {
            output.push_str(&format!("- {}\n", element));
        }

        output.push_str("```\n");
        output
    }

    pub fn render_claude_template(context: &str) -> String {
        let mut output = String::from("# CLAUDE.md\n\n");
        output.push_str("## Project Context\n\n");
        output.push_str(context);
        output.push('\n');
        output
    }

    pub fn render_file_summary(
        file_path: &str,
        elements: &[String],
        relationships: &[String],
    ) -> String {
        let mut output = String::new();
        output.push_str(&format!("# {}\n\n", file_path));
        output.push_str("## Elements\n\n");
        for elem in elements {
            output.push_str(&format!("- {}\n", elem));
        }
        output.push_str("\n## Relationships\n\n");
        for rel in relationships {
            output.push_str(&format!("- {}\n", rel));
        }
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_agents_template_empty() {
        let out = TemplateEngine::render_agents_template(&[]);
        assert!(out.contains("# AGENTS.md"));
        assert!(out.contains("## Codebase Structure"));
    }

    #[test]
    fn test_render_agents_template_with_elements() {
        let elements = vec!["foo".to_string(), "bar".to_string()];
        let out = TemplateEngine::render_agents_template(&elements);
        assert!(out.contains("- foo"));
        assert!(out.contains("- bar"));
    }

    #[test]
    fn test_render_claude_template() {
        let out = TemplateEngine::render_claude_template("test context");
        assert!(out.contains("# CLAUDE.md"));
        assert!(out.contains("test context"));
    }

    #[test]
    fn test_render_file_summary() {
        let elements = vec!["func1".to_string(), "func2".to_string()];
        let relationships = vec!["imports: mod".to_string()];
        let out = TemplateEngine::render_file_summary("src/main.rs", &elements, &relationships);
        assert!(out.contains("# src/main.rs"));
        assert!(out.contains("- func1"));
        assert!(out.contains("- imports: mod"));
    }
}
