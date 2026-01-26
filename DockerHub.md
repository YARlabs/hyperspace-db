```markdown
# [H] HyperspaceDB

![Banner](https://img.shields.io/badge/Status-v1.0_Gold-00FFFF?style=for-the-badge)
![License](https://img.shields.io/badge/License-AGPL_v3-blue?style=for-the-badge)
![Size](https://img.shields.io/docker/image-size/yarlabs/hyperspacedb/latest?style=for-the-badge)

**The Spatial Memory for AI.**
HyperspaceDB is a high-performance, hyperbolic vector database written in Rust. It features 1-bit quantization, async replication, and native support for hierarchical datasets (Poincaré ball model).

---

## 🚀 Quick Reference

* **Maintained by:** [YARlabs](https://github.com/yarlabs)
* **Where to get help:** [GitHub Issues](https://github.com/yarlabs/hyperspace-db/issues), [Discord](https://discord.gg/hyperspace-db)
* **Supported architectures:** `linux/amd64`, `linux/arm64` (Apple Silicon compatible)

---

## 🐳 How to use this image

### 1. Start a single instance

To start the database and expose the gRPC port (50051):

```bash
docker run -d \
  --name hyperspace \
  -p 50051:50051 \
  glukhota/hyperspace-db:latest

```

### 2. Persisting Data (Critical)

By default, data is stored inside the container. To prevent data loss when the container is removed, you **must** mount a volume to `/data`.

```bash
docker run -d \
  --name hyperspace \
  -p 50051:50051 \
  -v $(pwd)/hs_data:/data \
  glukhota/hyperspace-db:latest

```

### 3. Using Docker Compose

The easiest way to run HyperspaceDB in production or development.

```yaml
services:
  hyperspace:
    image: glukhota/hyperspace-db:latest
    container_name: hyperspace
    restart: unless-stopped
    ports:
      - "50051:50051"
    volumes:
      - ./data:/data
    environment:
      - RUST_LOG=info
      - HS_PORT=50051

```

---

## ⚙️ Configuration

HyperspaceDB is configured via environment variables passed to the container.

| Variable | Default | Description |
| --- | --- | --- |
| `HS_PORT` | `50051` | The gRPC listening port. |
| `HS_DATA_DIR` | `/data` | Path inside the container for storing segments and WAL. |
| `RUST_LOG` | `info` | Log verbosity (`error`, `warn`, `info`, `debug`, `trace`). |
| `HS_API_KEY` | *(None)* | If set, enables SHA-256 authentication for all requests. |

---

## 🏷 Image Variants

### `glukhota/hyperspace-db:latest`

This is the defacto image. It contains the latest stable release of the database. Use this for most use cases.

### `glukhota/hyperspace-db:1.0.0`

Specific version tags. Use these in production to ensure immutability and prevent unexpected updates.

---

## 🔒 License

HyperspaceDB is licensed under a dual-license model:

1. **Open Source (AGPLv3):** Free for open source projects.
2. **Commercial:** Required for proprietary/closed-source products.

View full license details on [GitHub](https://github.com/yarlabs/hyperspace-db/blob/main/LICENSE).

```

---