
import os
import time
import subprocess
import requests

# --- CONFIGURATION ---
SERVER_BIN = os.path.abspath("./target/debug/hyperspace-server")
LEADER_PORT = "50070"
FOLLOWER_PORT = "50072"
LEADER_HTTP = "50071"
FOLLOWER_HTTP = "50073"

def test_local_via_http():
    print("\n🧪 Starting LOCAL Replication Test (CURL/HTTP Mode)")
    
    # Cleanup
    subprocess.run("rm -rf test_leader test_follower", shell=True)
    os.makedirs("test_leader", exist_ok=True)
    os.makedirs("test_follower", exist_ok=True)
    subprocess.run("pkill -9 hyperspace-server", shell=True)
    time.sleep(1)

    # 1. Start Leader
    print(f"1. Starting Leader on HTTP :{LEADER_HTTP}...")
    env = os.environ.copy()
    env["HYPERSPACE_API_KEY"] = "test_key"
    leader_sub = subprocess.Popen(
        [SERVER_BIN, "--port", LEADER_PORT, "--http-port", LEADER_HTTP, "--role", "leader", "--node-id", "L1"],
        cwd="test_leader", stdout=open("test_leader/out.log", "w"), stderr=subprocess.STDOUT, env=env
    )
    time.sleep(15)

    # 2. Start Follower
    print(f"2. Starting Follower on HTTP :{FOLLOWER_HTTP} -> Leader gRPC :{LEADER_PORT}")
    follower_sub = subprocess.Popen(
        [SERVER_BIN, "--port", FOLLOWER_PORT, "--http-port", FOLLOWER_HTTP, "--role", "follower", "--leader", f"http://127.0.0.1:{LEADER_PORT}", "--node-id", "F1"],
        cwd="test_follower", stdout=open("test_follower/out.log", "w"), stderr=subprocess.STDOUT, env=env
    )
    time.sleep(15)

    headers = {"x-api-key": "test_key"}

    try:
        # 3. Create Collection via HTTP
        print("3. Creating collection 'http_test'...")
        res = requests.post(f"http://127.0.0.1:{LEADER_HTTP}/api/collections", json={
            "name": "http_test", "dimension": 8, "metric": "l2"
        }, headers=headers)
        print(f"   Response: {res.status_code} {res.text}")

        # 4. Insert data via HTTP
        print("4. Inserting data...")
        res = requests.post(f"http://127.0.0.1:{LEADER_HTTP}/api/collections/http_test/insert", json={
            "vector": [0.1] * 8,
            "id": 1,
            "metadata": {"source": "http_test"}
        }, headers=headers)
        print(f"   Response: {res.status_code} {res.text}")

        print("⏳ Waiting for sync (5s)...")
        time.sleep(5)

        # 5. Check Follower via HTTP
        print("5. Checking Follower...")
        res = requests.get(f"http://127.0.0.1:{FOLLOWER_HTTP}/api/collections", headers=headers)
        print(f"📋 Follower Collections: {res.text}")

        if "http_test" in res.text:
            print("\n✅ SUCCESS: Local replication logic is PERFECT!")
        else:
            print("\n❌ FAIL: Collection not found on follower.")
            print("Check test_follower/out.log for gRPC errors.")

    finally:
        leader_sub.terminate()
        follower_sub.terminate()
        leader_sub.wait()
        follower_sub.wait()

if __name__ == "__main__":
    test_local_via_http()
