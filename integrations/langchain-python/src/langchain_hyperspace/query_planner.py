"""
LangChain Tool for HyperspaceDB Query Planning & Intent Disambiguation.

Evaluates user intent prior to vector retrieval against HyperspaceVectorStore:
- Disambiguates vague queries by formulating multiple-choice questions for the user.
- Deconstructs multi-part queries into parallel sub-queries for scatter-gather search.
"""

from __future__ import annotations

import json
from typing import Any, Dict, List, Optional, Type
from pydantic import BaseModel, Field

try:
    from langchain_core.tools import BaseTool, tool
except ImportError:
    # Graceful fallback if langchain_core is not present
    class BaseTool:  # type: ignore
        pass
    def tool(*args, **kwargs):  # type: ignore
        def decorator(f):
            return f
        return decorator

# Import core planning logic
import os
import sys

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


class HyperspaceQueryPlannerInput(BaseModel):
    """Input parameters for Hyperspace Query Planner tool."""

    raw_user_query: str = Field(
        ...,
        description="The raw natural language query or prompt submitted by the user."
    )
    context: Optional[str] = Field(
        default=None,
        description="Optional recent conversation history or context for coreference resolution."
    )
    collections: Optional[List[str]] = Field(
        default=None,
        description="Optional target HyperspaceDB collections available to query."
    )
    max_sub_queries: Optional[int] = Field(
        default=5,
        description="Maximum number of parallel sub-queries to generate for multi-intent questions."
    )


class HyperspaceQueryPlannerTool(BaseTool):
    """
    Enterprise LangChain tool that inspects user queries before hitting HyperspaceDB.
    Detects ambiguity (requesting user clarification) and splits multi-part queries into parallel searches.
    """

    name: str = "hyperspace_query_planner"
    description: str = (
        "Evaluates a user query prior to vector search against HyperspaceDB. "
        "Returns 'ask_user_clarification' with multiple-choice options if the query is vague/ambiguous, "
        "or 'parallel_search' / 'direct_search' with isolated sub-queries if clear."
    )
    args_schema: Type[BaseModel] = HyperspaceQueryPlannerInput

    def _run(
        self,
        raw_user_query: str,
        context: Optional[str] = None,
        collections: Optional[List[str]] = None,
        max_sub_queries: Optional[int] = 5,
        **kwargs: Any
    ) -> str:
        """Executes the query planner and returns a JSON payload."""
        result = _core_plan_query(
            raw_user_query=raw_user_query,
            context=context,
            collections=collections,
            max_sub_queries=max_sub_queries or 5,
        )
        return result.to_json()

    async def _arun(
        self,
        raw_user_query: str,
        context: Optional[str] = None,
        collections: Optional[List[str]] = None,
        max_sub_queries: Optional[int] = 5,
        **kwargs: Any
    ) -> str:
        """Asynchronous execution of the query planner."""
        return self._run(
            raw_user_query=raw_user_query,
            context=context,
            collections=collections,
            max_sub_queries=max_sub_queries,
            **kwargs
        )


@tool("hyperspace_query_planner", args_schema=HyperspaceQueryPlannerInput)
def hyperspace_query_planner(
    raw_user_query: str,
    context: Optional[str] = None,
    collections: Optional[List[str]] = None,
    max_sub_queries: Optional[int] = 5
) -> str:
    """
    Evaluates a user query prior to vector search against HyperspaceDB.
    Returns structured instructions to clarify ambiguity or execute parallel sub-searches.
    """
    result = _core_plan_query(
        raw_user_query=raw_user_query,
        context=context,
        collections=collections,
        max_sub_queries=max_sub_queries or 5,
    )
    return result.to_json()


__all__ = [
    "HyperspaceQueryPlannerInput",
    "HyperspaceQueryPlannerTool",
    "hyperspace_query_planner",
    "QUERY_PLANNER_TOOL_SCHEMA",
    "LOCAL_LLM_PLANNER_PROMPT",
]
