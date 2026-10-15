"""
HyperspaceDB secSDK / cyberSDK: Threat Intelligence & Causal Attack Graph Engine.

Provides Minkowski spacetime causal light cone verification for incident forensics,
MITRE ATT&CK tactical subsumption hierarchies, lateral movement prediction via Koopman
linearization, and behavioral zero-day anomaly detection via local entropy.
"""

from typing import Sequence, List, Dict, Any, Optional, Union
from dataclasses import dataclass, field
from . import math as hs_math


@dataclass
class SecurityEvent:
    event_id: Union[str, int]
    timestamp: float
    host_id: str
    technique_id: str
    vector: Sequence[float]
    metadata: Dict[str, Any] = field(default_factory=dict)


def to_spacetime_lorentz(
    event: SecurityEvent,
    c_time: float = 1.0
) -> List[float]:
    """
    Transforms a security event into Minkowski spacetime coordinates (c*t, x_1, ..., x_d).
    Enables physical causal light cone calculations to verify whether an event could have triggered another.
    """
    t = float(event.timestamp) * c_time
    spatial = list(event.vector)
    return [t] + spatial


def is_causally_linked(
    source_event_vec: Sequence[float],
    target_event_vec: Sequence[float],
    epsilon: float = 1e-5
) -> bool:
    """
    Proves or disproves causality between two security events using Minkowski spacetime separation:
    Event A can only be the cause of Event B if:
    1. B occurred after or simultaneously with A (t_B >= t_A)
    2. B lies within or on the future light cone of A: -(t_B - t_A)^2 + ||x_B - x_A||^2 <= 0.
    """
    return hs_math.is_minkowski_subsumed(source_event_vec, target_event_vec, epsilon=epsilon)


def mitre_attack_cone(
    depth_level: int,
    technique_vector: Sequence[float],
    level_scale: float = 2.0
) -> List[float]:
    """
    Embeds MITRE ATT&CK tactics, techniques, and procedures (TTPs) into Lorentz subsumption cones.
    Depth 1: Tactic (e.g. TA0002 Execution)
    Depth 2: Technique (e.g. T1059 Command/Scripting Interpreter)
    Depth 3: Sub-technique (e.g. T1059.001 PowerShell)
    """
    if depth_level < 1:
        raise ValueError("Depth level must be >= 1")
    t = float(depth_level) * level_scale
    spatial = list(technique_vector)
    return [t] + spatial


def is_technique_subsumed(
    tactic_cone: Sequence[float],
    technique_cone: Sequence[float],
    epsilon: float = 1e-5
) -> bool:
    """Verifies if a specific technique or procedure is topologically subsumed by a tactical campaign."""
    return hs_math.is_minkowski_subsumed(tactic_cone, technique_cone, epsilon=epsilon)


def predict_adversary_next_hop(
    attack_path_vectors: Sequence[Sequence[float]],
    c: float = 1.0,
    steps: float = 1.0
) -> List[float]:
    """
    Predicts the adversary's next lateral movement hop or credential access target in behavioral
    feature space by projecting momentum along the hyperbolic manifold via Koopman operator.
    """
    if len(attack_path_vectors) < 2:
        raise ValueError("At least 2 attack path vectors are required to evaluate velocity")
    return hs_math.koopman_extrapolate(
        attack_path_vectors[-2],
        attack_path_vectors[-1],
        steps=steps,
        c=c
    )


def detect_behavioral_intrusion(
    action_vector: Sequence[float],
    baseline_profile: Sequence[Sequence[float]],
    c: float = 1.0,
    threshold: float = 0.2
) -> Dict[str, Any]:
    """
    Detects zero-day anomalies, abnormal privilege escalations, or data exfiltration by computing
    the local spatial entropy against the host's historical behavioral manifold.
    """
    if not baseline_profile:
        raise ValueError("Baseline profile cannot be empty")

    cand_norm_sq = hs_math._norm_sq(action_vector)
    if cand_norm_sq >= (1.0 / c):
        return {
            "is_intrusion_detected": True,
            "anomaly_entropy": 1.0,
            "threshold": threshold,
            "risk_level": "CRITICAL",
            "status": "out_of_manifold_boundary"
        }

    entropy = hs_math.local_entropy(action_vector, baseline_profile, c=c)
    is_anomaly = entropy >= threshold

    return {
        "is_intrusion_detected": is_anomaly,
        "anomaly_entropy": entropy,
        "threshold": threshold,
        "risk_level": "CRITICAL" if entropy > 0.5 else ("SUSPICIOUS" if is_anomaly else "NORMAL")
    }


class SecSDK:
    """
    High-level Cyber Threat Intelligence & Causal Forensics SDK for HyperspaceDB.
    """
    def __init__(self, client: Optional[Any] = None, collection: str = "threat_intel", c: float = 1.0):
        self.client = client
        self.collection = collection
        self.c = c

    def to_lorentz_event(self, event: SecurityEvent, c_time: float = 1.0) -> List[float]:
        return to_spacetime_lorentz(event, c_time=c_time)

    def verify_causality(self, event_a_vec: Sequence[float], event_b_vec: Sequence[float]) -> bool:
        return is_causally_linked(event_a_vec, event_b_vec)

    def forecast_killchain(self, attack_hops: Sequence[Sequence[float]], steps: float = 1.0) -> List[float]:
        return predict_adversary_next_hop(attack_hops, c=self.c, steps=steps)

    def assess_telemetry(self, current_action: Sequence[float], baseline: Sequence[Sequence[float]]) -> Dict[str, Any]:
        return detect_behavioral_intrusion(current_action, baseline, c=self.c)

    def index_event(self, event: SecurityEvent, c_time: float = 1.0):
        if not self.client:
            raise RuntimeError("HyperspaceClient is required for database operations")
        vec = self.to_lorentz_event(event, c_time=c_time)
        meta = dict(event.metadata)
        meta["host_id"] = event.host_id
        meta["technique_id"] = event.technique_id
        meta["timestamp"] = event.timestamp
        return self.client.insert(id=int(event.event_id), vector=vec, metadata=meta, collection=self.collection)
