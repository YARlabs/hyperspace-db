import unittest
import numpy as np
import hashlib
from hyperspace import crypto
from hyperspace import math as hs_math

class TestZKPrivacy(unittest.TestCase):
    def setUp(self):
        self.password = "super-secret-password-123"
        self.collection_name = "test_private_collection"
        self.aes_key, self.hmac_key = crypto.derive_keys(self.password, self.collection_name)

    def test_payload_encryption_decryption(self):
        """Test AES-256-GCM encryption and decryption of payload."""
        payload = b"Sensitive bot instruction: act like a polite assistant."
        
        # Encrypt
        ciphertext = crypto.encrypt_payload(payload, self.aes_key)
        self.assertNotEqual(payload, ciphertext)
        self.assertTrue(len(ciphertext) > 12)  # Should contain IV and tag
        
        # Decrypt
        decrypted = crypto.decrypt_payload(ciphertext, self.aes_key)
        self.assertEqual(payload, decrypted)

    def test_metadata_obfuscation(self):
        """Test HMAC-SHA256 deterministic hashing of metadata keys and values."""
        key = "author"
        val = "John Doe"
        
        ek1 = crypto.hash_metadata_key(key, self.hmac_key)
        ev1 = crypto.hash_metadata_value(val, self.hmac_key)
        
        # Hashing should be deterministic (same outputs)
        ek2 = crypto.hash_metadata_key(key, self.hmac_key)
        ev2 = crypto.hash_metadata_value(val, self.hmac_key)
        
        self.assertEqual(ek1, ek2)
        self.assertEqual(ev1, ev2)
        self.assertTrue(ek1.startswith("tag_"))
        self.assertTrue(ev1.startswith("val_"))
        
        # Hashing should produce different outputs for different keys/values
        ek_diff = crypto.hash_metadata_key("department", self.hmac_key)
        ev_diff = crypto.hash_metadata_value("Jane", self.hmac_key)
        
        self.assertNotEqual(ek1, ek_diff)
        self.assertNotEqual(ev1, ev_diff)

    def test_anisotropic_noise(self):
        """Test that deterministic anisotropic noise is correctly injected and preserves the norm."""
        dim = 128
        rng = np.random.default_rng(100)
        u = rng.standard_normal(dim).tolist()
        
        # Inject noise with sigma = 0.02
        u_noisy = hs_math.inject_anisotropic_noise(u, self.hmac_key, sigma=0.02)
        
        # Noise should be deterministic for the same vector and seed
        u_noisy_again = hs_math.inject_anisotropic_noise(u, self.hmac_key, sigma=0.02)
        self.assertEqual(u_noisy, u_noisy_again)
        
        # Noise should be different for a different vector
        v = rng.standard_normal(dim).tolist()
        v_noisy = hs_math.inject_anisotropic_noise(v, self.hmac_key, sigma=0.02)
        self.assertNotEqual(u_noisy, v_noisy)
        
        # The norm should be perfectly preserved (for cosine/hyperbolic metric consistency)
        norm_orig = np.linalg.norm(u)
        norm_noisy = np.linalg.norm(u_noisy)
        self.assertAlmostEqual(norm_orig, norm_noisy, places=10)
        
        # The noisy vector should be close but not identical to the original vector
        self.assertNotEqual(u, u_noisy)
        dist = np.linalg.norm(np.array(u) - np.array(u_noisy))
        # Expected distance should be small
        self.assertTrue(dist < 0.5)

    def test_euclidean_orthogonal_projection(self):
        """Test that generated orthogonal matrices preserve L2 distance and dot products."""
        dim = 128
        O = hs_math.generate_orthogonal_matrix(dim, self.hmac_key)
        
        # Verify O is orthogonal (O^T * O = I)
        I_approx = np.matmul(O.T, O)
        np.testing.assert_array_almost_equal(I_approx, np.eye(dim), decimal=10)
        
        # Generate two random vectors
        rng = np.random.default_rng(42)
        u = rng.standard_normal(dim).tolist()
        v = rng.standard_normal(dim).tolist()
        
        # Project vectors
        u_proj = hs_math.project_vector(u, O)
        v_proj = hs_math.project_vector(v, O)
        
        # Verify L2 distance is preserved
        dist_orig = np.linalg.norm(np.array(u) - np.array(v))
        dist_proj = np.linalg.norm(np.array(u_proj) - np.array(v_proj))
        self.assertAlmostEqual(dist_orig, dist_proj, places=10)
        
        # Verify inner product is preserved
        dot_orig = np.dot(u, v)
        dot_proj = np.dot(u_proj, v_proj)
        self.assertAlmostEqual(dot_orig, dot_proj, places=10)

    def test_lorentz_hyperbolic_projection(self):
        """Test that Lorentz transformation matrices preserve Minkowski product and hyperbolic distance."""
        dim = 33  # 1 time dimension + 32 spatial dimensions
        Lambda = hs_math.generate_lorentz_matrix(dim, self.hmac_key)
        
        # Verify Lambda is a Lorentz matrix (Lambda^T * J * Lambda = J)
        J = np.eye(dim)
        J[0, 0] = -1.0
        
        J_approx = np.matmul(np.matmul(Lambda.T, J), Lambda)
        np.testing.assert_array_almost_equal(J_approx, J, decimal=10)
        
        # Generate two valid Lorentz vectors (on the hyperboloid: -x0^2 + |x|^2 = -1)
        rng = np.random.default_rng(24)
        
        # Generate spatial components
        u_spatial = rng.standard_normal(dim - 1)
        # Compute time component: u0 = sqrt(1 + |u_spatial|^2)
        u0 = np.sqrt(1.0 + np.sum(u_spatial**2))
        u = [u0] + u_spatial.tolist()
        
        v_spatial = rng.standard_normal(dim - 1)
        v0 = np.sqrt(1.0 + np.sum(v_spatial**2))
        v = [v0] + v_spatial.tolist()
        
        # Verify original vectors satisfy constraint
        self.assertAlmostEqual(-u[0]**2 + sum(x**2 for x in u[1:]), -1.0, places=10)
        self.assertAlmostEqual(-v[0]**2 + sum(x**2 for x in v[1:]), -1.0, places=10)
        
        # Project vectors
        u_proj = hs_math.project_vector(u, Lambda)
        v_proj = hs_math.project_vector(v, Lambda)
        
        # Verify projected vectors also satisfy hyperboloid constraint (are valid Lorentz points)
        self.assertAlmostEqual(-u_proj[0]**2 + sum(x**2 for x in u_proj[1:]), -1.0, places=10)
        self.assertAlmostEqual(-v_proj[0]**2 + sum(x**2 for x in v_proj[1:]), -1.0, places=10)
        
        # Verify Minkowski inner product is preserved
        dot_orig = hs_math.lorentz_product(u, v)
        dot_proj = hs_math.lorentz_product(u_proj, v_proj)
        self.assertAlmostEqual(dot_orig, dot_proj, places=10)
        
        # Verify hyperbolic distance is preserved
        dist_orig = hs_math.lorentz_dist(u, v)
        dist_proj = hs_math.lorentz_dist(u_proj, v_proj)
        self.assertAlmostEqual(dist_orig, dist_proj, places=10)


