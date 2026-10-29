//! # Hyperspace Plugin Development Kit (`hyperspace-pdk` / `hs-pdk`)
//!
//! An extension architecture for **HyperspaceDB v4.2.0**.
//!
//! `hyperspace-pdk` enables first-party and third-party developers to extend HyperspaceDB
//! without modifying or bloating the core storage/indexing engine:
//! - **Pipeline Lifecycle Hooks** (`BeforeInsert`, `AfterInsert`, `BeforeSearch`, `AfterSearch`, `OnLink`)
//! - **Custom Engine Procedures** (`call_extension(plugin, method, payload)`) with full JSON Schema introspection
//! - **Custom Metric Kernels** (`MetricDescriptor` + runtime evaluation)
//! - **Adaptive SDK Introspection** (`GET /v1/extensions/schema` & `ListExtensions` gRPC) so Python,
//!   TypeScript, and Rust SDKs dynamically bind `client.ext.<plugin>.<method>(...)`.
//!
//! ## Built-in Standard Flagship Plugins
//! 1. **`hs-embed` (`embed`)**: Zero-Python in-engine Markdown/text chunker, multi-manifold vectorizer,
//!    and automatic KAG sequential/hierarchical graph linker.
//! 2. **`hs-git-ast` (`git_ast`)**: Instant codebase AST parser & zero-cost KAG topology builder for
//!    Rust, Python, TypeScript/JavaScript, and Go.
//! 3. **`hs-shield` (`shield`)**: Zero-Knowledge Orthogonal Projection (`O(D log D)` keyed FWHT)
//!    defending embeddings against `Vec2Text` inversion attacks with zero accuracy loss.

pub mod embed;
pub mod git_ast;
pub mod shield;

pub use embed::{
    ApiProvider, ChunkingConfig, HsEmbedPlugin, Metric, MultiVectorizer, OnnxVectorizer,
    RemoteVectorizer, Vectorizer,
};
pub use git_ast::HsGitAstPlugin;
pub use shield::HsShieldPlugin;

/// Version of the Hyperspace Plugin Development Kit (`hyperspace-pdk`).
pub const PDK_VERSION: &str = "4.2.0";
pub type PluginPoint = InsertPoint;
pub type PluginSearchContext = SearchContext;

use async_trait::async_trait;
use dashmap::DashMap;
use hyperspace_core::{Collection, Durability, SearchParams, SearchResult};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Pipeline hook stages supported by the HyperspaceDB extension host.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HookStage {
    BeforeInsert,
    AfterInsert,
    BeforeSearch,
    AfterSearch,
    OnLink,
}

/// JSON Schema descriptor for a callable plugin procedure (`client.ext.<plugin>.<method>`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcedureSchema {
    pub name: String,
    pub description: String,
    pub requires_collection: bool,
    pub input_schema: Value,
    pub output_schema: Value,
}

/// Descriptor for a custom distance metric registered by a plugin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricDescriptor {
    pub name: String,
    pub description: String,
    pub is_symmetric: bool,
    pub preserves_unit_norm: bool,
}

/// Self-describing plugin manifest exposed via `ListExtensions` and `/v1/extensions/schema`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub runtime: String,
    pub hooks: Vec<HookStage>,
    pub procedures: Vec<ProcedureSchema>,
    pub custom_metrics: Vec<MetricDescriptor>,
}

/// Mutable point representation passed through `BeforeInsert` pipeline hooks.
#[derive(Debug, Clone)]
pub struct InsertPoint {
    pub id: u32,
    pub vector: Vec<f64>,
    pub metadata: HashMap<String, String>,
    pub payload: Option<Vec<u8>>,
}

/// Mutable search request context passed through `BeforeSearch` pipeline hooks.
#[derive(Debug, Clone)]
pub struct SearchContext {
    pub collection: String,
    pub metric: String,
    pub query: Vec<f64>,
    pub filter: HashMap<String, String>,
    pub top_k: usize,
}

