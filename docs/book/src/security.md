
# 🔒 Security & Auth

HyperspaceDB includes built-in security features for production deployments.

## API Authentication

We use a simple but effective **API Key** mechanism.

### Enabling Auth

Set the `HYPERSPACE_API_KEY` environment variable when starting the server.

```bash
export HYPERSPACE_API_KEY="my-secret-key-123"
./hyperspace-server
```

If this variable is NOT set, authentication is **disabled** (dev mode).

### Client Usage

Clients must pass the key in the `x-api-key` metadata header.

**Python:**
```python
client = HyperspaceClient(host="localhost:50051", api_key="my-secret-key-123")
```

**Rust:**
```rust
// Internally configured via Tonic Interceptor if you implement it or pass metadata manually.
// The SDK handles this if extended in future. 
// Current Rust SDK v0.1 does not explicit expose auth arg yet, planned for v0.2.
```

## Security Implementation

* **SHA-256 Hashing**: The server computes `SHA256(env_key)` at startup and stores only the hash.
* **Constant-Time Comparison**: Incoming keys are hashed and compared to prevent timing attacks.
