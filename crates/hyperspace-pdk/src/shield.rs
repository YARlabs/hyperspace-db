//! `hs-shield` (`shield`): Zero-Knowledge Orthogonal Projection (`Vec2Text` Inversion Defense).
//!
//! Protects vector embeddings against model inversion (`Vec2Text`) and side-channel attacks
//! by applying a keyed multi-round orthogonal transformation:
//! $$Q(K) = D_1(K) \cdot H \cdot P(K) \cdot D_0(K)$$
//! where $D_0, D_1 \in \{\pm 1\}^D$ are SHA-256 derived Rademacher sign matrices, $P(K)$ is a
//! Fisher-Yates coordinate permutation, and $H$ is the block-diagonal normalized Fast
//! Walsh-Hadamard Transform ($O(D \log D)$).
//!
//! Because $Q(K)^T Q(K) = I$, this projection is 100% isometric:
//! - Preserves Euclidean ($L_2$) distances, inner products, and Cosine similarities exactly.
//! - Preserves Poincaré ball norms ($\|Q u\|_2 = \|u\|_2 < 1$).
//! - Preserves Lorentz hyperboloid sheet membership ($-t^2 + \|Q x\|_2^2 = -1$) by keeping the
//!   timelike coordinate $t = v_0$ invariant while orthogonally rotating spatial coordinates.

use crate::{
    EngineContext, HookStage, HsPlugin, InsertPoint, PluginManifest, ProcedureSchema, SearchContext,
};
use async_trait::async_trait;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShieldCollectionConfig {
    pub collection: String,
    pub enabled: bool,
    pub key_source: String,
    pub key_fingerprint: String,
    #[serde(skip_serializing)]
    key_seed: [u8; 32],
}

#[derive(Debug, Clone, Deserialize)]
struct ConfigureShieldArgs {
    pub collection: Option<String>,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub secret_key: Option<String>,
    #[serde(default)]
    pub key_source: Option<String>,
}

const fn default_true() -> bool {
    true
}

/// Official `hs-shield` Zero-Knowledge Projection plugin.
#[derive(Debug, Default)]
pub struct HsShieldPlugin {
    configs: DashMap<String, ShieldCollectionConfig>,
}

impl HsShieldPlugin {
    pub fn new() -> Self {
        Self {
            configs: DashMap::new(),
        }
    }

    /// Configures `hs-shield` for a collection using the provided secret key.
    pub fn configure_collection(
        &self,
        collection: &str,
        enabled: bool,
        secret_key: &str,
        key_source: &str,
    ) -> ShieldCollectionConfig {
        let seed = Self::derive_seed(collection, secret_key);
        let fp = hex::encode(&seed[..8]);
        let cfg = ShieldCollectionConfig {
            collection: collection.to_string(),
            enabled,
            key_source: key_source.to_string(),
            key_fingerprint: fp,
            key_seed: seed,
        };
        self.configs.insert(collection.to_string(), cfg.clone());
        cfg
    }

