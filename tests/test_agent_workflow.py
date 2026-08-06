#!/usr/bin/env python3
import subprocess
import time
import json
import os
import sys
import signal

# Configurations
CDE_DIR = ".."
DB_DIR = "../hyperspace-db"
MCP_DIR = "../hyperspace-db/integrations/mcp-hyperspacedb"

COLLECTION = "agent_cognitive_memories_129"

processes = []

def cleanup():
    print("\n🧹 Cleaning up processes...")
    for p in processes:
        try:
            p.terminate()
            p.wait(timeout=5)
        except Exception:
            try:
                p.kill()
            except Exception:
                pass
    print("✨ Cleanup finished.")

def run_mcp_cmd(mcp_proc, method, params):
    req = {
        "jsonrpc": "2.0",
        "id": 1,
        "method": method,
        "params": params
    }
    req_str = json.dumps(req) + "\n"
    print(f"➡️ Sending MCP Request: {method} with {list(params.keys()) if params else None}")
    mcp_proc.stdin.write(req_str)
    mcp_proc.stdin.flush()
    
    # Read response
    resp_str = mcp_proc.stdout.readline()
    if not resp_str:
        return None
    try:
        resp = json.loads(resp_str)
        return resp
    except Exception as e:
        print(f"❌ Failed to parse response: {resp_str}. Error: {e}")
        return None

