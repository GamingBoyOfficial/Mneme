use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::access_control::AccessScope;
use crate::backend::{StorageBackend, AccessGrant};
use crate::embedder::Embedder;
use crate::audit::{AuditAction, AuditEvent};
use crate::retrieval::{RecallOptions, RetrievedMemory, SortOrder};
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MemoryType {
    Episodic,
    Semantic,
    Procedural,
}

impl ToString for MemoryType {
    fn to_string(&self) -> String {
        match self {
            MemoryType::Episodic => "Episodic".to_string(),
            MemoryType::Semantic => "Semantic".to_string(),
            MemoryType::Procedural => "Procedural".to_string(),
        }
    }
}

impl From<String> for MemoryType {
    fn from(s: String) -> Self {
        match s.as_str() {
            "Episodic" => MemoryType::Episodic,
            "Semantic" => MemoryType::Semantic,
            "Procedural" => MemoryType::Procedural,
            _ => MemoryType::Episodic,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryRecord {
    pub id: String,
    pub agent_id: String,
    pub user_id: String,
    pub session_id: String,
    pub memory_type: MemoryType,
    pub content: String,
    pub embedding: Option<Vec<f32>>,
    pub timestamp: DateTime<Utc>,
    pub source: String,
    pub confidence: f32,
    pub ttl: Option<i64>,
    pub tags: Vec<String>,
    pub scope: AccessScope,
    pub decay_factor: f32,
    pub last_accessed: DateTime<Utc>,
    pub access_count: u32,
}

impl MemoryRecord {
    pub fn new(
        agent_id: &str,
        user_id: &str,
        session_id: &str,
        memory_type: MemoryType,
        content: &str,
        source: &str,
        confidence: f32,
        ttl: Option<i64>,
        tags: Vec<String>,
        scope: AccessScope,
        decay_factor: f32,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            agent_id: agent_id.to_string(),
            user_id: user_id.to_string(),
            session_id: session_id.to_string(),
            memory_type,
            content: content.to_string(),
            embedding: None,
            timestamp: now,
            source: source.to_string(),
            confidence,
            ttl,
            tags,
            scope,
            decay_factor,
            last_accessed: now,
            access_count: 0,
        }
    }
}

pub struct MemoryStore {
    backend: Arc<dyn StorageBackend>,
    agent_id: String,
    embedder: Arc<dyn Embedder>,
    _lock: Mutex<()>,
}

impl MemoryStore {
    pub async fn new(
        agent_id: &str,
        backend: Arc<dyn StorageBackend>,
        embedder: Arc<dyn Embedder>,
    ) -> Self {
        Self {
            backend,
            agent_id: agent_id.to_string(),
            embedder,
            _lock: Mutex::new(()),
        }
    }

    pub async fn remember(
        &self,
        content: &str,
        memory_type: MemoryType,
        user_id: &str,
        session_id: &str,
        source: &str,
        confidence: f32,
        ttl: Option<i64>,
        tags: Vec<String>,
        scope: AccessScope,
        decay_factor: f32,
        external_embedding: Option<Vec<f32>>,
    ) -> Result<MemoryRecord> {
        let embedding = external_embedding.or_else(|| Some(self.embedder.embed(content)));

        let mut record = MemoryRecord::new(
            &self.agent_id,
            user_id,
            session_id,
            memory_type,
            content,
            source,
            confidence,
            ttl,
            tags,
            scope,
            decay_factor,
        );
        record.embedding = embedding;

        self.backend.store(record.clone()).await?;

        let event = AuditEvent::new(
            AuditAction::Write,
            &self.agent_id,
            user_id,
            session_id,
            Some(&record.id),
            "remember",
        );
        self.backend.append_audit(event).await?;

        Ok(record)
    }

    pub async fn recall(
        &self,
        query: &str,
        options: RecallOptions,
        external_query_embedding: Option<Vec<f32>>,
    ) -> Result<Vec<RetrievedMemory>> {
        let query_embedding = external_query_embedding.or_else(|| Some(self.embedder.embed(query))).unwrap();

        // Get all records and grants
        let all_records = self.backend.get_all().await?;
        let grants = self.backend.get_access_grants().await?;

        // Filter records accessible by this agent
        let mut accessible_records = Vec::new();
        for record in all_records {
            if record.agent_id == self.agent_id {
                accessible_records.push(record);
                continue;
            }
            // Check if any grant from record.agent_id to self.agent_id exists
            let has_grant = grants.iter().any(|g| {
                g.owner_agent == record.agent_id
                    && g.granted_agent == self.agent_id
                    && (g.tags.is_empty() || record.tags.iter().any(|t| g.tags.contains(t)))
            });
            if has_grant {
                accessible_records.push(record);
            }
        }

        // Apply filters
        let mut records = accessible_records;
        records.retain(|r| options.memory_types.contains(&r.memory_type));
        records.retain(|r| r.confidence >= options.min_confidence);
        if !options.include_expired {
            let now = chrono::Utc::now();
            records.retain(|r| {
                if let Some(ttl) = r.ttl {
                    r.timestamp + chrono::Duration::seconds(ttl) > now
                } else {
                    true
                }
            });
        }

        // Score and sort
        let mut scored: Vec<(MemoryRecord, f32, String)> = Vec::new();
        for record in records {
            let mut score = 0.0;
            let mut explanation = String::new();

            if let (Some(record_emb), Some(query_emb)) = (&record.embedding, Some(&query_embedding)) {
                let sim = cosine_similarity(record_emb, query_emb);
                score += sim * options.similarity_weight;
                explanation.push_str(&format!("similarity={:.2} ", sim));
            } else {
                score += options.similarity_weight;
                explanation.push_str("similarity=1.00 ");
            }

            let age_seconds = (chrono::Utc::now() - record.timestamp).num_seconds() as f32;
            let recency = 1.0 / (1.0 + age_seconds / (24.0 * 3600.0));
            score += recency * options.recency_weight;
            explanation.push_str(&format!("recency={:.2} ", recency));

            score += record.confidence * options.importance_weight;
            explanation.push_str(&format!("confidence={:.2}", record.confidence));

            scored.push((record, score, explanation));
        }

        match options.sort_by {
            SortOrder::Relevance => scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap()),
            SortOrder::Recency => scored.sort_by(|a, b| b.0.timestamp.cmp(&a.0.timestamp)),
            SortOrder::Importance => scored.sort_by(|a, b| b.0.confidence.partial_cmp(&a.0.confidence).unwrap()),
            SortOrder::Hybrid => scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap()),
        }

        let mut token_count = 0;
        let mut result = Vec::new();
        for (mut record, score, explanation) in scored {
            let approx_tokens = record.content.split_whitespace().count();
            if token_count + approx_tokens > options.token_budget {
                continue;
            }
            token_count += approx_tokens;
            record.last_accessed = chrono::Utc::now();
            record.access_count += 1;
            self.backend.store(record.clone()).await?;
            result.push(RetrievedMemory { record, score, explanation });
            if result.len() >= options.limit {
                break;
            }
        }

        let event = AuditEvent::new(
            AuditAction::Read,
            &self.agent_id,
            "",
            "",
            None,
            &format!("recall query: {}", query),
        );
        self.backend.append_audit(event).await?;

        Ok(result)
    }

