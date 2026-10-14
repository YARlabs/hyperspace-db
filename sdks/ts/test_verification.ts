import { HyperspaceClient, VerificationResult } from './src/client';

function testVerificationTyping() {
    const mockResult: VerificationResult = {
        trust_score: 0.94,
        is_hallucination: false,
        is_valid: true,
        min_distance: 0.14,
        avg_distance: 0.22,
        geodesic_distance: 0.31,
        geodesic_hops: 1,
        cone_satisfied: true,
        details: "Topologically sound context alignment",
        resolved_context_ids: [100, 200]
    };

    if (typeof mockResult.trust_score !== 'number' || typeof mockResult.is_hallucination !== 'boolean') {
        throw new Error('Type validation failed');
    }

    const client = new HyperspaceClient('localhost:50051');
    if (typeof client.verifyGeneration !== 'function' || typeof client.verify_generation !== 'function') {
        throw new Error('Method verifyGeneration / verify_generation not defined on HyperspaceClient');
    }
    client.close();

    console.log('TypeScript SDK verification tests passed successfully.');
}

testVerificationTyping();
