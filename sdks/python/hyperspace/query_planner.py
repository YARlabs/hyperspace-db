"""
HyperspaceDB Formal Proof-Obligation Engine & Query Planner (Python)

Upgrades query planning from naive sub-query splitting into a rigorous
"Questions as Proof Obligations" paradigm for regulated enterprise domains.
Decomposes user queries into mutually exclusive, collectively exhaustive (MECE)
proof obligations to guide vector, light-cone, and geodesic traversal in HyperspaceDB.

Obligation Types:
1. MANDATORY_PRECONDITION: A clause that must be proven true for the answer to be valid.
2. EXCEPTION_RULE: A check for conditions that override, void, or amend the general rule.
3. ABSENCE_CHECK: A strict verification that a disqualifying clause is explicitly ABSENT.
"""

from __future__ import annotations

import json
import re
from dataclasses import asdict, dataclass, field
from enum import Enum
from typing import Any, Callable, Dict, List, Optional, Tuple


class ObligationType(str, Enum):
    MANDATORY_PRECONDITION = "MANDATORY_PRECONDITION"
    EXCEPTION_RULE = "EXCEPTION_RULE"
    ABSENCE_CHECK = "ABSENCE_CHECK"


class EvidenceType(str, Enum):
    EXPLICIT_TEXT = "EXPLICIT_TEXT"
    STRUCTURAL_CHILD = "STRUCTURAL_CHILD"
    ABSENCE_PROOF = "ABSENCE_PROOF"


class ObligationStatus(str, Enum):
    UNRESOLVED = "UNRESOLVED"
    SATISFIED = "SATISFIED"
    VIOLATED = "VIOLATED"
    ABSENT = "ABSENT"


class ResolutionStrategy(str, Enum):
    ALL_MANDATORY_MET = "ALL_MANDATORY_MET"
    CONDITIONAL_ON_EXCEPTION = "CONDITIONAL_ON_EXCEPTION"


@dataclass
class ProofObligation:
    """A formal condition that must be proven in the corpus."""

    obligation_id: str
    type: str  # MANDATORY_PRECONDITION | EXCEPTION_RULE | ABSENCE_CHECK
    statement_to_verify: str
    search_focus: str
    required_evidence_type: str  # EXPLICIT_TEXT | STRUCTURAL_CHILD | ABSENCE_PROOF
    status: str = "UNRESOLVED"   # UNRESOLVED | SATISFIED | VIOLATED | ABSENT
    dependencies: List[str] = field(default_factory=list)
    target_manifold: Optional[str] = "hybrid"

    def to_dict(self) -> Dict[str, Any]:
        d: Dict[str, Any] = {
            "obligation_id": self.obligation_id,
            "type": self.type,
            "statement_to_verify": self.statement_to_verify,
            "search_focus": self.search_focus,
            "required_evidence_type": self.required_evidence_type,
            "status": self.status,
        }
        if self.dependencies:
            d["dependencies"] = self.dependencies
        if self.target_manifold:
            d["target_manifold"] = self.target_manifold
        return d


@dataclass
class ProofObligationPlan:
    """Formal proof plan schema for enterprise compliance & RAG."""

    primary_intent: str
    obligations: List[ProofObligation]
    resolution_strategy: str  # ALL_MANDATORY_MET | CONDITIONAL_ON_EXCEPTION

    def to_dict(self) -> Dict[str, Any]:
        return {
            "primary_intent": self.primary_intent,
            "obligations": [o.to_dict() for o in self.obligations],
            "resolution_strategy": self.resolution_strategy,
        }

    def to_json(self, indent: int = 2) -> str:
        return json.dumps(self.to_dict(), indent=indent)


@dataclass
class ClarificationPayload:
    """Details of the clarifying question requested from the user."""

    question: str
    options: List[str]
    reason: str

    def to_dict(self) -> Dict[str, Any]:
        return asdict(self)


@dataclass
class ExecutionStrategy:
    """Recommended search execution strategy for the agent."""

    parallel: bool = False
    target_collections: List[str] = field(default_factory=lambda: ["default"])
    top_k_per_query: int = 5

    def to_dict(self) -> Dict[str, Any]:
        return asdict(self)


