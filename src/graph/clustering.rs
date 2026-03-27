use crate::db::schema::CozoDb;
use crate::graph::GraphEngine;
use std::collections::HashMap;

pub struct CommunityDetector {
    graph_engine: GraphEngine,
}

impl CommunityDetector {
    pub fn new(db: &CozoDb) -> Self {
        Self {
            graph_engine: GraphEngine::new(db.clone()),
        }
    }

    pub fn detect_communities(
        &self,
    ) -> Result<HashMap<String, Cluster>, Box<dyn std::error::Error>> {
        let elements = self.graph_engine.all_elements()?;
        let relationships = self.graph_engine.all_relationships()?;

        if elements.is_empty() {
            return Ok(HashMap::new());
        }

        let mut adjacency: HashMap<String, Vec<String>> = HashMap::new();
        for elem in &elements {
            adjacency.entry(elem.qualified_name.clone()).or_default();
        }

        for rel in &relationships {
            if rel.rel_type == "calls" || rel.rel_type == "imports" {
                adjacency
                    .entry(rel.source_qualified.clone())
                    .or_default()
                    .push(rel.target_qualified.clone());
                adjacency
                    .entry(rel.target_qualified.clone())
                    .or_default()
                    .push(rel.source_qualified.clone());
            }
        }

        let mut labels: HashMap<String, String> = HashMap::new();
        let element_names: Vec<String> =
            elements.iter().map(|e| e.qualified_name.clone()).collect();
        for (i, name) in element_names.iter().enumerate() {
            labels.insert(name.clone(), format!("cluster_{}", i % 10));
        }

        for _ in 0..5 {
            for name in &element_names {
                if let Some(neighbors) = adjacency.get(name) {
                    if neighbors.is_empty() {
                        continue;
                    }
                    let mut label_counts: HashMap<String, usize> = HashMap::new();
                    for neighbor in neighbors {
                        if let Some(label) = labels.get(neighbor) {
                            *label_counts.entry(label.clone()).or_insert(0) += 1;
                        }
                    }
                    if let Some(max_label) = label_counts
                        .into_iter()
                        .max_by_key(|(_, count)| *count)
                        .map(|(label, _)| label)
                    {
                        labels.insert(name.clone(), max_label);
                    }
                }
            }
        }

        let mut clusters: HashMap<String, Cluster> = HashMap::new();
        for elem in &elements {
            let label = labels
                .get(&elem.qualified_name)
                .cloned()
                .unwrap_or_else(|| "unknown".to_string());
            let cluster_label = self.generate_cluster_label(&label, &elem.file_path);

            clusters
                .entry(label.clone())
                .or_insert_with(|| Cluster {
                    id: label.clone(),
                    label: cluster_label,
                    members: Vec::new(),
                    representative_files: Vec::new(),
                })
                .members
                .push(elem.qualified_name.clone());
        }

        for cluster in clusters.values_mut() {
            let mut file_counts: HashMap<String, usize> = HashMap::new();
            for member in &cluster.members {
                if let Some(elem) = elements.iter().find(|e| &e.qualified_name == member) {
                    *file_counts.entry(elem.file_path.clone()).or_insert(0) += 1;
                }
            }
            let mut files: Vec<(String, usize)> = file_counts.into_iter().collect();
            files.sort_by(|a, b| b.1.cmp(&a.1));
            cluster.representative_files =
                files.into_iter().take(3).map(|(path, _)| path).collect();
        }

        Ok(clusters)
    }

    fn generate_cluster_label(&self, cluster_id: &str, file_path: &str) -> String {
        let path_parts: Vec<&str> = file_path.split('/').collect();
        if path_parts.len() >= 2 {
            let dir = path_parts[path_parts.len() - 2];
            let normalized = dir
                .chars()
                .map(|c| {
                    if c.is_alphanumeric() {
                        c.to_ascii_lowercase()
                    } else {
                        '_'
                    }
                })
                .collect::<String>();
            if !normalized.is_empty() && normalized != "_" {
                return normalized;
            }
        }
        cluster_id.replace("cluster_", "module_")
    }

    pub fn assign_clusters_to_elements(&self) -> Result<(), Box<dyn std::error::Error>> {
        let clusters = self.detect_communities()?;

        for cluster in clusters.values() {
            for member_qualified in &cluster.members {
                self.graph_engine.update_element_cluster(
                    member_qualified,
                    Some(cluster.id.clone()),
                    Some(cluster.label.clone()),
                )?;
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Cluster {
    pub id: String,
    pub label: String,
    pub members: Vec<String>,
    pub representative_files: Vec<String>,
}

pub fn get_cluster_stats(clusters: &HashMap<String, Cluster>) -> ClusterStats {
    let total_members: usize = clusters.values().map(|c| c.members.len()).sum();
    let avg_cluster_size = if clusters.is_empty() {
        0.0
    } else {
        total_members as f64 / clusters.len() as f64
    };

    ClusterStats {
        total_clusters: clusters.len(),
        total_members,
        avg_cluster_size,
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ClusterStats {
    pub total_clusters: usize,
    pub total_members: usize,
    pub avg_cluster_size: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cluster_stats() {
        let mut clusters = HashMap::new();
        clusters.insert(
            "c1".to_string(),
            Cluster {
                id: "c1".to_string(),
                label: "auth".to_string(),
                members: vec!["a".to_string(), "b".to_string()],
                representative_files: vec!["auth.rs".to_string()],
            },
        );
        clusters.insert(
            "c2".to_string(),
            Cluster {
                id: "c2".to_string(),
                label: "api".to_string(),
                members: vec!["c".to_string(), "d".to_string(), "e".to_string()],
                representative_files: vec!["api.rs".to_string()],
            },
        );

        let stats = get_cluster_stats(&clusters);
        assert_eq!(stats.total_clusters, 2);
        assert_eq!(stats.total_members, 5);
        assert!((stats.avg_cluster_size - 2.5).abs() < 0.001);
    }
}
