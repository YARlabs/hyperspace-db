//! # Hyperspace Plugin Development Kit (`hyperspace-pdk` / `hs-pdk`)
//!
//! A `pgrx`-inspired extension architecture for **HyperspaceDB v4.2.0**.
//!
//! `hyperspace-pdk` enables both built-in first-party extensions and external third-party
//! dynamic plugins (`cdylib`: `.dylib` / `.so` / `.dll` in `./plugins/`) to extend HyperspaceDB
//! without modifying or bloating the core storage/indexing engine:
//! - **Pipeline Lifecycle Hooks** (`BeforeInsert`, `AfterInsert`, `BeforeSearch`, `AfterSearch`, `OnLink`)
//! - **Custom Engine Procedures** (`call_extension(plugin, method, payload)`) with full JSON Schema introspection
//! - **Custom Metric Kernels** (`MetricDescriptor` + runtime evaluation)
//! - **Dynamic External Plugin Loading (`hs_export_plugin!` + `libloading`)** from `./plugins/` (`HYPERSPACE_PLUGINS_DIR`)
//! - **Runtime Activation & Deactivation** via `.env` (`HYPERSPACE_ENABLED_PLUGINS`, `HYPERSPACE_DISABLED_PLUGINS`)
//!   and HTTP/gRPC endpoints (`POST /v1/extensions/{plugin}/enable`, `POST /v1/extensions/{plugin}/disable`, `POST /v1/extensions/reload`)
//! - **Adaptive SDK Introspection** (`GET /v1/extensions/schema` & `ListExtensions` gRPC) so Python,
//!   TypeScript, and Rust SDKs dynamically bind `client.ext.<plugin>.<method>(...)`.
//!
//! ## Built-in Core Plugins (`crates/hyperspace-pdk`)
//! 1. **`hs-embed` (`embed`)**: Zero-Python in-engine Markdown/text chunker, ONNX/HF/Remote model driver,
//!    multi-manifold vectorizer, and automatic KAG sequential/hierarchical graph linker.
//! 2. **`hs-shield` (`shield`)**: Zero-Knowledge Orthogonal Projection (`O(D log D)` keyed FWHT)
//!    defending embeddings against `Vec2Text` inversion attacks with zero accuracy loss.
//!
//! ## Official External Plugin (`plugins/hs-git-ast`)
//! 3. **`hs-git-ast` (`git_ast`)**: Standalone external `cdylib` plugin placed in `./plugins/`, providing
//!    instant codebase AST parsing & zero-cost KAG topology building for Rust, Python, TypeScript/JavaScript, and Go.

#![allow(clippy::doc_markdown)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::if_not_else)]
#![allow(clippy::explicit_iter_loop)]

pub mod embed;
pub mod shield;

pub use embed::{
    ApiProvider, ChunkingConfig, HsEmbedPlugin, Metric, MultiVectorizer, OnnxVectorizer,
    RemoteVectorizer, Vectorizer,
};
pub use shield::HsShieldPlugin;

/// Version string of the Hyperspace Plugin Development Kit (`hyperspace-pdk`).
pub const PDK_VERSION: &str = "4.2.0";
/// Numeric ABI/API version checked when loading external dynamic plugins (`4.2.0` -> `40200`).
pub const PDK_API_VERSION: u32 = 40200;

pub type PluginPoint = InsertPoint;
pub type PluginSearchContext = SearchContext;

use async_trait::async_trait;
use dashmap::DashMap;
use hyperspace_core::{Collection, Durability, SearchParams, SearchResult};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

/// Convenience prelude for authoring external HyperspaceDB plugins (`use hyperspace_pdk::prelude::*;`).
pub mod prelude {
    pub use crate::{
        hs_export_plugin, hs_plugin_manifest, EngineContext, HookStage, HsPlugin, InsertPoint,
        MetricDescriptor, PluginManifest, ProcedureSchema, SearchContext, PDK_API_VERSION,
        PDK_VERSION,
    };
    pub use async_trait::async_trait;
    pub use hyperspace_core::{Durability, SearchResult};
    pub use serde::{Deserialize, Serialize};
    pub use serde_json::{json, Value};
}

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

fn default_enabled_true() -> bool {
    true
}

fn default_source_builtin() -> String {
    "builtin".to_string()
}

