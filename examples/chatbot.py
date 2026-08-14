import mneme

# Persistent storage using plain file name
memory = mneme.Store(agent_id="chatbot-1", backend="chatbot.db")

# Remember facts and preferences
memory.remember("User's name is Alice", memory_type="semantic")
memory.remember("Alice prefers short, direct answers", memory_type="semantic")
memory.remember("Yesterday Alice asked about weather in Paris", memory_type="episodic")

# Recall relevant memories
context = memory.recall("What do I know about Alice?", limit=3)
for item in context:
    print(f"[{item['score']:.2f}] {item['content']}")

# Advanced operations using property (no parentheses)
memory.advanced.export("chatbot_memory.mneme")
print("Memory exported.")