use hyperspace_core::{Collection, EuclideanMetric, LorentzMetric, Metric, PoincareMetric};
#[cfg(feature = "embed")]
use hyperspace_embed::MultiVectorizer;
use hyperspace_proto::hyperspace::{VerifyGenerationRequest, VerifyGenerationResponse};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{HashSet, VecDeque};
#[cfg(feature = "embed")]
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TribunalVerificationDetails {
    pub metric: String,
    pub collection: String,
    pub resolved_contexts_count: usize,
    pub min_distance: f64,
    pub avg_distance: f64,
    pub max_threshold: f64,
    pub cone_satisfied: bool,
    pub geodesic_hops: u32,
    pub trust_score: f64,
    pub is_valid: bool,
    pub reason: String,
}

/// Fallback deterministic pseudo-vectorizer when external embedding engine is not configured
/// or for air-gapped / test environments. Maps text onto the collection's target manifold.
pub fn generate_deterministic_embedding(text: &str, metric: &str, dim: usize) -> Vec<f64> {
    if dim == 0 {
        return Vec::new();
    }

    let mut coords = vec![0.0f64; dim];
    let mut words_count = 0;
    for (word_idx, word) in text.split_whitespace().enumerate() {
        words_count += 1;
        let mut hasher = Sha256::new();
        hasher.update(word.as_bytes());
        let hash = hasher.finalize();

        for i in 0..dim {
            let byte = hash[i % hash.len()];
            let weight = 1.0 / ((word_idx + 1) as f64).sqrt();
            let val = ((byte as f64 - 128.0) / 128.0) * weight;
            coords[i] += val;
        }
    }

    // If text was empty or had no words, set small non-zero values
    if words_count == 0 {
        for (i, c) in coords.iter_mut().enumerate() {
            *c = 0.01 * ((i % 5) as f64 + 1.0);
        }
    }

    match metric {
        "lorentz" => {
            // Coordinate 0 is time-like (t >= 1.0), Coordinates 1..dim are spatial
            // On unit hyperboloid: -t^2 + sum(x_i^2) = -1 ==> t = sqrt(1.0 + sum(x_i^2))
            if dim == 1 {
                return vec![1.0];
            }
            let spatial_norm_sq: f64 = coords[1..].iter().map(|&x| (x * 0.1).powi(2)).sum();
            let t = (1.0 + spatial_norm_sq).sqrt();
            coords[0] = t;
            for c in &mut coords[1..] {
                *c *= 0.1; // Scale spatial coordinates to moderate radius
            }
            coords
        }
        "poincare" => {
            // Must have Euclidean norm strictly < 1.0
            let norm: f64 = coords.iter().map(|&x| x * x).sum::<f64>().sqrt();
            let target_norm = 0.75;
            if norm > 1e-12 {
                for c in &mut coords {
                    *c = (*c / norm) * target_norm;
                }
            } else {
                coords[0] = 0.1;
            }
            coords
        }
        "cosine" => {
            // Unit sphere normalization
            let norm: f64 = coords.iter().map(|&x| x * x).sum::<f64>().sqrt();
            if norm > 1e-12 {
                for c in &mut coords {
                    *c /= norm;
                }
            } else {
                coords[0] = 1.0;
            }
            coords
        }
        _ => coords, // Euclidean or default
    }
}

