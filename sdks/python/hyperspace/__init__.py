from .client import HyperspaceClient, Durability, VerificationResult
from .depin import DePINClient
from .embedders import (
    BaseEmbedder,
    OpenAIEmbedder,
    OpenRouterEmbedder,
    CohereEmbedder,
    VoyageEmbedder,
    GoogleEmbedder,
    SentenceTransformerEmbedder
)
from .math import (
    mobius_add, 
    exp_map, 
    log_map, 
    parallel_transport, 
    riemannian_gradient, 
    frechet_mean,
    poincare_dist,
    minkowski_interval,
    is_minkowski_subsumed,
    local_entropy,
    lyapunov_convergence,
    koopman_extrapolate,
    context_resonance,
    lorentz_product,
    lorentz_dist,
    lorentz_to_poincare,
    poincare_to_lorentz,
    project_to_hyperboloid,
    generate_orthogonal_matrix,
    generate_lorentz_matrix,
    project_vector,
    inject_anisotropic_noise
)
from .agents import TribunalContext
from . import query_planner
from .query_planner import (
    plan_query,
    QueryPlannerResult,
    ProofObligation,
    ProofObligationPlan,
    ClarificationPayload,
    ExecutionStrategy,
    QUERY_PLANNER_TOOL_SCHEMA,
    LOCAL_LLM_PLANNER_PROMPT,
)

# Specialized Domain SDK modules
from . import chem
from . import bio
from . import bio as genomics
from . import sec
from . import sec as cyber
from . import legal
from . import legal as patent
from . import onto
from . import onto as kg
from . import fin
from . import fin as quant

# chemSDK exports
from .chem import (
    ChemSDK,
    Molecule,
    smiles_to_fingerprint,
    tanimoto_similarity,
    scaffold_to_lorentz,
    is_substructure_subsumed,
    reaction_step,
    estimate_reaction_vector,
    verify_chemical_validity,
)

# bioSDK exports
from .bio import (
    BioSDK,
    PhyloNode,
    embed_phylogenetic_tree,
    evolutionary_distance,
    ancestral_state_reconstruction,
    protein_domain_lorentz,
    is_domain_subsumed,
    analyze_differentiation_trajectory,
)

# secSDK exports
from .sec import (
    SecSDK,
    SecurityEvent,
    to_spacetime_lorentz,
    is_causally_linked,
    mitre_attack_cone,
    is_technique_subsumed,
    predict_adversary_next_hop,
    detect_behavioral_intrusion,
)

# legalSDK exports
from .legal import (
    LegalSDK,
    LegalNorm,
    norm_to_lorentz_cone,
    is_norm_valid_under,
    patent_claim_cone,
    is_claim_subsumed,
    doctrine_of_equivalents,
    synthesize_precedents,
)

# ontoSDK exports
from .onto import (
    OntoSDK,
    Triple,
    hyperbolic_translate,
    score_triple,
    concept_subsumption_cone,
    is_concept_subsumed,
    align_ontologies,
)

# finSDK exports
from .fin import (
    FinSDK,
    FinancialEntity,
    to_contagion_cone,
    is_in_default_cascade,
    analyze_market_stability,
    forecast_liquidity_trajectory,
)

# Convenient camelCase aliases
chemSDK = ChemSDK
bioSDK = BioSDK
genomicsSDK = BioSDK
secSDK = SecSDK
cyberSDK = SecSDK
legalSDK = LegalSDK
patentSDK = LegalSDK
kgSDK = OntoSDK
ontoSDK = OntoSDK
finSDK = FinSDK
quantSDK = FinSDK

__all__ = [
    "HyperspaceClient",
    "DePINClient",
    "Durability",
    "VerificationResult",
    "BaseEmbedder",
    "OpenAIEmbedder",
    "OpenRouterEmbedder",
    "CohereEmbedder",
    "VoyageEmbedder",
    "GoogleEmbedder",
    "SentenceTransformerEmbedder",
    "mobius_add",
    "exp_map",
    "log_map",
    "parallel_transport",
    "riemannian_gradient",
    "frechet_mean",
    "poincare_dist",
    "minkowski_interval",
    "is_minkowski_subsumed",
    "local_entropy",
    "lyapunov_convergence",
    "koopman_extrapolate",
    "context_resonance",
    "lorentz_product",
    "lorentz_dist",
    "lorentz_to_poincare",
    "poincare_to_lorentz",
    "project_to_hyperboloid",
    "generate_orthogonal_matrix",
    "generate_lorentz_matrix",
    "project_vector",
    "inject_anisotropic_noise",
    "TribunalContext",
    # Modules
    "chem",
    "bio",
    "genomics",
    "sec",
    "cyber",
    "legal",
    "patent",
    "onto",
    "kg",
    "fin",
    "quant",
    # chemSDK
    "ChemSDK",
    "Molecule",
    "smiles_to_fingerprint",
    "tanimoto_similarity",
    "scaffold_to_lorentz",
    "is_substructure_subsumed",
    "reaction_step",
    "estimate_reaction_vector",
    "verify_chemical_validity",
    # bioSDK
    "BioSDK",
    "PhyloNode",
    "embed_phylogenetic_tree",
    "evolutionary_distance",
    "ancestral_state_reconstruction",
    "protein_domain_lorentz",
    "is_domain_subsumed",
    "analyze_differentiation_trajectory",
    # secSDK
    "SecSDK",
    "SecurityEvent",
    "to_spacetime_lorentz",
    "is_causally_linked",
    "mitre_attack_cone",
    "is_technique_subsumed",
    "predict_adversary_next_hop",
    "detect_behavioral_intrusion",
    # legalSDK
    "LegalSDK",
    "LegalNorm",
    "norm_to_lorentz_cone",
    "is_norm_valid_under",
    "patent_claim_cone",
    "is_claim_subsumed",
    "doctrine_of_equivalents",
    "synthesize_precedents",
    # ontoSDK
    "OntoSDK",
    "Triple",
    "hyperbolic_translate",
    "score_triple",
    "concept_subsumption_cone",
    "is_concept_subsumed",
    "align_ontologies",
    # finSDK
    "FinSDK",
    "FinancialEntity",
    "to_contagion_cone",
    "is_in_default_cascade",
    "analyze_market_stability",
    "forecast_liquidity_trajectory",
    # Aliases
    "chemSDK",
    "bioSDK",
    "genomicsSDK",
    "secSDK",
    "cyberSDK",
    "legalSDK",
    "patentSDK",
    "kgSDK",
    "ontoSDK",
    "finSDK",
    "quantSDK",
]