/// Self-describing plugin manifest exposed via `ListExtensions` and `/v1/extensions/schema`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub runtime: String,
    #[serde(default = "default_enabled_true")]
    pub enabled: bool,
    #[serde(default = "default_source_builtin")]
    pub source: String,
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

/// Core `pgrx`-style trait implemented by all HyperspaceDB plugins (built-in and external `cdylib`).
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
            enabled: true,
            source: "external".to_string(),
            hooks: vec![$($hook),*],
            procedures: vec![$($proc),*],
            custom_metrics: vec![],
        }
    };
}

/// Exports a [`HsPlugin`] implementation as a dynamically loadable shared library (`cdylib`).
///
/// This is the HyperspaceDB equivalent of `pgrx::pg_module_magic!()`.
/// When compiled as a `.dylib` (macOS), `.so` (Linux), or `.dll` (Windows) and placed in `./plugins/`,
/// `PluginRegistry` verifies `hs_pdk_api_version()` and instantiates the plugin via `hs_plugin_create()`.
#[macro_export]
macro_rules! hs_export_plugin {
    ($constructor:expr) => {
        #[cfg(not(feature = "builtin"))]
        #[no_mangle]
        pub extern "C" fn hs_pdk_api_version() -> u32 {
            $crate::PDK_API_VERSION
        }

        #[cfg(not(feature = "builtin"))]
        #[no_mangle]
        pub extern "C" fn hs_plugin_create() -> *mut ::std::boxed::Box<dyn $crate::HsPlugin> {
            let plugin: ::std::boxed::Box<dyn $crate::HsPlugin> =
                ::std::boxed::Box::new(($constructor)());
            ::std::boxed::Box::into_raw(::std::boxed::Box::new(plugin))
        }
    };
}

struct RegisteredPluginEntry {
    plugin: Arc<dyn HsPlugin>,
    enabled: AtomicBool,
    source: String,
}

static LOADED_LIBS_CACHE: parking_lot::RwLock<Option<HashMap<PathBuf, Arc<libloading::Library>>>> =
    parking_lot::RwLock::new(None);

/// Thread-safe registry managing built-in and dynamically loaded external HyperspaceDB plugins,
/// activation states (`enabled` / `disabled`), lifecycle pipeline hooks, and RPC procedures.
impl PluginRegistry {
    pub fn get_plugins_dir(&self) -> PathBuf {
        self.plugins_dir.read().clone()
    }
}

pub struct PluginRegistry {
    plugins: DashMap<String, Arc<RegisteredPluginEntry>>,
    hook_order: RwLock<Vec<String>>,
    loaded_libraries: RwLock<Vec<Arc<libloading::Library>>>,
    plugins_dir: RwLock<PathBuf>,
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::with_standard_plugins()
    }
}

impl PluginRegistry {
    /// Creates an empty `PluginRegistry`.
    pub fn new() -> Self {
        let dir = Self::resolve_default_plugins_dir();
        Self {
            plugins: DashMap::new(),
            hook_order: RwLock::new(Vec::new()),
            loaded_libraries: RwLock::new(Vec::new()),
            plugins_dir: RwLock::new(dir),
        }
    }

