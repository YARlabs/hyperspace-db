"""LangChain integration for HyperspaceDB."""

from langchain_hyperspace.vectorstores import HyperspaceVectorStore
from langchain_hyperspace.embeddings import YARLabsEmbeddings

__version__ = "0.1.0"
__all__ = ["HyperspaceVectorStore", "YARLabsEmbeddings"]
