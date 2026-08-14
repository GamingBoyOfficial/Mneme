use async_trait::async_trait;
use sqlx::postgres::{PgPool, PgPoolOptions};
use crate::store::{MemoryRecord, MemoryType};
use crate::access_control::AccessScope;
use anyhow::Result;
use chrono::{DateTime, Utc};
use std::sync::Arc;
use tokio::sync::Mutex;
use pgvector::Vector;

pub struct PostgresBackend {
    pool: PgPool,
    _vectors: Arc<Mutex<Vec<(String, Vec<f32>)>>>, // placeholder for any extra state
}

impl PostgresBackend {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await?;

        // Create tables (pgvector extension must be enabled beforehand)
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS memories (
                id TEXT PRIMARY KEY,
                agent_id TEXT NOT NULL,
                user_id TEXT NOT NULL,
                session_id TEXT NOT NULL,
                memory_type TEXT NOT NULL,
                content TEXT NOT NULL,
                embedding vector(384),
                timestamp TIMESTAMPTZ NOT NULL,
                source TEXT NOT NULL,
                confidence REAL NOT NULL,
                ttl BIGINT,
                tags TEXT NOT NULL,
                scope TEXT NOT NULL,
                decay_factor REAL NOT NULL,
                last_accessed TIMESTAMPTZ NOT NULL,
                access_count INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS audit_log (
                id TEXT PRIMARY KEY,
                timestamp TIMESTAMPTZ NOT NULL,
                action TEXT NOT NULL,
                agent_id TEXT NOT NULL,
                user_id TEXT NOT NULL,
                session_id TEXT NOT NULL,
                memory_id TEXT,
                details TEXT NOT NULL
            );
            "#
        )
        .execute(&pool)
        .await?;

        Ok(Self {
            pool,
            _vectors: Arc::new(Mutex::new(Vec::new())),
        })
    }
}

#[async_trait]
impl super::backend::StorageBackend for PostgresBackend {
    async fn store(&self, record: MemoryRecord) -> Result<()> {
        let embedding = match &record.embedding {
            Some(emb) => Some(Vector::from(emb.clone())),
            None => None,
        };
        let tags_json = serde_json::to_string(&record.tags).unwrap_or_default();
        let scope_json = serde_json::to_string(&record.scope).unwrap_or_default();

        sqlx::query(
            r#"
            INSERT INTO memories
            (id, agent_id, user_id, session_id, memory_type, content, embedding, timestamp, source, confidence, ttl, tags, scope, decay_factor, last_accessed, access_count)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
            ON CONFLICT (id) DO UPDATE SET
                agent_id = EXCLUDED.agent_id,
                user_id = EXCLUDED.user_id,
                session_id = EXCLUDED.session_id,
                memory_type = EXCLUDED.memory_type,
                content = EXCLUDED.content,
                embedding = EXCLUDED.embedding,
                timestamp = EXCLUDED.timestamp,
                source = EXCLUDED.source,
                confidence = EXCLUDED.confidence,
                ttl = EXCLUDED.ttl,
                tags = EXCLUDED.tags,
                scope = EXCLUDED.scope,
                decay_factor = EXCLUDED.decay_factor,
                last_accessed = EXCLUDED.last_accessed,
                access_count = EXCLUDED.access_count
            "#
        )
        .bind(&record.id)
        .bind(&record.agent_id)
        .bind(&record.user_id)
        .bind(&record.session_id)
        .bind(record.memory_type.to_string())
        .bind(&record.content)
        .bind(embedding)
        .bind(record.timestamp)
        .bind(&record.source)
        .bind(record.confidence)
        .bind(record.ttl)
        .bind(tags_json)
        .bind(scope_json)
        .bind(record.decay_factor)
        .bind(record.last_accessed)
        .bind(record.access_count)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn query(&self, query_embedding: &[f32], limit: usize) -> Result<Vec<MemoryRecord>> {
        let query_vec = Vector::from(query_embedding.to_vec());
        let rows = sqlx::query_as::<_, (String, String, String, String, String, String, Option<Vector>, DateTime<Utc>, String, f32, Option<i64>, String, String, f32, DateTime<Utc>, i32)>(
            r#"
            SELECT id, agent_id, user_id, session_id, memory_type, content, embedding, timestamp, source, confidence, ttl, tags, scope, decay_factor, last_accessed, access_count
            FROM memories
            ORDER BY embedding <=> $1
            LIMIT $2
            "#
        )
        .bind(query_vec)
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|row| {
            MemoryRecord {
                id: row.0,
                agent_id: row.1,
                user_id: row.2,
                session_id: row.3,
                memory_type: MemoryType::from(row.4),
                content: row.5,
                embedding: row.6.map(|v| v.into()),
                timestamp: row.7,
                source: row.8,
                confidence: row.9,
                ttl: row.10,
                tags: serde_json::from_str(&row.11).unwrap_or_default(),
                scope: serde_json::from_str(&row.12).unwrap_or_default(),
                decay_factor: row.13,
                last_accessed: row.14,
                access_count: row.15 as u32,
            }
        }).collect())
    }

    async fn delete(&self, memory_id: &str) -> Result<()> {
        sqlx::query("DELETE FROM memories WHERE id = $1")
            .bind(memory_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn get_all(&self) -> Result<Vec<MemoryRecord>> {
        let rows = sqlx::query_as::<_, (String, String, String, String, String, String, Option<Vector>, DateTime<Utc>, String, f32, Option<i64>, String, String, f32, DateTime<Utc>, i32)>(
            "SELECT id, agent_id, user_id, session_id, memory_type, content, embedding, timestamp, source, confidence, ttl, tags, scope, decay_factor, last_accessed, access_count FROM memories"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|row| {
            MemoryRecord {
                id: row.0,
                agent_id: row.1,
                user_id: row.2,
                session_id: row.3,
                memory_type: MemoryType::from(row.4),
                content: row.5,
                embedding: row.6.map(|v| v.into()),
                timestamp: row.7,
                source: row.8,
                confidence: row.9,
                ttl: row.10,
                tags: serde_json::from_str(&row.11).unwrap_or_default(),
                scope: serde_json::from_str(&row.12).unwrap_or_default(),
                decay_factor: row.13,
                last_accessed: row.14,
                access_count: row.15 as u32,
            }
        }).collect())
    }

    async fn migrate(&self) -> Result<()> {
        // no-op for now
        Ok(())
    }

    async fn get_audit_log(&self) -> Result<Vec<crate::audit::AuditEvent>> {
        let rows = sqlx::query_as::<_, (String, DateTime<Utc>, String, String, String, String, Option<String>, String)>(
            "SELECT id, timestamp, action, agent_id, user_id, session_id, memory_id, details FROM audit_log ORDER BY timestamp"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|row| {
            crate::audit::AuditEvent {
                id: row.0,
                timestamp: row.1,
                action: crate::audit::AuditAction::from(row.2),
                agent_id: row.3,
                user_id: row.4,
                session_id: row.5,
                memory_id: row.6,
                details: row.7,
            }
        }).collect())
    }

    async fn append_audit(&self, event: crate::audit::AuditEvent) -> Result<()> {
        sqlx::query(
            "INSERT INTO audit_log (id, timestamp, action, agent_id, user_id, session_id, memory_id, details) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"
        )
        .bind(&event.id)
        .bind(event.timestamp)
        .bind(event.action.to_string())
        .bind(&event.agent_id)
        .bind(&event.user_id)
        .bind(&event.session_id)
        .bind(&event.memory_id)
        .bind(&event.details)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}