
import numpy as np
import time
import sys
import os

# Add SDK path
sys.path.append(os.path.abspath("sdks/python"))
from hyperspace import HyperspaceClient

def test_batch_insert():
    client = HyperspaceClient(api_key="I_LOVE_HYPERSPACEDB")
    name = "batch_test"
    dim = 1024
    count = 10_000
    batch_size = 100

    print(f"Creating collection {name}...")
    try:
        client.delete_collection(name)
    except:
        pass
    
    client.create_collection(name, dim, "l2")

    print(f"Generating {count} vectors...")
    vectors = np.random.randn(count, dim).tolist()
    ids = list(range(count))
    metadatas = [{"key": f"val_{i}"} for i in range(count)]

    print("Starting Batch Insert...")
    start_time = time.time()

    for i in range(0, count, batch_size):
        end = min(i + batch_size, count)
        batch_vecs = vectors[i:end]
        batch_ids = ids[i:end]
        batch_meta = metadatas[i:end]
        
        success = client.batch_insert(batch_vecs, batch_ids, batch_meta, collection=name)
        if not success:
            print("Batch failed!")
            return

    total_time = time.time() - start_time
    qps = count / total_time
    print(f"✅ Batch Insert Done!")
    print(f"Vectors: {count}")
    print(f"Time: {total_time:.2f}s")
    print(f"QPS: {qps:.2f}")

    # Check count
    stats = client.get_collection_stats(name)
    print(f"Collection Count: {stats['count']}")
    
    # Cleanup
    # client.delete_collection(name)

if __name__ == "__main__":
    test_batch_insert()
