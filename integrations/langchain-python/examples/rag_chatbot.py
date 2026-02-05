"""
RAG Chatbot Example using HyperspaceDB and LangChain

This example demonstrates how to build a Retrieval-Augmented Generation (RAG)
chatbot using HyperspaceDB as the vector store.

Requirements:
    pip install langchain-hyperspace langchain-openai langchain

Usage:
    export OPENAI_API_KEY=your_key_here
    python rag_chatbot.py
"""

import os
from typing import List

from langchain.chains import ConversationalRetrievalChain
from langchain.memory import ConversationBufferMemory
from langchain.text_splitter import RecursiveCharacterTextSplitter
from langchain_openai import ChatOpenAI, OpenAIEmbeddings

from langchain_hyperspace import HyperspaceVectorStore


def load_sample_documents() -> List[str]:
    """Load sample documents about HyperspaceDB."""
    return [
        """
        HyperspaceDB is a hyperbolic vector database designed for hierarchical embeddings.
        It uses the Poincaré ball model to represent data in hyperbolic space, which is
        particularly effective for hierarchical and tree-like data structures.
        """,
        """
        HyperspaceDB features Edge-Cloud Federation, allowing it to work offline-first.
        Data is stored locally on edge devices and automatically synchronized with cloud
        servers when connectivity is available. This makes it ideal for mobile and IoT applications.
        """,
        """
        The database uses Merkle Trees for efficient synchronization. Each collection
        maintains 256 buckets, and changes are tracked using a rolling XOR hash.
        This allows for quick detection of differences between nodes and minimal
        data transfer during sync operations.
        """,
        """
        HyperspaceDB implements 1-bit binary quantization, reducing memory usage by 64x
        compared to full-precision vectors. Despite this aggressive compression, it
        maintains high accuracy for most similarity search tasks.
        """,
        """
        Built in Rust, HyperspaceDB achieves high performance with insert rates exceeding
        10,000 vectors per second and search latencies under 10ms at p99. The use of
        memory-mapped files (mmap) allows for efficient handling of large datasets.
        """,
    ]


def create_vectorstore() -> HyperspaceVectorStore:
    """Create and populate HyperspaceDB vector store."""
    print("🔧 Initializing HyperspaceDB...")
    
    # Initialize embeddings
    embeddings = OpenAIEmbeddings()
    
    # Create vector store
    vectorstore = HyperspaceVectorStore(
        host="localhost",
        port=50051,
        collection_name="hyperspace_docs",
        embedding_function=embeddings,
        enable_deduplication=True,
    )
    
    # Load and split documents
    print("📚 Loading sample documents...")
    documents = load_sample_documents()
    
    # Split into smaller chunks
    text_splitter = RecursiveCharacterTextSplitter(
        chunk_size=500,
        chunk_overlap=50,
        separators=["\n\n", "\n", " ", ""],
    )
    
    texts = []
    for doc in documents:
        texts.extend(text_splitter.split_text(doc.strip()))
    
    # Add to vector store
    print(f"💾 Adding {len(texts)} text chunks to HyperspaceDB...")
    vectorstore.add_texts(texts)
    
    print("✅ Vector store ready!")
    return vectorstore


def create_chatbot(vectorstore: HyperspaceVectorStore) -> ConversationalRetrievalChain:
    """Create conversational RAG chatbot."""
    print("🤖 Creating chatbot...")
    
    # Initialize LLM
    llm = ChatOpenAI(
        model_name="gpt-3.5-turbo",
        temperature=0.7,
    )
    
    # Create memory for conversation history
    memory = ConversationBufferMemory(
        memory_key="chat_history",
        return_messages=True,
        output_key="answer",
    )
    
    # Create retrieval chain
    qa_chain = ConversationalRetrievalChain.from_llm(
        llm=llm,
        retriever=vectorstore.as_retriever(
            search_kwargs={"k": 3}  # Retrieve top 3 relevant chunks
        ),
        memory=memory,
        return_source_documents=True,
    )
    
    print("✅ Chatbot ready!")
    return qa_chain


def main():
    """Run the RAG chatbot."""
    print("=" * 60)
    print("🚀 HyperspaceDB RAG Chatbot Example")
    print("=" * 60)
    print()
    
    # Check for OpenAI API key
    if not os.getenv("OPENAI_API_KEY"):
        print("❌ Error: OPENAI_API_KEY environment variable not set")
        print("   Please set it with: export OPENAI_API_KEY=your_key_here")
        return
    
    # Create vector store and chatbot
    vectorstore = create_vectorstore()
    chatbot = create_chatbot(vectorstore)
    
    print()
    print("=" * 60)
    print("💬 Chat with the bot! (Type 'quit' to exit)")
    print("=" * 60)
    print()
    
    # Sample questions
    sample_questions = [
        "What is HyperspaceDB?",
        "How does it handle offline scenarios?",
        "What makes it fast?",
    ]
    
    print("📝 Sample questions:")
    for i, q in enumerate(sample_questions, 1):
        print(f"   {i}. {q}")
    print()
    
    # Chat loop
    while True:
        try:
            question = input("You: ").strip()
            
            if not question:
                continue
            
            if question.lower() in ["quit", "exit", "bye"]:
                print("👋 Goodbye!")
                break
            
            # Get response
            print("🤔 Thinking...")
            result = chatbot({"question": question})
            
            # Print answer
            print(f"\n🤖 Bot: {result['answer']}\n")
            
            # Optionally show sources
            if result.get("source_documents"):
                print("📚 Sources:")
                for i, doc in enumerate(result["source_documents"], 1):
                    preview = doc.page_content[:100].replace("\n", " ")
                    print(f"   {i}. {preview}...")
                print()
            
        except KeyboardInterrupt:
            print("\n👋 Goodbye!")
            break
        except Exception as e:
            print(f"❌ Error: {e}")
            print("Please make sure HyperspaceDB server is running on localhost:50051")
            break


if __name__ == "__main__":
    main()