@dataclass
class QueryPlannerResult:
    """Result payload produced by the Hyperspace query planner."""

    status: str  # "ready_for_search" | "clarification_needed"
    action: str  # "direct_search" | "parallel_search" | "ask_user_clarification"
    is_intent_clear: bool
    has_multiple_questions: bool
    search_queries: List[str]
    clarification: Optional[ClarificationPayload]
    reason: str
    execution_strategy: ExecutionStrategy
    primary_intent: str = ""
    obligations: List[ProofObligation] = field(default_factory=list)
    resolution_strategy: str = "ALL_MANDATORY_MET"

    def to_obligation_plan(self) -> Dict[str, Any]:
        """Returns the formal proof obligation schema."""
        return {
            "primary_intent": self.primary_intent,
            "obligations": [o.to_dict() for o in self.obligations],
            "resolution_strategy": self.resolution_strategy,
        }

    def to_dict(self) -> Dict[str, Any]:
        return {
            "primary_intent": self.primary_intent,
            "obligations": [o.to_dict() for o in self.obligations],
            "resolution_strategy": self.resolution_strategy,
            "status": self.status,
            "action": self.action,
            "is_intent_clear": self.is_intent_clear,
            "has_multiple_questions": self.has_multiple_questions,
            "search_queries": self.search_queries,
            "clarification": self.clarification.to_dict() if self.clarification else None,
            "reason": self.reason,
            "execution_strategy": self.execution_strategy.to_dict(),
        }

    def to_json(self, indent: int = 2) -> str:
        return json.dumps(self.to_dict(), indent=indent)


# Standard JSON Schema for Tool Registrations (MCP, LangChain, LlamaIndex, OpenAI)
QUERY_PLANNER_TOOL_SCHEMA: Dict[str, Any] = {
    "name": "hyperspace_query_planner",
    "description": (
        "Enterprise Formal Proof-Obligation Engine. Replaces naive query splitting with a "
        "'Questions as Proof Obligations' framework for regulated enterprise domains. Decomposes "
        "queries into MECE proof obligations (MANDATORY_PRECONDITION, EXCEPTION_RULE, ABSENCE_CHECK) "
        "and guides vector/geometric traversal in HyperspaceDB."
    ),
    "parameters": {
        "type": "object",
        "properties": {
            "raw_user_query": {
                "type": "string",
                "description": "The raw user query or prompt submitted to the agent.",
            },
            "context": {
                "type": "string",
                "description": "Optional previous conversation context or chat history to resolve coreferences.",
            },
            "collections": {
                "type": "array",
                "items": {"type": "string"},
                "description": "Optional list of available HyperspaceDB collections.",
            },
            "max_sub_queries": {
                "type": "integer",
                "default": 5,
                "description": "Maximum number of parallel proof obligations to generate.",
            },
        },
        "required": ["raw_user_query"],
    },
}

LOCAL_LLM_PLANNER_PROMPT = """You are the HyperspaceDB Enterprise Proof-Obligation Engine.
Your role is to analyze a raw user query before it is vectorized and sent to the HyperspaceDB vector database.
Instead of returning naive text strings, decompose the query into a formal proof obligation plan.

Obligation Types:
- MANDATORY_PRECONDITION: A clause that must be proven true for the operation/answer to be valid.
- EXCEPTION_RULE: A check for conditions that override, void, or amend the general rule.
- ABSENCE_CHECK: A strict verification that a certain disqualifying clause is explicitly ABSENT from the corpus.

Resolution Strategies:
- ALL_MANDATORY_MET: All preconditions must hold, and absence checks must show zero non-conformances.
- CONDITIONAL_ON_EXCEPTION: Default rule holds unless an exception rule is satisfied.

Output strictly valid JSON matching this schema:
{
  "primary_intent": str,
  "obligations": [
    {
      "obligation_id": str,
      "type": "MANDATORY_PRECONDITION" | "EXCEPTION_RULE" | "ABSENCE_CHECK",
      "statement_to_verify": str,
      "search_focus": str,
      "required_evidence_type": "EXPLICIT_TEXT" | "STRUCTURAL_CHILD" | "ABSENCE_PROOF",
      "status": "UNRESOLVED"
    }
  ],
  "resolution_strategy": "ALL_MANDATORY_MET" | "CONDITIONAL_ON_EXCEPTION",
  "is_intent_clear": bool,
  "reason": str,
  "search_queries": list[str]
}"""

