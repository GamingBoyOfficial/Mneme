class MnemeClient {
  constructor(baseUrl = "http://127.0.0.1:8000") {
    this.baseUrl = baseUrl;
  }

  async remember(content, memoryType = "semantic", extra = {}) {
    const response = await fetch(`${this.baseUrl}/remember`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ content, memory_type: memoryType, ...extra }),
    });
    return response.json();
  }

  async recall(query, limit = 5, queryEmbedding = null) {
    const payload = { query, limit };
    if (queryEmbedding) payload.query_embedding = queryEmbedding;
    const response = await fetch(`${this.baseUrl}/recall`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(payload),
    });
    return response.json();
  }

  async forget(memoryId) {
    const response = await fetch(`${this.baseUrl}/forget`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ memory_id: memoryId }),
    });
    return response.json();
  }

  async export(path) {
    const response = await fetch(`${this.baseUrl}/advanced/export`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ path }),
    });
    return response.json();
  }

  async auditLog() {
    const response = await fetch(`${this.baseUrl}/advanced/audit_log`);
    return response.json();
  }
}

// Support both browser and Node.js
if (typeof module !== "undefined" && module.exports) {
  module.exports = { MnemeClient };
}