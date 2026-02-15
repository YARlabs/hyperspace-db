
import subprocess
import time
import urllib.request
import json
import os
import sys

SERVER_URL = "http://localhost:50050"
SERVER_BIN = "./target/release/hyperspace-server"
COLLECTION_NAME = "test_idle"
API_KEY = "I_LOVE_HYPERSPACEDB"

def log(msg):
    print(f"[TEST] {msg}")

def request(path, method="GET", data=None):
    url = f"{SERVER_URL}{path}"
    headers = {"x-api-key": API_KEY}
    if data:
        data = json.dumps(data).encode()
        headers["Content-Type"] = "application/json"
    
    req = urllib.request.Request(url, data=data, headers=headers, method=method)
    return urllib.request.urlopen(req)

def wait_for_server():
    for _ in range(30):
        try:
            with request("/api/status") as response:
                if response.status == 200:
                    return True
        except Exception as e:
            # log(f"Wait err: {e}")
            time.sleep(1)
    return False

def api_create():
    try:
        request("/api/collections", method="POST", data={"name": COLLECTION_NAME, "dimension": 8, "metric": "cosine"})
        log(f"Collection '{COLLECTION_NAME}' created.")
    except Exception as e:
        log(f"Create failed (maybe exists): {e}")

def api_search():
    with request(f"/api/collections/{COLLECTION_NAME}/search", method="POST", data={"vector": [0.1]*8, "top_k": 1}) as response:
        log("Search executed successfully.")

def run_test():
    # Remove old data if exists to start fresh? No, assume server handles unique names or create error is fine.
    
    # 1. Start Server
    env = os.environ.copy()
    env["MALLOC_CONF"] = "background_thread:true,dirty_decay_ms:0,muzzy_decay_ms:0"
    
    log("Starting server...")
    # Use different data dir to avoid messing with real data
    tmp_data = "./data_test_mem"
    
    proc = subprocess.Popen(
        [SERVER_BIN, "--http-port", "50050", "--port", "50051", "--role", "leader"],
        env=env,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True
    )

    try:
        if not wait_for_server():
            log("Server failed to start (timeout). DUMPING LOGS:")
            # In case subprocess terminates early
            if proc.poll() is not None:
                print(proc.stdout.read())
            return

        # 2. Create & Active
        api_create()
        api_search()
        log("Collection is HOT.")

        # 3. Idle Wait
        log("Waiting 15 seconds for idle eviction (timeout=5s)...")
        time.sleep(15)

        # 4. Wake Up
        log("Waking up collection...")
        try:
            api_search()
        except:
             log("Search failed on wake up!")

        # 5. Check Logs
        log("Checking logs...")
        proc.terminate()
        try:
            outs, _ = proc.communicate(timeout=5)
        except subprocess.TimeoutExpired:
            proc.kill()
            outs, _ = proc.communicate()

        if f"Idling collection '{COLLECTION_NAME}' unloaded from memory" in outs:
            print("✅ PASS: Found 'unloaded from memory' in logs.")
        else:
            print("❌ FAIL: Did not find unload message.")
            
        if f"Waking up cold collection: '{COLLECTION_NAME}'" in outs:
            print("✅ PASS: Found 'Waking up cold collection' in logs.")
        else:
            print("❌ FAIL: Did not find wake up message.")
            
        # Optional dump on failure
        if "FAIL" in outs or "FAIL" in str(sys.stdout): # Simplification
             pass # Already printed fail message

    finally:
        if proc.poll() is None:
            proc.kill()

if __name__ == "__main__":
    run_test()
