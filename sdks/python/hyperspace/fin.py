"""
HyperspaceDB finSDK / quantSDK: Systemic Contagion & Market Regime Manifolds Engine.

Provides systemic financial contagion light-cone modeling (interbank counterparty exposure),
market regime stability analysis via Lyapunov exponents (mean-reverting vs liquidity crisis),
and hyperbolic trajectory momentum forecasting via Koopman operators.
"""

from typing import Sequence, List, Dict, Any, Optional, Union
from dataclasses import dataclass, field
from . import math as hs_math


@dataclass
class FinancialEntity:
    entity_id: Union[str, int]
    tier: int  # 1: Central Bank/Sovereign, 2: G-SIB / Tier 1 Bank, 3: Regional Bank/Fund, 4: Corporation
    leverage_ratio: float
    vector: Sequence[float]
    metadata: Dict[str, Any] = field(default_factory=dict)


def to_contagion_cone(
    entity: FinancialEntity,
    tier_scale: float = 2.0
) -> List[float]:
    """
    Embeds a financial institution into Lorentz spacetime to model systemic default cascades.
    Higher systemic tier institutions (tier=1) have wider contagion cones that can engulf
    downstream counterparties in a credit liquidity crisis.
    """
    if entity.tier < 1:
        raise ValueError("Entity tier must be >= 1")
    t = float(entity.tier) * tier_scale
    spatial = list(entity.vector)
    return [t] + spatial


def is_in_default_cascade(
    source_institution_cone: Sequence[float],
    downstream_institution_cone: Sequence[float],
    epsilon: float = 1e-5
) -> bool:
    """Verifies whether a downstream institution lies within the default cascade cone of a failing institution."""
    return hs_math.is_minkowski_subsumed(source_institution_cone, downstream_institution_cone, epsilon=epsilon)


def analyze_market_stability(
    price_trajectory: Sequence[Sequence[float]],
    c: float = 1.0
) -> Dict[str, Any]:
    """
    Evaluates market regime stability on the financial manifold using Lyapunov exponents:
    - Negative Lyapunov derivative indicates orderly mean-reverting price discovery (attractor).
    - Positive Lyapunov derivative indicates market phase transition, chaotic breakdown, or flash crash cascade.
    """
    if len(price_trajectory) < 3:
        raise ValueError("At least 3 sequential market states required for trajectory evaluation")

    trend = hs_math.lyapunov_convergence(price_trajectory, c=c)
    is_stable = trend < 0.0

    return {
        "is_orderly_regime": is_stable,
        "lyapunov_derivative": trend,
        "regime": "MEAN_REVERTING_EQUILIBRIUM" if is_stable else "CHAOTIC_RUNAWAY_INSTABILITY"
    }


def forecast_liquidity_trajectory(
    historical_states: Sequence[Sequence[float]],
    steps: float = 1.0,
    c: float = 1.0
) -> List[float]:
    """
    Forecasts future market/liquidity state along the hyperbolic manifold using
    Koopman operator linearization.
    """
    if len(historical_states) < 2:
        raise ValueError("At least 2 historical market states are required")
    return hs_math.koopman_extrapolate(
        historical_states[-2],
        historical_states[-1],
        steps=steps,
        c=c
    )


class FinSDK:
    """
    High-level Quantitative Risk & Market Dynamics SDK for HyperspaceDB.
    """
    def __init__(self, client: Optional[Any] = None, collection: str = "market_dynamics", c: float = 1.0):
        self.client = client
        self.collection = collection
        self.c = c

    def to_cone(self, entity: FinancialEntity) -> List[float]:
        return to_contagion_cone(entity)

    def verify_cascade(self, source_cone: Sequence[float], target_cone: Sequence[float]) -> bool:
        return is_in_default_cascade(source_cone, target_cone)

    def evaluate_stability(self, price_trajectory: Sequence[Sequence[float]]) -> Dict[str, Any]:
        return analyze_market_stability(price_trajectory, c=self.c)

    def forecast_regime(self, historical_states: Sequence[Sequence[float]], steps: float = 1.0) -> List[float]:
        return forecast_liquidity_trajectory(historical_states, steps=steps, c=self.c)
