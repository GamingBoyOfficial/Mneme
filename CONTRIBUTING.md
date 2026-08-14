# Contributing to Mneme

We welcome contributions! Mneme is designed so adapters and backends can be added without touching the core.

## Getting Started

1. Fork the repository.
2. Clone your fork.
3. Create a new branch: `git checkout -b feature/your-feature`.
4. Make changes and test.
5. Submit a pull request.

## Development Setup

```bash
cargo build --release
cargo test --all
cd bindings/python && maturin develop --release
```

## Code Style

- **Rust:** `rustfmt`, `clippy`
- **Python:** `black`, `ruff`

## Good First Issues

Look for issues labeled `good first issue`. They are simple and self‑contained.

## Code of Conduct

Please follow our Code of Conduct.