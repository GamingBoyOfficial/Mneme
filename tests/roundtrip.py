import mneme
import os

def test_export_import_roundtrip():
    # Create first store
    store1 = mneme.Store(agent_id="test", backend=":memory:")
    mem_id = store1.remember("Test memory content", memory_type="semantic")
    assert mem_id is not None

    # Export using advanced property (no parentheses)
    path = "test_roundtrip.mneme"
    store1.advanced.export(path)
    assert os.path.exists(path)

    # Import into new store
    store2 = mneme.Store(agent_id="test", backend=":memory:")
    count = store2.advanced.import_from(path)
    assert count == 1

    # Recall
    results = store2.recall("Test memory content")
    assert len(results) > 0
    assert results[0]["content"] == "Test memory content"

    # Cleanup
    os.remove(path)
    print("Roundtrip test passed!")

if __name__ == "__main__":
    test_export_import_roundtrip()