AMBIGUOUS_KEYWORDS: Dict[str, Dict[str, Any]] = {
    "performance": {
        "question": "Which specific aspect of HyperspaceDB performance would you like to examine?",
        "options": [
            "Vector search query latency and QPS benchmarks",
            "Ingestion and indexing throughput with SuperKMeans clustering",
            "Memory footprint and RAM optimization under 21.2B scale",
            "Disk I/O and WAL persistence throughput",
        ],
        "reason": "The term 'performance' is underspecified without identifying latency, throughput, memory, or disk metrics.",
    },
    "latency": {
        "question": "Which latency metric are you looking for?",
        "options": [
            "End-to-end vector search query latency (p50 / p95 / p99)",
            "Point insert and batch write latency (WAL + memory index)",
            "Cross-region replication and P2P gossip propagation latency",
        ],
        "reason": "Query asks for 'latency' without specifying whether read, write, or replication latency is desired.",
    },
    "memory": {
        "question": "Which memory subsystem would you like to explore?",
        "options": [
            "Hierarchical memory clustering with SuperKMeans & Voronoi cells",
            "ADC 1-bit / Scalar-I8 quantization RAM reduction benchmarks",
            "Cognitive working memory (Fréchet Mean consolidation & Lorentz spaces)",
        ],
        "reason": "The query refers to 'memory' without specifying database RAM indexing, quantization, or cognitive agent memory.",
    },
    "clustering": {
        "question": "Which clustering capability do you want to inspect?",
        "options": [
            "SuperKMeans native SIMD clustering algorithm (AVX2/NEON)",
            "Two-level hierarchical macro-routing in MetaRouter",
            "Background Semantic Compactor partition lifecycle",
        ],
        "reason": "Clustering is broad; please specify algorithm mechanics, routing, or background compaction.",
    },
    "sharding": {
        "question": "Which sharding topic would you like to review?",
        "options": [
            "Thread-per-core ShardedCollection architecture and CPU auto-detection",
            "64-bit vector ID decomposition across shards for 21.2B+ scale",
            "Inter-shard scatter-gather search aggregation",
        ],
        "reason": "The query lacks details on whether thread architecture, ID decomposition, or query routing is requested.",
    },
    "pricing": {
        "question": "Which Hyperspace deployment or licensing tier are you asking about?",
        "options": [
            "Open-source Apache-2.0 community edition",
            "Hyperspace Cloud / SaaS hosted clusters",
            "Enterprise on-premise SLA and high-availability deployment",
        ],
        "reason": "Pricing depends on whether community, SaaS, or enterprise licensing is intended.",
    },
    "status": {
        "question": "Which component status do you want to inspect?",
        "options": [
            "Local node health and cluster membership",
            "Active indexing tasks and write buffer queue depth",
            "WAL sync status and storage compaction progress",
        ],
        "reason": "Status query needs a specific subsystem or cluster target.",
    },
    "security": {
        "question": "Which security aspect do you want information on?",
        "options": [
            "Tenant isolation and API authentication keys",
            "TLS/mTLS encrypted gRPC transport",
            "Zero-Knowledge cryptographic preservation on vector embeddings",
        ],
        "reason": "Security encompasses authentication, network encryption, and ZK vector protection.",
    },
    "deployment": {
        "question": "How are you planning to deploy HyperspaceDB?",
        "options": [
            "Docker multi-arch container (x86_64 / arm64)",
            "Kubernetes cluster with Helm charts",
            "Standalone native binary (macOS / Linux)",
        ],
        "reason": "Deployment architecture varies by runtime environment.",
    },
}

