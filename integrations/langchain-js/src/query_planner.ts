/**
 * HyperspaceDB Formal Proof-Obligation Engine & Query Planner for LangChain.js
 *
 * Upgrades query planning from naive sub-query splitting into a rigorous
 * "Questions as Proof Obligations" paradigm for regulated enterprise domains.
 * Decomposes user queries into mutually exclusive, collectively exhaustive (MECE)
 * proof obligations to guide vector, light-cone, and geodesic traversal in HyperspaceDB.
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

function cleanQuery(text: string): string {
  let cleaned = text.trim();
  cleaned = cleaned.replace(
    /^(?:please\s+|can\s+you\s+(?:please\s+)?(?:tell\s+me\s+|explain\s+|show\s+me\s+)?|could\s+you\s+|i\s+(?:would\s+like|want)\s+to\s+know\s+|tell\s+me\s+about\s+)/i,
    ""
  );
  return cleaned.replace(/[.!?]+$/, "").trim();
}

function decomposeIntoProofObligations(
  rawQuery: string
): { primaryIntent: string; obligations: ProofObligation[]; resolutionStrategy: ResolutionStrategy } {
  const cleaned = cleanQuery(rawQuery);
  const qLower = cleaned.toLowerCase();

  // Pattern 1: Regulatory Exception / Exemption Analysis
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

    return {
      primaryIntent,
      obligations: [
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
      ],
      resolutionStrategy: "CONDITIONAL_ON_EXCEPTION"
    };
  }

  // Pattern 2: Proof of Absence / Negative Assurance
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

    return {
      primaryIntent,
      obligations: [
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
      ],
      resolutionStrategy: "ALL_MANDATORY_MET"
    };
  }

  // Pattern 3: Multi-Hop Root Cause Investigation
  if (
    /\b(?:why\s+did|what\s+caused|root\s+cause\s+of|reason\s+for|investigate\s+discrepancy\s+in)\b/i.test(qLower) &&
    /\b(?:fail(?:ed|ure)?|quarantine|reject(?:ed|ion)?|defect|discrepancy|out\s+of\s+spec|halt(?:ed)?)\b/i.test(qLower)
  ) {
    const primaryIntent = `Root-cause multi-hop investigation across records, specs, and procedures for: '${cleaned}'`;
    const batchMatch = cleaned.match(/\b(batch|lot|sample|order|unit)\s+([a-zA-Z0-9_\-]+)/i);
    const targetSubject = batchMatch ? batchMatch[0].replace(/\b\w/g, l => l.toUpperCase()) : "Subject Batch";

    return {
      primaryIntent,
      obligations: [
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
      ],
      resolutionStrategy: "ALL_MANDATORY_MET"
    };
  }

  // Fallback: General inquiry
  return {
    primaryIntent: `Evaluate verified compliance and factual resolution for: '${cleaned}'`,
    obligations: [
      {
        obligation_id: "obl_01",
        type: "MANDATORY_PRECONDITION",
        statement_to_verify: `Factual verification of mandatory condition: '${cleaned}'.`,
        search_focus: cleaned,
        required_evidence_type: "EXPLICIT_TEXT",
        status: "UNRESOLVED",
        target_manifold: "lorentz"
      }
    ],
    resolutionStrategy: "ALL_MANDATORY_MET"
  };
}

export function evaluateQueryLocally(input: QueryPlannerInput): QueryPlannerResult {
  const raw = (input.raw_user_query || "").trim();
  const maxQueries = input.max_sub_queries || 5;
  const collections = input.collections || ["default"];

  if (!raw) {
    return {
      status: "clarification_needed",
      action: "ask_user_clarification",
      is_intent_clear: false,
      has_multiple_questions: false,
      search_queries: [],
      clarification: {
        question: "Please state your question or search query for HyperspaceDB.",
        options: ["General search", "Compliance policy review", "Technical specifications"],
        reason: "Empty query"
      },
      reason: "No query provided.",
      execution_strategy: { parallel: false, target_collections: collections, top_k_per_query: 5 },
      primary_intent: "Empty query",
      obligations: [],
      resolution_strategy: "ALL_MANDATORY_MET"
    };
  }

  const { primaryIntent, obligations, resolutionStrategy } = decomposeIntoProofObligations(raw);
  const truncatedObligations = obligations.slice(0, maxQueries);
  const isMulti = truncatedObligations.length > 1;

  return {
    status: "ready_for_search",
    action: isMulti ? "parallel_search" : "direct_search",
    is_intent_clear: true,
    has_multiple_questions: isMulti,
    search_queries: truncatedObligations.map(o => o.search_focus),
    clarification: null,
    reason: `Formulated ${truncatedObligations.length} MECE proof obligations with resolution strategy '${resolutionStrategy}'.`,
    execution_strategy: {
      parallel: isMulti,
      target_collections: collections,
      top_k_per_query: isMulti ? 3 : 5
    },
    primary_intent: primaryIntent,
    obligations: truncatedObligations,
    resolution_strategy: resolutionStrategy
  };
}

export async function clarifyAndSplitIntent(input: QueryPlannerInput): Promise<QueryPlannerResult> {
  return evaluateQueryLocally(input);
}

export class HyperspaceQueryPlanner {
  public name = "hyperspace_query_planner";
  public description = "Formal Proof-Obligation Engine. Decomposes enterprise questions into MECE proof obligations (MANDATORY_PRECONDITION, EXCEPTION_RULE, ABSENCE_CHECK).";
  public schema = QUERY_PLANNER_TOOL_SCHEMA;

  async call(input: QueryPlannerInput | string): Promise<QueryPlannerResult> {
    if (typeof input === "string") {
      return evaluateQueryLocally({ raw_user_query: input });
    }
    return evaluateQueryLocally(input);
  }

  async planAndExecute(query: string, store?: any): Promise<any> {
    const plan = evaluateQueryLocally({ raw_user_query: query });
    const steps: Array<{ sub_query: string; results: any[] }> = [];

    if (store && store.similaritySearch) {
      for (const obl of plan.obligations) {
        try {
          const res = await store.similaritySearch(obl.search_focus, 3);
          steps.push({ sub_query: obl.search_focus, results: res });
        } catch {
          steps.push({ sub_query: obl.search_focus, results: [] });
        }
      }
    }

    return {
      plan,
      steps,
      primary_intent: plan.primary_intent,
      resolution_strategy: plan.resolution_strategy,
      obligations: plan.obligations
    };
  }
}
