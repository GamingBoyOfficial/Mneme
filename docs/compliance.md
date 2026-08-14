# Mneme API Reference

The public API is intentionally minimal. The only top‑level methods are `remember()`, `recall()`, and `forget()`. Everything else is in the `.advanced` namespace.

## Core Verbs

### `remember(content, memory_type, ...)`

Persist a new memory.

```python
memory.remember(
    "User prefers dark mode",
    memory_type="semantic",
    user_id="user123",
    session_id="sess456",
    confidence=0.9,
    ttl=None,               # optional time‑to‑live in seconds
    tags=["preference"],
    scope=mneme.AccessScope.default(),
    decay_factor=1.0
)
```

### `recall(query, options)`

Retrieve relevant memories. Returns list of objects with `content`, `score`, `explanation`, and `memory_id`.

```python
results = memory.recall(
    "What are the user's preferences?",
    limit=5,               # max number of memories
    token_budget=500,      # approximate token limit of returned content
    min_confidence=0.5,
    memory_types=["semantic", "episodic"],
    include_expired=False,
    sort_by="hybrid"       # relevance | recency | importance | hybrid
)
```

### `forget(memory_id)`

Delete a specific memory. The deletion is recorded in the audit log and is provably complete.

```python
memory.forget(memory_id="...")
```

## Advanced Namespace

All advanced capabilities are under `memory.advanced`.

| Method | Description |
| --- | --- |
| `forget_all(user_id=...)` | Delete all memories for a given user (compliance). |
| `export(path)` | Export the entire memory store to a `.mneme` archive. |
| `import_from(path)` | Import a `.mneme` archive. |
| `audit_log(since=None)` | Retrieve the full audit trail, optionally filtered by RFC3339 timestamp. |
| `deduplicate(threshold=0.9)` | Remove duplicate memories (Jaccard similarity > threshold). |

### Example

```python
memory.advanced.export("backup.mneme")
memory.advanced.import_from("backup.mneme")
memory.advanced.forget_all(user_id="user_42")
log = memory.advanced.audit_log(since="2026-01-01T00:00:00Z")
```