/// Compute distance according to the collection's target manifold.
/// Generic over quantization because `Collection::get_points()` produces
/// dequantized, full-precision `f64` coordinate vectors.
pub fn compute_manifold_distance(metric_name: &str, a: &[f64], b: &[f64]) -> f64 {
    if a.is_empty() || b.is_empty() {
        return f64::MAX;
    }

    match metric_name {
        "lorentz" => {
            if a.len() < 2 || b.len() < 2 {
                return EuclideanMetric::distance(a, b);
            }
            LorentzMetric::distance(a, b)
        }
        "poincare" => {
            let norm_a: f64 = a.iter().map(|&x| x * x).sum::<f64>().sqrt();
            let norm_b: f64 = b.iter().map(|&x| x * x).sum::<f64>().sqrt();
            if norm_a >= 1.0 || norm_b >= 1.0 {
                // Out of Poincaré ball boundary fallback
                EuclideanMetric::distance(a, b)
            } else {
                PoincareMetric::distance(a, b)
            }
        }
        "cosine" => {
            let mut dot = 0.0;
            let mut norm_a = 0.0;
            let mut norm_b = 0.0;
            for (&x, &y) in a.iter().zip(b.iter()) {
                dot += x * y;
                norm_a += x * x;
                norm_b += y * y;
            }
            if norm_a <= 1e-12 || norm_b <= 1e-12 {
                1.0
            } else {
                let sim = (dot / (norm_a.sqrt() * norm_b.sqrt())).clamp(-1.0, 1.0);
                (1.0 - sim).max(0.0)
            }
        }
        "euclidean" | "l2" => EuclideanMetric::distance(a, b),
        _ => EuclideanMetric::distance(a, b),
    }
}

/// Evaluates whether a candidate vector falls within the forward causal light cone
/// of any context vector in the Lorentz hyperbolic model.
///
/// In Lorentz geometry, a parent/root node near the apex (t ~ 1.0, spatial ~ 0)
/// subsumes child nodes in its forward direction. For directional nodes, containment
/// evaluates angular aperture alignment and forward depth bounds.
pub fn check_lorentz_cone_containment<V: AsRef<[f64]>>(
    answer_vec: &[f64],
    context_vecs: &[V],
) -> bool {
    if answer_vec.len() < 2 || context_vecs.is_empty() {
        return true;
    }

    let a0 = answer_vec[0];
    let mut norm_a_sq = 0.0;
    for &ai in &answer_vec[1..] {
        norm_a_sq += ai * ai;
    }
    let norm_a = norm_a_sq.sqrt();

    for ctx_ref in context_vecs {
        let ctx = ctx_ref.as_ref();
        if ctx.len() != answer_vec.len() {
            continue;
        }

        let c0 = ctx[0];

        let mut norm_c_sq = 0.0;
        let mut dot_ca = 0.0;

        for i in 1..answer_vec.len() {
            let ai = answer_vec[i];
            let ci = ctx[i];
            norm_c_sq += ci * ci;
            dot_ca += ci * ai;
        }

        // 1. Apex Subsumption: If context is near the apex (c0 ~ 1.0, spatial norm ~ 0),
        // it serves as the root policy and causally subsumes downstream nodes.
        if norm_c_sq < 0.05 {
            if a0 >= (c0 - 0.15) && norm_a_sq <= 3.5 {
                return true;
            }
        }

        // 2. Angular Aperture Subsumption: Evaluate angular alignment along context branch
        let norm_c = norm_c_sq.sqrt();

        if norm_c > 1e-6 && norm_a > 1e-6 {
            let cos_angle = (dot_ca / (norm_c * norm_a)).clamp(-1.0, 1.0);
            if cos_angle >= 0.40 && a0 >= (c0 - 0.25) {
                return true;
            }
        }

        // 3. Geodesic Proximity: In-cone if within tight hyperbolic geodesic radius
        let d = LorentzMetric::distance(answer_vec, ctx);
        if d <= 1.20 {
            return true;
        }
    }

    false
}

/// Resolves raw context ID strings (which may be UUIDs or numeric strings)
/// to point IDs in the given collection.
pub fn resolve_context_ids(col: &dyn Collection, context_ids: &[String]) -> Vec<u32> {
    let mut resolved = Vec::new();

    for id_str in context_ids {
        let trimmed = id_str.trim();
        if trimmed.is_empty() {
            continue;
        }

        // 1. Try numeric u32 parse directly
        if let Ok(num_id) = trimmed.parse::<u32>() {
            let pts = col.get_points(&[num_id]);
            if !pts.is_empty() {
                resolved.push(num_id);
                continue;
            }
        }

        // 2. Try parsing as UUID
        if let Ok(parsed_uuid) = Uuid::parse_str(trimmed) {
            // Check lower 32-bits hash representation
            let u32_hash = (parsed_uuid.as_u128() & 0xFFFF_FFFF) as u32;
            let pts = col.get_points(&[u32_hash]);
            if !pts.is_empty() {
                resolved.push(u32_hash);
                continue;
            }

            // Also check if any existing point metadata matches the UUID string
            let peeked = col.peek(500, 0);
            for (pid, _, meta) in &peeked {
                if meta.get("uuid").map(|s| s.as_str()) == Some(trimmed)
                    || meta.get("id").map(|s| s.as_str()) == Some(trimmed)
                    || meta.get("chunk_id").map(|s| s.as_str()) == Some(trimmed)
                {
                    if !resolved.contains(pid) {
                        resolved.push(*pid);
                    }
                    break;
                }
            }
        }
    }

    resolved
}

