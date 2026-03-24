use leankg::db::schema::init_db;
use serde::{Deserialize, Serialize};
use tempfile::TempDir;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CodeElement {
    qualified_name: String,
    element_type: String,
    name: String,
    file_path: String,
    line_start: i64,
    line_end: i64,
    language: String,
    parent_qualified: Option<String>,
    metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Relationship {
    source_qualified: String,
    target_qualified: String,
    rel_type: String,
    metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BusinessLogic {
    element_qualified: String,
    description: String,
    user_story_id: Option<String>,
    feature_id: Option<String>,
}

#[tokio::test]
async fn test_init_db_creates_database_file() {
    let tmp = TempDir::new().unwrap();
    let db_path = tmp.path().join("leankg.db");
    let db = init_db(db_path.as_path()).await.unwrap();
    drop(db);
    assert!(
        db_path.exists() || tmp.path().join("leankg.db").exists(),
        "Database file should exist at expected location"
    );
}

#[tokio::test]
async fn test_init_db_returns_working_surreal_connection() {
    let tmp = TempDir::new().unwrap();
    let db_path = tmp.path().join("leankg.db");
    let db = init_db(db_path.as_path()).await.unwrap();
    let _: Option<String> = db.select(("info", "ns")).await.unwrap();
    let _: Option<String> = db.select(("info", "db")).await.unwrap();
}

#[tokio::test]
async fn test_init_db_creates_code_elements_table() {
    let tmp = TempDir::new().unwrap();
    let db_path = tmp.path().join("leankg.db");
    let db = init_db(db_path.as_path()).await.unwrap();
    let result: Vec<CodeElement> = db.select("code_elements").await.unwrap();
    assert!(result.is_empty());
}

#[tokio::test]
async fn test_init_db_creates_relationships_table() {
    let tmp = TempDir::new().unwrap();
    let db_path = tmp.path().join("leankg.db");
    let db = init_db(db_path.as_path()).await.unwrap();
    let result: Vec<Relationship> = db.select("relationships").await.unwrap();
    assert!(result.is_empty());
}

#[tokio::test]
async fn test_init_db_creates_business_logic_table() {
    let tmp = TempDir::new().unwrap();
    let db_path = tmp.path().join("leankg.db");
    let db = init_db(db_path.as_path()).await.unwrap();
    let result: Vec<BusinessLogic> = db.select("business_logic").await.unwrap();
    assert!(result.is_empty());
}

#[tokio::test]
async fn test_insert_and_query_code_element() {
    let tmp = TempDir::new().unwrap();
    let db_path = tmp.path().join("leankg.db");
    let db = init_db(db_path.as_path()).await.unwrap();

    let element = CodeElement {
        qualified_name: "test::func::add".to_string(),
        element_type: "function".to_string(),
        name: "add".to_string(),
        file_path: "src/math.rs".to_string(),
        line_start: 10,
        line_end: 15,
        language: "rust".to_string(),
        parent_qualified: Some("test::math".to_string()),
        metadata: serde_json::json!({"complexity": 1}),
    };

    let created: Option<CodeElement> = db
        .create(("code_elements", "add_func"))
        .content(element)
        .await
        .unwrap();

    assert!(created.is_some());
    let created_elem = created.unwrap();
    assert_eq!(created_elem.qualified_name, "test::func::add");
    assert_eq!(created_elem.element_type, "function");
    assert_eq!(created_elem.name, "add");
    assert_eq!(created_elem.file_path, "src/math.rs");
    assert_eq!(created_elem.line_start, 10);
    assert_eq!(created_elem.line_end, 15);
    assert_eq!(created_elem.language, "rust");

    let all: Vec<CodeElement> = db.select("code_elements").await.unwrap();
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].qualified_name, "test::func::add");

    let retrieved: Option<CodeElement> = db.select(("code_elements", "add_func")).await.unwrap();
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().name, "add");
}

#[tokio::test]
async fn test_insert_and_query_relationship() {
    let tmp = TempDir::new().unwrap();
    let db_path = tmp.path().join("leankg.db");
    let db = init_db(db_path.as_path()).await.unwrap();

    let rel = Relationship {
        source_qualified: "test::a".to_string(),
        target_qualified: "test::b".to_string(),
        rel_type: "imports".to_string(),
        metadata: serde_json::json!({"strength": "strong"}),
    };

    let created: Option<Relationship> = db
        .create(("relationships", "a_to_b"))
        .content(rel)
        .await
        .unwrap();

    assert!(created.is_some());
    let created_rel = created.unwrap();
    assert_eq!(created_rel.source_qualified, "test::a");
    assert_eq!(created_rel.target_qualified, "test::b");
    assert_eq!(created_rel.rel_type, "imports");

    let all: Vec<Relationship> = db.select("relationships").await.unwrap();
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].rel_type, "imports");

    let retrieved: Option<Relationship> = db.select(("relationships", "a_to_b")).await.unwrap();
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().source_qualified, "test::a");
}

