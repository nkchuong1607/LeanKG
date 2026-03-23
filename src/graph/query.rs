use crate::db::models::{CodeElement, Relationship, BusinessLogic};
use surrealdb::engine::local::Db;
use surrealdb::Surreal;

pub struct GraphEngine {
    db: Surreal<Db>,
}

impl GraphEngine {
    pub fn new(db: Surreal<Db>) -> Self {
        Self { db }
    }

    pub async fn find_element(&self, qualified_name: &str) -> Result<Option<CodeElement>, Box<dyn std::error::Error>> {
        let name = qualified_name.to_string();
        let result: Option<CodeElement> = self.db
            .query("SELECT * FROM code_elements WHERE qualified_name = $name")
            .bind(("name", name))
            .await?
            .take(0)?;
        Ok(result)
    }

    pub async fn get_dependencies(&self, file_path: &str) -> Result<Vec<CodeElement>, Box<dyn std::error::Error>> {
        let path = file_path.to_string();
        let result: Vec<CodeElement> = self.db
            .query("SELECT * FROM code_elements WHERE qualified_name = $path")
            .bind(("path", path))
            .await?
            .take(0)?;
        Ok(result)
    }

    pub async fn get_relationships(&self, source: &str) -> Result<Vec<Relationship>, Box<dyn std::error::Error>> {
        let src = source.to_string();
        let result: Vec<Relationship> = self.db
            .query("SELECT * FROM relationships WHERE source_qualified = $source")
            .bind(("source", src))
            .await?
            .take(0)?;
        Ok(result)
    }

    pub async fn get_dependents(&self, target: &str) -> Result<Vec<Relationship>, Box<dyn std::error::Error>> {
        let tgt = target.to_string();
        let result: Vec<Relationship> = self.db
            .query("SELECT * FROM relationships WHERE target_qualified = $target")
            .bind(("target", tgt))
            .await?
            .take(0)?;
        Ok(result)
    }

    pub async fn all_elements(&self) -> Result<Vec<CodeElement>, Box<dyn std::error::Error>> {
        let result: Vec<CodeElement> = self.db
            .query("SELECT * FROM code_elements")
            .await?
            .take(0)?;
        Ok(result)
    }

    pub async fn all_relationships(&self) -> Result<Vec<Relationship>, Box<dyn std::error::Error>> {
        let result: Vec<Relationship> = self.db
            .query("SELECT * FROM relationships")
            .await?
            .take(0)?;
        Ok(result)
    }

    pub async fn get_children(&self, parent_qualified: &str) -> Result<Vec<CodeElement>, Box<dyn std::error::Error>> {
        let parent = parent_qualified.to_string();
        let result: Vec<CodeElement> = self.db
            .query("SELECT * FROM code_elements WHERE parent_qualified = $parent")
            .bind(("parent", parent))
            .await?
            .take(0)?;
        Ok(result)
    }

    pub async fn get_annotation(&self, element_qualified: &str) -> Result<Option<BusinessLogic>, Box<dyn std::error::Error>> {
        let name = element_qualified.to_string();
        let result: Option<BusinessLogic> = self.db
            .query("SELECT * FROM business_logic WHERE element_qualified = $name")
            .bind(("name", name))
            .await?
            .take(0)?;
        Ok(result)
    }

    pub async fn search_annotations(&self, query: &str) -> Result<Vec<BusinessLogic>, Box<dyn std::error::Error>> {
        let q = format!("%{}%", query.to_lowercase());
        let result: Vec<BusinessLogic> = self.db
            .query("SELECT * FROM business_logic WHERE string::lowercase(description) CONTAINS $q")
            .bind(("q", q))
            .await?
            .take(0)?;
        Ok(result)
    }

    pub async fn all_annotations(&self) -> Result<Vec<BusinessLogic>, Box<dyn std::error::Error>> {
        let result: Vec<BusinessLogic> = self.db
            .query("SELECT * FROM business_logic")
            .await?
            .take(0)?;
        Ok(result)
    }

    pub async fn insert_elements(&self, elements: &[CodeElement]) -> Result<(), Box<dyn std::error::Error>> {
        for element in elements {
            let _: Option<CodeElement> = self.db
                .create("code_elements")
                .content(element.clone())
                .await?;
        }
        Ok(())
    }

    pub async fn insert_relationships(&self, relationships: &[Relationship]) -> Result<(), Box<dyn std::error::Error>> {
        for relationship in relationships {
            let _: Option<Relationship> = self.db
                .create("relationships")
                .content(relationship.clone())
                .await?;
        }
        Ok(())
    }

    pub async fn remove_elements_by_file(&self, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let path = file_path.to_string();
        self.db
            .query("DELETE FROM code_elements WHERE file_path = $path")
            .bind(("path", path))
            .await?;
        Ok(())
    }

    pub async fn remove_relationships_by_source(&self, source: &str) -> Result<(), Box<dyn std::error::Error>> {
        let src = source.to_string();
        self.db
            .query("DELETE FROM relationships WHERE source_qualified = $source")
            .bind(("source", src))
            .await?;
        Ok(())
    }

    pub async fn get_elements_by_file(&self, file_path: &str) -> Result<Vec<CodeElement>, Box<dyn std::error::Error>> {
        let path = file_path.to_string();
        let result: Vec<CodeElement> = self.db
            .query("SELECT * FROM code_elements WHERE file_path = $path")
            .bind(("path", path))
            .await?
            .take(0)?;
        Ok(result)
    }
}
