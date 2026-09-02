import os
import sys
import unittest
from llama_index.core.schema import TextNode
from llama_index.core.vector_stores.types import VectorStoreQuery, VectorStoreQueryMode
from llama_index_hyperspace.vector_stores import HyperspaceVectorStore
from hyperspace.client import HyperspaceClient

HOST = os.environ.get("HYPERSPACE_HOST", "localhost:50051")
API_KEY = os.environ.get("HYPERSPACE_API_KEY", "I_LOVE_HYPERSPACEDB")
is_local = HOST.startswith("localhost") or HOST.startswith("127.0.0.1")
grpc_key = "I_LOVE_HYPERSPACEDB" if (is_local and API_KEY.startswith("sk_")) else API_KEY
TEST_COLLECTION = f"test_llamaindex_py_{int(os.times().system * 1000 + 42)}"

print("==========================================================================")
print("🦙 LLAMINDEX-PYTHON: COMPREHENSIVE VECTORSTORE TEST SUITE")
print("==========================================================================")
print(f"   Host: {HOST}")
print(f"   Collection: {TEST_COLLECTION}\n")

passed = 0
failed = 0

def pass_test(label):
    global passed
    print(f"   ✅ {label}")
    passed += 1

def fail_test(label, detail=None):
    global failed
    print(f"   ❌ {label}{': ' + detail if detail else ''}")
    failed += 1

try:
    # ── STEP 1: Collection Setup ─────────────────────────────────────────────
    print("--------------------------------------------------------------------------")
    print("📦 STEP 1: Creating Collection with 801D Hybrid Metric")
    print("--------------------------------------------------------------------------")
    
    client = HyperspaceClient(host=HOST, api_key=grpc_key)
    schema = {
        "components": [
            {
                "name": "default",
                "metric": "hybrid",
                "full_dimension": 801,
                "weight": 1.0
            }
        ]
    }
    client.create_collection(TEST_COLLECTION, schema=schema)
    pass_test(f"Collection {TEST_COLLECTION} created successfully")

    host_ip, port = HOST.split(":") if ":" in HOST else (HOST, 50051)
    vector_store = HyperspaceVectorStore(
        collection_name=TEST_COLLECTION,
        host=host_ip,
        port=int(port),
        api_key=grpc_key
    )

    # ── STEP 2: Ingest Nodes ─────────────────────────────────────────────────
    print("\n--------------------------------------------------------------------------")
    print("💾 STEP 2: Adding Multi-Domain TextNodes (Medical, Math, Finance)")
    print("--------------------------------------------------------------------------")

    vec_med = client.vectorize("Patient exhibits acute hyperglycemia with blood glucose 14.5 mmol/L.", "hybrid")
    vec_math = client.vectorize("The Poincaré ball and hyperboloid model are isometric representations of hyperbolic space.", "hybrid")
    vec_fin = client.vectorize("Quarterly recurring revenue expanded by 34% driven by enterprise contracts.", "hybrid")

    node_med = TextNode(
        id_="201",
        text="Patient exhibits acute hyperglycemia with blood glucose 14.5 mmol/L.",
        embedding=vec_med,
        metadata={"domain": "medical", "severity": "high"}
    )
    node_math = TextNode(
        id_="202",
        text="The Poincaré ball and hyperboloid model are isometric representations of hyperbolic space.",
        embedding=vec_math,
        metadata={"domain": "math", "geometry": "hyperbolic"}
    )
    node_fin = TextNode(
        id_="203",
        text="Quarterly recurring revenue expanded by 34% driven by enterprise contracts.",
        embedding=vec_fin,
        metadata={"domain": "finance", "type": "revenue"}
    )

    added = vector_store.add([node_med, node_math, node_fin])
    pass_test(f"Added {len(added)} nodes to vector store (IDs: {', '.join(added)})")

    # ── STEP 3: Semantic Query ───────────────────────────────────────────────
    print("\n--------------------------------------------------------------------------")
    print("🔍 STEP 3: Semantic Vector Search (Math / Geometry)")
    print("--------------------------------------------------------------------------")

    query_vec = client.vectorize("The Poincare ball model of hyperbolic space", "hybrid")
    query_obj = VectorStoreQuery(
        query_embedding=query_vec,
        similarity_top_k=2,
        mode=VectorStoreQueryMode.DEFAULT
    )
    res = vector_store.query(query_obj)
    
    if res.nodes and len(res.nodes) > 0:
        pass_test(f"Query returned {len(res.nodes)} node(s). Top ID: {res.ids[0]}")
        if res.ids[0] == "202":
            pass_test("Top hit correctly identified as math/hyperbolic node #202")
        else:
            fail_test(f"Expected top hit 202, got {res.ids[0]}")
    else:
        fail_test("Query returned no nodes")

    # ── STEP 4: Delete Node ──────────────────────────────────────────────────
    print("\n--------------------------------------------------------------------------")
    print("🗑️  STEP 4: Delete Node (Finance #203)")
    print("--------------------------------------------------------------------------")

    vector_store.delete("203")
    pass_test("Deleted node #203")

    # ── STEP 5: Verify Deletion ──────────────────────────────────────────────
    print("\n--------------------------------------------------------------------------")
    print("✅ STEP 5: Verify Node Deletion from Search Results")
    print("--------------------------------------------------------------------------")

    verify_query = VectorStoreQuery(
        query_embedding=vec_fin,
        similarity_top_k=5,
        mode=VectorStoreQueryMode.DEFAULT
    )
    verify_res = vector_store.query(verify_query)
    is_present = any(nid == "203" for nid in verify_res.ids)
    if not is_present:
        pass_test("Confirmed node #203 is no longer in search results")
    else:
        fail_test("Deleted node #203 unexpectedly still returned in query results")

    # ── STEP 6: Cleanup ──────────────────────────────────────────────────────
    print("\n--------------------------------------------------------------------------")
    print("🧹 STEP 6: Collection Cleanup")
    print("--------------------------------------------------------------------------")

    client.delete_collection(TEST_COLLECTION)
    pass_test(f"Deleted test collection {TEST_COLLECTION}")

    # ── Summary ─────────────────────────────────────────────────────────────
    print("\n==========================================================================")
    print("📊 LLAMINDEX-PYTHON TEST RESULTS")
    print("==========================================================================")
    print(f"   ✅ PASSED: {passed}")
    print(f"   ❌ FAILED: {failed}")
    print(f"   Total:    {passed + failed}")
    print("==========================================================================")

    if failed > 0:
        sys.exit(1)
    else:
        print("🎉 ALL LLAMINDEX-PYTHON TESTS PASSED 100%!\n")
        sys.exit(0)

except Exception as err:
    print(f"\n❌ LlamaIndex-Python Test Exception: {err}")
    import traceback
    traceback.print_exc()
    try:
        client.delete_collection(TEST_COLLECTION)
    except Exception:
        pass
    sys.exit(1)
