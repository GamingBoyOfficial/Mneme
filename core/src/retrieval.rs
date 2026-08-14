use serde::{Deserialize, Serialize};
use crate::store::MemoryType;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecallOptions {
    pub limit: usize,
    pub token_budget: usize,
    pub min_confidence: f32,
    pub memory_types: Vec<MemoryType>,
    pub include_expired: bool,
    pub sort_by: SortOrder,
    pub recency_weight: f32,
    pub similarity_weight: f32,
    pub importance_weight: f32,
}

impl Default for RecallOptions {
    fn default() -> Self {
        Self {
            limit: 5,
            token_budget: 500,
            min_confidence: 0.0,
            memory_types: vec![MemoryType::Episodic, MemoryType::Semantic, MemoryType::Procedural],
            include_expired: false,
            sort_by: SortOrder::Hybrid,
            recency_weight: 0.2,
            similarity_weight: 0.6,
            importance_weight: 0.2,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortOrder {
    Relevance,
    Recency,
    Importance,
    Hybrid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievedMemory {
    pub record: crate::store::MemoryRecord,
    pub score: f32,
    pub explanation: String,
}