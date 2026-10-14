"""
Unit tests for HyperspaceDB Query Planner (Python).
"""

import unittest
from hyperspace_memory.query_planner import (
    plan_query,
    QueryPlannerResult,
    QUERY_PLANNER_TOOL_SCHEMA,
)


class TestHyperspaceQueryPlanner(unittest.TestCase):

    def test_ambiguous_keyword(self):
        res = plan_query("performance")
        self.assertEqual(res.status, "clarification_needed")
        self.assertEqual(res.action, "ask_user_clarification")
        self.assertFalse(res.is_intent_clear)
        self.assertIsNotNone(res.clarification)
        self.assertGreaterEqual(len(res.clarification.options), 3)
        self.assertIn("performance", res.clarification.reason.lower())

    def test_dangling_pronoun(self):
        res = plan_query("how does it work?")
        self.assertEqual(res.status, "clarification_needed")
        self.assertFalse(res.is_intent_clear)
        self.assertIsNotNone(res.clarification)
        self.assertIn("it", res.clarification.question)

    def test_multi_question_conjoined_with_coreference(self):
        res = plan_query("What is HyperspaceDB SuperKMeans and how does it prune distances?")
        self.assertEqual(res.status, "ready_for_search")
        self.assertEqual(res.action, "parallel_search")
        self.assertTrue(res.is_intent_clear)
        self.assertTrue(res.has_multiple_questions)
        self.assertEqual(len(res.search_queries), 2)
        self.assertIn("superkmeans", res.search_queries[1].lower())

    def test_multiple_question_marks(self):
        res = plan_query("What is vector throughput? How to configure 64-bit IDs in HyperspaceDB?")
        self.assertEqual(res.status, "ready_for_search")
        self.assertTrue(res.has_multiple_questions)
        self.assertEqual(len(res.search_queries), 2)

    def test_comparative_multidimensional(self):
        res = plan_query("Compare Scalar-I8 and Binary-1bit in terms of latency, RAM, and recall")
        self.assertEqual(res.status, "ready_for_search")
        self.assertTrue(res.has_multiple_questions)
        self.assertEqual(len(res.search_queries), 3)

    def test_clear_single_query(self):
        res = plan_query("How to configure Direct I/O in HyperspaceDB storage engine?")
        self.assertEqual(res.status, "ready_for_search")
        self.assertEqual(res.action, "direct_search")
        self.assertTrue(res.is_intent_clear)
        self.assertFalse(res.has_multiple_questions)
        self.assertEqual(len(res.search_queries), 1)
        self.assertIsNone(res.clarification)

    def test_json_schema_validity(self):
        self.assertEqual(QUERY_PLANNER_TOOL_SCHEMA["name"], "hyperspace_query_planner")
        self.assertIn("raw_user_query", QUERY_PLANNER_TOOL_SCHEMA["parameters"]["required"])

    def test_to_dict_and_to_json(self):
        res = plan_query("How to configure Direct I/O?")
        d = res.to_dict()
        self.assertIsInstance(d, dict)
        j = res.to_json()
        self.assertIsInstance(j, str)
        self.assertIn("direct_search", j)


if __name__ == "__main__":
    unittest.main()