# Pre-compiled regular expressions for zero per-query compilation overhead
_RE_CLEAN_PREFIX = re.compile(
    r"^(?:please\s+|can\s+you\s+(?:please\s+)?(?:tell\s+me\s+|explain\s+|show\s+me\s+)?|could\s+you\s+|i\s+(?:would\s+like|want)\s+to\s+know\s+|tell\s+me\s+about\s+)",
    re.IGNORECASE,
)
_RE_CLEAN_SUFFIX = re.compile(r"[.!?]+$")
_RE_CAPITALIZED_WORDS = re.compile(r"\b[A-Z][a-zA-Z0-9_-]*(?:\s+[A-Z][a-zA-Z0-9_-]*)*\b")
_RE_SUBJECT_EXTRACT = re.compile(
    r"(?:what\s+is|how\s+does|explain|about)\s+([a-zA-Z0-9_\-\s]+?)(?:\s+(?:work|scale|compare|perform|mean|and|or|\?|$))",
    re.IGNORECASE,
)
_RE_NON_ALNUM = re.compile(r"[^a-z0-9_-]")
_RE_VAGUE_PRONOUN = re.compile(
    r"\b(how|what|why|does|can|will|is|explain)\s+(?:does\s+|is\s+)?(it|this|that|them)\b"
)
_RE_COMPARE_ONLY = re.compile(r"^compare\s*(\?|$)")
_RE_NUMBERED_LIST = re.compile(r"(?:\r?\n|\s+)(?:\d+[\.\)]|\-|\*)\s+")
_RE_COMPARE_MULTI = re.compile(
    r"compare\s+([^,]+?)\s+(?:and|vs\.?|versus)\s+([^,]+?)\s+(?:in\s+terms\s+of|regarding|with\s+respect\s+to|for)\s+(.+)",
    re.IGNORECASE,
)
_RE_CONJ_SPLIT = re.compile(
    r"^(.+?)(?:,\s*and\s+|,\s*also\s+|,\s*as\s+well\s+as\s+|\s+and\s+(?:also\s+)?|\s+as\s+well\s+as\s+|\s+in\s+addition\s+to\s+|\s+also\s+)(.+)$",
    re.IGNORECASE,
)
_RE_QUESTION_START = re.compile(r"^(?:how|what|why|where|when|can|does|is|explain|show)\b", re.IGNORECASE)
_RE_EXEMPT = re.compile(r"\b(?:exempt|exemption|waived|waiver|except(?:ed|ion)?|derogation)\b")
_RE_STATUTE = re.compile(
    r"\b(?:under|pursuant\s+to|per|section|article|regulation|amended|rule|statute|law|standard|510\(?k\)?|fda|ce\s+mark|iso\s*\d+|hipaa|gdpr)\b"
)
_RE_NEG_1 = re.compile(
    r"\b(?:has|have|did|were|was|is|are)\b.+\b(?:any|ever\s+had\s+any)\b.+\b(?:audit\s+failure|non[- ]conformance|violation|penalty|breach|defect|warning\s+letter|recall|adverse\s+event|incident|hold)\b"
)
_RE_NEG_2 = re.compile(
    r"\b(?:proof\s+of\s+absence|verify\s+absence|ensure\s+no|confirm\s+no|check\s+if\s+no)\b"
)
_RE_NEG_3 = re.compile(
    r"\bany\s+(?:audit\s+failure|violation|non[- ]conformance|incident|defect|penalty|recall|warning\s+letter)\b"
)
_RE_ROOT_CAUSE = re.compile(
    r"\b(?:why\s+did|what\s+caused|root\s+cause\s+of|reason\s+for|investigate\s+discrepancy\s+in)\b"
)
_RE_FAILURE_TERM = re.compile(
    r"\b(?:fail(?:ed|ure)?|quarantine|reject(?:ed|ion)?|defect|discrepancy|out\s+of\s+spec|halt(?:ed)?)\b"
)


def _clean_query(text: str) -> str:
    cleaned = _RE_CLEAN_PREFIX.sub("", text.strip())
    return _RE_CLEAN_SUFFIX.sub("", cleaned).strip()


def _extract_main_subject(text: str) -> Optional[str]:
    matches = _RE_CAPITALIZED_WORDS.findall(text)
    question_words = {
        "what", "how", "why", "where", "when", "which",
        "can", "does", "is", "are", "tell", "explain", "please",
    }
    for m in matches:
        if m.lower().strip() not in question_words:
            return m.strip()

    q_match = _RE_SUBJECT_EXTRACT.search(text)
    if q_match and len(q_match.group(1).strip()) > 2:
        return q_match.group(1).strip()

    return None


def _evaluate_ambiguity(
    query: str, context: Optional[str] = None
) -> Tuple[bool, Optional[ClarificationPayload]]:
    trimmed = query.strip().lower()
    words = trimmed.split()

    single_word = _RE_NON_ALNUM.sub("", trimmed)
    if single_word in AMBIGUOUS_KEYWORDS and (not context or len(context.strip()) < 10):
        entry = AMBIGUOUS_KEYWORDS[single_word]
        return True, ClarificationPayload(
            question=entry["question"],
            options=entry["options"],
            reason=entry["reason"],
        )

    if len(words) <= 2:
        for kw, entry in AMBIGUOUS_KEYWORDS.items():
            if kw in words and (not context or len(context.strip()) < 10):
                return True, ClarificationPayload(
                    question=entry["question"],
                    options=entry["options"],
                    reason=entry["reason"],
                )

    vague_pronouns = {"it", "this", "that", "them", "these", "those"}
    if any(p in words for p in vague_pronouns) and (not context or len(context.strip()) < 10):
        subject = _extract_main_subject(query)
        if not subject and _RE_VAGUE_PRONOUN.search(trimmed):
            return True, ClarificationPayload(
                question="Could you please specify which entity, database feature, or component you are referring to?",
                options=[
                    "HyperspaceDB Storage Engine (WAL & Direct I/O)",
                    "SuperKMeans Indexing & ADSampling",
                    "Cognitive Memory & Lorentz Geometry",
                    "Thread-per-Core Sharding Architecture",
                ],
                reason="The query uses a demonstrative pronoun ('it' / 'this' / 'that') without an antecedent in conversational context.",
            )

    if _RE_COMPARE_ONLY.match(trimmed) or trimmed in ("compare", "compare them", "make a comparison"):
        return True, ClarificationPayload(
            question="Which features or architectures would you like to compare?",
            options=[
                "Thread-per-core v4 vs Shared-memory v3 architecture",
                "SuperKMeans vs standard k-means++ / IVF",
                "Scalar-I8 vs Binary 1-bit ADC quantization",
                "Product-Manifold Hybrid (33D Lorentz + Tail) vs Euclidean geometry",
            ],
            reason="The comparison query does not specify the items being compared.",
        )

    return False, None


