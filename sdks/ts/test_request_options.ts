// Offline tests for request building (no server): efSearch and compact vectors.
// Run: npm test
import { HyperspaceClient } from './src/client';

type Captured = { [rpc: string]: any };

function capture(client: HyperspaceClient): Captured {
    const calls: Captured = {};
    const fakeResp = {
        getResultsList: () => [],
        getResponsesList: () => [],
        getResponsesMap: () => new Map(),
        toObject: () => ({}),
    };
    (client as any).client = new Proxy({}, {
        get: (_t, rpc: string) => (req: any, _md: any, cb: (e: any, r: any) => void) => {
            calls[rpc] = req;
            cb(null, fakeResp);
        },
    });
    return calls;
}

function assert(cond: boolean, msg: string) {
    if (!cond) throw new Error(`FAIL: ${msg}`);
}

async function main() {
    const c = new HyperspaceClient('localhost:1');
    const calls = capture(c);

    await c.search([0.1, 0.2], 5, 'c', { efSearch: 300 }).catch(() => undefined);
    assert(calls.search.hasEfSearch() && calls.search.getEfSearch() === 300, 'search forwards efSearch');

    await c.search([0.1, 0.2], 5, 'c').catch(() => undefined);
    assert(!calls.search.hasEfSearch(), 'efSearch omitted when not set');

    await c.searchBatch([[0.1, 0.2], [0.3, 0.4]], 3, 'c', { efSearch: 128 }).catch(() => undefined);
    assert(calls.searchBatch.getSearchesList().every((s: any) => s.getEfSearch() === 128), 'batch forwards efSearch');

    await c.searchMultiCollection(['a', 'b'], [0.1, 0.2], 7, { efSearch: 64 }).catch(() => undefined);
    assert(calls.searchMultiCollection.getEfSearch() === 64, 'multi forwards efSearch');
    assert(calls.searchMultiCollection.getTopK() === 7, 'multi honours topK (was fixed at 10)');

    let threw = false;
    try { HyperspaceClient.checkEfSearch(-1); } catch { threw = true; }
    assert(threw, 'negative efSearch rejected');
    threw = false;
    try { HyperspaceClient.checkEfSearch(1.5); } catch { threw = true; }
    assert(threw, 'fractional efSearch rejected');

    const compact = new HyperspaceClient('localhost:1', undefined, undefined, { compactVectors: true });
    const ccalls = capture(compact);
    await compact.search([0.5, 0.25], 1, 'c').catch(() => undefined);
    assert(ccalls.search.getVectorList().length === 0 && ccalls.search.getVectorF32List().length === 2,
        'compactVectors sends vector_f32');

    // createCollection over gRPC carries x-expected-vectors; HTTP endpoint resolution.
    const sized = new HyperspaceClient('localhost:1');
    let createMd: any = null;
    (sized as any).client = new Proxy({}, {
        get: (_t, _rpc: string) => (_req: any, md: any, cb: (e: any, r: any) => void) => { createMd = md; cb(null, {}); },
    });
    await sized.createCollection('big', { components: [{ name: 'v', fullDimension: 8, metric: 'cosine' }] } as any,
        '', 0.02, undefined, 5_000_000);
    assert(createMd && createMd.get('x-expected-vectors')[0] === '5000000', 'expectedVectors sent as x-expected-vectors');
    assert(HyperspaceClient.collectionsHttpUrl('db.internal:7051') === 'http://db.internal:7050/api/collections',
        'self-hosted HTTP = gRPC port - 1 on the same host (never the SaaS)');
    assert(HyperspaceClient.collectionsHttpUrl('10.0.0.5') === 'http://10.0.0.5:50050/api/collections', 'default port');
    assert(HyperspaceClient.collectionsHttpUrl('grpc.yar.ink:443') === 'https://the.yar.ink/api/collections', 'SaaS host');

    console.log('ok  test_request_options (11 checks)');
}

main().catch((e) => { console.error(e); process.exit(1); });
