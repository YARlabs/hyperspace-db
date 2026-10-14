/**
 * HyperspaceDB Formal Proof-Obligation Engine & Query Planner (TypeScript)
 *
 * Upgrades query planning from naive sub-query splitting into a rigorous
 * "Questions as Proof Obligations" paradigm for regulated enterprise domains.
 * Decomposes user queries into mutually exclusive, collectively exhaustive (MECE)
 * proof obligations to guide vector, light-cone, and geodesic traversal in HyperspaceDB.
 *
 * Obligation Types:
 * 1. MANDATORY_PRECONDITION: A clause that must be proven true for the answer to be valid.
 * 2. EXCEPTION_RULE: A check for conditions that override, void, or amend the general rule.
 * 3. ABSENCE_CHECK: A strict verification that a disqualifying clause is explicitly ABSENT.
 */

export type ObligationType = "MANDATORY_PRECONDITION" | "EXCEPTION_RULE" | "ABSENCE_CHECK";
export type EvidenceType = "EXPLICIT_TEXT" | "STRUCTURAL_CHILD" | "ABSENCE_PROOF";
export type ObligationStatus = "UNRESOLVED" | "SATISFIED" | "VIOLATED" | "ABSENT";
export type ResolutionStrategy = "ALL_MANDATORY_MET" | "CONDITIONAL_ON_EXCEPTION";

export interface ProofObligation {
  obligation_id: string;
  type: ObligationType;
  statement_to_verify: string;
  search_focus: string;
  required_evidence_type: EvidenceType;
  status: ObligationStatus;
  dependencies?: string[];
  target_manifold?: "lorentz" | "poincare" | "cosine" | "euclidean";
}

export interface ProofObligationPlan {
  primary_intent: string;
  obligations: ProofObligation[];
  resolution_strategy: ResolutionStrategy;
}

export interface QueryPlannerInput {
  raw_user_query: string;
  context?: string;
  collections?: string[];
  max_sub_queries?: number;
}

export interface ClarificationPayload {
  question: string;
  options: string[];
  reason: string;
}

export interface ExecutionStrategy {
  parallel: boolean;
  target_collections: string[];
  top_k_per_query: number;
}

export interface QueryPlannerResult {
  primary_intent: string;
  obligations: ProofObligation[];
  resolution_strategy: ResolutionStrategy;
  status: "ready_for_search" | "clarification_needed";
  action: "direct_search" | "parallel_search" | "ask_user_clarification";
  is_intent_clear: boolean;
  has_multiple_questions: boolean;
  search_queries: string[];
  clarification: ClarificationPayload | null;
  reason: string;
  execution_strategy: ExecutionStrategy;
}

export const QUERY_PLANNER_TOOL_SCHEMA = {
  type: "object",
  properties: {
    raw_user_query: {
      type: "string",
      description: "The raw user query or compliance condition submitted to the agent."
    },
    context: {
      type: "string",
      description: "Optional previous conversation context or chat history to resolve coreferences."
    },
    collections: {
      type: "array",
      items: { type: "string" },
      description: "Optional list of available HyperspaceDB collections."
    },
    max_sub_queries: {
      type: "number",
      default: 5,
      description: "Maximum number of parallel proof obligations to generate."
    }
  },
  required: ["raw_user_query"]
} as const;

export const LOCAL_LLM_PLANNER_PROMPT = `You are the HyperspaceDB Formal Proof-Obligation Engine.
Analyze a raw user query before vector search in regulated enterprise domains.
Decompose it into formal proof obligations matching this schema:
{
  "primary_intent": "string",
  "obligations": [
    {
      "obligation_id": "obl_01",
      "type": "MANDATORY_PRECONDITION | EXCEPTION_RULE | ABSENCE_CHECK",
      "statement_to_verify": "string",
      "search_focus": "string",
      "required_evidence_type": "EXPLICIT_TEXT | STRUCTURAL_CHILD | ABSENCE_PROOF",
      "status": "UNRESOLVED"
    }
  ],
  "resolution_strategy": "ALL_MANDATORY_MET | CONDITIONAL_ON_EXCEPTION",
  "is_intent_clear": true,
  "reason": "string"
}`;

