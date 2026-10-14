"""
LlamaIndex Tool for HyperspaceDB Query Planning & Intent Disambiguation.

Evaluates user intent prior to vector retrieval against HyperspaceVectorStore:
- Disambiguates vague queries by formulating multiple-choice questions for the user.
- Deconstructs multi-part queries into parallel sub-queries for scatter-gather search.
"""

from __future__ import annotations

import os
import sys
from typing import Any, Dict, List, Optional

_memory_py_path = os.path.abspath(
    os.path.join(os.path.dirname(__file__), "../../../hyperspace-memory-py")
)
if _memory_py_path not in sys.path:
    sys.path.insert(0, _memory_py_path)

from hyperspace_memory.query_planner import (
    plan_query as _core_plan_query,
    QueryPlannerResult,
    QUERY_PLANNER_TOOL_SCHEMA,
    LOCAL_LLM_PLANNER_PROMPT,
)

try:
    from llama_index.core.tools import FunctionTool, BaseTool
except ImportError:
    # Graceful fallback when llama_index_core is absent
    class BaseTool:  # type: ignore
        pass
    class FunctionTool:  # type: ignore
        @classmethod
        def from_defaults(cls, fn: Any, name: Optional[str] = None, description: Optional[str] = None):
            return fn


def hyperspace_query_planner(
    raw_user_query: str,
    context: Optional[str] = None,
    collections: Optional[List[str]] = None,
    max_sub_queries: Optional[int] = 5
) -> str:
    """
    Evaluates a user query prior to vector search against HyperspaceDB.
    Returns structured instructions to clarify ambiguity or execute parallel sub-searches.

    Args:
        raw_user_query: Raw natural language question or prompt from the user.
        context: Optional conversation context to resolve pronouns/references.
        collections: Optional target HyperspaceDB collection names.
        max_sub_queries: Maximum number of parallel sub-queries to generate.

    Returns:
        JSON string matching QueryPlannerResult schema.
    """
    result = _core_plan_query(
        raw_user_query=raw_user_query,
        context=context,
        collections=collections,
        max_sub_queries=max_sub_queries or 5,
    )
    return result.to_json()


def get_query_planner_tool(
    name: str = "hyperspace_query_planner",
    description: Optional[str] = None,
) -> Any:
    """
    Returns a LlamaIndex FunctionTool wrapping the Hyperspace query planner.
    """
    desc = description or (
        "Enterprise RAG Query Planner. Evaluates user intent before vector search: "
        "detects ambiguous/underspecified questions (requesting multiple-choice user clarification) "
        "and decomposes multi-part queries into parallel sub-queries."
    )
    return FunctionTool.from_defaults(
        fn=hyperspace_query_planner,
        name=name,
        description=desc,
    )


__all__ = [
    "hyperspace_query_planner",
    "get_query_planner_tool",
    "QUERY_PLANNER_TOOL_SCHEMA",
    "LOCAL_LLM_PLANNER_PROMPT",
]
