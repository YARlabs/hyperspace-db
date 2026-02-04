#!/usr/bin/env python3
"""
Integration tests for HyperspaceDB Dashboard API
Tests all HTTP endpoints for correctness.
"""

import requests
import json
import sys
import time

API_URL = "http://localhost:50050/api"
API_KEY = "test_key_12345"  # Set HYPERSPACE_API_KEY=test_key_12345 when running server

HEADERS = {"x-api-key": API_KEY}

def test_status():
    """Test /api/status endpoint"""
    print("Testing /api/status...")
    r = requests.get(f"{API_URL}/status", headers=HEADERS)
    assert r.status_code == 200, f"Expected 200, got {r.status_code}"
    data = r.json()
    assert data["status"] == "ONLINE", f"Expected ONLINE status"
    assert "version" in data
    assert "config" in data
    print("✅ /api/status passed")
    return data

def test_metrics():
    """Test /api/metrics endpoint"""
    print("Testing /api/metrics...")
    r = requests.get(f"{API_URL}/metrics", headers=HEADERS)
    assert r.status_code == 200
    data = r.json()
    assert "total_vectors" in data
    assert "total_collections" in data
    print("✅ /api/metrics passed")
    return data

def test_logs():
    """Test /api/logs endpoint"""
    print("Testing /api/logs...")
    r = requests.get(f"{API_URL}/logs", headers=HEADERS)
    assert r.status_code == 200
    data = r.json()
    assert isinstance(data, list), "Logs should be array"
    print(f"✅ /api/logs passed (got {len(data)} log entries)")
    return data

def test_collections_lifecycle():
    """Test collection CRUD operations"""
    print("\nTesting Collections Lifecycle...")
    
    # 1. List collections
    print("  1. Listing collections...")
    r = requests.get(f"{API_URL}/collections", headers=HEADERS)
    assert r.status_code == 200
    initial_collections = r.json()
    print(f"     Found {len(initial_collections)} collections")
    
    # 2. Create collection
    print("  2. Creating test collection...")
    test_col = {
        "name": "test_dashboard_collection",
        "dimension": 128,
        "metric": "l2"
    }
    r = requests.post(f"{API_URL}/collections", json=test_col, headers=HEADERS)
    assert r.status_code == 201, f"Expected 201, got {r.status_code}"
    print("     ✅ Collection created")
    
    # 3. Verify it appears in list
    print("  3. Verifying collection appears in list...")
    r = requests.get(f"{API_URL}/collections", headers=HEADERS)
    collections = r.json()
    assert len(collections) == len(initial_collections) + 1
    
    # Check if response is detailed (with stats) or just names
    if isinstance(collections[0], dict):
        names = [c["name"] for c in collections]
        assert "test_dashboard_collection" in names
        # Verify stats
        test_col_data = next(c for c in collections if c["name"] == "test_dashboard_collection")
        assert test_col_data["dimension"] == 128
        assert test_col_data["metric"] == "l2"
        print(f"     ✅ Collection found with correct stats")
    else:
        assert "test_dashboard_collection" in collections
        print(f"     ✅ Collection found in list")
    
    # 4. Get stats
    print("  4. Getting collection stats...")
    r = requests.get(f"{API_URL}/collections/test_dashboard_collection/stats", headers=HEADERS)
    assert r.status_code == 200
    stats = r.json()
    assert stats["count"] == 0  # Empty collection
    print(f"     ✅ Stats retrieved (count: {stats['count']})")
    
    # 5. Peek (should be empty)
    print("  5. Peeking at empty collection...")
    r = requests.get(f"{API_URL}/collections/test_dashboard_collection/peek?limit=10", headers=HEADERS)
    assert r.status_code == 200
    items = r.json()
    assert len(items) == 0, "Empty collection should return empty array"
    print("     ✅ Peek returned empty array")
    
    # 6. Search (should work but return empty)
    print("  6. Testing search on empty collection...")
    search_req = {
        "vector": [0.1] * 128,
        "top_k": 5
    }
    r = requests.post(f"{API_URL}/collections/test_dashboard_collection/search", 
                     json=search_req, headers=HEADERS)
    assert r.status_code == 200
    results = r.json()
    assert isinstance(results, list)
    assert len(results) == 0, "Empty collection should return no results"
    print("     ✅ Search returned empty results")
    
    # 7. Delete collection
    print("  7. Deleting test collection...")
    r = requests.delete(f"{API_URL}/collections/test_dashboard_collection", headers=HEADERS)
    assert r.status_code == 200
    print("     ✅ Collection deleted")
    
    # 8. Verify deletion
    print("  8. Verifying deletion...")
    r = requests.get(f"{API_URL}/collections", headers=HEADERS)
    collections = r.json()
    if isinstance(collections[0], dict):
        names = [c["name"] for c in collections]
        assert "test_dashboard_collection" not in names
    else:
        assert "test_dashboard_collection" not in collections
    print("     ✅ Collection no longer in list")
    
    print("✅ Collections lifecycle tests passed")

def test_search_with_data():
    """Test search functionality with actual data (requires gRPC client)"""
    print("\nTesting Search with Data...")
    print("⚠️  Skipping - requires gRPC client to insert vectors")
    print("    Manual test: Use gRPC to insert vectors, then test /search endpoint")

def main():
    print("=" * 60)
    print("HyperspaceDB Dashboard API Integration Tests")
    print("=" * 60)
    print(f"API URL: {API_URL}")
    print(f"Using API Key: {API_KEY[:8]}...")
    print()
    
    try:
        # Wait for server
        print("Waiting for server to be ready...")
        for i in range(5):
            try:
                requests.get(f"{API_URL}/status", headers=HEADERS, timeout=1)
                break
            except:
                if i == 4:
                    print("❌ Server not responding. Start server with:")
                    print("   HYPERSPACE_API_KEY=test_key_12345 cargo run --bin hyperspace-server")
                    sys.exit(1)
                time.sleep(1)
        
        print("✅ Server is ready\n")
        
        # Run tests
        test_status()
        test_metrics()
        test_logs()
        test_collections_lifecycle()
        test_search_with_data()
        
        print("\n" + "=" * 60)
        print("✅ ALL TESTS PASSED")
        print("=" * 60)
        
    except AssertionError as e:
        print(f"\n❌ TEST FAILED: {e}")
        sys.exit(1)
    except Exception as e:
        print(f"\n❌ ERROR: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)

if __name__ == "__main__":
    main()