/// Calculates geodesic graph connectivity and hop distance on HNSW layer 0.
pub fn verify_hnsw_geodesic_coherence(
    col: &dyn Collection,
    resolved_ids: &[u32],
    closest_id: u32,
) -> (u32, f64) {
    if resolved_ids.is_empty() {
        return (0, 0.0);
    }

    if resolved_ids.len() == 1 {
        // Single context node: check local clustering degree of closest context node
        if let Ok(neighbors) = col.graph_neighbors(closest_id, 0, 10) {
            let hops = if neighbors.is_empty() { 2 } else { 1 };
            return (hops, 0.2);
        }
        return (1, 0.2);
    }

    // Multi-context nodes: compute geodesic hops between the closest node and others
    let target_set: HashSet<u32> = resolved_ids.iter().copied().collect();
    let mut visited: HashSet<u32> = HashSet::new();
    let mut queue: VecDeque<(u32, u32)> = VecDeque::new(); // (node_id, hops)

    queue.push_back((closest_id, 0));
    visited.insert(closest_id);

    let mut reachable_contexts = 0;
    let mut total_hops = 0;
    let max_search_depth = 4;

    while let Some((curr, hops)) = queue.pop_front() {
        if target_set.contains(&curr) && curr != closest_id {
            reachable_contexts += 1;
            total_hops += hops;
        }

        if hops < max_search_depth {
            if let Ok(neighbors) = col.graph_neighbors(curr, 0, 15) {
                for n in neighbors {
                    if visited.insert(n) {
                        queue.push_back((n, hops + 1));
                    }
                }
            }
        }
    }

    if reachable_contexts > 0 {
        let avg_hops = (total_hops as f64 / reachable_contexts as f64).round() as u32;
        (avg_hops.max(1), avg_hops as f64 * 0.25)
    } else {
        // Disconnected or beyond max_search_depth
        (5, 1.25)
    }
}

/// Core Tribunal Router Engine: Verifies drafted LLM answer against context chunks.
pub async fn verify_generation_against_context(
    col: &dyn Collection,
    req: VerifyGenerationRequest,
) -> Result<VerifyGenerationResponse, String> {
    #[cfg(feature = "embed")]
    return verify_generation_against_context_with_vectorizer(col, req, None).await;

    #[cfg(not(feature = "embed"))]
    return verify_generation_against_context_with_vectorizer(col, req).await;
}