/// Safe execution context passed to plugin procedures (`HsPlugin::call_procedure`),
/// providing high-throughput access to the target collection, Sidecar Payloads, and `GraphEngine`.
#[derive(Clone)]
pub struct EngineContext {
    pub owner: String,
    pub collection_name: Option<String>,
    pub collection: Option<Arc<dyn Collection>>,
    clock: Arc<AtomicU64>,
}

impl EngineContext {
    pub fn new(
        owner: impl Into<String>,
        collection_name: Option<String>,
        collection: Option<Arc<dyn Collection>>,
        clock: u64,
    ) -> Self {
        Self {
            owner: owner.into(),
            collection_name,
            collection,
            clock: Arc::new(AtomicU64::new(clock)),
        }
    }

    /// Returns a reference to the active collection, or an error if none was bound.
    pub fn collection(&self) -> Result<&Arc<dyn Collection>, String> {
        self.collection.as_ref().ok_or_else(|| {
            "This extension procedure requires an active target collection".to_string()
        })
    }

    /// Returns the vector dimensionality of the bound collection.
    pub fn dimension(&self) -> Result<usize, String> {
        Ok(self.collection()?.dimension())
    }

    /// Returns the metric name of the bound collection.
    pub fn metric_name(&self) -> Result<&'static str, String> {
        Ok(self.collection()?.metric_name())
    }

    /// Batch-inserts points (vectors, metadata, and optional Sidecar Payloads) into the bound collection.
    pub async fn insert_points(
        &self,
        points: Vec<InsertPoint>,
        durability: Durability,
    ) -> Result<usize, String> {
        let col = self.collection()?;
        let count = points.len();
        if count == 0 {
            return Ok(0);
        }

        let mut vec_batch = Vec::with_capacity(count);
        let mut payload_batch = Vec::new();
        for pt in points {
            vec_batch.push((pt.vector, pt.id, pt.metadata));
            if let Some(p) = pt.payload {
                if !p.is_empty() {
                    payload_batch.push((pt.id, p));
                }
            }
        }

        let clk = self.clock.fetch_add(1, Ordering::Relaxed) + 1;
        col.insert_batch(vec_batch, clk, durability).await?;
        if !payload_batch.is_empty() {
            col.insert_payload_batch(payload_batch).await?;
        }
        Ok(count)
    }

    /// Batch-links directed labeled edges `(source_id, target_id, relation_id, weight)` in `GraphEngine`.
    pub fn link_batch(&self, edges: &[(u32, u32, u16, f32)]) -> Result<usize, String> {
        let col = self.collection()?;
        col.graph_link_batch(edges)
    }

    /// Executes a similarity search on the bound collection.
    pub async fn search(
        &self,
        query: &[f64],
        top_k: usize,
        include_payload: bool,
    ) -> Result<Vec<SearchResult>, String> {
        let col = self.collection()?;
        let params = SearchParams {
            top_k,
            ef_search: (top_k * 2).max(64),
            include_payload,
            ..Default::default()
        };
        col.search(query, &HashMap::new(), &[], &params).await
    }
}

/// Core `pgrx`-style trait implemented by all HyperspaceDB plugins.
#[async_trait]
pub trait HsPlugin: Send + Sync + 'static {
    /// Returns the plugin's self-describing manifest and JSON Schemas.
    fn manifest(&self) -> PluginManifest;

    /// Invoked before vectors are inserted into a collection.
    /// Plugins may enrich metadata, generate embeddings (`hs-embed`), or apply orthogonal
    /// cryptographic rotations (`hs-shield`).
    async fn before_insert(
        &self,
        _collection: &str,
        _metric: &str,
        _dimension: usize,
        _points: &mut Vec<InsertPoint>,
    ) -> Result<(), String> {
        Ok(())
    }

    /// Invoked before a similarity search is executed on a collection.
    async fn before_search(&self, _ctx: &mut SearchContext) -> Result<(), String> {
        Ok(())
    }

    /// Invoked after a similarity search finishes on a collection.
    async fn after_search(
        &self,
        _collection: &str,
        _results: &mut Vec<SearchResult>,
    ) -> Result<(), String> {
        Ok(())
    }

    /// Executes a named plugin procedure (`client.ext.<plugin>.<method>(args)`).
    async fn call_procedure(
        &self,
        method: &str,
        args: Value,
        ctx: &EngineContext,
    ) -> Result<Value, String>;

    /// Evaluates a custom distance metric registered by this plugin, if supported.
    fn evaluate_custom_metric(&self, _metric_name: &str, _a: &[f64], _b: &[f64]) -> Option<f64> {
        None
    }
}

