import base64
import json
import os
import struct
import subprocess
import sys
import time
import uuid
import threading
from http.server import HTTPServer, BaseHTTPRequestHandler
import grpc

# Add paths to look up hyperspace module
sys.path.append(os.path.dirname(os.path.abspath(__file__)))
sys.path.append(os.path.join(os.path.dirname(os.path.abspath(__file__)), "hyperspace"))
sys.path.append(os.path.join(os.path.dirname(os.path.abspath(__file__)), "hyperspace", "proto"))

from hyperspace import DePINClient
from hyperspace.proto import hyperspace_pb2
from hyperspace.proto import hyperspace_pb2_grpc

# ─── Mock Coordinator ─────────────────────────────────────────────────────────

class MockCoordinatorHandler(BaseHTTPRequestHandler):
    billing_status = "active"

    def log_message(self, format, *args):
        pass  # Suppress logging

    def do_GET(self):
        if self.path == "/api/depin/nodes":
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.end_headers()
            # Return active node list with dummy hex public key (32 bytes = 64 characters)
            data = [{
                "peerId": "test-peer-id",
                "ip": "127.0.0.1",
                "port": 7777,
                "publicKey": "00" * 32,
                "semanticZone": [0.0] * 33,
                "isActive": True
            }]
            self.wfile.write(json.dumps(data).encode("utf-8"))
        else:
            self.send_response(404)
            self.end_headers()

    def do_POST(self):
        content_length = int(self.headers.get('Content-Length', 0))
        post_data = self.rfile.read(content_length) if content_length > 0 else b""

        if self.path == "/api/depin/nodes/register":
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.end_headers()
            self.wfile.write(json.dumps({"success": True, "node_id": "test-node-1"}).encode("utf-8"))
        elif self.path == "/api/depin/nodes/heartbeat":
            payload = json.loads(post_data.decode("utf-8"))
            print(f"💓 Heartbeat received on Mock Coordinator: {payload}")
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.end_headers()
            self.wfile.write(json.dumps({"success": True, "isFull": False}).encode("utf-8"))
        elif self.path == "/api/depin/sync":
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.end_headers()
            self.wfile.write(json.dumps({"success": True, "status": MockCoordinatorHandler.billing_status, "tokenBalance": 100.0}).encode("utf-8"))
        else:
            self.send_response(404)
            self.end_headers()

def start_mock_coordinator(port):
    server = HTTPServer(("localhost", port), MockCoordinatorHandler)
    t = threading.Thread(target=server.serve_forever, daemon=True)
    t.start()
    return server

# ─── Main Test Runner ─────────────────────────────────────────────────────────