/// Core Tribunal Router Engine with optional vectorizer engine.
pub async fn verify_generation_against_context_with_vectorizer(
    col: &dyn Collection,
    req: VerifyGenerationRequest,
    #[cfg(feature = "embed")] vectorizer: Option<&Arc<MultiVectorizer>>,
) -> Result<VerifyGenerationResponse, String> {
    let metric = col.metric_name();
    let dim = col.dimension();

    // 1. Vectorize the llm_answer on the fly
    let answer_vec: Vec<f64> = {
        #[cfg(feature = "embed")]
        if let Some(multi) = vectorizer {
            match multi
                .vectorize_for_dim(vec![req.llm_answer.clone()], metric, dim)
                .await
            {
                Ok(mut vecs) if !vecs.is_empty() => vecs.remove(0),
                _ => generate_deterministic_embedding(&req.llm_answer, metric, dim),
            }
        } else {
            generate_deterministic_embedding(&req.llm_answer, metric, dim)
        }
        #[cfg(not(feature = "embed"))]
        {
            generate_deterministic_embedding(&req.llm_answer, metric, dim)
        }
    };

    if answer_vec.is_empty() {
        return Err("Failed to vectorize LLM answer".into());
    }

    // 2. Resolve context IDs
    let resolved_ids = resolve_context_ids(col, &req.context_ids);

    if resolved_ids.is_empty() {
        return Ok(VerifyGenerationResponse {
            trust_score: 0.10,
            is_valid: false,
            min_distance: f64::MAX,
            avg_distance: f64::MAX,
            geodesic_distance: f64::MAX,
            geodesic_hops: 0,
            cone_satisfied: false,
            details: "No valid context chunk IDs could be resolved from the database".into(),
            resolved_context_ids: Vec::new(),
        });
    }

    // 3. Fetch context points (generic over quantization via Collection::get_points)
    let context_pts = col.get_points(&resolved_ids);

    if context_pts.is_empty() {
        return Ok(VerifyGenerationResponse {
            trust_score: 0.15,
            is_valid: false,
            min_distance: f64::MAX,
            avg_distance: f64::MAX,
            geodesic_distance: f64::MAX,
            geodesic_hops: 0,
            cone_satisfied: false,
            details: "Context chunk IDs exist in request but have no vectors stored in collection"
                .into(),
            resolved_context_ids: resolved_ids,
        });
    }

    // 4. Calculate geometric distances
    let mut min_dist = f64::MAX;
    let mut total_dist = 0.0;
    let mut closest_node_id = context_pts[0].0;

    for (node_id, ctx_vec, _) in &context_pts {
        let d = compute_manifold_distance(metric, &answer_vec, ctx_vec);
        if d < min_dist {
            min_dist = d;
            closest_node_id = *node_id;
        }
        total_dist += d;
    }

    let avg_dist = total_dist / context_pts.len() as f64;

    // 5. Evaluate Metric Boundaries & Thresholds
    let default_threshold = match metric {
        "lorentz" => 1.20,
        "poincare" => 2.20,
        "cosine" => 0.45,
        _ => 1.00,
    };
    let threshold = req.max_distance_threshold.unwrap_or(default_threshold);

    // 6. Evaluate Lorentz light cone containment (if metric == "lorentz")
    let cone_satisfied = if metric == "lorentz" {
        let ctx_slices: Vec<&[f64]> = context_pts.iter().map(|p| p.1.as_slice()).collect();
        check_lorentz_cone_containment(&answer_vec, &ctx_slices)
    } else {
        true
    };

    // 7. Verify geodesic paths in HNSW
    let valid_resolved_ids: Vec<u32> = context_pts.iter().map(|p| p.0).collect();
    let check_geodesic = req.check_geodesic.unwrap_or(true);
    let (geodesic_hops, geodesic_dist) = if check_geodesic {
        verify_hnsw_geodesic_coherence(col, &valid_resolved_ids, closest_node_id)
    } else {
        (1, min_dist)
    };

    // 8. Calculate Trust Score [0.0, 1.0]
    let base_dist_score = 1.0 / (1.0 + 2.0 * min_dist);

    let threshold_factor = if min_dist <= threshold {
        1.0
    } else {
        (threshold / min_dist).clamp(0.05, 0.90)
    };

    let cone_factor = if metric == "lorentz" {
        if cone_satisfied {
            1.0
        } else {
            0.20 // severe penalty for breaking Lorentz causal cone
        }
    } else {
        1.0
    };

    let geodesic_factor = match geodesic_hops {
        1..=2 => 1.0,
        3..=4 => 0.85,
        5..=6 => 0.65,
        _ => 0.45,
    };

    let mut trust_score =
        (base_dist_score * threshold_factor * cone_factor * geodesic_factor).clamp(0.0, 1.0);

    // Hard cap if boundaries are violated
    if !cone_satisfied && metric == "lorentz" {
        trust_score = trust_score.min(0.28);
    }
    if min_dist > threshold {
        trust_score = trust_score.min(0.42);
    }

    let is_valid = trust_score >= 0.70 && (metric != "lorentz" || cone_satisfied);

    // Generate descriptive reason/diagnostic details
    let details = if is_valid {
        format!(
            "Generation verified: geometrically grounded in context manifold (min_dist={:.4}, cone_satisfied={}, geodesic_hops={})",
            min_dist, cone_satisfied, geodesic_hops
        )
    } else if metric == "lorentz" && !cone_satisfied {
        format!(
            "Hallucination detected: answer lies OUTSIDE Lorentz causal light cone of grounding context (min_dist={:.4}, threshold={:.4})",
            min_dist, threshold
        )
    } else if min_dist > threshold {
        format!(
            "Logic break / semantic drift: minimum distance ({:.4}) exceeds maximum acceptable threshold ({:.4})",
            min_dist, threshold
        )
    } else {
        format!(
            "Low trust score ({:.2}): answer is weakly connected across context geodesic neighborhood",
            trust_score
        )
    };

    Ok(VerifyGenerationResponse {
        trust_score,
        is_valid,
        min_distance: min_dist,
        avg_distance: avg_dist,
        geodesic_distance: geodesic_dist,
        geodesic_hops,
        cone_satisfied,
        details,
        resolved_context_ids: resolved_ids,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deterministic_embedding_lorentz_hyperboloid() {
        let vec = generate_deterministic_embedding(
            "Commercial loan automated approval up to $1M",
            "lorentz",
            33,
        );
        assert_eq!(vec.len(), 33);
        assert!(vec[0] >= 1.0, "Coordinate 0 must be time-like (t >= 1.0)");

        // Verify Lorentz hyperboloid condition: -t^2 + sum(x_i^2) = -1
        let spatial_sq: f64 = vec[1..].iter().map(|&x| x * x).sum();
        let minkowski_norm = -vec[0].powi(2) + spatial_sq;
        assert!(
            (minkowski_norm + 1.0).abs() < 1e-6,
            "Must be strictly on unit hyperboloid, got {}",
            minkowski_norm
        );
    }

    #[test]
    fn test_deterministic_embedding_cosine_sphere() {
        let vec = generate_deterministic_embedding("Arbitrary test text", "cosine", 16);
        assert_eq!(vec.len(), 16);
        let norm: f64 = vec.iter().map(|&x| x * x).sum::<f64>().sqrt();
        assert!(
            (norm - 1.0).abs() < 1e-6,
            "Must be normalized on unit sphere"
        );
    }

    #[test]
    fn test_lorentz_cone_containment_positive_and_negative() {
        // Parent context along +X branch
        let mut parent = vec![0.0; 33];
        parent[1] = 0.5;
        parent[0] = (1.0 + 0.25f64).sqrt();

        // Child vector along the same direction (inside cone)
        let mut inside_child = vec![0.0; 33];
        inside_child[1] = 0.8;
        inside_child[2] = 0.05;
        let child_spatial_sq: f64 = 0.8 * 0.8 + 0.05 * 0.05;
        inside_child[0] = (1.0f64 + child_spatial_sq).sqrt();

        // Check positive containment
        assert!(check_lorentz_cone_containment(
            &inside_child,
            &[parent.clone()]
        ));

        // Outside vector along the opposite direction (-X branch)
        let mut opposite_point = vec![0.0; 33];
        opposite_point[1] = -0.8;
        let opp_spatial_sq: f64 = 0.64;
        opposite_point[0] = (1.0f64 + opp_spatial_sq).sqrt();

        assert!(!check_lorentz_cone_containment(
            &opposite_point,
            &[parent.clone()]
        ));

        // Apex root parent subsumes moderate child
        let mut apex_root = vec![0.0; 33];
        apex_root[0] = 1.0;
        assert!(check_lorentz_cone_containment(
            &inside_child,
            &[apex_root.clone()]
        ));

        // Distant outside point exceeding manifold threshold
        let mut distant_point = vec![0.0; 33];
        distant_point[1] = 3.0;
        distant_point[0] = (1.0 + 9.0f64).sqrt();
        assert!(!check_lorentz_cone_containment(
            &distant_point,
            &[apex_root]
        ));
    }

    #[test]
    fn test_manifold_distance_cosine() {
        let v1 = vec![1.0, 0.0, 0.0];
        let v2 = vec![1.0, 0.0, 0.0];
        let v3 = vec![0.0, 1.0, 0.0];

        assert!(compute_manifold_distance("cosine", &v1, &v2).abs() < 1e-6);
        assert!((compute_manifold_distance("cosine", &v1, &v3) - 1.0).abs() < 1e-6);
    }

    #[tokio::test]
    async fn test_tribunal_verify_generation_integration() {
        let uuid = Uuid::new_v4();
        let tmp_dir = std::env::temp_dir().join(format!("hs_tribunal_test_{uuid}"));
        let _ = std::fs::create_dir_all(&tmp_dir);

        let (tx, _rx) = tokio::sync::broadcast::channel(100);
        let (etx, _) = tokio::sync::broadcast::channel(100);
        let manager = crate::manager::CollectionManager::new(tmp_dir.clone(), tx, etx);

        let col_name = "test_tribunal_col";
        let schema = hyperspace_proto::hyperspace::CollectionSchema {
            components: vec![hyperspace_proto::hyperspace::VectorComponent {
                name: "default".to_string(),
                metric: "lorentz".to_string(),
                full_dimension: 33,
                weight: 1.0,
            }],
            cascade_pipeline: vec![],
        };
        manager
            .create_collection("default_admin", col_name, schema)
            .await
            .expect("Create collection failed");

        let col = manager
            .get("default_admin", col_name)
            .await
            .expect("Collection exists");

        // Insert context chunk: ID 101 with metadata UUID
        let policy_vec = generate_deterministic_embedding(
            "Commercial credit automated approvals up to $1,000,000",
            "lorentz",
            33,
        );

        let mut meta1 = std::collections::HashMap::new();
        meta1.insert(
            "uuid".to_string(),
            "4c2049d5-1b26-4d1a-8c9e-2b5a19213ef2".to_string(),
        );
        meta1.insert(
            "text".to_string(),
            "Commercial credit automated approvals up to $1,000,000".to_string(),
        );

        col.insert(
            &policy_vec,
            101,
            meta1,
            1,
            hyperspace_core::Durability::Strict,
        )
        .await
        .expect("Insert 101 failed");

        // 1. Verify Grounded Generation (High trust score >= 0.70)
        let grounded_req = VerifyGenerationRequest {
            collection: col_name.to_string(),
            llm_answer: "Commercial credit automated approvals up to $1,000,000".to_string(),
            context_ids: vec![
                "101".to_string(),
                "4c2049d5-1b26-4d1a-8c9e-2b5a19213ef2".to_string(),
            ],
            max_distance_threshold: Some(1.2),
            check_geodesic: Some(true),
        };

        let grounded_res = verify_generation_against_context(&*col, grounded_req)
            .await
            .expect("Verification must succeed");

        assert!(
            grounded_res.is_valid,
            "Grounded generation must be valid: {:?}",
            grounded_res
        );
        assert!(
            grounded_res.trust_score >= 0.70,
            "Trust score must be >= 0.70, got {}",
            grounded_res.trust_score
        );
        assert!(grounded_res.cone_satisfied, "Cone must be satisfied");

        // 2. Verify Hallucinated / Out-of-bounds Generation (Low trust score, marked invalid)
        let hallucinated_req = VerifyGenerationRequest {
            collection: col_name.to_string(),
            llm_answer: "Unrelated quantum electrodynamics and astrophysical nebula synthesis"
                .to_string(),
            context_ids: vec!["101".to_string()],
            max_distance_threshold: Some(0.3),
            check_geodesic: Some(true),
        };

        let hallucinated_res = verify_generation_against_context(&*col, hallucinated_req)
            .await
            .expect("Verification call must succeed");

        assert!(
            !hallucinated_res.is_valid,
            "Hallucinated generation must be marked invalid"
        );
        assert!(
            hallucinated_res.trust_score < 0.70,
            "Trust score for hallucination must be low, got {}",
            hallucinated_res.trust_score
        );

        let _ = std::fs::remove_dir_all(&tmp_dir);
    }
}