def _split_multi_intent(query: str) -> List[str]:
    cleaned = _clean_query(query)

    if "?" in cleaned:
        raw_parts = [p.strip() for p in cleaned.split("?") if len(p.strip()) > 3]
        if len(raw_parts) > 1:
            subject = _extract_main_subject(raw_parts[0])
            queries = []
            for idx, part in enumerate(raw_parts):
                q = _clean_query(part)
                if idx > 0 and subject and subject.lower() not in q.lower():
                    q = f"{q} in {subject}"
                queries.append(q if q.endswith("?") else f"{q}?")
            return queries

    numbered = [p.strip() for p in _RE_NUMBERED_LIST.split(cleaned) if len(p.strip()) > 5]
    if len(numbered) > 1:
        return [_clean_query(p) for p in numbered]

    comp_match = _RE_COMPARE_MULTI.match(cleaned)
    if comp_match:
        item_a = comp_match.group(1).strip()
        item_b = comp_match.group(2).strip()
        dims_raw = comp_match.group(3).strip()
        dims = [
            re.sub(r"[.?]$", "", d.strip())
            for d in re.split(r"(?:,\s*(?:and\s+)?|\s+and\s+)", dims_raw, flags=re.I)
            if len(d.strip()) > 1
        ]
        if len(dims) > 1:
            return [f"Comparison of {item_a} vs {item_b} regarding {d}" for d in dims]

    conj_match = _RE_CONJ_SPLIT.match(cleaned)
    if conj_match:
        part1 = conj_match.group(1).strip()
        part2 = conj_match.group(2).strip()
        is_part2_q = bool(_RE_QUESTION_START.match(part2)) or len(part2.split()) >= 4
        if is_part2_q and len(part1.split()) >= 3:
            main_subject = _extract_main_subject(part1)
            q1 = _clean_query(part1)
            q2 = _clean_query(part2)
            if main_subject:
                q2 = re.sub(
                    r"\b(it|its|the\s+system|the\s+algorithm)\b",
                    main_subject,
                    q2,
                    flags=re.I,
                )
                if main_subject.lower() not in q2.lower() and re.match(r"^(how|what|why)\b", q2, re.I):
                    q2 = f"{q2} for {main_subject}"
            return [
                q1 if q1.endswith("?") else f"{q1}?",
                q2 if q2.endswith("?") else f"{q2}?",
            ]

    return [cleaned]