/// Declarative helper macro for constructing a [`PluginManifest`] concisely (`pgrx` style).
#[macro_export]
macro_rules! hs_plugin_manifest {
    (
        name: $name:expr,
        version: $version:expr,
        author: $author:expr,
        description: $desc:expr,
        hooks: [$($hook:expr),* $(,)?],
        procedures: [$($proc:expr),* $(,)?]
    ) => {
        $crate::PluginManifest {
            name: ($name).to_string(),
            version: ($version).to_string(),
            author: ($author).to_string(),
            description: ($desc).to_string(),
            runtime: "native-rust".to_string(),
            hooks: vec![$($hook),*],
            procedures: vec![$($proc),*],
            custom_metrics: vec![],
        }
    };
}

/// Thread-safe registry managing all active HyperspaceDB plugins, pipeline hooks, and procedures.
pub struct PluginRegistry {
    plugins: DashMap<String, Arc<dyn HsPlugin>>,
    hook_order: RwLock<Vec<String>>,
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::with_standard_plugins()
    }
}

impl PluginRegistry {
    /// Creates an empty `PluginRegistry`.
    pub fn new() -> Self {
        Self {
            plugins: DashMap::new(),
            hook_order: RwLock::new(Vec::new()),
        }
    }

    /// Creates a `PluginRegistry` pre-populated with the 3 official HyperspaceDB v4.2.0 plugins:
    /// 1. `embed` (`hs-embed`) — runs first on `BeforeInsert` to materialize vectors from text
    /// 2. `git_ast` (`hs-git-ast`) — AST codebase parser and zero-cost KAG topology builder
    /// 3. `shield` (`hs-shield`) — runs after `embed` on `BeforeInsert` and on `BeforeSearch`
    pub fn with_standard_plugins() -> Self {
        let registry = Self::new();
        registry.register(Arc::new(HsEmbedPlugin::new()));
        registry.register(Arc::new(HsGitAstPlugin::new()));
        registry.register(Arc::new(HsShieldPlugin::new()));
        registry
    }

    /// Attaches a configured `MultiVectorizer` (ONNX / HuggingFace Hub / Remote API)
    /// to the registered `hs-embed` (`embed`) plugin.
    pub fn set_embed_vectorizer(&self, vectorizer: Option<Arc<MultiVectorizer>>) {
        self.register(Arc::new(HsEmbedPlugin::with_vectorizer(vectorizer)));
    }

    /// Registers a plugin in the runtime registry.
    pub fn register(&self, plugin: Arc<dyn HsPlugin>) {
        let manifest = plugin.manifest();
        let name = manifest.name.clone();
        self.plugins.insert(name.clone(), plugin);
        let mut order = self.hook_order.write();
        if !order.contains(&name) {
            order.push(name);
        }
    }

    /// Returns a registered plugin by name (also accepts `hs-<name>` or `hs_<name>` aliases).
    pub fn get(&self, name: &str) -> Option<Arc<dyn HsPlugin>> {
        let normalized = Self::normalize_plugin_name(name);
        self.plugins.get(&normalized).map(|p| Arc::clone(p.value()))
    }

    /// Normalizes plugin aliases (`"hs-git-ast"` / `"hs_git_ast"` -> `"git_ast"`).
    pub fn normalize_plugin_name(name: &str) -> String {
        let trimmed = name.trim();
        let stripped = trimmed
            .strip_prefix("hs-")
            .or_else(|| trimmed.strip_prefix("hs_"))
            .unwrap_or(trimmed);
        stripped.replace('-', "_")
    }

    /// Lists manifests for all registered plugins in deterministic registration order.
    pub fn list_manifests(&self) -> Vec<PluginManifest> {
        let order = self.hook_order.read().clone();
        let mut out = Vec::with_capacity(order.len());
        for name in order {
            if let Some(p) = self.plugins.get(&name) {
                out.push(p.manifest());
            }
        }
        out
    }

