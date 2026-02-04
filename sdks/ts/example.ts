import { HyperspaceClient } from '../../sdks/ts/src/client';

async function main() {
    console.log('Connecting...');
    const client = new HyperspaceClient('localhost:50051', 'I_LOVE_HYPERSPACEDB');
    const colName = 'ts_sdk_test';

    try {
        console.log('Creating collection...');
        await client.deleteCollection(colName).catch(() => { });
        await client.createCollection(colName, 8, 'l2');
        console.log('Collection created');

        console.log('Inserting...');
        for (let i = 0; i < 10; i++) {
            await client.insert(i, Array(8).fill(0.1 * i), { cat: 'test' }, colName);
        }
        console.log('Inserted 10 vectors.');

        await new Promise(r => setTimeout(r, 1000));

        console.log('Searching...');
        const res = await client.search(Array(8).fill(0.1), 5, colName);
        console.log('Results:', res);

        await client.deleteCollection(colName);
        client.close(); // Important to close handle
    } catch (e) {
        console.error('Error:', e);
        client.close();
    }
}

main();
