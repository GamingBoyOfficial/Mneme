# Design Decision Log

- **Rust core**: Performance and safety.
- **SQLite default**: Zero‑config, local‑first.
- **Linear scan vector search for MVP**: Avoids heavy dependencies; HNSW can be added later without API change.
- **`.mneme` format as JSON**: Simplicity and inspectability; Parquet planned for scale.
- **Three‑verb API**: Prevents API creep; everything else in `.advanced`.
- **Audit log in same SQLite database**: Single‑file portability and simplicity.
- **FastEmbed as default embedding**: Local, offline, multilingual.