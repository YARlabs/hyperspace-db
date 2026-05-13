# HyperspaceDB ROS2 Integration

This package provides a bridge between HyperspaceDB and the ROS2 environment, allowing robots to utilize hyperbolic vector memory and advanced cognitive math.

## Services

- `hyperspace/create_collection`: Create a collection using a JSON schema definition.
- `hyperspace/insert_text`: Vectorize and store text in the database.
- `hyperspace/search_text`: Unified search with support for hybrid ranking.

## Creating a Collection with Schema

You can now define complex schemas (Multi-Vector, MRL, Hybrid) using JSON via the ROS2 service:

```bash
ros2 service call /hyperspace/create_collection hyperspace_interfaces/srv/CreateCollection "{name: 'robot_memory', schema_json: '{\"components\": [{\"name\": \"main\", \"metric\": \"lorentz\", \"full_dimension\": 1025}]}'}"
```

## Hybrid Search Usage

In ROS2, you can specify `hybrid_alpha` and `hybrid_query` in the `Search` service:

```bash
ros2 service call /hyperspace/search hyperspace_interfaces/srv/Search "{vector: [0.1, 0.2, 0.3], top_k: 5, collection: 'robot_memory', hybrid_alpha: 0.5, hybrid_query: 'docking station'}"
```

## Configuration

Set the gRPC endpoint and API key via ROS2 parameters:
```bash
ros2 run ros2_hyperspace_node tribunal_router_node --ros-args -p hyperspace_endpoint:="192.168.1.50:50051" -p api_key:="YOUR_SECRET"
```