    /// Returns the full JSON Schema catalog of all registered extensions and supported metrics.
    pub fn schema_catalog_json(&self) -> Value {
        let manifests = self.list_manifests();
        json!({
            "pdk_version": "4.2.0",
            "supported_metrics": hyperspace_core::SUPPORTED_METRICS,
            "extensions": manifests,
        })
    }

    /// Runs all registered `BeforeInsert` hooks in deterministic pipeline order (`embed` -> `shield`).
    pub async fn run_before_insert(
        &self,
        collection: &str,
        metric: &str,
        dimension: usize,
        points: &mut Vec<InsertPoint>,
    ) -> Result<(), String> {
        let order = self.hook_order.read().clone();
        for name in order {
            if let Some(plugin) = self.plugins.get(&name).map(|e| Arc::clone(e.value())) {
                plugin
                    .before_insert(collection, metric, dimension, points)
                    .await?;
            }
        }
        Ok(())
    }

    /// Runs all registered `BeforeSearch` hooks in deterministic pipeline order.
    pub async fn run_before_search(&self, ctx: &mut SearchContext) -> Result<(), String> {
        let order = self.hook_order.read().clone();
        for name in order {
            if let Some(plugin) = self.plugins.get(&name).map(|e| Arc::clone(e.value())) {
                plugin.before_search(ctx).await?;
            }
        }
        Ok(())
    }

    /// Dispatches a procedure call (`plugin.method(args)`) with the provided [`EngineContext`].
    pub async fn call_procedure(
        &self,
        plugin_name: &str,
        method: &str,
        args: Value,
        ctx: &EngineContext,
    ) -> Result<Value, String> {
        let plugin = self
            .get(plugin_name)
            .ok_or_else(|| format!("Extension plugin '{plugin_name}' is not registered"))?;
        plugin.call_procedure(method, args, ctx).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_standard_plugins_manifest_and_embed() {
        let registry = PluginRegistry::with_standard_plugins();
        let manifests = registry.list_manifests();
        assert_eq!(manifests.len(), 3);
        assert_eq!(manifests[0].name, "embed");
        assert_eq!(manifests[1].name, "git_ast");
        assert_eq!(manifests[2].name, "shield");

        let ctx = EngineContext::new("admin", None, None, 1);

        // Test embed.chunk_text
        let chunk_res = registry
            .call_procedure(
                "hs-embed",
                "chunk_text",
                json!({
                    "text": "# Architecture\nHyperspaceDB is a multi-metric spatial runtime.\n# Extensions\nhp-pdk brings pgrx-style plugins.",
                    "chunk_size": 128
                }),
                &ctx,
            )
            .await
            .unwrap();
        let chunks = chunk_res["chunks"].as_array().unwrap();
        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0]["heading"], "Architecture");
        assert_eq!(chunks[1]["heading"], "Extensions");

