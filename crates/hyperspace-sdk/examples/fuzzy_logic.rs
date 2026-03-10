use hyperspace_sdk::fuzzy::{FuzzyQuery, TConorm, TNorm};
use hyperspace_sdk::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Connecting to HyperspaceDB...");
    let mut client = Client::connect("http://[::1]:50051".to_string(), None, None).await?;

    let col_name = "fuzzy_test";

    // Ignore error if it doesn't exist
    let _ = client.delete_collection(col_name.to_string()).await;

    println!("Creating collection `{col_name}` for fuzzy queries (L2 metric)...");
    client
        .create_collection(col_name.to_string(), 3, "l2".to_string())
        .await?;

    println!("Inserting test data...");
    // Item 1 is very close to A and B
    client
        .insert(
            1,
            vec![1.0, 0.0, 0.0],
            std::collections::HashMap::new(),
            Some(col_name.to_string()),
        )
        .await?;
    // Item 2 is very close to A and NOT C
    client
        .insert(
            2,
            vec![0.0, 1.0, 0.0],
            std::collections::HashMap::new(),
            Some(col_name.to_string()),
        )
        .await?;
    // Item 3 is close to B and C
    client
        .insert(
            3,
            vec![0.0, 0.0, 1.0],
            std::collections::HashMap::new(),
            Some(col_name.to_string()),
        )
        .await?;

    // A = [1.0, 0.0, 0.0]
    // B = [0.0, 1.0, 0.0]
    // C = [0.0, 0.0, 1.0]

    // Query: A AND (B OR NOT C)
    // - Item 1: Distance to A=0. To B=1.41. To C=1.41. Has A, somewhat lacks B and C.
    // - Item 2: Distance to A=1.41. To B=0. To C=1.41. Has B.
    // Let's set the target queries to exactly select what we want.

    let a_q = FuzzyQuery::Vector(vec![1.0, 0.0, 0.0]); // Matches Item 1
    let b_q = FuzzyQuery::Vector(vec![0.0, 1.0, 0.0]); // Matches Item 2
    let c_q = FuzzyQuery::Vector(vec![0.0, 0.0, 1.0]); // Matches Item 3

    let query = FuzzyQuery::And(
        Box::new(a_q),
        Box::new(FuzzyQuery::Or(
            Box::new(b_q),
            Box::new(FuzzyQuery::Not(Box::new(c_q))),
            TConorm::Max,
        )),
        TNorm::Product,
    );

    println!("Searching with FuzzyQuery: A AND (B OR NOT C)...");
    let results = client
        .search_fuzzy(&query, 5, Some(col_name.to_string()))
        .await?;

    println!("Search results for FuzzyQuery:");
    for (id, score) in results {
        println!("  - Node {id}: membership score = {score:.4}");
    }

    Ok(())
}
