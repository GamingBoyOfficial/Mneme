# Mneme JavaScript Client

A minimal, zero‑dependency client for the Mneme HTTP server. Use it in the browser or Node.js to store, recall, and manage agent memories.

## Requirements

- Mneme server running (default: `http://127.0.0.1:8000`). Start with:
  ```bash
  cargo run --bin mneme-server
  ```

## Installation

No installation needed. Just copy `mneme-client.js` into your project.

## Usage

### Browser

```html
<script src="mneme-client.js"></script>
<script>
  const client = new MnemeClient("[http://127.0.0.1:8000](http://127.0.0.1:8000)");
</script>
```

### Node.js

```javascript
const { MnemeClient } = require("./mneme-client");
const client = new MnemeClient("[http://127.0.0.1:8000](http://127.0.0.1:8000)");
```

## API

### `new MnemeClient(baseUrl?)`
Creates a new client. `baseUrl` defaults to `http://127.0.0.1:8000`.

### `remember(content, memoryType?, extra?)`
Stores a new memory.
- `content` (string) – the memory text.
- `memoryType` (string, optional) – `"semantic"` or `"episodic"`. Default `"semantic"`.
- `extra` (object, optional) – additional fields like `user_id`, `session_id`, `tags`, `embedding`, `confidence`, `ttl`.

Returns a `Promise` resolving to the created memory object.

```javascript
client.remember("User likes coffee", "semantic").then(console.log);
```

### `recall(query, limit?, queryEmbedding?)`
Retrieves relevant memories.
- `query` (string) – search query.
- `limit` (number, optional) – max results (default 5).
- `queryEmbedding` (array, optional) – precomputed embedding vector. If omitted, the server computes it.

Returns a `Promise` resolving to an array of memory objects with `content`, `score`, `explanation`, and `memory_id`.

```javascript
client.recall("What does the user like?", 3).then(console.log);
```

### `forget(memoryId)`
Deletes a memory by its ID.

```javascript
client.forget("memory-id").then(console.log);
```

### `export(path)`
Exports the entire memory store to a `.mneme` archive.

```javascript
client.export("backup.mneme").then(console.log);
```

### `auditLog()`
Retrieves the full audit log.

```javascript
client.auditLog().then(console.log);
```

## Example

```javascript
const { MnemeClient } = require("./mneme-client");

async function demo() {
  const client = new MnemeClient("[http://127.0.0.1:8000](http://127.0.0.1:8000)");

  // Store facts
  await client.remember("User prefers email over Slack", "semantic");
  await client.remember("User's birthday is in March", "semantic");

  // Recall
  const results = await client.recall("How does the user like to be contacted?", 3);
  console.log(results);
}

demo();
```

## Notes

- The client uses the global `fetch` API. In Node.js 18+, `fetch` is built-in. For older Node versions, you may need a polyfill.
- If you need to send custom embeddings, pass them as arrays to `remember` (`extra.embedding`) or `recall` (`queryEmbedding`).
- The server must have CORS enabled (it is by default in development).

## License

Apache 2.0