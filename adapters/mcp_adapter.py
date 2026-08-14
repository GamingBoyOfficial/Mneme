"""
Mneme MCP (Model Context Protocol) Adapter

This adapter wraps Mneme's memory store so it can be exposed as tools
in an MCP server. It provides functions that map to Mneme's core verbs
and advanced operations.

Usage:
    from adapters.mcp_adapter import MnemeMCPTools
    tools = MnemeMCPTools(agent_id="my-agent", backend="memory.db")
    # tools.remember(...), tools.recall(...), etc.
"""

import mneme
from typing import Optional, List, Dict, Any


class MnemeMCPTools:
    def __init__(self, agent_id: str, backend: str = "mneme_mcp.db"):
        self.store = mneme.Store(agent_id=agent_id, backend=backend)

    def remember(self, content: str, memory_type: str = "semantic", tags: Optional[List[str]] = None) -> Dict[str, Any]:
        """Store a new memory."""
        tags = tags or []
        result = self.store.remember(
            content,
            memory_type=memory_type,
            tags=tags,  # Note: current Python bindings may not accept tags; adjust if needed
        )
        return result

    def recall(self, query: str, limit: int = 5) -> List[Dict[str, Any]]:
        """Recall relevant memories."""
        return self.store.recall(query, limit=limit)

    def forget(self, memory_id: str) -> None:
        """Forget a specific memory."""
        self.store.forget(memory_id)

    def forget_all(self, user_id: str) -> int:
        """Forget all memories for a user."""
        return self.store.advanced.forget_all(user_id)

    def export(self, path: str) -> None:
        """Export memory to a .mneme archive."""
        self.store.advanced.export(path)

    def import_from(self, path: str) -> int:
        """Import memory from a .mneme archive."""
        return self.store.advanced.import_from(path)

    def audit_log(self, since: Optional[str] = None) -> List[Dict[str, Any]]:
        """Get audit log, optionally filtered by RFC3339 timestamp."""
        return self.store.advanced.audit_log(since=since) if since else self.store.advanced.audit_log()

    def deduplicate(self, threshold: float = 0.9) -> int:
        """Remove duplicate memories."""
        return self.store.advanced.deduplicate(threshold)

    def grant_access(self, granted_agent: str, tags: List[str], permission: str) -> str:
        """Grant access to another agent."""
        return self.store.advanced.grant_access(granted_agent, tags, permission)

    def revoke_access(self, grant_id: str) -> None:
        """Revoke an access grant."""
        self.store.advanced.revoke_access(grant_id)

    def consolidate(self, user_id: str) -> int:
        """Consolidate episodic memories into a semantic summary."""
        return self.store.advanced.consolidate(user_id)