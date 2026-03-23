from langchain_hyperspace import HyperspaceVectorStore
from langchain_core.documents import Document

def main():
    """Example using HyperspaceDB server-side vectorization.
    
    No Embeddings function (like OpenAI) is needed here.
    """
    
    # 1. Initialize Store with use_server_side_embedding=True
    vectorstore = HyperspaceVectorStore(
        host="localhost",
        port=50051,
        collection_name="server_side_python_demo",
        use_server_side_embedding=True,
        api_key="I_LOVE_HYPERSPACEDB"
    )

    # 2. Add Documents (server will vectorize them)
    print("Adding documents (server-side embedding)...")
    documents = [
        Document(
            page_content="Hyperbolic geometry is useful for hierarchical data.",
            metadata={"topic": "geometry"}
        ),
        Document(
            page_content="Deep learning models are trained on massive datasets.",
            metadata={"topic": "ai"}
        )
    ]
    ids = vectorstore.add_documents(documents)
    print(f"Added documents with IDs: {ids}")

    # 3. Search (server will vectorize the query)
    print("Searching for 'mathematics'...")
    results = vectorstore.similarity_search("mathematics", k=1)
    if results:
        print(f"Top result: {results[0].page_content}")
        print(f"Metadata: {results[0].metadata}")

    # Success!
    print("Done!")

if __name__ == "__main__":
    main()