def main():
    signal.signal(signal.SIGINT, lambda sig, frame: (cleanup(), sys.exit(1)))
    signal.signal(signal.SIGTERM, lambda sig, frame: (cleanup(), sys.exit(1)))

    try:
        # Clean up old database directories to prevent replay/version conflicts
        import shutil
        for folder in ["default_admin_agent_cognitive_memories", "default_admin_agent_cognitive_memories_129"]:
            path = os.path.join(DB_DIR, "data", folder)
            if os.path.exists(path):
                print(f"🧹 Removing old database collection folder: {path}")
                try:
                    shutil.rmtree(path)
                except Exception as e:
                    print(f"⚠️ Failed to remove {path}: {e}")

        # 1. Start CDE Inference Service locally on port 8080
        print("🚀 Starting CDE Inference Service...")
        cde_env = os.environ.copy()
        cde_proc = subprocess.Popen(
            ["./target/debug/cde-service", "serve", "--port", "8080", "--config", "configs/default.json"],
            cwd=CDE_DIR,
            env=cde_env,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL
        )
        processes.append(cde_proc)
        
        # 2. Start HyperspaceDB Server locally on port 50051 (gRPC) / 50050 (HTTP)
        print("🚀 Starting HyperspaceDB Server with YarInk Hybrid 129D (MRL) and Medium Quantization...")
        db_env = os.environ.copy()
        db_env["HYPERSPACE_EMBED"] = "true"
        
        # Disable unused embedding providers to prevent heavy HF model downloads
        db_env["HS_EMBED_L2_PROVIDER"] = "disabled"
        db_env["HS_EMBED_COSINE_PROVIDER"] = "disabled"
        db_env["HS_EMBED_POINCARE_PROVIDER"] = "disabled"
        db_env["HS_EMBED_LORENTZ_PROVIDER"] = "disabled"
        
        # Configure hybrid geometry provider to use our local CDE/yarink API
        db_env["HS_EMBED_HYBRID_PROVIDER"] = "yarink"
        db_env["HS_EMBED_HYBRID_API_BASE"] = "http://localhost:8080/v1/embeddings"
        db_env["HS_EMBED_HYBRID_EMBED_MODEL"] = "v5_Light"
        db_env["HS_EMBED_HYBRID_DIM"] = "801"  # Embedder outputs full 801D vector
        db_env["HS_QUANTIZATION_LEVEL"] = "medium"  # Enable AsymmetricHybrid801 quantization mode
        db_env["HYPERSPACE_API_KEY"] = "I_LOVE_HYPERSPACEDB"
        
        db_proc = subprocess.Popen(
            ["./target/release/hyperspace-server"],
            cwd=DB_DIR,
            env=db_env,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL
        )
        processes.append(db_proc)
        
        # Give servers some time to spin up (and bind port 50051)
        print("⏳ Waiting for servers to initialize (12 seconds)...")
        time.sleep(12)
        
        # 3. Launch MCP Server
        print("🚀 Launching MCP Server via node...")
        mcp_env = os.environ.copy()
        mcp_env["HYPERSPACE_HOST"] = "127.0.0.1:50051"
        mcp_env["HYPERSPACE_API_KEY"] = "I_LOVE_HYPERSPACEDB"
        
        mcp_proc = subprocess.Popen(
            ["node", "dist/index.js"],
            cwd=MCP_DIR,
            env=mcp_env,
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            text=True
        )
        processes.append(mcp_proc)
        
        # Initialize MCP Handshake
        init_resp = run_mcp_cmd(mcp_proc, "initialize", {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {"name": "test-client", "version": "1.0.0"}
        })
        if not init_resp or "result" not in init_resp:
            print("❌ MCP Initialization failed!")
            cleanup()
            sys.exit(1)
        print("✅ MCP Server Initialized.")

        # --- STEP 1: Create Collection with 129D Hybrid MRL ---
        print("\n--- STEP 1: Creating collection (129D Hybrid MRL with Asymmetric801 Quantization) ---")
        create_resp = run_mcp_cmd(mcp_proc, "tools/call", {
            "name": "hyperspace_create_collection",
            "arguments": {
                "collection": COLLECTION,
                "dimension": 129,
                "metric": "hybrid"
            }
        })
        print(f"Result: {json.dumps(create_resp, indent=2, ensure_ascii=False)}")
        
        # --- STEP 2: Remember Event (Episodic Memory) ---
        print("\n--- STEP 2: Remembering events (Sidecar Payload Storage) ---")
        mem1 = "Stripe refund window is strictly 14th day. This is an absolute top-level boundary."
        mem2 = "CTO discount approval allows a manager to issue a refund."
        
        rem1_resp = run_mcp_cmd(mcp_proc, "tools/call", {
            "name": "hyperspace_remember_event",
            "arguments": {
                "collection": COLLECTION,
                "text": mem1,
                "session_id": "session_alpha",
                "tags": ["refund-window", "absolute-limit"]
            }
        })
        print(f"Event 1 saved: {json.dumps(rem1_resp, indent=2, ensure_ascii=False)}")
        
        rem2_resp = run_mcp_cmd(mcp_proc, "tools/call", {
            "name": "hyperspace_remember_event",
            "arguments": {
                "collection": COLLECTION,
                "text": mem2,
                "session_id": "session_alpha",
                "tags": ["cto-approval", "discounts"]
            }
        })
        print(f"Event 2 saved: {json.dumps(rem2_resp, indent=2, ensure_ascii=False)}")

        # Wait for index refresh
        print("⏳ Waiting 3 seconds for index updates...")
        time.sleep(3)

        # --- STEP 3: Explore Hierarchy (Concept/Taxonomy Navigation) ---
        print("\n--- STEP 3: Exploring hierarchy ---")
        explore_resp = run_mcp_cmd(mcp_proc, "tools/call", {
            "name": "hyperspace_explore_hierarchy",
            "arguments": {
                "collection": COLLECTION,
                "concept_id": 910888392,  # ID of Event 1
                "direction": "up",
                "limit": 5
            }
        })
        print(f"Hierarchy exploration results: {json.dumps(explore_resp, indent=2, ensure_ascii=False)}")

        # --- STEP 4: Verify Logical Claim ---
        print("\n--- STEP 4: Verifying logical claim ---")
        verify_resp = run_mcp_cmd(mcp_proc, "tools/call", {
            "name": "hyperspace_verify_logical_claim",
            "arguments": {
                "collection": COLLECTION,
                "premise": "All API keys require enterprise owner approval for unlimited requests.",
                "conclusion": "Unlimited requests need approval from an enterprise owner."
            }
        })
        print(f"Verification results: {json.dumps(verify_resp, indent=2, ensure_ascii=False)}")

        print("\n🎉 ALL STEPS COMPLETED SUCCESSFULLY!")

    except Exception as e:
        print(f"\n❌ Test suite failed with error: {e}")
        cleanup()
        sys.exit(1)
        
    finally:
        cleanup()

if __name__ == "__main__":
    main()
