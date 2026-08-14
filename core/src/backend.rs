use async_trait::async_trait;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;
use crate::store::{MemoryRecord, MemoryType};
use anyhow::Result;
use chrono::{DateTime, Utc};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::str::FromStr;

#[async_trait]
pub trait StorageBackend: Send + Sync {
    async fn store(&self, record: MemoryRecord) -> Result<()>;
    async fn query(&self, query_embedding: &[f32], limit: usize) -> Result<Vec<MemoryRecord>>;
    async fn delete(&self, memory_id: &str) -> Result<()>;
    async fn get_all(&self) -> Result<Vec<MemoryRecord>>;
    async fn migrate(&self) -> Result<()>;
    async fn get_audit_log(&self) -> Result<Vec<crate::audit::AuditEvent>>;
    async fn append_audit(&self, event: crate::audit::AuditEvent) -> Result<()>;
    async fn grant_access(&self, grant: AccessGrant) -> Result<()>;
    async fn revoke_access(&self, grant_id: &str) -> Result<()>;
    async fn get_access_grants(&self) -> Result<Vec<AccessGrant>>;
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AccessGrant {
    pub id: String,
    pub owner_agent: String,
    pub granted_agent: String,
    pub tags: Vec<String>,
    pub permission: String, // "ReadOnly" or "ReadWrite"
    pub created_at: DateTime<Utc>,
}

pub struct SqliteBackend {
    pool: SqlitePool,
    vectors: Arc<Mutex<Vec<(String, Vec<f32>)>>>,
}

impl SqliteBackend {
    pub async fn new(database_path: &str) -> Result<Self> {
        let options = if database_path == ":memory:" || database_path == "sqlite::memory:" {
            SqliteConnectOptions::from_str("sqlite::memory:")?
                .create_if_missing(true)
        } else {
            SqliteConnectOptions::new()
                .filename(database_path)
                .create_if_missing(true)
                .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        };

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await?;

        sqlx::query("PRAGMA journal_mode=WAL;")
            .execute(&pool)
            .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS memories (
                id TEXT PRIMARY KEY,
                agent_id TEXT NOT NULL,
                user_id TEXT NOT NULL,
                session_id TEXT NOT NULL,
                memory_type TEXT NOT NULL,
                content TEXT NOT NULL,
                embedding TEXT,
                timestamp TEXT NOT NULL,
                source TEXT NOT NULL,
                confidence REAL NOT NULL,
                ttl INTEGER,
                tags TEXT NOT NULL,
                scope TEXT NOT NULL,
                decay_factor REAL NOT NULL,
                last_accessed TEXT NOT NULL,
                access_count INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS audit_log (
                id TEXT PRIMARY KEY,
                timestamp TEXT NOT NULL,
                action TEXT NOT NULL,
                agent_id TEXT NOT NULL,
                user_id TEXT NOT NULL,
                session_id TEXT NOT NULL,
                memory_id TEXT,
                details TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS access_grants (
                id TEXT PRIMARY KEY,
                owner_agent TEXT NOT NULL,
                granted_agent TEXT NOT NULL,
                tags TEXT NOT NULL,
                permission TEXT NOT NULL,
                created_at TEXT NOT NULL
            );
            "#
        )
        .execute(&pool)
        .await?;

        Ok(Self {
            pool,
            vectors: Arc::new(Mutex::new(Vec::new())),
        })
    }
}

#[async_trait]
impl StorageBackend for SqliteBackend {
    async fn store(&self, record: MemoryRecord) -> Result<()> {
        if let Some(embedding) = &record.embedding {
            let mut vectors = self.vectors.lock().await;
            vectors.retain(|(id, _)| id != &record.id);
            vectors.push((record.id.clone(), embedding.clone()));
        }

        let embedding_json = record.embedding.as_ref().map(|e| serde_json::to_string(e).unwrap_or_default());
        let tags_json = serde_json::to_string(&record.tags).unwrap_or_default();
        let scope_json = serde_json::to_string(&record.scope).unwrap_or_default();

        sqlx::query(
            r#"
            INSERT OR REPLACE INTO memories
            (id, agent_id, user_id, session_id, memory_type, content, embedding, timestamp, source, confidence, ttl, tags, scope, decay_factor, last_accessed, access_count)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&record.id)
        .bind(&record.agent_id)
        .bind(&record.user_id)
        .bind(&record.session_id)
        .bind(record.memory_type.to_string())
        .bind(&record.content)
        .bind(embedding_json)
        .bind(record.timestamp.to_rfc3339())
        .bind(&record.source)
        .bind(record.confidence)
        .bind(record.ttl)
        .bind(tags_json)
        .bind(scope_json)
        .bind(record.decay_factor)
        .bind(record.last_accessed.to_rfc3339())
        .bind(record.access_count)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn query(&self, query_embedding: &[f32], limit: usize) -> Result<Vec<MemoryRecord>> {
        let all_records = self.get_all().await?;
        let mut scored: Vec<(MemoryRecord, f32)> = Vec::new();
        for record in all_records {
            if let Some(emb) = &record.embedding {
                let sim = cosine_similarity(query_embedding, emb);
                scored.push((record, sim));
            }
        }
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        let results = scored.into_iter().take(limit).map(|(r, _)| r).collect();
        Ok(results)
    }

    async fn delete(&self, memory_id: &str) -> Result<()> {
        let mut vectors = self.vectors.lock().await;
        vectors.retain(|(id, _)| id != memory_id);
        drop(vectors);

        sqlx::query("DELETE FROM memories WHERE id = ?")
            .bind(memory_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn get_all(&self) -> Result<Vec<MemoryRecord>> {
        let rows = sqlx::query_as::<_, (String, String, String, String, String, String, Option<String>, String, String, f32, Option<i64>, String, String, f32, String, u32)>(
            "SELECT id, agent_id, user_id, session_id, memory_type, content, embedding, timestamp, source, confidence, ttl, tags, scope, decay_factor, last_accessed, access_count FROM memories"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|row| {
            let (id, agent_id, user_id, session_id, memory_type, content, embedding, timestamp, source, confidence, ttl, tags, scope, decay_factor, last_accessed, access_count) = row;
            MemoryRecord {
                id,
                agent_id,
                user_id,
                session_id,
                memory_type: MemoryType::from(memory_type),
                content,
                embedding: embedding.and_then(|e| serde_json::from_str(&e).ok()),
                timestamp: DateTime::parse_from_rfc3339(&timestamp).unwrap().with_timezone(&Utc),
                source,
                confidence,
                ttl,
                tags: serde_json::from_str(&tags).unwrap_or_default(),
                scope: serde_json::from_str(&scope).unwrap_or_default(),
                decay_factor,
                last_accessed: DateTime::parse_from_rfc3339(&last_accessed).unwrap().with_timezone(&Utc),
                access_count,
            }
        }).collect())
    }

    async fn migrate(&self) -> Result<()> {
        Ok(())
    }

    async fn get_audit_log(&self) -> Result<Vec<crate::audit::AuditEvent>> {
        let rows = sqlx::query_as::<_, (String, String, String, String, String, String, Option<String>, String)>(
            "SELECT id, timestamp, action, agent_id, user_id, session_id, memory_id, details FROM audit_log ORDER BY timestamp"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|row| {
            crate::audit::AuditEvent {
                id: row.0,
                timestamp: DateTime::parse_from_rfc3339(&row.1).unwrap().with_timezone(&Utc),
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
            "INSERT INTO audit_log (id, timestamp, action, agent_id, user_id, session_id, memory_id, details) VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&event.id)
        .bind(event.timestamp.to_rfc3339())
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

    async fn grant_access(&self, grant: AccessGrant) -> Result<()> {
        let tags_json = serde_json::to_string(&grant.tags).unwrap_or_default();
        sqlx::query(
            "INSERT OR REPLACE INTO access_grants (id, owner_agent, granted_agent, tags, permission, created_at) VALUES (?, ?, ?, ?, ?, ?)"
        )
        .bind(&grant.id)
        .bind(&grant.owner_agent)
        .bind(&grant.granted_agent)
        .bind(tags_json)
        .bind(&grant.permission)
        .bind(grant.created_at.to_rfc3339())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn revoke_access(&self, grant_id: &str) -> Result<()> {
        sqlx::query("DELETE FROM access_grants WHERE id = ?")
            .bind(grant_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn get_access_grants(&self) -> Result<Vec<AccessGrant>> {
        let rows = sqlx::query_as::<_, (String, String, String, String, String, String)>(
            "SELECT id, owner_agent, granted_agent, tags, permission, created_at FROM access_grants"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|row| {
            AccessGrant {
                id: row.0,
                owner_agent: row.1,
                granted_agent: row.2,
                tags: serde_json::from_str(&row.3).unwrap_or_default(),
                permission: row.4,
                created_at: DateTime::parse_from_rfc3339(&row.5).unwrap().with_timezone(&Utc),
            }
        }).collect())
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