const AMBIGUOUS_KEYWORDS: Record<string, { question: string; options: string[]; reason: string }> = {
  performance: {
    question: "Which specific aspect of HyperspaceDB performance would you like to examine?",
    options: [
      "Vector search query latency and QPS benchmarks",
      "Ingestion and indexing throughput with SuperKMeans clustering",
      "Memory footprint and RAM optimization under 21.2B scale",
      "Disk I/O and WAL persistence throughput"
    ],
    reason: "The term 'performance' is underspecified without identifying latency, throughput, memory, or disk metrics."
  },
  latency: {
    question: "Which latency metric are you looking for?",
    options: [
      "End-to-end vector search query latency (p50 / p95 / p99)",
      "Point insert and batch write latency (WAL + memory index)",
      "Cross-region replication and P2P gossip propagation latency"
    ],
    reason: "Query asks for 'latency' without specifying whether read, write, or replication latency is desired."
  },
  memory: {
    question: "Which memory subsystem would you like to explore?",
    options: [
      "Hierarchical memory clustering with SuperKMeans & Voronoi cells",
      "ADC 1-bit / Scalar-I8 quantization RAM reduction benchmarks",
      "Cognitive working memory (Fréchet Mean consolidation & Lorentz spaces)"
    ],
    reason: "The query refers to 'memory' without specifying database RAM indexing, quantization, or cognitive agent memory."
  },
  clustering: {
    question: "Which clustering capability do you want to inspect?",
    options: [
      "SuperKMeans native SIMD clustering algorithm (AVX2/NEON)",
      "Two-level hierarchical macro-routing in MetaRouter",
      "Background Semantic Compactor partition lifecycle"
    ],
    reason: "Clustering is broad; please specify algorithm mechanics, routing, or background compaction."
  },
  sharding: {
    question: "Which sharding topic would you like to review?",
    options: [
      "Thread-per-core ShardedCollection architecture and CPU auto-detection",
      "64-bit vector ID decomposition across shards for 21.2B+ scale",
      "Inter-shard scatter-gather search aggregation"
    ],
    reason: "The query lacks details on whether thread architecture, ID decomposition, or query routing is requested."
  },
  pricing: {
    question: "Which Hyperspace deployment or licensing tier are you asking about?",
    options: [
      "Open-source Apache-2.0 community edition",
      "Hyperspace Cloud / SaaS hosted clusters",
      "Enterprise on-premise SLA and high-availability deployment"
    ],
    reason: "Pricing depends on whether community, SaaS, or enterprise licensing is intended."
  },
  status: {
    question: "Which component status do you want to inspect?",
    options: [
      "Local node health and cluster membership",
      "Active indexing tasks and write buffer queue depth",
      "WAL sync status and storage compaction progress"
    ],
    reason: "Status query needs a specific subsystem or cluster target."
  },
  security: {
    question: "Which security aspect do you want information on?",
    options: [
      "Tenant isolation and API authentication keys",
      "TLS/mTLS encrypted gRPC transport",
      "Zero-Knowledge cryptographic preservation on vector embeddings"
    ],
    reason: "Security encompasses authentication, network encryption, and ZK vector protection."
  },
  deployment: {
    question: "How are you planning to deploy HyperspaceDB?",
    options: [
      "Docker multi-arch container (x86_64 / arm64)",
      "Kubernetes cluster with Helm charts",
      "Standalone native binary (macOS / Linux)"
    ],
    reason: "Deployment architecture varies by runtime environment."
  }
};

function cleanQuery(text: string): string {
  let cleaned = text.trim();
  cleaned = cleaned.replace(
    /^(?:please\s+|can\s+you\s+(?:please\s+)?(?:tell\s+me\s+|explain\s+|show\s+me\s+)?|could\s+you\s+|i\s+(?:would\s+like|want)\s+to\s+know\s+|tell\s+me\s+about\s+)/i,
    ""
  );
  return cleaned.replace(/[.!?]+$/, "").trim();
}

function extractMainSubject(text: string): string | null {
  const matches = text.match(/\b[A-Z][a-zA-Z0-9_-]*(?:\s+[A-Z][a-zA-Z0-9_-]*)*\b/g);
  const questionWords = new Set(["what", "how", "why", "where", "when", "which", "can", "does", "is", "are", "tell", "explain", "please"]);
  if (matches) {
    for (const m of matches) {
      if (!questionWords.has(m.toLowerCase().trim())) {
        return m.trim();
      }
    }
  }

  const qMatch = text.match(/(?:what\s+is|how\s+does|explain|about)\s+([a-zA-Z0-9_\-\s]+?)(?:\s+(?:work|scale|compare|perform|mean|and|or|\?|$))/i);
  if (qMatch && qMatch[1].trim().length > 2) {
    return qMatch[1].trim();
  }

  return null;
}

