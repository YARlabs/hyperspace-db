import sys
import os
import time
import random

# Ensure we can import the SDK
sys.path.append(os.path.abspath(os.path.join(os.path.dirname(__file__), '../../sdks/python')))

from hyperspace.client import HyperspaceClient

def main():
    print("Connecting to HyperspaceDB...")
    client = HyperspaceClient(host="localhost:50051", api_key="I_LOVE_HYPERSPACEDB")

    col_name = "py_sdk_test"
    
    # Clean up potentially stale collection
    client.delete_collection(col_name)
    
    print(f"Creating collection '{col_name}'...")
    if client.create_collection(col_name, 8, "l2"):
        print("Collection created.")
    else:
        print("Failed to create collection.")
        return

    print("Inserting vectors...")
    for i in range(10):
        vector = [0.1 * i] * 8
        meta = {"category": "test", "val": str(i)}
        success = client.insert(id=i, vector=vector, metadata=meta, collection=col_name)
        if not success:
            print(f"Failed to insert {i}")

    print("Vectors inserted. Waiting for async index...")
    time.sleep(1.0) 

    print("Searching...")
    query = [0.1] * 8
    results = client.search(vector=query, top_k=5, collection=col_name)
    
    print("Results:")
    for res in results:
        print(f"  ID: {res['id']}, Distance: {res['distance']:.4f}")

    # cleanup
    client.delete_collection(col_name)
    print("Done.")

if __name__ == "__main__":
    main()