class MockDatabaseStub:
    def __init__(self):
        self.last_insert_request = None
        self.last_search_request = None
        self.last_update_payload_request = None
        self.last_scroll_request = None
        self.last_count_request = None
        self.database = {}  # id -> payload

    def Insert(self, request, metadata=None):
        self.last_insert_request = request
        self.database[request.id] = request.payload
        class Response:
            success = True
        return Response()

    def Search(self, request, metadata=None):
        self.last_search_request = request
        payload = list(self.database.values())[0] if self.database else None
        class Response:
            class Result:
                def __init__(self, id, distance, payload, metadata_dict):
                    self.id = id
                    self.distance = distance
                    self.payload = payload
                    self.metadata = metadata_dict
                    self.typed_metadata = {}
            results = [Result(42, 0.1, payload, {})]
        return Response()

    def UpdatePayload(self, request, metadata=None):
        self.last_update_payload_request = request
        class Response:
            code = 0
        return Response()

    def Scroll(self, request, metadata=None):
        self.last_scroll_request = request
        class Response:
            points = []
        return Response()

    def Count(self, request, metadata=None):
        self.last_count_request = request
        class Response:
            count = 5
        return Response()


class TestClientTransparentWrapping(unittest.TestCase):
    def test_transparent_client_wrapping(self):
        """Test that HyperspaceClient encrypts inserts and decrypts search results transparently."""
        from hyperspace import HyperspaceClient
        
        # Create client and register key
        client = HyperspaceClient(host="dummy_host", collection_keys={"secure_box": "secret_pass"})
        client.register_collection_key("secure_box", "secret_pass", metric="l2")
        
        # Mock the gRPC stub
        mock_stub = MockDatabaseStub()
        client.stubs = [mock_stub] * client.num_channels
        client._thread_local.stub = mock_stub
        
        # Insert a document and metadata
        vector = [0.1, 0.2, 0.3]
        doc = "Top Secret Finance Report"
        meta = {"category": "private"}
        
        client.insert(id=1, vector=vector, document=doc, metadata=meta, collection="secure_box")
        
        # Verify gRPC insert request was obfuscated
        inserted_req = mock_stub.last_insert_request
        self.assertIsNotNone(inserted_req)
        
        # Vector should be projected (different from original)
        self.assertNotEqual(list(inserted_req.vector), vector)
        
        # Payload should be encrypted bytes
        self.assertNotEqual(inserted_req.payload, doc.encode('utf-8'))
        
        # Metadata key and value should be hashed
        self.assertNotIn("category", inserted_req.metadata)
        self.assertNotIn("private", inserted_req.metadata.values())
        
        # Retrieve through search
        results = client.search(vector=vector, filter={"category": "private"}, collection="secure_box")
        
        # Verify query vector and filter were obfuscated
        search_req = mock_stub.last_search_request
        self.assertIsNotNone(search_req)
        self.assertNotEqual(list(search_req.vector), vector)
        self.assertNotIn("category", search_req.filter)
        
        # Verify returned result was transparently decrypted
        self.assertEqual(len(results), 1)
        self.assertEqual(results[0]["id"], 42)
        self.assertEqual(results[0]["payload"], doc)

        # Test update_payload hashing
        client.update_payload(id=1, metadata={"category": "private"}, collection="secure_box")
        update_req = mock_stub.last_update_payload_request
        self.assertIsNotNone(update_req)
        self.assertNotIn("category", update_req.metadata)
        self.assertNotIn("private", update_req.metadata.values())
        
        # Test scroll filter hashing
        client.scroll(limit=10, filters=[{"match": {"key": "category", "value": "private"}}], collection="secure_box")
        scroll_req = mock_stub.last_scroll_request
        self.assertIsNotNone(scroll_req)
        self.assertTrue(len(scroll_req.filters) > 0)
        self.assertTrue(scroll_req.filters[0].match.key.startswith("tag_"))
        self.assertTrue(scroll_req.filters[0].match.value.startswith("val_"))
        
        # Test count filter hashing
        client.count(filters=[{"match": {"key": "category", "value": "private"}}], collection="secure_box")
        count_req = mock_stub.last_count_request
        self.assertIsNotNone(count_req)
        self.assertTrue(len(count_req.filters) > 0)
        self.assertTrue(count_req.filters[0].match.key.startswith("tag_"))
        self.assertTrue(count_req.filters[0].match.value.startswith("val_"))