function evaluateAmbiguity(query: string, context?: string): { isAmbiguous: boolean; clarification: ClarificationPayload | null } {
  const trimmed = query.trim().toLowerCase();
  const words = trimmed.split(/\s+/);

  const singleWord = trimmed.replace(/[^a-z0-9_-]/g, "");
  if (AMBIGUOUS_KEYWORDS[singleWord] && (!context || context.trim().length < 10)) {
    const entry = AMBIGUOUS_KEYWORDS[singleWord];
    return {
      isAmbiguous: true,
      clarification: {
        question: entry.question,
        options: entry.options,
        reason: entry.reason
      }
    };
  }

  if (words.length <= 2) {
    for (const [kw, entry] of Object.entries(AMBIGUOUS_KEYWORDS)) {
      if (words.includes(kw) && (!context || context.trim().length < 10)) {
        return {
          isAmbiguous: true,
          clarification: {
            question: entry.question,
            options: entry.options,
            reason: entry.reason
          }
        };
      }
    }
  }

  const vaguePronouns = ["it", "this", "that", "them", "these", "those"];
  if (words.some(w => vaguePronouns.includes(w)) && (!context || context.trim().length < 10)) {
    const subject = extractMainSubject(query);
    if (!subject) {
      if (/\b(how|what|why|does|can|will|is|explain)\s+(?:does\s+|is\s+)?(it|this|that|them)\b/i.test(trimmed)) {
        return {
          isAmbiguous: true,
          clarification: {
            question: "Could you please specify which entity, database feature, or component you are referring to?",
            options: [
              "HyperspaceDB Storage Engine (WAL & Direct I/O)",
              "SuperKMeans Indexing & ADSampling",
              "Cognitive Memory & Lorentz Geometry",
              "Thread-per-Core Sharding Architecture"
            ],
            reason: "The query uses a demonstrative pronoun ('it' / 'this' / 'that') without an antecedent in conversational context."
          }
        };
      }
    }
  }

  if (/^compare\s*(\?|$)/i.test(trimmed) || ["compare", "compare them", "make a comparison"].includes(trimmed)) {
    return {
      isAmbiguous: true,
      clarification: {
        question: "Which features or architectures would you like to compare?",
        options: [
          "Thread-per-core v4 vs Shared-memory v3 architecture",
          "SuperKMeans vs standard k-means++ / IVF",
          "Scalar-I8 vs Binary 1-bit ADC quantization",
          "Poincaré vs Lorentz hyperbolic geometry"
        ],
        reason: "The comparison query does not specify the items being compared."
      }
    };
  }

  return { isAmbiguous: false, clarification: null };
}