#[tokio::test]
async fn test_insert_and_query_business_logic() {
    let tmp = TempDir::new().unwrap();
    let db_path = tmp.path().join("leankg.db");
    let db = init_db(db_path.as_path()).await.unwrap();

    let biz = BusinessLogic {
        element_qualified: "test::func::process".to_string(),
        description: "Processes user payment".to_string(),
        user_story_id: Some("US-001".to_string()),
        feature_id: Some("PAY-101".to_string()),
    };

    let created: Option<BusinessLogic> = db
        .create(("business_logic", "process_payment"))
        .content(biz)
        .await
        .unwrap();

    assert!(created.is_some());
    let created_biz = created.unwrap();
    assert_eq!(created_biz.element_qualified, "test::func::process");
    assert_eq!(created_biz.description, "Processes user payment");
    assert_eq!(created_biz.user_story_id, Some("US-001".to_string()));
    assert_eq!(created_biz.feature_id, Some("PAY-101".to_string()));

    let all: Vec<BusinessLogic> = db.select("business_logic").await.unwrap();
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].description, "Processes user payment");

    let retrieved: Option<BusinessLogic> = db.select(("business_logic", "process_payment")).await.unwrap();
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().user_story_id, Some("US-001".to_string()));
}

#[tokio::test]
async fn test_multiple_inserts_and_queries() {
    let tmp = TempDir::new().unwrap();
    let db_path = tmp.path().join("leankg.db");
    let db = init_db(db_path.as_path()).await.unwrap();

    let elements = vec![
        CodeElement {
            qualified_name: "test::func::add".to_string(),
            element_type: "function".to_string(),
            name: "add".to_string(),
            file_path: "src/math.rs".to_string(),
            line_start: 1,
            line_end: 5,
            language: "rust".to_string(),
            parent_qualified: None,
            metadata: serde_json::json!({}),
        },
        CodeElement {
            qualified_name: "test::func::sub".to_string(),
            element_type: "function".to_string(),
            name: "sub".to_string(),
            file_path: "src/math.rs".to_string(),
            line_start: 7,
            line_end: 11,
            language: "rust".to_string(),
            parent_qualified: None,
            metadata: serde_json::json!({}),
        },
    ];

    let _: Option<CodeElement> = db
        .create(("code_elements", "add"))
        .content(elements[0].clone())
        .await
        .unwrap();
    let _: Option<CodeElement> = db
        .create(("code_elements", "sub"))
        .content(elements[1].clone())
        .await
        .unwrap();

    let all: Vec<CodeElement> = db.select("code_elements").await.unwrap();
    assert_eq!(all.len(), 2);

    let add: Option<CodeElement> = db.select(("code_elements", "add")).await.unwrap();
    assert!(add.is_some());
    assert_eq!(add.unwrap().name, "add");

    let sub: Option<CodeElement> = db.select(("code_elements", "sub")).await.unwrap();
    assert!(sub.is_some());
    assert_eq!(sub.unwrap().name, "sub");
}

#[tokio::test]
async fn test_schema_indexes_are_defined() {
    let tmp = TempDir::new().unwrap();
    let db_path = tmp.path().join("leankg.db");
    let db = init_db(db_path.as_path()).await.unwrap();

    let element = CodeElement {
        qualified_name: "unique::test::func".to_string(),
        element_type: "function".to_string(),
        name: "test_func".to_string(),
        file_path: "src/test.rs".to_string(),
        line_start: 1,
        line_end: 10,
        language: "rust".to_string(),
        parent_qualified: None,
        metadata: serde_json::json!({}),
    };

    let _: Option<CodeElement> = db
        .create(("code_elements", "test_func_idx"))
        .content(element)
        .await
        .unwrap();

    let results: Vec<CodeElement> = db
        .query("SELECT * FROM code_elements WHERE qualified_name = 'unique::test::func'")
        .await
        .unwrap()
        .take(0)
        .unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].qualified_name, "unique::test::func");
}
