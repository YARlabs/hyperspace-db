# HyperspaceDB ROS2 Integration

This package provides a bridge between HyperspaceDB and the ROS2 environment, allowing robots to utilize hyperbolic vector memory and advanced cognitive math.

## Services

- `hyperspace/create_collection`: Create a collection using a JSON schema definition.
- `hyperspace/insert_text`: Vectorize and store text in the database.
- `hyperspace/search_text`: Unified search with support for hybrid ranking.
- `hyperspace/get_subsumption_tree`: Extract directed light-cone hierarchies for robotic navigation.
- `hyperspace/verify_generation`: Anti-hallucination mathematical verification (`trust_score`, `is_hallucination`) of drafted responses against source context geometric boundaries.
- `hyperspace/link`: Explicit directed edge creation between entities.
- `hyperspace/link_batch`: Atomic bulk edge linking.
- `hyperspace/unlink`: Zero-lag edge deletion.
- `hyperspace/induce_edges`: Automatic edge induction from metadata fields based on declarative rules.

## Creating a Collection with Schema

You can now define complex schemas (Multi-Vector, MRL, Hybrid) using JSON via the ROS2 service:

```bash
ros2 service call /hyperspace/create_collection hyperspace_interfaces/srv/CreateCollection "{name: 'robot_memory', schema_json: '{\"components\": [{\"name\": \"main\", \"metric\": \"lorentz\", \"full_dimension\": 1025}], \"quantization\": \"turbo\"}'}"
```

## Hybrid Search Usage

In ROS2, you can specify `hybrid_alpha` and `hybrid_query` in the `Search` service:

```bash
ros2 service call /hyperspace/search hyperspace_interfaces/srv/Search "{vector: [0.1, 0.2, 0.3], top_k: 5, collection: 'robot_memory', hybrid_alpha: 0.5, hybrid_query: 'docking station'}"
```

## Native Vector-Graph (KAG) & Topology Linking

Link robot affordance, spatial topology, and object hierarchy graphs directly in HyperspaceDB:

### 1. Linking Nodes

```bash
# Link a single edge
ros2 service call /hyperspace/link hyperspace_interfaces/srv/Link "{collection: 'robot_memory', edge: {source_id: 1, target_id: 2, relation: 'OWNS', weight: 1.0}}"

# Batch link edges
ros2 service call /hyperspace/link_batch hyperspace_interfaces/srv/LinkBatch "{collection: 'robot_memory', edges: [{source_id: 1, target_id: 2, relation: 'OWNS', weight: 1.0}, {source_id: 2, target_id: 3, relation: 'DEPENDS_ON', weight: 0.95}]}"

# Unlink edge
ros2 service call /hyperspace/unlink hyperspace_interfaces/srv/Unlink "{collection: 'robot_memory', source_id: 1, target_id: 2, relation: 'OWNS'}"
```

### 2. Declarative Metadata Induction

Automatically generate topological edges from metadata properties:

```bash
ros2 service call /hyperspace/induce_edges hyperspace_interfaces/srv/InduceEdges "{collection: 'robot_memory', rules: [{field: 'room_path', rule_type: 'path_hierarchy', relation: 'CONTAINS', weight: 1.0}]}"
```

## Configuration

Set the gRPC endpoint and API key via ROS2 parameters:
```bash
ros2 run ros2_hyperspace_node tribunal_router_node --ros-args -p hyperspace_endpoint:="192.168.1.50:50051" -p api_key:="YOUR_SECRET"
```

## Zero-Knowledge Client-Side Encryption (ZK-Privacy)

Since the ROS2 node compiles and links against the C++ SDK, it natively supports Zero-Knowledge client-side encryption. Robotic developers can call `RegisterCollectionKey` and `InsertSecure` on the underlying `HyperspaceClient` instance in their custom node extensions to ensure all sensor readings, maps, and private interaction logs are projected and encrypted before sending them to the cloud or a shared DePIN server.


