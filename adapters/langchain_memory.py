from typing import Any, Dict, List
import mneme


class MnemeMemory:
    """
    LangChain-compatible memory adapter backed by Mneme.
    Stores conversation history and facts persistently with semantic search.
    This version does NOT require langchain to be installed.
    """

    def __init__(
        self,
        agent_id: str,
        backend: str = "mneme_langchain.db",
        **kwargs
    ):
        self.store = mneme.Store(agent_id=agent_id, backend=backend)

    def save_context(
        self, inputs: Dict[str, Any], outputs: Dict[str, str]
    ) -> None:
        """Save context from this conversation to Mneme."""
        # Store user message as episodic
        if "input" in inputs and inputs["input"]:
            self.store.remember(
                inputs["input"],
                memory_type="episodic",
            )
        # Store assistant message as episodic
        if "output" in outputs and outputs["output"]:
            self.store.remember(
                outputs["output"],
                memory_type="episodic",
            )

    def load_memory_variables(self, inputs: Dict[str, Any]) -> Dict[str, Any]:
        """Return relevant conversation history from Mneme."""
        query = inputs.get("input", "")
        if not query:
            query = "recent conversation"

        results = self.store.recall(query, limit=5)
        # Return list of contents (strings) for simplicity
        history = [item["content"] for item in results]
        return {"history": history}

    def clear(self) -> None:
        """
        Clear all memories for this agent.
        For safety, this is a no-op. Use store.advanced().forget_all(user_id) if needed.
        """
        pass