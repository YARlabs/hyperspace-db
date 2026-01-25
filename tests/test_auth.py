
from hyperspace import HyperspaceClient
import time
import sys

def test_auth():
    print("--- Auth Test ---")
    
    # 1. Try without key (Should Fail)
    print("1. Attempting connection WITHOUT key...")
    try:
        client = HyperspaceClient("localhost:50051") # no key
        success = client.insert(999, [0.1]*8, {"test": "fail"})
        if success:
            print("❌ FAIL: Insert succeeded but should have failed!")
            sys.exit(1)
        else:
            print("✅ PASS: Insert failed (Expected)")
    except Exception as e:
        print(f"✅ PASS: Exception caught: {e}")

    # 2. Try with WRONG key (Should Fail)
    print("2. Attempting connection with WRONG key...")
    try:
        client = HyperspaceClient("localhost:50051", api_key="wrong_secret")
        success = client.insert(999, [0.1]*8, {"test": "fail"})
        if success:
             print("❌ FAIL: Insert succeeded with wrong key!")
             sys.exit(1)
        else:
             print("✅ PASS: Insert failed (Expected)")
    except Exception as e:
        print(f"✅ PASS: Exception caught: {e}")

    # 3. Try with CORRECT key (Should Success)
    print("3. Attempting connection with CORRECT key...")
    try:
        client = HyperspaceClient("localhost:50051", api_key="supersecret")
        success = client.insert(999, [0.1]*8, {"test": "ok"})
        if success:
             print("✅ PASS: Insert succeeded!")
        else:
             print("❌ FAIL: Insert failed with correct key!")
             sys.exit(1)
    except Exception as e:
        print(f"❌ FAIL: Exception with correct key: {e}")
        sys.exit(1)

if __name__ == "__main__":
    test_auth()
