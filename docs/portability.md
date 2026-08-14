# Portability in Mneme

Mneme's portability engine allows you to export an agent's entire memory and import it into a different agent or framework. This is a first‑class feature, not an afterthought.

## Export

```python
memory.advanced.export("backup.mneme")
```

This creates a `.mneme` archive containing:
- All memory records (episodic + semantic, with embeddings and metadata)
- Full audit log
- Version number and export timestamp

## Import

```python
new_memory = mneme.Store(agent_id="new-agent", backend="new.db")
new_memory.advanced.import_from("backup.mneme")
```

The import is lossless within Mneme’s own schema — same schema in, same schema out.

## Cross‑framework Mapping

When moving memory to/from third‑party frameworks, Mneme uses adapters that map as much as possible and produce a diff report. We never claim blanket zero data loss across frameworks; instead we provide a documented, inspectable mapping.

## CLI

```bash
mneme export --db mneme.db backup.mneme
mneme import --db mneme.db backup.mneme
mneme diff store1.mneme store2.mneme   # Phase 2
```

## Format

The `.mneme` format is JSON (MVP) and will move to Parquet in later versions. It is versioned for backward compatibility.

---

# Architecture Overview

Mneme consists of a Rust core, optional language bindings, a CLI, and an HTTP server.

## Components

- **Memory Store** – hybrid structured + vector storage. Three memory types: episodic, semantic (procedural Phase 2).
- **Retrieval Engine** – semantic similarity + recency + importance scoring with configurable weights and token budget.
- **Consolidation Engine** – deduplication (MVP) and future summarization/decay.
- **Portability Engine** – export/import to `.mneme` archive.
- **Access Control & Audit** – scoping by agent/user/session, permission levels, full audit log.
- **Storage Backend Interface** – SQLite default, Postgres+pgvector Phase 2.

## Data Flow

1. Client (Python/JS/HTTP) sends a `remember` call.
2. Core computes embedding (or uses provided one).
3. Record is stored in SQLite; vector is added to in‑memory vector store.
4. `recall` computes query embedding, performs vector search, applies filters, scores by hybrid formula, and returns top‑k within token budget.
5. Every operation writes an audit event to the same database.

## Backend Swap

Changing the storage backend requires only changing the connection string; no application code changes.