    pub async fn forget(&self, memory_id: &str) -> Result<()> {
        self.backend.delete(memory_id).await?;
        let event = AuditEvent::new(
            AuditAction::Forget,
            &self.agent_id,
            "",
            "",
            Some(memory_id),
            "forget",
        );
        self.backend.append_audit(event).await?;
        Ok(())
    }

    pub async fn forget_all(&self, user_id: &str) -> Result<usize> {
        let records = self.backend.get_all().await?;
        let mut count = 0;
        for rec in records {
            if rec.user_id == user_id {
                self.backend.delete(&rec.id).await?;
                count += 1;
            }
        }
        let event = AuditEvent::new(
            AuditAction::ForgetAll,
            &self.agent_id,
            user_id,
            "",
            None,
            &format!("forget_all user_id={}", user_id),
        );
        self.backend.append_audit(event).await?;
        Ok(count)
    }

    pub async fn deduplicate(&self, threshold: f32) -> Result<usize> {
        let records = self.backend.get_all().await?;
        let to_remove = crate::consolidation::find_duplicates(&records, threshold);
        let mut count = 0;
        for id in to_remove {
            self.backend.delete(&id).await?;
            count += 1;
        }
        let event = AuditEvent::new(
            AuditAction::Write,
            &self.agent_id,
            "",
            "",
            None,
            &format!("deduplicate threshold={}", threshold),
        );
        self.backend.append_audit(event).await?;
        Ok(count)
    }

