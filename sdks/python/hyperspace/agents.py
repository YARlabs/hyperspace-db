import math
from collections import deque

class TribunalContext:
    """
    Heterogeneous Tribunal Framework (Tribunal Router).
    Evaluates LLM claims by verifying geometric/logical paths between concepts
    using the HyperspaceDB Graph Traversal API.
    """
    def __init__(self, client, collection_name: str):
        self.client = client
        self.collection_name = collection_name

    def evaluate_claim(self, concept_a_id: int, concept_b_id: int, max_depth: int = 5, max_nodes: int = 256) -> float:
        """
        Calculates Graph-Geometric Trust Score by traversing from Concept A to Concept B.
        Returns a score in [0.0, 1.0]. A score of 0.0 means disconnected (Hallucination).
        
        Success Metric: Geometric Trust Score evaluates the spatial consistency of the claim.
        """
        if concept_a_id == concept_b_id:
            return 1.0

        # Extract local geometric subgraph via the Graph Traversal API
        nodes = self.client.traverse(
            start_id=concept_a_id,
            max_depth=max_depth,
            max_nodes=max_nodes,
            collection=self.collection_name
        )
        
        if not nodes:
            return 0.0

        # Build an adjacency list for path validation
        adj_list = {}
        for node in nodes:
            adj_list[node["id"]] = node["neighbors"]

        if concept_a_id not in adj_list:
            return 0.0

        # Perform BFS to find the shortest geometric path distance
        queue = deque([(concept_a_id, 0)])
        visited = {concept_a_id}

        path_length = -1
        while queue:
            current, depth = queue.popleft()
            if current == concept_b_id:
                path_length = depth
                break
                
            if depth >= max_depth:
                continue

            for neighbor in adj_list.get(current, []):
                if neighbor not in visited:
                    visited.add(neighbor)
                    queue.append((neighbor, depth + 1))

        if path_length == -1:
            return 0.0 # No logical pipeline -> hallucination / confabulation

        # Geometric Trust Score decays smoothly based on the length of the shortest path
        trust_score = math.exp(-0.4 * path_length)
        return float(trust_score)