        // Test embed.embed_text across Poincare, Lorentz, and Hybrid manifolds
        for (metric, dim) in [("poincare", 32usize), ("lorentz", 33), ("hybrid", 801)] {
            let emb_res = registry
                .call_procedure(
                    "embed",
                    "embed_text",
                    json!({
                        "text": "Contract #402 with Vector LLC",
                        "dimension": dim,
                        "metric": metric
                    }),
                    &ctx,
                )
                .await
                .unwrap();
            let vec: Vec<f64> = serde_json::from_value(emb_res["vector"].clone()).unwrap();
            assert_eq!(vec.len(), dim);
            match metric {
                "poincare" => {
                    assert!(<hyperspace_core::PoincareMetric as hyperspace_core::Metric>::validate(&vec).is_ok());
                }
                "lorentz" => {
                    assert!(<hyperspace_core::LorentzMetric as hyperspace_core::Metric>::validate(&vec).is_ok());
                }
                "hybrid" => {
                    assert!(<hyperspace_core::HybridMetric as hyperspace_core::Metric>::validate(&vec).is_ok());
                }
                _ => {}
            }
        }
    }

    #[tokio::test]
    async fn test_git_ast_multi_language_kag_extraction() {
        let registry = PluginRegistry::with_standard_plugins();
        let ctx = EngineContext::new("admin", None, None, 1);

        let res = registry
            .call_procedure(
                "hs-git-ast",
                "parse_source",
                json!({
                    "files": [
                        {
                            "path": "src/engine.rs",
                            "content": "pub trait StorageEngine {\n    fn flush(&self);\n}\npub struct MmapEngine;\nimpl StorageEngine for MmapEngine {\n    fn flush(&self) {\n        sync_disk();\n    }\n}\npub fn sync_disk() {}\n"
                        }
                    ]
                }),
                &ctx,
            )
            .await
            .unwrap();

        let symbols = res["symbols"].as_array().unwrap();
        let edges = res["edges"].as_array().unwrap();
        assert!(symbols.len() >= 4, "Should extract module, trait, struct, impl, and fn");
        assert!(
            edges.iter().any(|e| e["relation_name"] == "IMPLEMENTS"),
            "Should detect impl StorageEngine for MmapEngine"
        );
        assert!(
            edges.iter().any(|e| e["relation_name"] == "CALLS"),
            "Should detect flush() calling sync_disk()"
        );
    }

    #[tokio::test]
    async fn test_shield_orthogonal_isometry_and_roundtrip() {
        let registry = PluginRegistry::with_standard_plugins();
        let ctx = EngineContext::new("admin", Some("medical_records".to_string()), None, 1);

        // Configure shield for collection "medical_records"
        let cfg = registry
            .call_procedure(
                "hs-shield",
                "configure",
                json!({
                    "collection": "medical_records",
                    "enabled": true,
                    "secret_key": "top_secret_kms_key_2026"
                }),
                &ctx,
            )
            .await
            .unwrap();
        assert_eq!(cfg["enabled"], true);

        let v1 = HsEmbedPlugin::embed_for_manifold("Patient diagnosis: Type-1 diabetes", 64, "cosine");
        let v2 = HsEmbedPlugin::embed_for_manifold("Insulin therapy protocol", 64, "cosine");
        let orig_dist = <hyperspace_core::EuclideanMetric as hyperspace_core::Metric>::distance(&v1, &v2);

        let mut points = vec![
            InsertPoint {
                id: 1,
                vector: v1.clone(),
                metadata: HashMap::new(),
                payload: None,
            },
            InsertPoint {
                id: 2,
                vector: v2.clone(),
                metadata: HashMap::new(),
                payload: None,
            },
        ];

        registry
            .run_before_insert("medical_records", "cosine", 64, &mut points)
            .await
            .unwrap();

        // Rotated vectors must differ strongly from plaintext vectors
        let diff_from_plain =
            <hyperspace_core::EuclideanMetric as hyperspace_core::Metric>::distance(&v1, &points[0].vector);
        assert!(diff_from_plain > 0.1, "Shielded vector must not equal plaintext vector");

        // Pairwise distance between shielded vectors must equal original distance to machine precision!
        let shielded_dist = <hyperspace_core::EuclideanMetric as hyperspace_core::Metric>::distance(
            &points[0].vector,
            &points[1].vector,
        );
        assert!(
            (orig_dist - shielded_dist).abs() < 1e-12,
            "Orthogonal projection must preserve exact pairwise distances: orig={orig_dist}, shielded={shielded_dist}"
        );

        // Unrotating with the secret key must recover the exact original vector
        let unrot = registry
            .call_procedure(
                "shield",
                "unrotate_vector",
                json!({
                    "collection": "medical_records",
                    "secret_key": "top_secret_kms_key_2026",
                    "metric": "cosine",
                    "vector": points[0].vector
                }),
                &ctx,
            )
            .await
            .unwrap();
        let recovered: Vec<f64> = serde_json::from_value(unrot["vector"].clone()).unwrap();
        let recovery_err =
            <hyperspace_core::EuclideanMetric as hyperspace_core::Metric>::distance(&v1, &recovered);
        assert!(recovery_err < 1e-12, "Inverse projection must recover original vector");
    }
}
