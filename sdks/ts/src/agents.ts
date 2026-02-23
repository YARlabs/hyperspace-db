import { HyperspaceClient } from './client';

export class TribunalContext {
    private client: HyperspaceClient;
    private collectionName: string;

    /**
     * Heterogeneous Tribunal Framework (Tribunal Router).
     * Evaluates LLM claims by verifying geometric/logical paths between concepts
     * using the HyperspaceDB Graph Traversal API.
     */
    constructor(client: HyperspaceClient, collectionName: string) {
        this.client = client;
        this.collectionName = collectionName;
    }

    /**
     * Calculates Graph-Geometric Trust Score by traversing from Concept A to Concept B.
     * Returns a score in [0.0, 1.0]. A score of 0.0 means disconnected (Hallucination).
     */
    async evaluateClaim(conceptAId: number, conceptBId: number, maxDepth: number = 5, maxNodes: number = 256): Promise<number> {
        if (conceptAId === conceptBId) return 1.0;

        try {
            // Extract local geometric subgraph via the Graph Traversal API
            const nodes = await this.client.traverse(conceptAId, 0, maxDepth, maxNodes, this.collectionName);
            if (!nodes || nodes.length === 0) return 0.0;

            const adjList: Record<number, number[]> = {};
            for (const node of nodes) {
                adjList[node.id] = node.neighbors;
            }

            if (!adjList[conceptAId]) return 0.0;

            // Perform BFS to find the shortest geometric path distance
            const queue: [number, number][] = [[conceptAId, 0]];
            const visited = new Set<number>([conceptAId]);
            let pathLength = -1;

            while (queue.length > 0) {
                const [current, depth] = queue.shift()!;
                if (current === conceptBId) {
                    pathLength = depth;
                    break;
                }

                if (depth >= maxDepth) continue;

                const neighbors = adjList[current] || [];
                for (const neighbor of neighbors) {
                    if (!visited.has(neighbor)) {
                        visited.add(neighbor);
                        queue.push([neighbor, depth + 1]);
                    }
                }
            }

            if (pathLength === -1) return 0.0; // No logical pathway

            // Geometric Trust Score decays smoothly based on shortest path
            const trustScore = Math.exp(-0.4 * pathLength);
            return trustScore;
        } catch (e) {
            console.error("TribunalContext evaluateClaim Error:", e);
            return 0.0;
        }
    }
}
