import mneme
import time

store = mneme.Store(agent_id="bench", backend=":memory:")
n = 1000
start = time.time()
for i in range(n):
    store.remember(f"Memory number {i}", memory_type="episodic")
end = time.time()

total = end - start
avg_ms = (total / n) * 1000
print(f"Wrote {n} memories in {total:.2f}s")
print(f"Average write latency: {avg_ms:.3f} ms")