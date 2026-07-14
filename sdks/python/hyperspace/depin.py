import base64
import json
import os
import struct
import time
import uuid
from typing import Optional, Dict
import grpc

from .client import HyperspaceClient
from .proto import hyperspace_pb2_grpc

def create_signed_ticket(signing_key, recipient_pubkey: bytes, amount_microusd: int) -> str:
    """
    Creates and signs a DePIN SignedTicket, returning the base64-encoded JSON representation.
    """
    issuer_pubkey = signing_key.public_key().public_bytes_raw()
    
    # 30 seconds expiration time
    expires_at = int(time.time()) + 30
    request_id = uuid.uuid4().bytes  # 16 bytes
    
    # Pack amount (u64 le) and expires_at (u64 le)
    amount_bytes = struct.pack("<Q", amount_microusd)
    expires_at_bytes = struct.pack("<Q", expires_at)
    
    # Construct signing bytes:
    # issuer_pubkey (32) + recipient_pubkey (32) + amount (8) + request_id (16) + expires_at (8)
    signing_bytes = issuer_pubkey + recipient_pubkey + amount_bytes + request_id + expires_at_bytes
    
    # Sign
    signature = signing_key.sign(signing_bytes)
    
    # Create the JSON dict
    # issuer_pubkey, recipient_pubkey, and request_id are lists of integers (standard serde [u8] representation)
    # signature is serialized as a base64 string because of #[serde(with = "sig_serde")] in Rust
    ticket = {
        "issuer_pubkey": list(issuer_pubkey),
        "recipient_pubkey": list(recipient_pubkey),
        "amount_microusd": amount_microusd,
        "request_id": list(request_id),
        "expires_at": expires_at,
        "signature": base64.b64encode(signature).decode('utf-8')
    }
    
    ticket_json = json.dumps(ticket)
    return base64.b64encode(ticket_json.encode('utf-8')).decode('utf-8')


class DePINClientInterceptor(grpc.UnaryUnaryClientInterceptor):
    def __init__(self, signing_key, recipient_pubkey: bytes):
        self.signing_key = signing_key
        self.recipient_pubkey = recipient_pubkey

    def intercept_unary_unary(self, continuation, client_call_details, request):
        method = client_call_details.method
        cost = 0
        if "Insert" in method:
            cost = 1  # 1 micro-USD
        elif "Search" in method:
            cost = 10  # 10 micro-USD
            
        if cost > 0:
            ticket_b64 = create_signed_ticket(self.signing_key, self.recipient_pubkey, cost)
            metadata = list(client_call_details.metadata or [])
            metadata.append(("x-hs-ticket", ticket_b64))
            
            # Reconstruct details with updated metadata
            class _Details(grpc.ClientCallDetails):
                def __init__(self, d, meta):
                    self.method = d.method
                    self.timeout = d.timeout
                    self.metadata = meta
                    self.credentials = d.credentials
                    self.wait_for_ready = d.wait_for_ready
                    self.compression = d.compression
            client_call_details = _Details(client_call_details, metadata)
            
        return continuation(client_call_details, request)


class DePINClient(HyperspaceClient):
    def __init__(self, host: str = "localhost:50051", coordinator_url: str = "http://localhost:8080", signing_key_path: Optional[str] = None, *args, **kwargs):
        from cryptography.hazmat.primitives.asymmetric import ed25519
        import requests
        
        # Load or generate ed25519 keypair
        if signing_key_path and os.path.exists(signing_key_path):
            with open(signing_key_path, "rb") as f:
                private_bytes = f.read()
            self.signing_key = ed25519.Ed25519PrivateKey.from_private_bytes(private_bytes)
        else:
            self.signing_key = ed25519.Ed25519PrivateKey.generate()
            if signing_key_path:
                with open(signing_key_path, "wb") as f:
                    f.write(self.signing_key.private_bytes_raw())
                    
        self.issuer_pubkey = self.signing_key.public_key().public_bytes_raw()
        
        # Auto-fetch node_pubkey (recipient_pubkey) from coordinator
        recipient_pubkey = None
        try:
            resp = requests.get(f"{coordinator_url}/api/depin/nodes", timeout=5)
            if resp.status_code == 200:
                nodes = resp.json()
                for node in nodes:
                    if node.get("isActive"):
                        recipient_pubkey = bytes.fromhex(node.get("publicKey"))
                        break
        except Exception as e:
            # Non-fatal: coordinator might be down, or we might be testing directly
            pass
            
        if not recipient_pubkey:
            # Fallback to zero-key if coordinator not available
            recipient_pubkey = b"\x00" * 32
            
        self.recipient_pubkey = recipient_pubkey
        
        # Initialize base client
        super().__init__(host=host, *args, **kwargs)
        
        # Apply gRPC interceptor to all channels/stubs
        interceptor = DePINClientInterceptor(self.signing_key, self.recipient_pubkey)
        self.channels = [grpc.intercept_channel(c, interceptor) for c in self.channels]
        self.stubs = [hyperspace_pb2_grpc.DatabaseStub(c) for c in self.channels]
        self.channel = self.channels[0]
