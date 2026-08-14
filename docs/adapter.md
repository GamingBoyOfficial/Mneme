# Writing a New Framework Adapter (Phase 2)

Framework adapters allow Mneme to map its memory schema to external formats (LangChain, MCP, etc.).

## Design

- Implement best‑effort mapping with a diff report.
- Never claim lossless conversion across frameworks; only within Mneme's own schema.
- Adapters live in the `adapters/` directory and are independent of the core.

## Example Adapter Structure

```python
class MyFrameworkAdapter:
    def save_context(self, inputs, outputs):
        # map to Mneme remember()
        pass

    def load_memory_variables(self, inputs):
        # map from Mneme recall()
        pass
```

Full adapters will be added in Phase 2.