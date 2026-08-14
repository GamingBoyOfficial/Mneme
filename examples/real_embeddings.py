import mneme
from fastembed import TextEmbedding

# Initialize FastEmbed model (downloads on first run, small)
model = TextEmbedding()

# Create a persistent store
memory = mneme.Store(agent_id="semantic-agent", backend="semantic.db")

# Helper to get embedding vector
def embed(text):
    return list(model.query_embed(text))[0]

# Remember some facts with real embeddings
memory.remember("User prefers email over Slack", memory_type="semantic", embedding=embed("User prefers email over Slack"))
memory.remember("User likes coffee", memory_type="semantic", embedding=embed("User likes coffee"))
memory.remember("User's birthday is in March", memory_type="semantic", embedding=embed("User's birthday is in March"))

# Recall using semantic similarity
query = "How does the user like to be contacted?"
context = memory.recall(query, query_embedding=embed(query))
for item in context:
    print(f"[{item['score']:.3f}] {item['content']} ({item['explanation']})")