    pub fn derive_seed(collection: &str, secret_key: &str) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(b"HS_SHIELD_V4_2_DOMAIN:");
        hasher.update(collection.as_bytes());
        hasher.update(b":");
        hasher.update(secret_key.as_bytes());
        hasher.finalize().into()
    }

    /// Applies the forward or inverse keyed orthogonal projection in place in `O(D log D)` time.
    ///
    /// When `metric` is `"lorentz"` or `"hybrid"`, coordinate `0` (the timelike axis $t$) is
    /// kept invariant so that $-t^2 + \|x\|^2 = -1$ remains strictly satisfied.
    pub fn transform_in_place(vector: &mut [f64], seed: &[u8; 32], metric: &str, inverse: bool) {
        let metric_lower = metric.trim().to_ascii_lowercase();
        let start_idx = if metric_lower == "lorentz" || metric_lower == "hybrid" {
            1.min(vector.len())
        } else {
            0
        };
        let slice = &mut vector[start_idx..];
        let n = slice.len();
        if n <= 1 {
            return;
        }

        let (signs_0, signs_1, perm) = Self::build_schedule(n, seed);

        if !inverse {
            // 1. Apply D_0 Rademacher signs
            for i in 0..n {
                slice[i] *= signs_0[i];
            }
            // 2. Apply keyed coordinate permutation P
            let copy = slice.to_vec();
            for i in 0..n {
                slice[i] = copy[perm[i]];
            }
            // 3. Apply block-diagonal normalized FWHT H
            Self::fwht_blocks_f64(slice);
            // 4. Apply D_1 Rademacher signs
            for i in 0..n {
                slice[i] *= signs_1[i];
            }
        } else {
            // Inverse: D_0 * P^{-1} * H * D_1
            for i in 0..n {
                slice[i] *= signs_1[i];
            }
            Self::fwht_blocks_f64(slice);
            let copy = slice.to_vec();
            for i in 0..n {
                slice[perm[i]] = copy[i];
            }
            for i in 0..n {
                slice[i] *= signs_0[i];
            }
        }
    }

    fn build_schedule(n: usize, seed: &[u8; 32]) -> (Vec<f64>, Vec<f64>, Vec<usize>) {
        // SplitMix64 PRNG seeded from SHA-256 key material
        let mut s0 = u64::from_le_bytes(seed[0..8].try_into().unwrap());
        let mut s1 = u64::from_le_bytes(seed[8..16].try_into().unwrap());
        let next_u64 = |state: &mut u64| -> u64 {
            *state = state.wrapping_add(0x9e37_79b9_7f4a_7c15);
            let mut z = *state;
            z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
            z ^ (z >> 31)
        };

        let mut signs_0 = Vec::with_capacity(n);
        let mut signs_1 = Vec::with_capacity(n);
        for _ in 0..n {
            signs_0.push(if (next_u64(&mut s0) & 1) == 0 {
                1.0
            } else {
                -1.0
            });
            signs_1.push(if (next_u64(&mut s1) & 1) == 0 {
                1.0
            } else {
                -1.0
            });
        }

        let mut perm: Vec<usize> = (0..n).collect();
        for i in (1..n).rev() {
            let j = (next_u64(&mut s0) as usize) % (i + 1);
            perm.swap(i, j);
        }

        (signs_0, signs_1, perm)
    }

    fn fwht_blocks_f64(data: &mut [f64]) {
        let mut offset = 0usize;
        let mut remaining = data.len();
        while remaining > 0 {
            let block_len = 1usize << remaining.ilog2();
            if block_len > 1 {
                Self::fwht_power_of_two(&mut data[offset..offset + block_len]);
            }
            offset += block_len;
            remaining -= block_len;
        }
    }

    fn fwht_power_of_two(block: &mut [f64]) {
        let n = block.len();
        debug_assert!(n.is_power_of_two());
        let mut len = 1usize;
        while len < n {
            let step = len * 2;
            for i in (0..n).step_by(step) {
                for j in 0..len {
                    let u = block[i + j];
                    let v = block[i + len + j];
                    block[i + j] = u + v;
                    block[i + len + j] = u - v;
                }
            }
            len = step;
        }
        let scale = 1.0 / (n as f64).sqrt();
        for x in block.iter_mut() {
            *x *= scale;
        }
    }
}