def _decompose_into_proof_obligations(
    raw_query: str,
    context: Optional[str] = None,
) -> Tuple[str, List[ProofObligation], str]:
    cleaned = _clean_query(raw_query)
    q_lower = cleaned.lower()

    if _RE_EXEMPT.search(q_lower) and _RE_STATUTE.search(q_lower):
        primary_intent = f"Determine statutory exemption status and applicable compliance conditions for: '{cleaned}'"
        dev_match = re.search(
            r"\b(class\s+[i|ii|iii|iv|1|2|3]+(?:\s+devices?)?|devices?|in\s+vitro|software|saamd)\b",
            q_lower,
        )
        target_entity = dev_match.group(0).title() if dev_match else "Target Subject"
        sec_match = re.search(r"(?:under|per|pursuant\s+to|in)\s+([^?]+)", cleaned, re.I)
        sec_name = sec_match.group(1).strip() if sec_match else "applicable statutory amendment"

        obligations = [
            ProofObligation(
                obligation_id="obl_01",
                type=ObligationType.MANDATORY_PRECONDITION.value,
                statement_to_verify=(
                    f"{target_entity} are subject to baseline regulatory premarket notification "
                    "requirements under standard statutory classification."
                ),
                search_focus=f"{target_entity} regulatory baseline 510k premarket notification requirement statutory classification",
                required_evidence_type=EvidenceType.EXPLICIT_TEXT.value,
                status=ObligationStatus.UNRESOLVED.value,
                target_manifold="hybrid",
            ),
            ProofObligation(
                obligation_id="obl_02",
                type=ObligationType.EXCEPTION_RULE.value,
                statement_to_verify=(
                    f"Statutory authority under {sec_name} establishes specific product-code or "
                    f"categorical exemptions waiving standard premarket notification for qualifying {target_entity}."
                ),
                search_focus=f"{sec_name} categorical exemption list product codes waiving requirements",
                required_evidence_type=EvidenceType.STRUCTURAL_CHILD.value,
                status=ObligationStatus.UNRESOLVED.value,
                target_manifold="hybrid",
            ),
            ProofObligation(
                obligation_id="obl_03",
                type=ObligationType.ABSENCE_CHECK.value,
                statement_to_verify=(
                    "Absence of overriding special controls, warning letters, reclassifications, "
                    "or statutory revocation orders barring exemption for the subject device."
                ),
                search_focus="Overriding special controls statutory revocation orders safety alerts barring exemption",
                required_evidence_type=EvidenceType.ABSENCE_PROOF.value,
                status=ObligationStatus.UNRESOLVED.value,
                target_manifold="hybrid",
            ),
        ]
        return primary_intent, obligations, ResolutionStrategy.CONDITIONAL_ON_EXCEPTION.value

    if _RE_NEG_1.search(q_lower) or _RE_NEG_2.search(q_lower) or _RE_NEG_3.search(q_lower):
        primary_intent = f"Negative assurance proof: verify strict absence of non-conformances or failures for: '{cleaned}'"
        supplier_match = re.search(
            r"(?:supplier|vendor|partner|facility|manufacturer|batch|lot|component)\s+([a-zA-Z0-9_\-]+)",
            cleaned,
            re.I,
        )
        entity_name = supplier_match.group(0).title() if supplier_match else "Target Organization"
        event_match = re.search(
            r"\b(audit\s+failure|non[- ]conformance|violation|penalty|breach|defect|warning\s+letter|recall|incident)\b",
            q_lower,
        )
        event_type = event_match.group(0).lower() if event_match else "non-conformance"
        time_match = re.search(r"\b(q[1-4](?:\s+\d{4})?|\d{4}|fiscal\s+year\s+\d+|h[1-2])\b", q_lower)
        time_frame = f" in {time_match.group(0).upper()}" if time_match else ""

        obligations = [
            ProofObligation(
                obligation_id="obl_01",
                type=ObligationType.ABSENCE_CHECK.value,
                statement_to_verify=(
                    f"Absence of recorded {event_type} reports, critical quality notifications, "
                    f"or formal non-conformance findings for {entity_name}{time_frame}."
                ),
                search_focus=f"{entity_name} recorded {event_type} non-conformance observation inspection failure logs{time_frame}",
                required_evidence_type=EvidenceType.ABSENCE_PROOF.value,
                status=ObligationStatus.UNRESOLVED.value,
                target_manifold="hybrid",
            ),
            ProofObligation(
                obligation_id="obl_02",
                type=ObligationType.MANDATORY_PRECONDITION.value,
                statement_to_verify=(
                    f"{entity_name} active compliance monitoring logs, evaluation coverage records, "
                    f"and scheduled audit documentation{time_frame} are confirmed present and uncorrupted."
                ),
                search_focus=f"{entity_name} scheduled audit execution coverage compliance monitoring documentation records{time_frame}",
                required_evidence_type=EvidenceType.EXPLICIT_TEXT.value,
                status=ObligationStatus.UNRESOLVED.value,
                target_manifold="hybrid",
            ),
        ]
        return primary_intent, obligations, ResolutionStrategy.ALL_MANDATORY_MET.value

    if _RE_ROOT_CAUSE.search(q_lower) and _RE_FAILURE_TERM.search(q_lower):
        primary_intent = f"Root-cause multi-hop investigation across records, specs, and procedures for: '{cleaned}'"
        target_batch_match = re.search(
            r"\b(batch|lot|sample|order|unit)\s+([a-zA-Z0-9_\-]+)",
            cleaned,
            re.I,
        )
        target_subject = target_batch_match.group(0).title() if target_batch_match else "Subject Batch"

        obligations = [
            ProofObligation(
                obligation_id="obl_01",
                type=ObligationType.MANDATORY_PRECONDITION.value,
                statement_to_verify=(
                    f"{target_subject} specific non-conformance records, out-of-specification test results, "
                    "and quarantine inspection failure defect codes."
                ),
                search_focus=f"{target_subject} quarantine inspection defect failure non-conformance testing observations certificate",
                required_evidence_type=EvidenceType.EXPLICIT_TEXT.value,
                status=ObligationStatus.UNRESOLVED.value,
                target_manifold="hybrid",
            ),
            ProofObligation(
                obligation_id="obl_02",
                type=ObligationType.MANDATORY_PRECONDITION.value,
                statement_to_verify=(
                    f"Material technical specifications, acceptable quality limits (AQL), "
                    f"and baseline tolerance thresholds governing {target_subject}."
                ),
                search_focus=f"{target_subject} supplier material technical specifications AQL tolerance thresholds baseline limits",
                required_evidence_type=EvidenceType.EXPLICIT_TEXT.value,
                status=ObligationStatus.UNRESOLVED.value,
                dependencies=["obl_01"],
                target_manifold="hybrid",
            ),
            ProofObligation(
                obligation_id="obl_03",
                type=ObligationType.MANDATORY_PRECONDITION.value,
                statement_to_verify=(
                    "Quarantine standard operating procedure (SOP), quarantine disposition protocol, "
                    "and mandatory rejection criteria triggered by the detected defects."
                ),
                search_focus="Quarantine standard operating procedure SOP quarantine release threshold disposition workflow mandatory rejection criteria",
                required_evidence_type=EvidenceType.STRUCTURAL_CHILD.value,
                status=ObligationStatus.UNRESOLVED.value,
                dependencies=["obl_01", "obl_02"],
                target_manifold="hybrid",
            ),
        ]
        return primary_intent, obligations, ResolutionStrategy.ALL_MANDATORY_MET.value

    sub_queries = _split_multi_intent(cleaned)
    primary_intent = f"Evaluate verified compliance and factual resolution for: '{cleaned}'"
    obligations = []
    has_exception = False

    for idx, sq in enumerate(sub_queries):
        obl_id = f"obl_{idx+1:02d}"
        sq_clean = _clean_query(sq)
        sq_l = sq_clean.lower()

        if re.search(r"\b(?:any|no|not|never|absence|without|zero|fail(?:ure)?|violation)\b", sq_l):
            obl_type = ObligationType.ABSENCE_CHECK.value
            ev_type = EvidenceType.ABSENCE_PROOF.value
            stmt = f"Strict verification of absence or non-occurrence regarding '{sq_clean}'."
        elif re.search(r"\b(?:unless|except|waiver|exempt|override|special\s+case)\b", sq_l):
            obl_type = ObligationType.EXCEPTION_RULE.value
            ev_type = EvidenceType.STRUCTURAL_CHILD.value
            has_exception = True
            stmt = f"Verification of exception rule or conditional waiver regarding '{sq_clean}'."
        else:
            obl_type = ObligationType.MANDATORY_PRECONDITION.value
            ev_type = EvidenceType.EXPLICIT_TEXT.value
            stmt = f"Factual verification of mandatory condition: '{sq_clean}'."

        obligations.append(
            ProofObligation(
                obligation_id=obl_id,
                type=obl_type,
                statement_to_verify=stmt,
                search_focus=sq_clean,
                required_evidence_type=ev_type,
                status=ObligationStatus.UNRESOLVED.value,
                target_manifold="hybrid",
            )
        )

    res_strategy = (
        ResolutionStrategy.CONDITIONAL_ON_EXCEPTION.value
        if has_exception
        else ResolutionStrategy.ALL_MANDATORY_MET.value
    )
    return primary_intent, obligations, res_strategy


