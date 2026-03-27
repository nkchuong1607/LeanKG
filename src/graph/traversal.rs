use crate::db::models::{CodeElement, Relationship};
use crate::graph::GraphEngine;
use std::collections::{HashSet, VecDeque};

pub struct ImpactAnalyzer<'a> {
    graph: &'a GraphEngine,
}

impl<'a> ImpactAnalyzer<'a> {
    pub fn new(graph: &'a GraphEngine) -> Self {
        Self { graph }
    }

    pub fn calculate_impact_radius(
        &self,
        start_file: &str,
        depth: u32,
    ) -> Result<ImpactResult, Box<dyn std::error::Error>> {
        self.calculate_impact_radius_with_confidence(start_file, depth, 0.0)
    }

    pub fn calculate_impact_radius_with_confidence(
        &self,
        start_file: &str,
        depth: u32,
        min_confidence: f64,
    ) -> Result<ImpactResult, Box<dyn std::error::Error>> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut affected_with_confidence = Vec::new();

        queue.push_back((start_file.to_string(), 0));
        visited.insert(start_file.to_string());

        while let Some((current, current_depth)) = queue.pop_front() {
            if current_depth >= depth {
                continue;
            }

            let relationships = self.graph.get_relationships(&current)?;

            for rel in relationships {
                if rel.confidence < min_confidence {
                    continue;
                }
                if !visited.contains(&rel.target_qualified) {
                    visited.insert(rel.target_qualified.clone());
                    queue.push_back((rel.target_qualified.clone(), current_depth + 1));

                    if let Ok(Some(element)) = self.graph.find_element(&rel.target_qualified) {
                        let severity = rel.severity(current_depth + 1);
                        affected_with_confidence.push(AffectedElementWithConfidence {
                            element,
                            confidence: rel.confidence,
                            severity: severity.to_string(),
                            depth: current_depth + 1,
                        });
                    }
                }
            }

            let dependents = self.graph.get_dependents(&current)?;
            for rel in dependents {
                if rel.confidence < min_confidence {
                    continue;
                }
                if !visited.contains(&rel.source_qualified) {
                    visited.insert(rel.source_qualified.clone());
                    queue.push_back((rel.source_qualified.clone(), current_depth + 1));

                    if let Ok(Some(element)) = self.graph.find_element(&rel.source_qualified) {
                        let severity = rel.severity(current_depth + 1);
                        affected_with_confidence.push(AffectedElementWithConfidence {
                            element,
                            confidence: rel.confidence,
                            severity: severity.to_string(),
                            depth: current_depth + 1,
                        });
                    }
                }
            }
        }

        let affected_elements: Vec<CodeElement> = affected_with_confidence
            .iter()
            .map(|a| a.element.clone())
            .collect();

        Ok(ImpactResult {
            start_file: start_file.to_string(),
            max_depth: depth,
            affected_elements,
            affected_with_confidence,
        })
    }
}

#[derive(Debug, Clone)]
pub struct AffectedElementWithConfidence {
    pub element: CodeElement,
    pub confidence: f64,
    pub severity: String,
    pub depth: u32,
}

#[derive(Debug)]
pub struct ImpactResult {
    pub start_file: String,
    pub max_depth: u32,
    pub affected_elements: Vec<CodeElement>,
    pub affected_with_confidence: Vec<AffectedElementWithConfidence>,
}
