import { HyperspaceClient } from '../../sdks/ts/src/client';

async function main() {
    const client = new HyperspaceClient('localhost:50051', 'I_LOVE_HYPERSPACEDB');
    const colName = 'ts_sdk_example';

    try {
        console.log(`Creating collection '${colName}'...`);
        // Cleanup if exists
        await client.deleteCollection(colName).catch(() => { });

        await client.createCollection(colName, 8, 'l2');
        console.log('Collection created.');

        console.log('Inserting vectors...');
        for (let i = 0; i < 10; i++) {
            const vec = Array(8).fill(0.1 * i);
            // Note: Metadata support currently pending serialization fix
            await client.insert(i, vec, undefined, colName);
        }
        console.log('Insertion complete.');

        console.log('Waiting for index...');
        await new Promise(r => setTimeout(r, 1000));

        console.log('Searching...');
        const query = Array(8).fill(0.1); // Matches ID 1
        const results = await client.search(query, 5, colName);

        console.log('Results:');
        results.forEach(r => console.log(`  ID: ${r.id}, Distance: ${r.distance.toFixed(4)}`));

        await client.deleteCollection(colName);
        client.close();
    } catch (e) {
        console.error('Error:', e);
        client.close();
    }
}

main();
