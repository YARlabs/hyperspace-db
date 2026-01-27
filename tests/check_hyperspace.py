import sys
import os
sys.path.append('sdks/python')

from hyperspace.client import HyperspaceClient

def check():
    print("Connecting to HyperspaceDB at localhost:50051...")
    try:
        client = HyperspaceClient("localhost:50051")
        print("Performing ping check (TriggerSnapshot)...")
        if client.trigger_snapshot():
             print("✅ Connection Successful!")
        else:
             print("❌ TriggerSnapshot returned False (See Client logs)")
             
        # Optional: Test Dimension Constraint
        try:
             print("Testing search with 8-dim vector...")
             client.search(vector=[0.1]*8)
             print("✅ 8-dim search OK")
        except Exception as e:
             print(f"⚠️ 8-dim search failed: {e}")

    except Exception as e:
        print(f"❌ Connection failed: {e}")

if __name__ == "__main__":
    check()