function splitMultiIntent(query: string): string[] {
  const cleaned = cleanQuery(query);

  if (cleaned.includes("?")) {
    const rawParts = cleaned.split("?").map(p => p.trim()).filter(p => p.length > 3);
    if (rawParts.length > 1) {
      const subject = extractMainSubject(rawParts[0]);
      return rawParts.map((part, idx) => {
        let q = cleanQuery(part);
        if (idx > 0 && subject && !q.toLowerCase().includes(subject.toLowerCase())) {
          q = `${q} in ${subject}`;
        }
        return q.endsWith("?") ? q : `${q}?`;
      });
    }
  }

  const numbered = cleaned.split(/(?:\r?\n|\s+)(?:\d+[\.\)]|\-|\*)\s+/).map(p => p.trim()).filter(p => p.length > 5);
  if (numbered.length > 1) {
    return numbered.map(cleanQuery);
  }

  const compMatch = cleaned.match(/compare\s+([^,]+?)\s+(?:and|vs\.?|versus)\s+([^,]+?)\s+(?:in\s+terms\s+of|regarding|with\s+respect\s+to|for)\s+(.+)/i);
  if (compMatch) {
    const itemA = compMatch[1].trim();
    const itemB = compMatch[2].trim();
    const dimsRaw = compMatch[3].trim();
    const dims = dimsRaw.split(/(?:,\s*(?:and\s+)?|\s+and\s+)/i).map(d => d.replace(/[.?]$/, "").trim()).filter(d => d.length > 1);
    if (dims.length > 1) {
      return dims.map(d => `Comparison of ${itemA} vs ${itemB} regarding ${d}`);
    }
  }

  const conjMatch = cleaned.match(/^(.+?)(?:,\s*and\s+|,\s*also\s+|,\s*as\s+well\s+as\s+|\s+and\s+(?:also\s+)?|\s+as\s+well\s+as\s+|\s+in\s+addition\s+to\s+|\s+also\s+)(.+)$/i);
  if (conjMatch) {
    const part1 = conjMatch[1].trim();
    const part2 = conjMatch[2].trim();
    const isPart2Q = /^(?:how|what|why|where|when|can|does|is|explain|show)\b/i.test(part2) || part2.split(/\s+/).length >= 4;
    if (isPart2Q && part1.split(/\s+/).length >= 3) {
      const mainSubject = extractMainSubject(part1);
      const q1 = cleanQuery(part1);
      let q2 = cleanQuery(part2);
      if (mainSubject) {
        q2 = q2.replace(/\b(it|its|the\s+system|the\s+algorithm)\b/gi, mainSubject);
        if (!q2.toLowerCase().includes(mainSubject.toLowerCase()) && /^(how|what|why)\b/i.test(q2)) {
          q2 = `${q2} for ${mainSubject}`;
        }
      }
      return [
        q1.endsWith("?") ? q1 : `${q1}?`,
        q2.endsWith("?") ? q2 : `${q2}?`
      ];
    }
  }

  return [cleaned];
}

