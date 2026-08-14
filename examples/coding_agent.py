import mneme

# Coding agent that remembers code patterns and user preferences
memory = mneme.Store(agent_id="coding-agent", backend="coding_agent.db")

# Remember user coding preferences
memory.remember("User prefers Python over JavaScript", memory_type="semantic")
memory.remember("User likes detailed docstrings", memory_type="semantic")
memory.remember("Yesterday user refactored login module", memory_type="episodic")

# Recall relevant context for a new coding task
query = "What language does the user prefer and how should I document code?"
context = memory.recall(query, limit=3)
for item in context:
    print(f"[{item['score']:.3f}] {item['content']}")

# Demonstrate export
memory.advanced.export("coding_agent_memory.mneme")
print("Memory exported.")