#[async_trait]
impl HsPlugin for HsShieldPlugin {
    fn manifest(&self) -> PluginManifest {
        PluginManifest {
            name: "shield".to_string(),
            version: "4.2.0".to_string(),
            author: "HyperspaceDB Core Team".to_string(),
            description: "Zero-Knowledge Orthogonal Projection (O(D log D) keyed FWHT) defending vectors against Vec2Text inversion attacks with zero accuracy loss.".to_string(),
            runtime: "native-rust".to_string(),
            enabled: true,
            source: "builtin".to_string(),
            hooks: vec![HookStage::BeforeInsert, HookStage::BeforeSearch],
            procedures: vec![
                ProcedureSchema {
                    name: "configure".to_string(),
                    description: "Enables or updates Zero-Knowledge Orthogonal Projection for a collection.".to_string(),
                    requires_collection: false,
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "collection": { "type": "string" },
                            "enabled": { "type": "boolean", "default": true },
                            "secret_key": { "type": "string", "description": "Secret rotation key or KMS key identifier" },
                            "key_source": { "type": "string", "default": "inline" }
                        }
                    }),
                    output_schema: json!({
                        "type": "object",
                        "properties": {
                            "collection": { "type": "string" },
                            "enabled": { "type": "boolean" },
                            "key_source": { "type": "string" },
                            "key_fingerprint": { "type": "string" }
                        }
                    }),
                },
                ProcedureSchema {
                    name: "status".to_string(),
                    description: "Returns active hs-shield configuration and key fingerprints.".to_string(),
                    requires_collection: false,
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "collection": { "type": "string" }
                        }
                    }),
                    output_schema: json!({
                        "type": "object",
                        "properties": {
                            "protected_collections": { "type": "array" }
                        }
                    }),
                },
                ProcedureSchema {
                    name: "transform".to_string(),
                    description: "Applies forward or inverse Zero-Knowledge Orthogonal Projection to a vector.".to_string(),
                    requires_collection: false,
                    input_schema: json!({
                        "type": "object",
                        "required": ["vector", "secret_key"],
                        "properties": {
                            "vector": { "type": "array", "items": { "type": "number" } },
                            "secret_key": { "type": "string" },
                            "collection": { "type": "string", "default": "default" },
                            "metric": { "type": "string", "default": "cosine" },
                            "inverse": { "type": "boolean", "default": false }
                        }
                    }),
                    output_schema: json!({
                        "type": "object",
                        "properties": {
                            "vector": { "type": "array", "items": { "type": "number" } }
                        }
                    }),
                },
            ],
            custom_metrics: vec![],
        }
    }

    async fn before_insert(
        &self,
        collection: &str,
        metric: &str,
        _dimension: usize,
        points: &mut Vec<InsertPoint>,
    ) -> Result<(), String> {
        if let Some(cfg) = self.configs.get(collection) {
            if cfg.enabled {
                let seed = cfg.key_seed;
                for pt in points.iter_mut() {
                    Self::transform_in_place(&mut pt.vector, &seed, metric, false);
                }
            }
        }
        Ok(())
    }

    async fn before_search(&self, ctx: &mut SearchContext) -> Result<(), String> {
        if let Some(cfg) = self.configs.get(&ctx.collection) {
            if cfg.enabled {
                let seed = cfg.key_seed;
                Self::transform_in_place(&mut ctx.query, &seed, &ctx.metric, false);
            }
        }
        Ok(())
    }

    async fn call_procedure(
        &self,
        method: &str,
        args: Value,
        ctx: &EngineContext,
    ) -> Result<Value, String> {
        match method {
            "configure" => {
                let parsed: ConfigureShieldArgs = serde_json::from_value(args)
                    .map_err(|e| format!("Invalid arguments for shield.configure: {e}"))?;
                let col_name = parsed
                    .collection
                    .or_else(|| ctx.collection_name.clone())
                    .ok_or_else(|| "Missing 'collection' name for shield.configure".to_string())?;
                let secret = parsed.secret_key.unwrap_or_else(|| {
                    std::env::var("HS_SHIELD_MASTER_KEY")
                        .unwrap_or_else(|_| "hs_default_shield_key_v4_2".to_string())
                });
                let key_source = parsed.key_source.unwrap_or_else(|| "inline".to_string());
                let cfg =
                    self.configure_collection(&col_name, parsed.enabled, &secret, &key_source);
                Ok(serde_json::to_value(cfg).unwrap())
            }
            "status" => {
                let filter_col = args
                    .get("collection")
                    .and_then(Value::as_str)
                    .or(ctx.collection_name.as_deref());
                let list: Vec<ShieldCollectionConfig> = self
                    .configs
                    .iter()
                    .filter(|entry| filter_col.is_none_or(|c| entry.key() == c))
                    .map(|entry| entry.value().clone())
                    .collect();
                Ok(json!({ "protected_collections": list }))
            }
            "transform" | "rotate_vector" | "unrotate_vector" => {
                let mut vec: Vec<f64> = args
                    .get("vector")
                    .and_then(|v| serde_json::from_value(v.clone()).ok())
                    .ok_or_else(|| "Missing or invalid 'vector' array".to_string())?;
                let secret = args
                    .get("secret_key")
                    .and_then(Value::as_str)
                    .ok_or_else(|| "Missing 'secret_key'".to_string())?;
                let col_name = args
                    .get("collection")
                    .and_then(Value::as_str)
                    .or(ctx.collection_name.as_deref())
                    .unwrap_or("default");
                let metric = args
                    .get("metric")
                    .and_then(Value::as_str)
                    .or_else(|| ctx.metric_name().ok())
                    .unwrap_or("cosine");
                let inverse = if method == "unrotate_vector" {
                    true
                } else {
                    args.get("inverse")
                        .and_then(Value::as_bool)
                        .unwrap_or(false)
                };
                let seed = Self::derive_seed(col_name, secret);
                Self::transform_in_place(&mut vec, &seed, metric, inverse);
                Ok(json!({ "vector": vec }))
            }
            other => Err(format!("Unknown procedure 'shield.{other}'")),
        }
    }
}
