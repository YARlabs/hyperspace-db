
import os
import time
import subprocess
import sys
import json
from hyperspace import HyperspaceClient

# --- CONFIGURATION ---
SERVER_BIN = os.path.abspath("./target/release/hyperspace-server")
LEADER_PORT = "50060"
FOLLOWER_PORT = "50062"
LEADER_HTTP = "50061"
FOLLOWER_HTTP = "50063"

def test_local_replication():
    print("\n🧪 Starting LOCAL Replication Test (Logic Check)")
    print(f"📁 Binary: {SERVER_BIN}")

    if not os.path.exists(SERVER_BIN):
        print(f"❌ Error: Binary not found. Run 'cargo build --bin hyperspace-server' first.")
        return

    # Cleanup
    subprocess.run("rm -rf test_leader test_follower", shell=True)
    os.makedirs("test_leader", exist_ok=True)
    os.makedirs("test_follower", exist_ok=True)
    subprocess.run("pkill -9 hyperspace-server", shell=True)
    time.sleep(1)

    # 1. Start Leader
    print(f"1. Starting Leader on :{LEADER_PORT}...")
    leader_sub = subprocess.Popen(
        [
            SERVER_BIN, 
            "--port", LEADER_PORT, 
            "--http-port", LEADER_HTTP, 
            "--role", "leader",
            "--node-id", "local-leader-node"
        ],
        cwd="test_leader",
        stdout=open("test_leader/out.log", "w"),
        stderr=subprocess.STDOUT
    )
    time.sleep(15) # Wait for leader models to load

    # 2. Start Follower
    print(f"2. Starting Follower on :{FOLLOWER_PORT} -> Leader :{LEADER_PORT}")
    follower_sub = subprocess.Popen(
        [
            SERVER_BIN, 
            "--port", FOLLOWER_PORT, 
            "--http-port", FOLLOWER_HTTP, 
            "--role", "follower", 
            "--leader", f"http://127.0.0.1:{LEADER_PORT}",
            "--node-id", "local-follower-node"
        ],
        cwd="test_follower",
        stdout=open("test_follower/out.log", "w"),
        stderr=subprocess.STDOUT
    )
    time.sleep(20) # Wait for follower models and sync handshake

    try:
        client_leader = HyperspaceClient(f"127.0.0.1:{LEADER_PORT}")
        client_follower = HyperspaceClient(f"127.0.0.1:{FOLLOWER_PORT}")

        # 3. Create Collection on Leader
        print("3. Creating collection 'sync_test' on Leader...")
        client_leader.create_collection("sync_test", dimension=8, metric="l2")
        
        # 4. Insert data into Leader
        print("4. Inserting data into Leader...")
        vec = [0.1] * 8
        client_leader.insert("sync_test", vector=vec, id=1, metadata={"name": "test_vector"})
        
        print("⏳ Waiting for replication (5s)...")
        time.sleep(5)

        # 5. Verify on Follower
        print("5. Querying Follower for 'sync_test' collection...")
        try:
            collections = client_follower.list_collections()
            print(f"📋 Collections on Follower: {collections}")
            
            # Note: List collections might return strings or dicts depending on SDK version
            has_col = any(c == "sync_test" or (isinstance(c, dict) and c.get('name') == "sync_test") for c in collections)
            
            if has_col:
                print("✅ SUCCESS: Collection replicated!")
                # Check data
                data = client_follower.peek("sync_test", limit=5)
                if data:
                    print(f"✅ SUCCESS: Data found on follower: {data}")
                else:
                    print("❌ FAIL: Collection exists but data is missing.")
            else:
                print("❌ FAIL: Collection not found on follower.")
                print("\n--- Follower Log Snippet ---")
                with open("test_follower/out.log", "r") as f:
                    print("".join(f.readlines()[-15:]))

        except Exception as e:
            print(f"❌ Error during query: {e}")

    finally:
        print("\n🛑 Stopping servers...")
        leader_sub.terminate()
        follower_sub.terminate()
        leader_sub.wait()
        follower_sub.wait()

if __name__ == "__main__":
    test_local_replication()
