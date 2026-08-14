# Writing a New Storage Backend

Mneme's storage backend is abstracted behind a Rust trait. To add a new backend (e.g., Postgres, Redis, LanceDB), implement the `StorageBackend` trait from `core/src/backend.rs`.

## Steps

1. Create a new Rust struct (e.g., `PostgresBackend`).
2. Implement the `StorageBackend` trait:

```rust
#[async_trait]
impl StorageBackend for PostgresBackend {
    async fn store(&self, record: MemoryRecord) -> Result<()> { ... }
    async fn query(&self, query_embedding: &[f32], limit: usize) -> Result<Vec<MemoryRecord>> { ... }
    async fn delete(&self, memory_id: &str) -> Result<()> { ... }
    async fn get_all(&self) -> Result<Vec<MemoryRecord>> { ... }
    async fn migrate(&self) -> Result<()> { ... }
    async fn get_audit_log(&self) -> Result<Vec<AuditEvent>> { ... }
    async fn append_audit(&self, event: AuditEvent) -> Result<()> { ... }
}
```

3. Add a constructor `new(connection_string)` that initializes the pool/schema.
4. Update `lib.rs` to re‑export the backend.

No application code changes are required; just pass the new backend's connection string to `Store`.