function decomposeIntoProofObligations(
  rawQuery: string,
  context?: string
): { primaryIntent: string; obligations: ProofObligation[]; resolutionStrategy: ResolutionStrategy } {
  const cleaned = cleanQuery(rawQuery);
  const qLower = cleaned.toLowerCase();

  // Pattern 1: Scenario A (Regulatory Exception / Statutory Exemption)
  if (
    /\b(?:exempt|exemption|waived|waiver|except(?:ed|ion)?|derogation)\b/i.test(qLower) &&
    (/\b(?:under|pursuant\s+to|per|section|article|regulation|amended|rule|statute|law|standard)\b/i.test(qLower) ||
     /\b(?:510\(?k\)?|fda|ce\s+mark|iso\s*\d+|hipaa|gdpr)\b/i.test(qLower))
  ) {
    const primaryIntent = `Determine statutory exemption status and applicable compliance conditions for: '${cleaned}'`;
    const devMatch = qLower.match(/\b(class\s+[i|ii|iii|iv|1|2|3]+(?:\s+devices?)?|devices?|in\s+vitro|software|saamd)\b/i);
    const targetEntity = devMatch ? devMatch[0].replace(/\b\w/g, l => l.toUpperCase()) : "Target Subject";

    const secMatch = cleaned.match(/(?:under|per|pursuant\s+to|in)\s+([^?]+)/i);
    const secName = secMatch ? secMatch[1].trim() : "applicable statutory amendment";

    const obligations: ProofObligation[] = [
      {
        obligation_id: "obl_01",
        type: "MANDATORY_PRECONDITION",
        statement_to_verify: `${targetEntity} are subject to baseline regulatory premarket notification requirements under standard statutory classification.`,
        search_focus: `${targetEntity} regulatory baseline 510k premarket notification requirement statutory classification`,
        required_evidence_type: "EXPLICIT_TEXT",
        status: "UNRESOLVED",
        target_manifold: "lorentz"
      },
      {
        obligation_id: "obl_02",
        type: "EXCEPTION_RULE",
        statement_to_verify: `Statutory authority under ${secName} establishes specific product-code or categorical exemptions waiving standard premarket notification for qualifying ${targetEntity}.`,
        search_focus: `${secName} categorical exemption list product codes waiving requirements`,
        required_evidence_type: "STRUCTURAL_CHILD",
        status: "UNRESOLVED",
        target_manifold: "lorentz"
      },
      {
        obligation_id: "obl_03",
        type: "ABSENCE_CHECK",
        statement_to_verify: "Absence of overriding special controls, warning letters, reclassifications, or statutory revocation orders barring exemption for the subject device.",
        search_focus: "Overriding special controls statutory revocation orders safety alerts barring exemption",
        required_evidence_type: "ABSENCE_PROOF",
        status: "UNRESOLVED",
        target_manifold: "lorentz"
      }
    ];

    return { primaryIntent, obligations, resolutionStrategy: "CONDITIONAL_ON_EXCEPTION" };
  }

  // Pattern 2: Scenario B (Proof of Absence / Negative Assurance)
  const isNeg = [
    /\b(?:has|have|did|were|was|is|are)\b.+\b(?:any|ever\s+had\s+any)\b.+\b(?:audit\s+failure|non[- ]conformance|violation|penalty|breach|defect|warning\s+letter|recall|adverse\s+event|incident|hold)\b/i,
    /\b(?:proof\s+of\s+absence|verify\s+absence|ensure\s+no|confirm\s+no|check\s+if\s+no)\b/i,
    /\bany\s+(?:audit\s+failure|violation|non[- ]conformance|incident|defect|penalty|recall|warning\s+letter)\b/i
  ].some(p => p.test(qLower));

  if (isNeg) {
    const primaryIntent = `Negative assurance proof: verify strict absence of non-conformances or failures for: '${cleaned}'`;
    const suppMatch = cleaned.match(/(?:supplier|vendor|partner|facility|manufacturer|batch|lot|component)\s+([a-zA-Z0-9_\-]+)/i);
    const entityName = suppMatch ? suppMatch[0].replace(/\b\w/g, l => l.toUpperCase()) : "Target Organization";

    const eventMatch = qLower.match(/\b(audit\s+failure|non[- ]conformance|violation|penalty|breach|defect|warning\s+letter|recall|incident)\b/i);
    const eventType = eventMatch ? eventMatch[0].toLowerCase() : "non-conformance";

    const timeMatch = qLower.match(/\b(q[1-4](?:\s+\d{4})?|\d{4}|fiscal\s+year\s+\d+|h[1-2])\b/i);
    const timeFrame = timeMatch ? ` in ${timeMatch[0].toUpperCase()}` : "";

    const obligations: ProofObligation[] = [
      {
        obligation_id: "obl_01",
        type: "ABSENCE_CHECK",
        statement_to_verify: `Absence of recorded ${eventType} reports, critical quality notifications, or formal non-conformance findings for ${entityName}${timeFrame}.`,
        search_focus: `${entityName} recorded ${eventType} non-conformance observation inspection failure logs${timeFrame}`,
        required_evidence_type: "ABSENCE_PROOF",
        status: "UNRESOLVED",
        target_manifold: "lorentz"
      },
      {
        obligation_id: "obl_02",
        type: "MANDATORY_PRECONDITION",
        statement_to_verify: `${entityName} active compliance monitoring logs, evaluation coverage records, and scheduled audit documentation${timeFrame} are confirmed present and uncorrupted.`,
        search_focus: `${entityName} scheduled audit execution coverage compliance monitoring documentation records${timeFrame}`,
        required_evidence_type: "EXPLICIT_TEXT",
        status: "UNRESOLVED",
        target_manifold: "lorentz"
      }
    ];

    return { primaryIntent, obligations, resolutionStrategy: "ALL_MANDATORY_MET" };
  }

  // Pattern 3: Scenario C (Multi-Hop Root Cause Investigation)
  if (
    /\b(?:why\s+did|what\s+caused|root\s+cause\s+of|reason\s+for|investigate\s+discrepancy\s+in)\b/i.test(qLower) &&
    /\b(?:fail(?:ed|ure)?|quarantine|reject(?:ed|ion)?|defect|discrepancy|out\s+of\s+spec|halt(?:ed)?)\b/i.test(qLower)
  ) {
    const primaryIntent = `Root-cause multi-hop investigation across records, specs, and procedures for: '${cleaned}'`;
    const batchMatch = cleaned.match(/\b(batch|lot|sample|order|unit)\s+([a-zA-Z0-9_\-]+)/i);
    const targetSubject = batchMatch ? batchMatch[0].replace(/\b\w/g, l => l.toUpperCase()) : "Subject Batch";

    const obligations: ProofObligation[] = [
      {
        obligation_id: "obl_01",
        type: "MANDATORY_PRECONDITION",
        statement_to_verify: `${targetSubject} specific non-conformance records, out-of-specification test results, and quarantine inspection failure defect codes.`,
        search_focus: `${targetSubject} quarantine inspection defect failure non-conformance testing observations certificate`,
        required_evidence_type: "EXPLICIT_TEXT",
        status: "UNRESOLVED",
        target_manifold: "lorentz"
      },
      {
        obligation_id: "obl_02",
        type: "MANDATORY_PRECONDITION",
        statement_to_verify: `Material technical specifications, acceptable quality limits (AQL), and baseline tolerance thresholds governing ${targetSubject}.`,
        search_focus: `${targetSubject} supplier material technical specifications AQL tolerance thresholds baseline limits`,
        required_evidence_type: "EXPLICIT_TEXT",
        status: "UNRESOLVED",
        dependencies: ["obl_01"],
        target_manifold: "lorentz"
      },
      {
        obligation_id: "obl_03",
        type: "MANDATORY_PRECONDITION",
        statement_to_verify: `Quarantine standard operating procedure (SOP), quarantine disposition protocol, and mandatory rejection criteria triggered by the detected defects.`,
        search_focus: "Quarantine standard operating procedure SOP quarantine release threshold disposition workflow mandatory rejection criteria",
        required_evidence_type: "STRUCTURAL_CHILD",
        status: "UNRESOLVED",
        dependencies: ["obl_01", "obl_02"],
        target_manifold: "lorentz"
      }
    ];

    return { primaryIntent, obligations, resolutionStrategy: "ALL_MANDATORY_MET" };
  }

  // Pattern 4: Conditional Authorization / Exemption without re-inspection
  if (/\b(?:can|may|is\s+it\s+permitted\s+for|allow(?:ed)?)\b/i.test(qLower) && qLower.includes("without")) {
    const primaryIntent = `Verify conditional delivery authorization and waiver applicability for: '${cleaned}'`;
    const suppMatch = cleaned.match(/supplier\s+([a-zA-Z0-9_\-]+)/i);
    const suppName = suppMatch ? suppMatch[0].replace(/\b\w/g, l => l.toUpperCase()) : "Supplier";

    const lotMatch = cleaned.match(/lot\s+([a-zA-Z0-9_\-]+)/i);
    const lotName = lotMatch ? lotMatch[0].replace(/\b\w/g, l => l.toUpperCase()) : "Lot";

    const obligations: ProofObligation[] = [
      {
        obligation_id: "obl_01",
        type: "MANDATORY_PRECONDITION",
        statement_to_verify: `${suppName} approved vendor qualification status and baseline incoming inspection protocol.`,
        search_focus: `${suppName} approved vendor qualification baseline incoming inspection protocol`,
        required_evidence_type: "EXPLICIT_TEXT",
        status: "UNRESOLVED",
        target_manifold: "lorentz"
      },
      {
        obligation_id: "obl_02",
        type: "EXCEPTION_RULE",
        statement_to_verify: `Skip-lot or direct-to-stock waiver provision under verified supplier quality partnership program.`,
        search_focus: `${suppName} skip-lot inspection waiver direct-to-stock qualification agreement`,
        required_evidence_type: "STRUCTURAL_CHILD",
        status: "UNRESOLVED",
        target_manifold: "lorentz"
      },
      {
        obligation_id: "obl_03",
        type: "ABSENCE_CHECK",
        statement_to_verify: `Absence of active quarantine holds, open CAPAs, or critical defect notices against ${lotName}.`,
        search_focus: `${lotName} active quarantine hold open CAPA critical defect notification`,
        required_evidence_type: "ABSENCE_PROOF",
        status: "UNRESOLVED",
        target_manifold: "lorentz"
      }
    ];

    return { primaryIntent, obligations, resolutionStrategy: "CONDITIONAL_ON_EXCEPTION" };
  }

  // Pattern 5: Multi-part or Standard Queries
  const subQueries = splitMultiIntent(cleaned);
  const primaryIntent = `Evaluate verified compliance and factual resolution for: '${cleaned}'`;
  const obligations: ProofObligation[] = [];
  let hasException = false;

  subQueries.forEach((sq, idx) => {
    const oblId = `obl_${String(idx + 1).padStart(2, "0")}`;
    const sqClean = cleanQuery(sq);
    const sqL = sqClean.toLowerCase();

    let oblType: ObligationType = "MANDATORY_PRECONDITION";
    let evType: EvidenceType = "EXPLICIT_TEXT";
    let stmt = `Factual verification of mandatory condition: '${sqClean}'.`;

    if (/\b(?:any|no|not|never|absence|without|zero|fail(?:ure)?|violation)\b/i.test(sqL)) {
      oblType = "ABSENCE_CHECK";
      evType = "ABSENCE_PROOF";
      stmt = `Strict verification of absence or non-occurrence regarding '${sqClean}'.`;
    } else if (/\b(?:unless|except|waiver|exempt|override|special\s+case)\b/i.test(sqL)) {
      oblType = "EXCEPTION_RULE";
      evType = "STRUCTURAL_CHILD";
      hasException = true;
      stmt = `Verification of exception rule or conditional waiver regarding '${sqClean}'.`;
    }

    obligations.push({
      obligation_id: oblId,
      type: oblType,
      statement_to_verify: stmt,
      search_focus: sqClean,
      required_evidence_type: evType,
      status: "UNRESOLVED",
      target_manifold: "lorentz"
    });
  });

  const resolutionStrategy: ResolutionStrategy = hasException ? "CONDITIONAL_ON_EXCEPTION" : "ALL_MANDATORY_MET";
  return { primaryIntent, obligations, resolutionStrategy };
}

