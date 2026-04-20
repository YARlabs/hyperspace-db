# HyperspaceDB ROS2 Integration

This package provides a bridge between HyperspaceDB and the ROS2 environment, allowing robots to utilize hyperbolic vector memory and advanced cognitive math.

## Services

- `hyperspace/insert_text`: Vectorize and store text in the database.
- `hyperspace/search_text`: Unified search with support for hybrid ranking.
- `hyperspace/vectorize`: Get raw vector for a string.
- `hyperspace/delete`: Remove entry by ID.
- `evaluate_claim_and_navigate`: Perform Riemannian evaluation of a "thought" vector relative to context and calculate next velocity.

## Hybrid Search Usage

In ROS2, you can now specify `hybrid_alpha` in the `SearchText` service to balance between semantic and lexical results:

```bash
ros2 service call /hyperspace/search_text hyperspace_interfaces/srv/SearchText "{query: 'charging station', top_k: 5, collection: 'map_descriptors', hybrid_alpha: 0.5}"
```

## Configuration

Set the gRPC endpoint and API key via ROS2 parameters:
```bash
ros2 run ros2_hyperspace_node tribunal_router_node --ros-args -p hyperspace_endpoint:="192.168.1.50:50051" -p api_key:="YOUR_SECRET"
```
