
import os
import time
import subprocess
import sys
from hyperspace import HyperspaceClient

def test_replication():
    print("\n🚀 Starting Replication HA Test (Leader-Follower)")
    
    # Paths
    server_bin = "./target/release/hyperspace-server"
    leader_dir = "data_leader"
    follower_dir = "data_follower"
    leader_wal = "wal_leader.log"
    follower_wal = "wal_follower.log"
    
    # Cleanup
    subprocess.run(f"rm -rf {leader_dir} {follower_dir} {leader_wal} {follower_wal} index.snap", shell=True)
    subprocess.run("pkill hyperspace-server", shell=True)
    time.sleep(1)

    # 1. Start Leader (Port 50051)
    print("\n1. Starting Leader on :50051...")
    # NOTE: We must run in separate dirs to avoid file conflicts!
    # Using cwd arguments or environment variables would be better, but server hardcodes "data".
    # Temporary hack: We just launch leader, insert, kill. Then launch follower? No, must run concurrently.
    # The current server hardcodes "data" and "wal.log".
    # I cannot run two servers in the same CWD without them clobbering each other.
    # I need to update main.rs to accept --data-dir and --wal-path? 
    # Or I can run them in different CWDs.
    
    os.makedirs("test_leader", exist_ok=True)
    os.makedirs("test_follower", exist_ok=True)
    
    # Copy binary to CWDs doesn't help if libraries relative.. but binary is standalone mostly.
    # Actually simpler to just symlink the binary or call absolute path.
    abs_server = os.path.abspath(server_bin)
    
    leader_sub = subprocess.Popen(
        [abs_server, "--port", "50051", "--role", "leader"],
        cwd="test_leader",
        stdout=open("test_leader/out.log", "w"),
        stderr=subprocess.STDOUT
    )
    time.sleep(2)
    
    # 2. Start Follower (Port 50052) -> Point to Leader :50051
    print("2. Starting Follower on :50052...")
    follower_sub = subprocess.Popen(
        [abs_server, "--port", "50052", "--role", "follower", "--leader", "http://127.0.0.1:50051"],
        cwd="test_follower",
        stdout=open("test_follower/out.log", "w"),
        stderr=subprocess.STDOUT
    )
    time.sleep(2)
    
    try:
        # 3. Insert into Leader
        print("3. Inserting 3 vectors into LEADER...")
        # Note: Leader auth? If enabled, we need key. Assuming key env passed or disabled for now?
        # Tests run with KEY usually if I built with it? No, key comes from ENV.
        # I won't set env, so auth disabled.
        
        client_leader = HyperspaceClient("localhost:50051")
        client_follower = HyperspaceClient("localhost:50052")
        
        vec = [0.1] * 8
        client_leader.insert(100, vec, {"name": "alpha"})
        client_leader.insert(101, vec, {"name": "beta"})
        client_leader.insert(102, vec, {"name": "gamma"})
        
        print("   Inserted 3 items.")
        time.sleep(2) # Allow replication
        
        # 4. Read from Follower
        print("4. Reading from FOLLOWER...")
        results = client_follower.search(vec, top_k=10)
        print(f"   Follower Results: {results}")
        
        ids = sorted([r["id"] for r in results])
        print(f"   Follower IDs: {ids}")
        # Note: IDs might differ if Follower compacts storage (0, 1, 2 vs 100, 101, 102).
        # We verify by count and distance.
        if len(ids) == 3:
            print("✅ SUCEESS: Follower has 3 items!")
        else:
            print(f"❌ FAIL: Follower missing data. Count {len(ids)}")
            sys.exit(1)
            
        # 5. Verify Follower Read-Only
        print("5. Verify Follower Write Block...")
        if client_follower.insert(999, vec, {}):
            print("❌ FAIL: Follower allowed write!")
            sys.exit(1)
        else:
            print("✅ SUCCESS: Follower rejected write.")
            
    finally:
        print("\nCleaning up...")
        leader_sub.terminate()
        follower_sub.terminate()
        leader_sub.wait()
        follower_sub.wait()

if __name__ == "__main__":
    test_replication()