    pub async fn grant_access(
        &self,
        granted_agent: &str,
        tags: Vec<String>,
        permission: &str,
    ) -> Result<AccessGrant> {
        let grant = AccessGrant {
            id: Uuid::new_v4().to_string(),
            owner_agent: self.agent_id.clone(),
            granted_agent: granted_agent.to_string(),
            tags,
            permission: permission.to_string(),
            created_at: Utc::now(),
        };
        self.backend.grant_access(grant.clone()).await?;
        let event = AuditEvent::new(
            AuditAction::Write,
            &self.agent_id,
            "",
            "",
            None,
            &format!("grant_access to {} with tags {:?}", granted_agent, grant.tags),
        );
        self.backend.append_audit(event).await?;
        Ok(grant)
    }

    pub async fn revoke_access(&self, grant_id: &str) -> Result<()> {
        self.backend.revoke_access(grant_id).await?;
        let event = AuditEvent::new(
            AuditAction::Write,
            &self.agent_id,
            "",
            "",
            None,
            &format!("revoke_access grant_id={}", grant_id),
        );
        self.backend.append_audit(event).await?;
        Ok(())
    }

    pub async fn consolidate(&self, user_id: &str) -> Result<usize> {
        // Simple extractive summarization: gather episodic memories and create a semantic summary
        let records = self.backend.get_all().await?;
        let mut episodic = records
            .iter()
            .filter(|r| r.memory_type == MemoryType::Episodic && r.user_id == user_id)
            .collect::<Vec<_>>();

        if episodic.is_empty() {
            return Ok(0);
        }

        // Sort by timestamp descending, take up to 100 memories
        episodic.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        let contents: Vec<String> = episodic
            .iter()
            .take(100)
            .map(|r| r.content.clone())
            .collect();

        let summary = contents.join("; ");
        let summary_record = self.remember(
            &summary,
            MemoryType::Semantic,
            user_id,
            "",
            "consolidation",
            0.7,
            None,
            vec!["summary".to_string()],
            AccessScope::default(),
            1.0,
            None,
        )
        .await?;

        // Optionally delete original episodic memories? No, keep for audit.
        Ok(1)
    }

    pub async fn export(&self, path: &str) -> Result<()> {
        let records = self.backend.get_all().await?;
        let audit_log = self.backend.get_audit_log().await?;
        crate::portability::export(records, audit_log, path)?;
        let event = AuditEvent::new(
            AuditAction::Export,
            &self.agent_id,
            "",
            "",
            None,
            &format!("export to {}", path),
        );
        self.backend.append_audit(event).await?;
        Ok(())
    }

    pub async fn import_from(&self, path: &str) -> Result<usize> {
        let archive = crate::portability::import(path)?;
        let mut count = 0;
        for rec in archive.records {
            self.backend.store(rec).await?;
            count += 1;
        }
        for event in archive.audit_log {
            self.backend.append_audit(event).await?;
        }
        let event = AuditEvent::new(
            AuditAction::Import,
            &self.agent_id,
            "",
            "",
            None,
            &format!("import from {}", path),
        );
        self.backend.append_audit(event).await?;
        Ok(count)
    }

    pub async fn get_audit_log(&self) -> Result<Vec<AuditEvent>> {
        self.backend.get_audit_log().await
    }
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let dot: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot / (norm_a * norm_b)
    }
}