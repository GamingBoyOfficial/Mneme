import mneme

# Persistent storage using plain file name
memory = mneme.Store(agent_id="support-bot", backend="support.db")

# Simulate remembering user preferences
memory.remember("User 42 prefers email over Slack for follow-ups", memory_type="semantic")
memory.remember("User 42 had issue with login on 2026-08-10", memory_type="episodic")

# Recall for a new support ticket
context = memory.recall("How does user 42 like to be contacted?")
for item in context:
    print(item["content"])

# Demonstrate export (portability)
memory.advanced.export("support_memory_backup.mneme")
print("Backup exported.")

# Demonstrate right-to-be-forgotten
deleted = memory.advanced.forget_all(user_id="user_42")
print(f"Deleted {deleted} memories for user_42.")