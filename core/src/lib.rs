pub mod store;
pub mod backend;
pub mod retrieval;
pub mod consolidation;
pub mod portability;
pub mod access_control;
pub mod audit;
pub mod embedder;

#[cfg(feature = "postgres")]
pub mod backend_postgres;

pub use store::{MemoryStore, MemoryRecord, MemoryType};
pub use backend::{StorageBackend, SqliteBackend};
pub use retrieval::{RecallOptions, RetrievedMemory, SortOrder};
pub use access_control::{AccessScope, Permission};
pub use audit::{AuditEvent, AuditAction};
pub use embedder::{Embedder, HashEmbedder};

#[cfg(feature = "postgres")]
pub use backend_postgres::PostgresBackend;