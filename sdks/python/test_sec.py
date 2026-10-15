import unittest
from hyperspace import (
    sec,
    cyber,
    SecSDK,
    cyberSDK,
    SecurityEvent,
    to_spacetime_lorentz,
    is_causally_linked,
    mitre_attack_cone,
    is_technique_subsumed,
    predict_adversary_next_hop,
    detect_behavioral_intrusion
)


class TestSecSDK(unittest.TestCase):
    def setUp(self):
        self.event_recon = SecurityEvent(
            event_id=101,
            timestamp=100.0,
            host_id="srv-web-01",
            technique_id="T1595",
            vector=[0.1, 0.2, 0.05, 0.1]
        )
        self.event_exploit = SecurityEvent(
            event_id=102,
            timestamp=105.0,
            host_id="srv-web-01",
            technique_id="T1190",
            vector=[0.15, 0.25, 0.08, 0.12]
        )
        self.event_past = SecurityEvent(
            event_id=100,
            timestamp=90.0,
            host_id="srv-web-01",
            technique_id="T1078",
            vector=[0.1, 0.2, 0.05, 0.1]
        )
        self.event_superluminal = SecurityEvent(
            event_id=103,
            timestamp=101.0,
            host_id="srv-db-09",
            technique_id="T1041",
            vector=[100.0, -100.0, 50.0, -50.0]  # Spacelike separation
        )

    def test_spacetime_lorentz_transformation(self):
        cone_vec = to_spacetime_lorentz(self.event_recon, c_time=2.0)
        self.assertEqual(len(cone_vec), 5)
        self.assertAlmostEqual(cone_vec[0], 200.0)  # 100 * 2.0
        self.assertEqual(cone_vec[1:], [0.1, 0.2, 0.05, 0.1])

    def test_causality_light_cone(self):
        c_time = 1.0
        v_recon = to_spacetime_lorentz(self.event_recon, c_time=c_time)
        v_exploit = to_spacetime_lorentz(self.event_exploit, c_time=c_time)
        v_past = to_spacetime_lorentz(self.event_past, c_time=c_time)
        v_distant = to_spacetime_lorentz(self.event_superluminal, c_time=c_time)

        # Recon (t=100) -> Exploit (t=105): Timelike future separation, causally linked
        self.assertTrue(is_causally_linked(v_recon, v_exploit))

        # Reverse in time: Exploit cannot cause Recon in the past
        self.assertFalse(is_causally_linked(v_exploit, v_recon))

        # Past event (t=90) cannot be caused by Recon (t=100)
        self.assertFalse(is_causally_linked(v_recon, v_past))

        # Spacelike separation: spatial jump is too large for the 1-second interval
        self.assertFalse(is_causally_linked(v_recon, v_distant))

    def test_mitre_attack_subsumption(self):
        tactic_vec = [0.2, 0.1, 0.05, 0.0]
        # Depth 1: Tactic (TA0002 Execution)
        tactic_cone = mitre_attack_cone(depth_level=1, technique_vector=tactic_vec, level_scale=5.0)

        # Depth 2: Technique (T1059 Command and Scripting Interpreter) - child of execution
        technique_vec = [0.22, 0.12, 0.06, 0.01]
        technique_cone = mitre_attack_cone(depth_level=2, technique_vector=technique_vec, level_scale=5.0)

        self.assertTrue(is_technique_subsumed(tactic_cone, technique_cone))
        self.assertFalse(is_technique_subsumed(technique_cone, tactic_cone))

        # Disparate technique far away
        unrelated_technique_vec = [15.0, -15.0, 10.0, -10.0]
        unrelated_cone = mitre_attack_cone(depth_level=2, technique_vector=unrelated_technique_vec, level_scale=5.0)
        self.assertFalse(is_technique_subsumed(tactic_cone, unrelated_cone))

    def test_adversary_next_hop_prediction(self):
        dim = 4
        c = 1.0
        # Lateral movement progression along network telemetry manifold
        kill_chain_hops = [
            [0.1, 0.05, 0.02, 0.0],
            [0.2, 0.10, 0.04, 0.01],
            [0.3, 0.15, 0.06, 0.02],
        ]

        predicted_next_hop = predict_adversary_next_hop(kill_chain_hops, c=c, steps=1.0)
        self.assertEqual(len(predicted_next_hop), dim)
        # Momentum should continue in positive direction
        self.assertGreater(predicted_next_hop[0], kill_chain_hops[-1][0])

    def test_behavioral_intrusion_detection(self):
        c = 1.0
        # Normal baseline telemetry cluster
        baseline = [
            [0.05, 0.05, 0.02, 0.01],
            [0.06, 0.04, 0.03, 0.01],
            [0.04, 0.06, 0.01, 0.02],
            [0.05, 0.05, 0.02, 0.02],
        ]

        # Normal activity consistent with baseline
        normal_action = [0.052, 0.048, 0.021, 0.012]
        res_normal = detect_behavioral_intrusion(normal_action, baseline, c=c, threshold=0.1)
        self.assertFalse(res_normal["is_intrusion_detected"])
        self.assertEqual(res_normal["risk_level"], "NORMAL")

        # Zero-day anomalous burst (in-ball distant point)
        malicious_action = [-0.4, -0.4, 0.2, -0.2]
        res_anomaly = detect_behavioral_intrusion(malicious_action, baseline, c=c, threshold=0.1)
        self.assertTrue(res_anomaly["is_intrusion_detected"])
        self.assertIn(res_anomaly["risk_level"], ["SUSPICIOUS", "CRITICAL"])

        # Extreme out-of-boundary action
        boundary_burst = [2.0, 0.0, 0.0, 0.0]
        res_boundary = detect_behavioral_intrusion(boundary_burst, baseline, c=c)
        self.assertTrue(res_boundary["is_intrusion_detected"])
        self.assertEqual(res_boundary["risk_level"], "CRITICAL")

    def test_sec_sdk_class_and_aliases(self):
        # Test that cyberSDK is an exact alias of SecSDK
        self.assertIs(cyberSDK, SecSDK)

        sdk = SecSDK(c=1.0)
        v = sdk.to_lorentz_event(self.event_recon)
        self.assertEqual(len(v), 5)

        with self.assertRaises(RuntimeError):
            sdk.index_event(self.event_recon)


if __name__ == "__main__":
    unittest.main()