    /// Resolves the active plugins directory from `HYPERSPACE_PLUGINS_DIR` or `./plugins`
    /// (falling back to workspace `<root>/plugins` during cargo tests).
    pub fn resolve_default_plugins_dir() -> PathBuf {
        if let Ok(custom) = std::env::var("HYPERSPACE_PLUGINS_DIR") {
            let trimmed = custom.trim();
            if !trimmed.is_empty() {
                let p = PathBuf::from(trimmed);
                if p.exists() {
                    return p;
                }
            }
        }
        let cwd_plugins = PathBuf::from("plugins");
        if cwd_plugins.exists() {
            return cwd_plugins;
        }
        // Fallback when running `cargo test` inside `crates/<crate>`
        let workspace_plugins = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|p| p.parent())
            .map(|root| root.join("plugins"));
        if let Some(wp) = workspace_plugins {
            if wp.exists() {
                return wp;
            }
        }
        cwd_plugins
    }

    /// Creates a `PluginRegistry` initialized with:
    /// 1. Built-in core plugins:
    ///    - `embed` (`hs-embed`) — runs first on `BeforeInsert` to materialize vectors from text
    ///    - `shield` (`hs-shield`) — runs on `BeforeInsert` and `BeforeSearch` for isometric encryption
    /// 2. External dynamic plugins discovered from `./plugins` (`HYPERSPACE_PLUGINS_DIR`), such as:
    ///    - `git_ast` (`hs-git-ast`) — external `cdylib` AST codebase parser and zero-cost KAG builder
    /// 3. Activation state configured via `.env` (`HYPERSPACE_ENABLED_PLUGINS`, `HYPERSPACE_DISABLED_PLUGINS`).
    pub fn with_standard_plugins() -> Self {
        let registry = Self::new();
        // 1. Register built-in plugins
        registry.register_with_source(Arc::new(HsEmbedPlugin::new()), "builtin", true);
        registry.register_with_source(Arc::new(HsShieldPlugin::new()), "builtin", true);

        // 2. Discover and dynamically load external plugins from `./plugins`
        let dir = registry.plugins_dir.read().clone();
        let _ = registry.load_plugins_from_dir(&dir);

        // 3. Apply `.env` activation/deactivation rules
        registry.apply_env_config();
        registry
    }

    /// Attaches a configured `MultiVectorizer` (ONNX / HuggingFace Hub / Remote API)
    /// to the built-in `hs-embed` (`embed`) plugin while preserving its `enabled` state.
    pub fn set_embed_vectorizer(&self, vectorizer: Option<Arc<MultiVectorizer>>) {
        let was_enabled = self.is_plugin_enabled("embed");
        self.register_with_source(
            Arc::new(HsEmbedPlugin::with_vectorizer(vectorizer)),
            "builtin",
            was_enabled,
        );
    }

    /// Registers a plugin in the runtime registry as a built-in active plugin.
    pub fn register(&self, plugin: Arc<dyn HsPlugin>) {
        self.register_with_source(plugin, "builtin", true);
    }

    /// Registers a plugin with explicit provenance (`"builtin"` or `"external:<path>"`) and initial `enabled` state.
    pub fn register_with_source(
        &self,
        plugin: Arc<dyn HsPlugin>,
        source: impl Into<String>,
        enabled: bool,
    ) {
        let manifest = plugin.manifest();
        let name = Self::normalize_plugin_name(&manifest.name);
        let entry = Arc::new(RegisteredPluginEntry {
            plugin,
            enabled: AtomicBool::new(enabled),
            source: source.into(),
        });
        self.plugins.insert(name.clone(), entry);
        let mut order = self.hook_order.write();
        if !order.contains(&name) {
            // Keep `shield` at the end of `BeforeInsert` pipeline so embeddings generated by `embed`
            // (or external ingestion plugins) are rotated last before storage.
            if name != "shield" {
                if let Some(shield_pos) = order.iter().position(|n| n == "shield") {
                    order.insert(shield_pos, name);
                    return;
                }
            }
            order.push(name);
        }
    }

    /// Dynamically loads an external plugin shared library (`.dylib` / `.so` / `.dll`) using `libloading`.
    pub fn load_dynamic_library(&self, path: impl AsRef<Path>) -> Result<PluginManifest, String> {
        let path_ref = path.as_ref();
        if !path_ref.exists() {
            return Err(format!(
                "External plugin shared library not found: {}",
                path_ref.display()
            ));
        }

        let stem = path_ref
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .trim_start_matches("lib");
        let normalized_stem = Self::normalize_plugin_name(stem);
        if let Some(entry) = self.plugins.get(&normalized_stem) {
            if entry.source == "builtin" {
                // Already statically registered as built-in, skip foreign dylib dlopen
                return self.get_manifest(&normalized_stem).ok_or_else(|| "Already loaded".to_string());
            }
        }

        let canonical_path = path_ref
            .canonicalize()
            .unwrap_or_else(|_| path_ref.to_path_buf());

        // Re-use already loaded shared library instance to prevent repeated dlopen / dyld lock
        // contention and avoid dropping/dlclosing active plugin symbols across test runs.
        let lib_arc = {
            let read_cache = LOADED_LIBS_CACHE.read();
            read_cache
                .as_ref()
                .and_then(|c| c.get(&canonical_path).cloned())
        };

        let lib = match lib_arc {
            Some(arc) => arc,
            None => {
                let opened = unsafe {
                    libloading::Library::new(path_ref).map_err(|e| {
                        format!(
                            "Failed to open dynamic plugin library '{}': {e}",
                            path_ref.display()
                        )
                    })?
                };
                let arc = Arc::new(opened);
                let mut write_cache = LOADED_LIBS_CACHE.write();
                let cache = write_cache.get_or_insert_with(HashMap::new);
                cache.insert(canonical_path, Arc::clone(&arc));
                arc
            }
        };

        // SAFETY: External plugins compiled with `hs_export_plugin!` expose C-ABI `hs_pdk_api_version`
        // and `hs_plugin_create` functions. We keep the `Library` handle alive permanently in the static cache.
        unsafe {
            let version_fn: libloading::Symbol<unsafe extern "C" fn() -> u32> =
                lib.get(b"hs_pdk_api_version\0").map_err(|e| {
                    format!(
                        "Library '{}' is missing `hs_pdk_api_version` symbol (use `hs_export_plugin!`): {e}",
                        path_ref.display()
                    )
                })?;

            let plugin_api_ver = version_fn();
            if plugin_api_ver != PDK_API_VERSION {
                return Err(format!(
                    "ABI version mismatch for '{}': plugin has {plugin_api_ver}, host expects {PDK_API_VERSION}",
                    path_ref.display()
                ));
            }

            let create_fn: libloading::Symbol<unsafe extern "C" fn() -> *mut Box<dyn HsPlugin>> =
                lib.get(b"hs_plugin_create\0").map_err(|e| {
                    format!(
                        "Library '{}' is missing `hs_plugin_create` symbol: {e}",
                        path_ref.display()
                    )
                })?;

            let raw_ptr = create_fn();
            if raw_ptr.is_null() {
                return Err(format!(
                    "`hs_plugin_create` returned null for '{}'",
                    path_ref.display()
                ));
            }

            let boxed_outer: Box<Box<dyn HsPlugin>> = Box::from_raw(raw_ptr);
            let plugin_arc: Arc<dyn HsPlugin> = Arc::from(*boxed_outer);
            let source_tag = format!("external:{}", path_ref.display());

            self.loaded_libraries.write().push(Arc::clone(&lib));
            self.register_with_source(Arc::clone(&plugin_arc), source_tag, true);

            let name = Self::normalize_plugin_name(&plugin_arc.manifest().name);
            self.get_manifest(&name)
                .ok_or_else(|| "Failed to retrieve manifest after registration".to_string())
        }
    }

    /// Scans a plugins directory (e.g. `./plugins`) for shared libraries (`.dylib`, `.so`, `.dll`)
    /// and plugin package directories containing `plugin.toml`, loading all discovered external plugins.
    pub fn load_plugins_from_dir(
        &self,
        dir: impl AsRef<Path>,
    ) -> Result<Vec<PluginManifest>, String> {
        let dir_path = dir.as_ref();
        *self.plugins_dir.write() = dir_path.to_path_buf();

        if !dir_path.exists() {
            return Ok(Vec::new());
        }

        let mut loaded = Vec::new();
        let mut loaded_libs_by_stem = HashSet::new();

        // 1. Scan top-level `.dylib` / `.so` / `.dll` files inside `dir`
        if let Ok(entries) = std::fs::read_dir(dir_path) {
            let mut paths: Vec<PathBuf> = entries.flatten().map(|e| e.path()).collect();
            paths.sort();

            for path in &paths {
                if path.is_file() && Self::is_shared_library(path) {
                    match self.load_dynamic_library(path) {
                        Ok(manifest) => {
                            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                                loaded_libs_by_stem
                                    .insert(stem.trim_start_matches("lib").to_string());
                            }
                            loaded.push(manifest);
                        }
                        Err(e) => {
                            if std::env::var("HYPERSPACE_DEBUG_PLUGINS").is_ok() {
                                eprintln!("⚠️ [PluginRegistry] Skipping dynamic library candidate '{}': {e}", path.display());
                            }
                        }
                    }
                }
            }

            // 2. Scan plugin subdirectories containing `plugin.toml` (e.g. `plugins/hs-git-ast/plugin.toml`)
            for path in &paths {
                if path.is_dir() {
                    let plugin_toml = path.join("plugin.toml");
                    if plugin_toml.exists() {
                        if let Some((lib_name, default_enabled)) =
                            Self::parse_plugin_toml(&plugin_toml)
                        {
                            if !loaded_libs_by_stem.contains(&lib_name) {
                                if let Some(cdylib_path) =
                                    Self::find_compiled_cdylib(dir_path, path, &lib_name)
                                {
                                    match self.load_dynamic_library(&cdylib_path) {
                                        Ok(manifest) => {
                                            if !default_enabled {
                                                let _ =
                                                    self.set_plugin_enabled(&manifest.name, false);
                                            }
                                            loaded_libs_by_stem.insert(lib_name);
                                            if let Some(updated) = self.get_manifest(&manifest.name)
                                            {
                                                loaded.push(updated);
                                            }
                                        }
                                        Err(e) => {
                                            if std::env::var("HYPERSPACE_DEBUG_PLUGINS").is_ok() {
                                                eprintln!("⚠️ [PluginRegistry] Failed to load compiled cdylib '{}': {e}", cdylib_path.display());
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        self.apply_env_config();
        Ok(loaded)
    }

    /// Re-scans the configured `HYPERSPACE_PLUGINS_DIR` (`./plugins`) at runtime and hot-loads external plugins.
    pub fn reload_external_plugins(&self) -> Result<Vec<PluginManifest>, String> {
        let dir = self.plugins_dir.read().clone();
        self.load_plugins_from_dir(dir)
    }

    /// Enables or disables a registered plugin at runtime (`POST /v1/extensions/{plugin}/enable|disable`).
    pub fn set_plugin_enabled(&self, name: &str, enabled: bool) -> Result<PluginManifest, String> {
        let normalized = Self::normalize_plugin_name(name);
        let entry = self
            .plugins
            .get(&normalized)
            .ok_or_else(|| format!("Extension plugin '{name}' is not registered"))?;
        entry.enabled.store(enabled, Ordering::Relaxed);
        drop(entry);
        self.get_manifest(&normalized)
            .ok_or_else(|| format!("Extension plugin '{name}' not found"))
    }

    /// Checks whether a plugin is currently registered and enabled.
    pub fn is_plugin_enabled(&self, name: &str) -> bool {
        let normalized = Self::normalize_plugin_name(name);
        self.plugins
            .get(&normalized)
            .is_some_and(|e| e.enabled.load(Ordering::Relaxed))
    }

    /// Applies `HYPERSPACE_ENABLED_PLUGINS` and `HYPERSPACE_DISABLED_PLUGINS` environment variables.
    pub fn apply_env_config(&self) {
        if let Ok(enabled_list) = std::env::var("HYPERSPACE_ENABLED_PLUGINS") {
            let trimmed = enabled_list.trim();
            if !trimmed.is_empty() && trimmed != "*" && !trimmed.eq_ignore_ascii_case("all") {
                let allowed: HashSet<String> = trimmed
                    .split(',')
                    .map(Self::normalize_plugin_name)
                    .filter(|s| !s.is_empty())
                    .collect();
                for item in self.plugins.iter() {
                    let is_on = allowed.contains(item.key());
                    item.value().enabled.store(is_on, Ordering::Relaxed);
                }
            }
        }

        if let Ok(disabled_list) = std::env::var("HYPERSPACE_DISABLED_PLUGINS") {
            for raw in disabled_list.split(',') {
                let norm = Self::normalize_plugin_name(raw);
                if !norm.is_empty() {
                    if let Some(entry) = self.plugins.get(&norm) {
                        entry.enabled.store(false, Ordering::Relaxed);
                    }
                }
            }
        }
    }

    fn is_shared_library(path: &Path) -> bool {
        matches!(
            path.extension().and_then(|e| e.to_str()),
            Some("dylib" | "so" | "dll")
        )
    }

    fn parse_plugin_toml(path: &Path) -> Option<(String, bool)> {
        let content = std::fs::read_to_string(path).ok()?;
        let mut library = None;
        let mut default_enabled = true;
        for line in content.lines() {
            let t = line.trim();
            if t.starts_with('#') || t.is_empty() {
                continue;
            }
            if let Some((k, v)) = t.split_once('=') {
                let key = k.trim();
                let val = v.trim().trim_matches('"').trim_matches('\'');
                if key == "library" {
                    library = Some(val.to_string());
                } else if key == "default_enabled" {
                    default_enabled = !val.eq_ignore_ascii_case("false");
                }
            }
        }
        library.map(|l| (l, default_enabled))
    }

    fn shared_lib_filenames(lib_name: &str) -> [String; 3] {
        [
            format!("lib{lib_name}.dylib"),
            format!("lib{lib_name}.so"),
            format!("{lib_name}.dll"),
        ]
    }

    fn find_compiled_cdylib(
        plugins_dir: &Path,
        plugin_crate_dir: &Path,
        lib_name: &str,
    ) -> Option<PathBuf> {
        let filenames = Self::shared_lib_filenames(lib_name);
        let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|p| p.parent())
            .map(Path::to_path_buf);

        let mut search_dirs = vec![
            plugins_dir.to_path_buf(),
            plugin_crate_dir.join("target").join("release"),
            plugin_crate_dir.join("target").join("debug"),
        ];
        if let Some(root) = workspace_root {
            search_dirs.push(root.join("plugins"));
            search_dirs.push(root.join("target").join("release"));
            search_dirs.push(root.join("target").join("debug"));
            search_dirs.push(root.join("target").join("release").join("deps"));
            search_dirs.push(root.join("target").join("debug").join("deps"));
        }

        for dir in search_dirs {
            for fname in &filenames {
                let candidate = dir.join(fname);
                if candidate.exists() {
                    return Some(candidate);
                }
            }
        }
        None
    }

    /// Returns a registered plugin by name if it exists (regardless of `enabled` state).
    pub fn get(&self, name: &str) -> Option<Arc<dyn HsPlugin>> {
        let normalized = Self::normalize_plugin_name(name);
        self.plugins
            .get(&normalized)
            .map(|p| Arc::clone(&p.value().plugin))
    }

    /// Returns the live [`PluginManifest`] (with runtime `enabled` and `source` fields populated).
    pub fn get_manifest(&self, name: &str) -> Option<PluginManifest> {
        let normalized = Self::normalize_plugin_name(name);
        self.plugins.get(&normalized).map(|entry| {
            let mut m = entry.plugin.manifest();
            m.enabled = entry.enabled.load(Ordering::Relaxed);
            m.source.clone_from(&entry.source);
            m
        })
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

    /// Lists manifests for all registered plugins (both enabled and disabled) in deterministic order.
    pub fn list_manifests(&self) -> Vec<PluginManifest> {
        let order = self.hook_order.read().clone();
        let mut out = Vec::with_capacity(order.len());
        for name in order {
            if let Some(m) = self.get_manifest(&name) {
                out.push(m);
            }
        }
        out
    }

    /// Returns the full JSON Schema catalog of all registered extensions, their activation status, and supported metrics.
    pub fn schema_catalog_json(&self) -> Value {
        let manifests = self.list_manifests();
        let plugins_dir = self.plugins_dir.read().display().to_string();
        json!({
            "pdk_version": PDK_VERSION,
            "pdk_api_version": PDK_API_VERSION,
            "plugins_dir": plugins_dir,
            "supported_metrics": hyperspace_core::SUPPORTED_METRICS,
            "extensions": manifests,
        })
    }

    /// Runs all **enabled** `BeforeInsert` hooks in deterministic pipeline order (`embed` -> ... -> `shield`).
    pub async fn run_before_insert(
        &self,
        collection: &str,
        metric: &str,
        dimension: usize,
        points: &mut Vec<InsertPoint>,
    ) -> Result<(), String> {
        let order = self.hook_order.read().clone();
        for name in order {
            let active_plugin = self.plugins.get(&name).and_then(|entry| {
                if entry.enabled.load(Ordering::Relaxed) {
                    Some(Arc::clone(&entry.plugin))
                } else {
                    None
                }
            });
            if let Some(plugin) = active_plugin {
                plugin
                    .before_insert(collection, metric, dimension, points)
                    .await?;
            }
        }
        Ok(())
    }

    /// Runs all **enabled** `BeforeSearch` hooks in deterministic pipeline order.
    pub async fn run_before_search(&self, ctx: &mut SearchContext) -> Result<(), String> {
        let order = self.hook_order.read().clone();
        for name in order {
            let active_plugin = self.plugins.get(&name).and_then(|entry| {
                if entry.enabled.load(Ordering::Relaxed) {
                    Some(Arc::clone(&entry.plugin))
                } else {
                    None
                }
            });
            if let Some(plugin) = active_plugin {
                plugin.before_search(ctx).await?;
            }
        }
        Ok(())
    }

    /// Dispatches a procedure call (`plugin.method(args)`) if the plugin is registered and enabled.
    pub async fn call_procedure(
        &self,
        plugin_name: &str,
        method: &str,
        args: Value,
        ctx: &EngineContext,
    ) -> Result<Value, String> {
        let normalized = Self::normalize_plugin_name(plugin_name);
        let entry = self
            .plugins
            .get(&normalized)
            .ok_or_else(|| format!("Extension plugin '{plugin_name}' is not registered"))?;
        if !entry.enabled.load(Ordering::Relaxed) {
            return Err(format!(
                "Extension plugin '{normalized}' is currently disabled. Enable it via POST /v1/extensions/{normalized}/enable or HYPERSPACE_ENABLED_PLUGINS in .env"
            ));
        }
        let plugin = Arc::clone(&entry.plugin);
        drop(entry);
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
        assert!(
            manifests
                .iter()
                .any(|m| m.name == "embed" && m.source == "builtin" && m.enabled),
            "embed must be registered as a builtin enabled plugin"
        );
        assert!(
            manifests
                .iter()
                .any(|m| m.name == "shield" && m.source == "builtin" && m.enabled),
            "shield must be registered as a builtin enabled plugin"
        );

        // Test runtime disable and re-enable of a plugin
        let disabled = registry.set_plugin_enabled("embed", false).unwrap();
        assert!(!disabled.enabled);
        let ctx = EngineContext::new("admin", None, None, 1);
        let err = registry
            .call_procedure("embed", "chunk_text", json!({"text": "hello"}), &ctx)
            .await;
        assert!(
            err.is_err(),
            "Calling a disabled plugin must return an error"
        );

        let enabled = registry.set_plugin_enabled("embed", true).unwrap();
        assert!(enabled.enabled);

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
                    assert!(
                        <hyperspace_core::PoincareMetric as hyperspace_core::Metric>::validate(
                            &vec
                        )
                        .is_ok()
                    );
                }
                "lorentz" => {
                    assert!(
                        <hyperspace_core::LorentzMetric as hyperspace_core::Metric>::validate(&vec)
                            .is_ok()
                    );
                }
                "hybrid" => {
                    assert!(
                        <hyperspace_core::HybridMetric as hyperspace_core::Metric>::validate(&vec)
                            .is_ok()
                    );
                }
                _ => {}
            }
        }
    }

    #[tokio::test]
    async fn test_external_git_ast_dynamic_plugin_loading() {
        let registry = PluginRegistry::with_standard_plugins();
        if registry.get("git_ast").is_none() {
            // If `libhs_git_ast` was not yet compiled before this unit test ran, skip or check manifest dir
            return;
        }
        let manifest = registry.get_manifest("git_ast").unwrap();
        assert_eq!(manifest.name, "git_ast");
        assert!(
            manifest.source.starts_with("external:"),
            "git_ast must be loaded as an external dynamic library plugin, got: {}",
            manifest.source
        );

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
        assert!(
            symbols.len() >= 4,
            "Should extract module, trait, struct, impl, and fn"
        );
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

        let v1 =
            HsEmbedPlugin::embed_for_manifold("Patient diagnosis: Type-1 diabetes", 64, "cosine");
        let v2 = HsEmbedPlugin::embed_for_manifold("Insulin therapy protocol", 64, "cosine");
        let orig_dist =
            <hyperspace_core::EuclideanMetric as hyperspace_core::Metric>::distance(&v1, &v2);

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
            <hyperspace_core::EuclideanMetric as hyperspace_core::Metric>::distance(
                &v1,
                &points[0].vector,
            );
        assert!(
            diff_from_plain > 0.1,
            "Shielded vector must not equal plaintext vector"
        );

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
        let recovery_err = <hyperspace_core::EuclideanMetric as hyperspace_core::Metric>::distance(
            &v1, &recovered,
        );
        assert!(
            recovery_err < 1e-12,
            "Inverse projection must recover original vector"
        );
    }
}
