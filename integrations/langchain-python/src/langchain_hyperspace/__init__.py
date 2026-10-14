"""LangChain integration for HyperspaceDB."""

from langchain_hyperspace.vectorstores import HyperspaceVectorStore
from langchain_hyperspace.embeddings import YARLabsEmbeddings
from langchain_hyperspace.query_planner import (
    HyperspaceQueryPlannerTool,
    HyperspaceQueryPlannerInput,
    hyperspace_query_planner,
)

__version__ = "4.0.0"
__all__ = [
    "HyperspaceVectorStore",
    "YARLabsEmbeddings",
    "HyperspaceQueryPlannerTool",
    "HyperspaceQueryPlannerInput",
    "hyperspace_query_planner",
]