class TestPoincareAndMRL(unittest.TestCase):
    def setUp(self):
        self.password = "mrl-poincare-secret-123"
        self.collection_name = "test_adv_collection"
        self.aes_key, self.hmac_key = crypto.derive_keys(self.password, self.collection_name)

    def test_poincare_model_projection(self):
        """Test that Poincaré ball vectors are projected into valid Poincaré vectors and distance is preserved."""
        dim = 32
        rng = np.random.default_rng(42)
        # Create vectors in Poincaré ball (norm < 1)
        u = (rng.standard_normal(dim) * 0.1).tolist()
        v = (rng.standard_normal(dim) * 0.1).tolist()
        
        # Verify norms are less than 1
        self.assertTrue(np.linalg.norm(u) < 1.0)
        self.assertTrue(np.linalg.norm(v) < 1.0)
        
        # Generate Poincaré Lorentz matrix (dim + 1 = 33)
        matrix = hs_math.generate_lorentz_matrix(dim + 1, self.hmac_key)
        
        # Project Poincaré vectors
        u_lorentz = hs_math.poincare_to_lorentz(u)
        u_lorentz_proj = hs_math.project_vector(u_lorentz, matrix)
        u_proj = hs_math.lorentz_to_poincare(u_lorentz_proj)
        
        v_lorentz = hs_math.poincare_to_lorentz(v)
        v_lorentz_proj = hs_math.project_vector(v_lorentz, matrix)
        v_proj = hs_math.lorentz_to_poincare(v_lorentz_proj)
        
        # Projected vectors must still lie inside the Poincaré ball (norm < 1)
        self.assertTrue(np.linalg.norm(u_proj) < 1.0)
        self.assertTrue(np.linalg.norm(v_proj) < 1.0)
        
        # Verify Poincaré distances are preserved
        # In Poincaré ball model, distance is: acosh(1 + 2 * ||u-v||^2 / ((1-||u||^2)(1-||v||^2)))
        def poincare_dist(x, y):
            x_arr = np.array(x)
            y_arr = np.array(y)
            x_sq = np.dot(x_arr, x_arr)
            y_sq = np.dot(y_arr, y_arr)
            diff_sq = np.sum((x_arr - y_arr) ** 2)
            denom = (1.0 - x_sq) * (1.0 - y_sq)
            val = 1.0 + 2.0 * diff_sq / denom
            return np.arccosh(val)
            
        dist_orig = poincare_dist(u, v)
        dist_proj = poincare_dist(u_proj, v_proj)
        
        self.assertAlmostEqual(dist_orig, dist_proj, places=9)

    def test_mrl_block_diagonal_projection(self):
        """Test that MRL schema correctly partitions the vector space and projects block-by-block."""
        from hyperspace import HyperspaceClient
        
        # Create client and register key with MRL schema
        client = HyperspaceClient(host="dummy_host")
        mrl_schema = {
            "components": [
                {"name": "primary", "metric": "cosine", "full_dimension": 128}
            ],
            "cascade_pipeline": [
                {"component_name": "primary", "cutoff_dimension": 32}
            ]
        }
        client.register_collection_key("mrl_box", "secret_pass", metric="cosine", schema=mrl_schema, noise_sigma=0.0)
        
        context = client._get_encryption_context("mrl_box")
        
        # Generate test vectors
        rng = np.random.default_rng(99)
        vec1 = rng.standard_normal(128).tolist()
        
        # vec2 shares the exact same first 32 coordinates as vec1, but differs in coordinates 32-128
        vec2 = list(vec1)
        for i in range(32, 128):
            vec2[i] = rng.standard_normal()
            
        # Project both vectors
        proj1 = client._project_collection_vector("mrl_box", vec1, context, "cosine")
        proj2 = client._project_collection_vector("mrl_box", vec2, context, "cosine")
        
        # Under block-diagonal projection:
        # The first 32 elements of proj1 and proj2 must be IDENTICAL, because the first 32 coordinates
        # are projected independently of the rest!
        self.assertEqual(proj1[:32], proj2[:32])
        
        # The rest of the coordinates should differ
        self.assertNotEqual(proj1[32:], proj2[32:])
        
        # Verify distance in the 32D head is preserved (cosine distance)
        def cosine_dist(a, b):
            a_arr = np.array(a)
            b_arr = np.array(b)
            return 1.0 - np.dot(a_arr, b_arr) / (np.linalg.norm(a_arr) * np.linalg.norm(b_arr))
            
        # Create a third random vector
        vec3 = rng.standard_normal(128).tolist()
        proj3 = client._project_collection_vector("mrl_box", vec3, context, "cosine")
        
        # Check cosine distance preservation in 32D subspace
        self.assertAlmostEqual(cosine_dist(vec1[:32], vec3[:32]), cosine_dist(proj1[:32], proj3[:32]), places=10)
        
        # Check cosine distance preservation in full 128D space
        self.assertAlmostEqual(cosine_dist(vec1, vec3), cosine_dist(proj1, proj3), places=10)


if __name__ == '__main__':
    unittest.main()
