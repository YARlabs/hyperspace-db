
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
client = HyperspaceClient(
    host="localhost:50051", 
    api_key="my-secret-key-123",
    user_id="tenant_name"  # Optional: For multi-tenancy
)
```

**Rust:**
```rust
// Use the updated connect function
let client = Client::connect(
    "http://0.0.0.0:50051".to_string(),
    Some("my-secret-key-123".to_string()),
    Some("tenant_name".to_string())
).await?;
```

## Multi-Tenancy Isolation

Use `x-hyperspace-user-id` header to isolate data per user.

*   **Gateway Responsibility**: Ensure your API Gateway validates user tokens and injects this header securely.
*   **Internal Scope**: Data created with a `user_id` is invisible to other users and the default admin scope.

## Security Implementation

* **SHA-256 Hashing**: The server computes `SHA256(env_key)` at startup and stores only the hash.
* **Constant-Time Comparison**: Incoming keys are hashed and compared to prevent timing attacks.
