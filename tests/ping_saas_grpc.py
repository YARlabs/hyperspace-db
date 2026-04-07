import grpc
import sys
import os

# --- CONFIGURATION FROM USER ---
HOST = "the.yar.ink:443"
API_KEY = "YOUR_KEY"

# Add SDK path to sys.path
sdk_path = os.path.abspath("./sdks/python")
if sdk_path not in sys.path:
    sys.path.append(sdk_path)

from hyperspace import HyperspaceClient

def test_saas_grpc_connection():
    print(f"🚀 Testing gRPC connection to {HOST}...")

    # Note: The standard HyperspaceClient uses insecure_channel.
    # For SaaS on 443, we need secure_channel (SSL/TLS).
    # I will create a modified client for this test.
    
    try:
        print("\n1. Testing via SECURE channel (SSL/TLS)...")
        credentials = grpc.ssl_channel_credentials()
        channel = grpc.secure_channel(HOST, credentials)
        
        # We'll use the metadata manually to see if it responds
        # Using deep imports from sdk structure
        from hyperspace.proto import hyperspace_pb2
        from hyperspace.proto import hyperspace_pb2_grpc
        stub = hyperspace_pb2_grpc.DatabaseStub(channel)
        
        metadata = [
            ('x-api-key', API_KEY)
        ]
        
        print("📡 Sending ListCollections request...")
        # Timeout after 5s to avoid hanging
        resp = stub.ListCollections(hyperspace_pb2.Empty(), metadata=metadata, timeout=5)
        
        print(f"✅ SUCCESS! Received {len(resp.collections)} collections from SaaS.")
        for col in resp.collections:
            print(f" - {col.name} ({col.count} vectors, {col.metric})")

    except grpc.RpcError as e:
        print(f"❌ gRPC Error: {e.code()} - {e.details()}")
        if e.code() == grpc.StatusCode.UNAUTHENTICATED:
            print("🔑 Auth failure: Check API_KEY and USER_ID match.")
        elif e.code() == grpc.StatusCode.UNAVAILABLE:
            print("🔌 Connection failure: Is the gRPC port exposed on the host?")
    except Exception as e:
        print(f"⚠️  Unexpected error: {e}")

    # Try insecure just in case (some proxies handle it on 80/443 without TLS or via plain http2)
    # But highly unlikely for Production SaaS.

if __name__ == "__main__":
    test_saas_grpc_connection()