def plan_query(
    raw_user_query: str,
    context: Optional[str] = None,
    collections: Optional[List[str]] = None,
    max_sub_queries: int = 5,
    llm_caller: Optional[Callable[[str], str]] = None,
) -> QueryPlannerResult:
    """
    Evaluates raw user query prior to vector retrieval against HyperspaceDB,
    decomposing it into formal proof obligations.
    """
    raw_query = (raw_user_query or "").strip()
    target_collections = collections if collections and len(collections) > 0 else ["default"]

    if not raw_query:
        return QueryPlannerResult(
            status="clarification_needed",
            action="ask_user_clarification",
            is_intent_clear=False,
            has_multiple_questions=False,
            search_queries=[],
            clarification=ClarificationPayload(
                question="Please provide a query or topic you would like to search for in HyperspaceDB.",
                options=[
                    "Explore document collections",
                    "Search vector similarities",
                    "Analyze graph subsumption hierarchies",
                ],
                reason="The query string was empty.",
            ),
            reason="No query provided.",
            execution_strategy=ExecutionStrategy(
                parallel=False,
                target_collections=target_collections,
                top_k_per_query=5,
            ),
            primary_intent="Empty query",
            obligations=[],
            resolution_strategy=ResolutionStrategy.ALL_MANDATORY_MET.value,
        )

    if llm_caller:
        try:
            prompt = (
                f"{LOCAL_LLM_PLANNER_PROMPT}\n\n"
                f'User Query: "{raw_query}"\n'
                f'Context: "{context or "None"}"\n'
                f"Available Collections: {json.dumps(target_collections)}"
            )
            response = llm_caller(prompt)
            json_match = re.search(r"\{[\s\S]*\}", response)
            if json_match:
                parsed = json.loads(json_match.group(0))
                if isinstance(parsed.get("is_intent_clear"), bool):
                    is_clear = parsed["is_intent_clear"]
                    obligations_raw = parsed.get("obligations", [])
                    obligations = [
                        ProofObligation(
                            obligation_id=o.get("obligation_id", f"obl_{i+1:02d}"),
                            type=o.get("type", ObligationType.MANDATORY_PRECONDITION.value),
                            statement_to_verify=o.get("statement_to_verify", ""),
                            search_focus=o.get("search_focus", ""),
                            required_evidence_type=o.get(
                                "required_evidence_type", EvidenceType.EXPLICIT_TEXT.value
                            ),
                            status=o.get("status", ObligationStatus.UNRESOLVED.value),
                            target_manifold=o.get("target_manifold", "hybrid"),
                        )
                        for i, o in enumerate(obligations_raw)
                    ][:max_sub_queries]

                    is_multi = len(obligations) > 1
                    search_queries = (
                        [o.search_focus for o in obligations]
                        if obligations
                        else parsed.get("search_queries", [raw_query])
                    )

                    return QueryPlannerResult(
                        status="ready_for_search" if is_clear else "clarification_needed",
                        action=(
                            "ask_user_clarification"
                            if not is_clear
                            else ("parallel_search" if is_multi else "direct_search")
                        ),
                        is_intent_clear=is_clear,
                        has_multiple_questions=is_multi,
                        search_queries=search_queries,
                        clarification=None,
                        reason=parsed.get("reason", "LLM formal obligation plan generated."),
                        execution_strategy=ExecutionStrategy(
                            parallel=is_multi,
                            target_collections=target_collections,
                            top_k_per_query=3 if is_multi else 5,
                        ),
                        primary_intent=parsed.get("primary_intent", raw_query),
                        obligations=obligations,
                        resolution_strategy=parsed.get(
                            "resolution_strategy", ResolutionStrategy.ALL_MANDATORY_MET.value
                        ),
                    )
        except Exception:
            pass

    is_ambiguous, clarification = _evaluate_ambiguity(raw_query, context)
    if is_ambiguous and clarification:
        return QueryPlannerResult(
            status="clarification_needed",
            action="ask_user_clarification",
            is_intent_clear=False,
            has_multiple_questions=False,
            search_queries=[],
            clarification=clarification,
            reason=clarification.reason,
            execution_strategy=ExecutionStrategy(
                parallel=False,
                target_collections=target_collections,
                top_k_per_query=5,
            ),
            primary_intent=f"Ambiguous query: '{raw_query}'",
            obligations=[],
            resolution_strategy=ResolutionStrategy.ALL_MANDATORY_MET.value,
        )

    primary_intent, obligations, resolution_strategy = _decompose_into_proof_obligations(
        raw_query, context
    )
    obligations = obligations[:max_sub_queries]
    is_multi = len(obligations) > 1
    search_queries = [o.search_focus for o in obligations]

    return QueryPlannerResult(
        status="ready_for_search",
        action="parallel_search" if is_multi else "direct_search",
        is_intent_clear=True,
        has_multiple_questions=is_multi,
        search_queries=search_queries,
        clarification=None,
        reason=(
            f"Formulated {len(obligations)} MECE proof obligations with resolution strategy '{resolution_strategy}'."
            if is_multi
            else "Formulated single atomic proof obligation ready for search."
        ),
        execution_strategy=ExecutionStrategy(
            parallel=is_multi,
            target_collections=target_collections,
            top_k_per_query=3 if is_multi else 5,
        ),
        primary_intent=primary_intent,
        obligations=obligations,
        resolution_strategy=resolution_strategy,
    )
