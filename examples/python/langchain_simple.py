from langchain_hyperspace import HyperspaceVectorStore, YARLabsEmbeddings
from langchain_core.documents import Document

def main():
    # 1. Setup YARLabs Local Embeddings
    print("Loading YARLabs Local Embedding Model (0.5B)...")
    embeddings = YARLabsEmbeddings()
    
    # 2. Initialize Store
    # This will automatically create the collection if it doesn't exist
    vectorstore = HyperspaceVectorStore(
        host="localhost",
        port=50051,
        collection_name="langchain_python_demo",
        embedding_function=embeddings,
        api_key="I_LOVE_HYPERSPACEDB"
    )

    # 3. Add Documents
    print("Adding documents...")
    documents = [
        Document(
            page_content="HyperspaceDB is a multi-geometry vector database.",
            metadata={"category": "tech", "level": "advanced"}
        ),
        Document(
            page_content="LangChain is a framework for developing AI applications.",
            metadata={"category": "ai", "level": "beginner"}
        )
    ]
    ids = vectorstore.add_documents(documents)
    print(f"Added {len(ids)} documents with IDs: {ids}")

    # 4. Simple Similarity Search
    print("Searching for 'vector database'...")
    results = vectorstore.similarity_search("vector database", k=1)
    if results:
        print(f"Top result: {results[0].page_content}")

    # 5. Search with Scores
    print("Searching for 'AI' with scores...")
    results_with_score = vectorstore.similarity_search_with_score("AI", k=1)
    for doc, score in results_with_score:
        print(f"Result: {doc.page_content} (Distance: {score})")

    # 6. Delete Documents
    # delete from the first added ID
    to_delete = ids[0:1]
    print(f"Deleting documents with IDs: {to_delete}...")
    vectorstore.delete(ids=to_delete)
    
    # Cleanup channel
    del vectorstore

if __name__ == "__main__":
    main()