def main():
    print("=== Start DePIN Client End-to-End Integration Test ===")
    
    # 1. Start Mock Coordinator
    coord_port = 8089
    coord_server = start_mock_coordinator(coord_port)
    print(f"Mock Coordinator running at http://localhost:{coord_port}")
    
    # 2. Setup paths for temporary test databases inside workspace
    workspace_dir = os.path.dirname(os.path.abspath(__file__))
    data_dir = os.path.join(workspace_dir, "test_depin_data")
    billing_db = os.path.join(workspace_dir, "test_billing.redb")
    identity_file = os.path.join(workspace_dir, "test_identity.json")
    
    # Clean up any past test data
    for path in [billing_db, billing_db + "_tickets.redb", identity_file]:
        if os.path.exists(path):
            os.remove(path)
            
    if not os.path.exists(data_dir):
        os.makedirs(data_dir)
        
    # 3. Start hyperspace-miner process
    # Find binary path (assuming workspace root target/debug/hyperspace-miner)
    target_bin = os.path.join(workspace_dir, "..", "..", "target", "debug", "hyperspace-miner")
    if not os.path.exists(target_bin):
        # Fallback to cargo run
        cmd = [
            "cargo", "run", "--bin", "hyperspace-miner", "--features", "depin", "--"
        ]
    else:
        cmd = [target_bin]
        
    cmd.extend([
        "--grpc-port", "50053",
        "--http-port", "8081",
        "--p2p-port", "7778",
        "--coordinator-url", f"http://localhost:{coord_port}",
        "--data-dir", data_dir,
        "--billing-db-path", billing_db,
        "--identity-file", identity_file,
        "--sync-interval-secs", "5"
    ])
    
    print(f"Launching miner: {' '.join(cmd)}")
    miner_proc = subprocess.Popen(
        cmd,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True
    )
    
    # Wait for the gRPC server to start serving
    time.sleep(4)
    if miner_proc.poll() is not None:
        stdout, stderr = miner_proc.communicate()
        print("Error: Miner failed to start:")
        print(f"STDOUT: {stdout}")
        print(f"STDERR: {stderr}")
        coord_server.shutdown()
        sys.exit(1)
        
    print("Miner successfully booted in DePIN mode on port 50053.")
    
    try:
        # 4. Test DePINClient
        print("Initializing DePINClient...")
        client = DePINClient(
            host="localhost:50053",
            coordinator_url=f"http://localhost:{coord_port}",
            api_key="I_LOVE_HYPERSPACEDB"
        )
        
        # Verify that client fetched recipient_pubkey (should be 32 bytes of 0x00 from mock coordinator)
        print(f"Recipient public key resolved: {client.recipient_pubkey.hex()}")
        assert client.recipient_pubkey == b"\x00" * 32, "Recipient public key should match mock coordinator"
        
        col_name = f"depin_test_{uuid.uuid4().hex[:8]}"
        # Create collection first (free of ticket billing because it's admin)
        print(f"Creating collection '{col_name}'...")
        schema = {
            "components": [
                {
                    "name": "comp1",
                    "metric": "l2",
                    "full_dimension": 4,
                    "weight": 1.0
                }
            ]
        }
        client.create_collection(col_name, schema)
        
        # Test insert: should attach x-hs-ticket automatically
        print("Inserting vector...")
        success = client.insert(id=1, vector=[0.1, 0.2, 0.3, 0.4], collection=col_name)
        print(f"Insert status: {success}")
        assert success is True, "Insert with DePIN ticket should succeed"
        
        # Test search: should attach x-hs-ticket automatically
        print("Searching collection...")
        results = client.search(collection=col_name, vector=[0.1, 0.2, 0.3, 0.4], top_k=2)
        print(f"Search results: {results}")
        assert len(results) > 0, "Search should return results"
        assert results[0]["id"] == 1, "First result ID should be 1"

        # Test delete: verify delete works in DePIN environment
        print("Deleting vector...")
        del_success = client.delete(id=1, collection=col_name)
        print(f"Delete status: {del_success}")
        assert del_success is True, "Delete should succeed"

        # ─── Throttling and Restoration Test ───
        print("\n=== Testing Billing Throttling & Restoration ===")
        # 1. Switch Mock Coordinator response to insufficient_funds
        print("Switching mock coordinator status to 'insufficient_funds'...")
        MockCoordinatorHandler.billing_status = "insufficient_funds"

        # 2. Wait for next SyncWorker tick (sync-interval-secs is 5, wait 7s to be sure)
        print("Waiting 7 seconds for SyncWorker to sync and apply throttle...")
        time.sleep(7)

        # 3. Verify that calling gRPC stub directly raises RESOURCE_EXHAUSTED
        print("Verifying request rejection with RESOURCE_EXHAUSTED...")
        req = hyperspace_pb2.InsertRequest(
            id=2,
            vector=[0.1, 0.2, 0.3, 0.4],
            collection=col_name
        )
        try:
            client.stub.Insert(req, metadata=client.metadata)
            assert False, "Insert should have been rejected with RESOURCE_EXHAUSTED"
        except grpc.RpcError as e:
            print(f"Correctly failed with: {e.code()} - {e.details()}")
            assert e.code() == grpc.StatusCode.RESOURCE_EXHAUSTED, "Expected StatusCode.RESOURCE_EXHAUSTED"

        # 4. Switch Mock Coordinator response back to active
        print("Restoring balance on mock coordinator to 'active'...")
        MockCoordinatorHandler.billing_status = "active"

        # 5. Wait for next SyncWorker tick (7s)
        print("Waiting 7 seconds for SyncWorker to sync and lift throttle...")
        time.sleep(7)

        # 6. Verify that inserts work correctly again
        print("Attempting insert after restoration...")
        success = client.insert(id=2, vector=[0.1, 0.2, 0.3, 0.4], collection=col_name)
        print(f"Insert status after restoration: {success}")
        assert success is True, "Insert should succeed after restoration"

        # ─── Full Node Capacity Write Rejection Test ───
        print("\n=== Testing Full Node Capacity Write Rejection ===")
        
        # Shut down first miner to release DB locks
        print("Shutting down first miner to release DB locks...")
        miner_proc.terminate()
        try:
            miner_proc.wait(timeout=5)
        except subprocess.TimeoutExpired:
            miner_proc.kill()

        env = os.environ.copy()
        env["HS_DISK_CAPACITY_BYTES"] = "1"  # Force 1-byte capacity limit
        env["HS_RAM_CAPACITY_BYTES"] = "1"
        
        cmd_full = cmd.copy()
        for i, arg in enumerate(cmd_full):
            if arg == "--grpc-port": cmd_full[i+1] = "50054"
            elif arg == "--http-port": cmd_full[i+1] = "8082"
            elif arg == "--p2p-port": cmd_full[i+1] = "7779"
            
        print(f"Launching second miner with 1-byte capacity: {' '.join(cmd_full)}")
        miner_full_proc = subprocess.Popen(
            cmd_full,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            env=env
        )
        
        time.sleep(4)
        if miner_full_proc.poll() is not None:
            stdout, stderr = miner_full_proc.communicate()
            print(f"STDOUT: {stdout}")
            print(f"STDERR: {stderr}")
            raise RuntimeError("Second miner failed to start")
            
        try:
            client_full = DePINClient(
                host="localhost:50054",
                coordinator_url=f"http://localhost:{coord_port}",
                api_key="I_LOVE_HYPERSPACEDB"
            )
            
            # Wait for capacity checker loop to flag the node full (runs every 5 seconds)
            print("Waiting 6 seconds for capacity checker loop to flag the node full...")
            time.sleep(6)
            
            print("Verifying insert rejection due to capacity limit...")
            req = hyperspace_pb2.InsertRequest(
                id=3,
                vector=[0.1, 0.2, 0.3, 0.4],
                collection=col_name
            )
            try:
                client_full.stub.Insert(req, metadata=client_full.metadata)
                assert False, "Insert should have been rejected due to capacity limit"
            except grpc.RpcError as e:
                print(f"Correctly failed with: {e.code()} - {e.details()}")
                assert e.code() == grpc.StatusCode.RESOURCE_EXHAUSTED, "Expected StatusCode.RESOURCE_EXHAUSTED"
                assert "capacity limit reached" in e.details().lower(), "Expected capacity limit details"
                
        finally:
            print("Shutting down second miner...")
            miner_full_proc.terminate()
            try:
                miner_full_proc.wait(timeout=5)
            except subprocess.TimeoutExpired:
                miner_full_proc.kill()

        print("\n🎉 DePIN SDK and Server ticket validation + throttling/restoration + capacity tests PASSED successfully!")
        
    finally:
        # 5. Clean up subprocesses
        print("Shutting down miner...")
        miner_proc.terminate()
        try:
            miner_proc.wait(timeout=5)
        except subprocess.TimeoutExpired:
            miner_proc.kill()
            
        coord_server.shutdown()
        
        # Clean up files
        import shutil
        for path in [billing_db, billing_db + "_tickets.redb", identity_file]:
            if os.path.exists(path):
                os.remove(path)
        if os.path.exists(data_dir):
            shutil.rmtree(data_dir)
            
        print("Shutdown and cleanup complete.")

if __name__ == "__main__":
    main()