export function planQuery(input: QueryPlannerInput | string): QueryPlannerResult {
  const params: QueryPlannerInput = typeof input === "string" ? { raw_user_query: input } : input;
  const rawQuery = (params.raw_user_query || "").trim();
  const targetCollections = params.collections && params.collections.length > 0 ? params.collections : ["default"];
  const maxSubQueries = params.max_sub_queries || 5;

  if (!rawQuery) {
    return {
      status: "clarification_needed",
      action: "ask_user_clarification",
      is_intent_clear: false,
      has_multiple_questions: false,
      search_queries: [],
      clarification: {
        question: "Please provide a query or topic you would like to search for in HyperspaceDB.",
        options: [
          "Explore document collections",
          "Search vector similarities",
          "Analyze graph subsumption hierarchies"
        ],
        reason: "The query string was empty."
      },
      reason: "No query provided.",
      execution_strategy: {
        parallel: false,
        target_collections: targetCollections,
        top_k_per_query: 5
      },
      primary_intent: "Empty query",
      obligations: [],
      resolution_strategy: "ALL_MANDATORY_MET"
    };
  }

  const { isAmbiguous, clarification } = evaluateAmbiguity(rawQuery, params.context);
  if (isAmbiguous && clarification) {
    return {
      status: "clarification_needed",
      action: "ask_user_clarification",
      is_intent_clear: false,
      has_multiple_questions: false,
      search_queries: [],
      clarification,
      reason: clarification.reason,
      execution_strategy: {
        parallel: false,
        target_collections: targetCollections,
        top_k_per_query: 5
      },
      primary_intent: `Ambiguous query: '${rawQuery}'`,
      obligations: [],
      resolution_strategy: "ALL_MANDATORY_MET"
    };
  }

  const { primaryIntent, obligations, resolutionStrategy } = decomposeIntoProofObligations(rawQuery, params.context);
  const truncatedObligations = obligations.slice(0, maxSubQueries);
  const isMulti = truncatedObligations.length > 1;
  const searchQueries = truncatedObligations.map(o => o.search_focus);

  return {
    status: "ready_for_search",
    action: isMulti ? "parallel_search" : "direct_search",
    is_intent_clear: true,
    has_multiple_questions: isMulti,
    search_queries: searchQueries,
    clarification: null,
    reason: isMulti
      ? `Formulated ${truncatedObligations.length} MECE proof obligations with resolution strategy '${resolutionStrategy}'.`
      : "Formulated single atomic proof obligation ready for search.",
    execution_strategy: {
      parallel: isMulti,
      target_collections: targetCollections,
      top_k_per_query: isMulti ? 3 : 5
    },
    primary_intent: primaryIntent,
    obligations: truncatedObligations,
    resolution_strategy: resolutionStrategy
  };
}

export const evaluateQueryLocally = planQuery;
export const clarifyAndSplitIntent = planQuery;

