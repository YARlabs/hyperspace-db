import unittest
from hyperspace import (
    fin,
    quant,
    FinSDK,
    quantSDK,
    FinancialEntity,
    to_contagion_cone,
    is_in_default_cascade,
    analyze_market_stability,
    forecast_liquidity_trajectory
)


class TestFinSDK(unittest.TestCase):
    def setUp(self):
        # Tier 1 Global Systemically Important Bank (G-SIB)
        self.gsib_bank = FinancialEntity(
            entity_id=1,
            tier=1,
            leverage_ratio=18.5,
            vector=[0.1, 0.2, 0.05, 0.0]
        )
        # Tier 2 Regional Bank with high direct exposure to G-SIB
        self.regional_bank = FinancialEntity(
            entity_id=2,
            tier=2,
            leverage_ratio=12.0,
            vector=[0.12, 0.22, 0.06, 0.01]
        )
        # Uncorrelated Hedge Fund in alternative assets
        self.uncorrelated_fund = FinancialEntity(
            entity_id=3,
            tier=2,
            leverage_ratio=5.0,
            vector=[12.0, -12.0, 8.0, -8.0]
        )

    def test_contagion_cone_transformation(self):
        cone = to_contagion_cone(self.gsib_bank, tier_scale=3.0)
        self.assertEqual(len(cone), 5)
        self.assertAlmostEqual(cone[0], 3.0)
        self.assertEqual(cone[1:], [0.1, 0.2, 0.05, 0.0])

        with self.assertRaises(ValueError):
            to_contagion_cone(FinancialEntity(entity_id=0, tier=0, leverage_ratio=1.0, vector=[0.1]))

    def test_default_cascade_causality(self):
        cone_gsib = to_contagion_cone(self.gsib_bank, tier_scale=5.0)
        cone_reg = to_contagion_cone(self.regional_bank, tier_scale=5.0)
        cone_uncorr = to_contagion_cone(self.uncorrelated_fund, tier_scale=5.0)

        # G-SIB default cascades down to exposed regional bank
        self.assertTrue(is_in_default_cascade(cone_gsib, cone_reg))

        # Reverse should not cascade: regional bank failure does not automatically engulf G-SIB
        self.assertFalse(is_in_default_cascade(cone_reg, cone_gsib))

        # Uncorrelated fund outside contagion cone is safe from direct default cascade
        self.assertFalse(is_in_default_cascade(cone_gsib, cone_uncorr))

    def test_market_stability_analysis(self):
        c = 1.0
        # Stable mean-reverting price trajectory
        stable_prices = [
            [0.3, 0.2, 0.1, 0.05],
            [0.15, 0.1, 0.05, 0.02],
            [0.05, 0.03, 0.01, 0.005],
            [0.01, 0.005, 0.002, 0.001],
        ]

        res_stable = analyze_market_stability(stable_prices, c=c)
        self.assertTrue(res_stable["is_orderly_regime"])
        self.assertEqual(res_stable["regime"], "MEAN_REVERTING_EQUILIBRIUM")
        self.assertLess(res_stable["lyapunov_derivative"], 0.0)

    def test_liquidity_trajectory_forecasting(self):
        c = 1.0
        # Market liquidity momentum trajectory
        historical_states = [
            [0.1, 0.05, 0.02, 0.01],
            [0.15, 0.08, 0.03, 0.02],
            [0.20, 0.11, 0.04, 0.03],
        ]

        forecast = forecast_liquidity_trajectory(historical_states, steps=1.0, c=c)
        self.assertEqual(len(forecast), 4)
        # Momentum forward step
        self.assertGreater(forecast[0], historical_states[-1][0])

    def test_fin_sdk_class_and_aliases(self):
        self.assertIs(quantSDK, FinSDK)

        sdk = FinSDK(c=1.0)
        cone = sdk.to_cone(self.gsib_bank)
        self.assertEqual(len(cone), 5)


if __name__ == "__main__":
    unittest.main()
