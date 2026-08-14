# Mneme — Portable Memory Layer for AI Agents

**Structured, consolidating, forgettable memory that works across any framework.**

Mneme gives AI agents a memory system that persists across sessions, supports semantic recall, and can be exported/imported wholesale. It is framework‑agnostic, local‑first, and production‑ready.

---

## ✨ What is Mneme?

Mneme is a memory infrastructure for AI agents, similar to how Redis is for caching. It provides:

- **Three memory types**: episodic (what happened), semantic (facts/preferences), procedural (how to behave).
- **Real semantic search** using local embedding models (FastEmbed).
- **Portability**: export/import your agent's entire memory to a `.mneme` archive.
- **Consolidation & forgetting**: deduplicate memories, summarise episodic into semantic, and delete memory with a full audit trail.
- **Access control & audit**: scoped memory per agent/user/session, with a queryable audit log.

---

## 🚀 Quickstart

### Option 1: Python SDK (build from source)

Prerequisites:
- Rust (stable 1.70+)
- Python 3.8+
- `maturin` installed (`pip install maturin`)

Steps:

```bash
# Clone the repository
git clone https://github.com/GamingBoyOfficial/mneme.git
cd mneme

# Build and install the Python module
pip install .
```

Now you can use Mneme in Python:

```python
import mneme

memory = mneme.Store(agent_id="my-agent", backend="memory.db")

# Remember facts and experiences
memory.remember("User prefers email over Slack", memory_type="semantic")
memory.remember("User clicked on settings", memory_type="episodic")

# Recall relevant memories
context = memory.recall("How does the user like to be contacted?")
print(context)

# Advanced operations
memory.advanced.export("backup.mneme")
memory.advanced.forget_all(user_id="user_42")
```

### Option 2: HTTP Server (any language)

Build and run the server:

```bash
cargo build --release
./target/release/mneme-server
```

The server starts at `http://127.0.0.1:8000` and stores data in `mneme_server.db`.

Then use any HTTP client:

```bash
# Remember
curl -X POST http://127.0.0.1:8000/remember \
  -H "Content-Type: application/json" \
  -d '{"content":"User likes coffee","memory_type":"semantic"}'

# Recall
curl -X POST http://127.0.0.1:8000/recall \
  -H "Content-Type: application/json" \
  -d '{"query":"What does the user like?"}'
```

You can also open the **web dashboard** (`dashboard.html`) in your browser while the server is running.

---

## 📦 Installation (Detailed)

### Python SDK

If you prefer `maturin develop` for development:

```bash
cd bindings/python
maturin develop --release
```

### CLI Tools

```bash
cargo build --release
./target/release/mneme-cli --help
./target/release/mneme-cli export --db mneme.db backup.mneme
./target/release/mneme-cli import --db mneme.db backup.mneme
./target/release/mneme-cli diff backup1.mneme backup2.mneme
```

---

## 🧠 Core API

Three verbs only. Everything else is in `.advanced`.

```python
# The only three verbs that matter day‑to‑day
memory.remember("User prefers email over Slack", memory_type="semantic")
context = memory.recall("how does this user like to be contacted?", limit=5)
memory.forget(memory_id="...")

# Advanced (separate namespace, never crowds the core three)
memory.advanced.forget_all(user_id="user_42")          # compliance
memory.advanced.export("backup.mneme")                 # portability
memory.advanced.import_from("backup.mneme")            # portability
memory.advanced.audit_log(since="2026-01-01")          # trust/compliance
memory.advanced.deduplicate(threshold=0.9)             # consolidation
memory.advanced.grant_access("other-agent", ["tag"], "ReadOnly")  # sharing
memory.advanced.consolidate(user_id="user_42")         # summarise episodic → semantic
```

---

## 📚 Documentation

- [Portability guide](docs/portability.md)
- [Architecture overview](docs/architecture.md)
- [Full API reference](docs/api.md)
- [Compliance & trust](docs/compliance.md)
- [How to write a new storage backend](docs/backend.md)
- [How to write a new framework adapter](docs/adapter.md)
- [Design decision log](docs/decisions.md)
- [Contributing](CONTRIBUTING.md)
- [Roadmap](ROADMAP.md)

---

## ⚡ Performance

Benchmarked on a synthetic dataset (10 queries, 1000 writes, local SQLite, FastEmbed):

- **Retrieval precision@1**: 1.00 (10/10 correct)
- **Average recall latency**: 7.34 ms
- **Average write latency**: 0.088 ms
- **Export/import round‑trip**: lossless, verified by CI

Run benchmarks locally:

```bash
python benchmarks/retrieval_eval.py
python benchmarks/write_bench.py
```

---

## 🌐 HTTP API Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/remember` | Store a memory |
| POST | `/recall` | Retrieve memories |
| POST | `/forget` | Delete a memory by ID |
| POST | `/advanced/export` | Export all memories to file |
| POST | `/advanced/import` | Import memories from file |
| POST | `/advanced/forget_all` | Delete all memories for a user |
| GET | `/advanced/audit_log` | Get full audit log |

The server has real embeddings (FastEmbed) built in, so no client‑side embedding is needed.

---

## 🕸️ Web Dashboard

Open `dashboard.html` in a browser while the server is running. You can add memories, search, view audit log, and export.

---

## 🌍 JavaScript Client

A zero‑dependency client is available in `clients/js/mneme-client.js`. Use it in the browser or Node.js.

```javascript
const { MnemeClient } = require("./clients/js/mneme-client");
const client = new MnemeClient("http://127.0.0.1:8000");
client.remember("User likes pizza").then(console.log);
client.recall("pizza").then(console.log);
```

---

## 🤝 Contributing

Contributions welcome! See [CONTRIBUTING.md](CONTRIBUTING.md).

---

## 📄 License

Apache License, Version 2.0. See [LICENSE](LICENSE).