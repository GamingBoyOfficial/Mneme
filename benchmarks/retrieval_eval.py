import mneme
from fastembed import TextEmbedding
import time

def embed(text):
    return list(model.query_embed(text))[0].tolist()

model = TextEmbedding()

# Create store
store = mneme.Store(agent_id="eval-agent", backend=":memory:")

# Synthetic dataset: fact -> query that should retrieve it
facts = [
    "User prefers email over Slack",
    "User likes coffee",
    "User's birthday is in March",
    "User enjoys hiking",
    "User works as a software engineer",
    "User lives in New York",
    "User has a cat named Whiskers",
    "User is allergic to peanuts",
    "User drives a Tesla",
    "User's favorite color is blue"
]

queries = [
    "How does the user like to be contacted?",
    "What does the user like to drink?",
    "When is the user's birthday?",
    "What outdoor activity does the user enjoy?",
    "What is the user's profession?",
    "Where does the user live?",
    "What pet does the user have?",
    "What food allergy does the user have?",
    "What car does the user drive?",
    "What is the user's favorite color?"
]

# Store facts with embeddings
for fact in facts:
    store.remember(fact, memory_type="semantic", embedding=embed(fact))

# Evaluate precision@1: for each query, check if top result is the correct fact
correct = 0
total = 0
latencies = []

for query, expected_fact in zip(queries, facts):
    start = time.time()
    results = store.recall(query, limit=1, query_embedding=embed(query))
    latency = time.time() - start
    latencies.append(latency)
    total += 1
    if results and results[0]["content"] == expected_fact:
        correct += 1

precision = correct / total
avg_latency = sum(latencies) / len(latencies)

print(f"Precision@1: {precision:.2f}")
print(f"Average recall latency: {avg_latency*1000:.2f} ms")
print(f"Total queries: {total}, Correct top-1: {correct}")