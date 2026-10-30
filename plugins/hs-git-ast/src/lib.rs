//! External HyperspaceDB Plugin: `hs-git-ast` (`git_ast`)
//!
//! Instant Codebase AST Topology & Zero-Cost KAG Builder.
//!
//! Parses source files across Rust, Python, TypeScript/JavaScript, and Go directly inside
//! HyperspaceDB, extracting semantic code symbols (`function`, `struct`, `class`, `trait`,
//! `interface`, `impl`, `module`) and wiring topological edges (`CONTAINS`, `CALLS`,
//! `IMPORTS`, `IMPLEMENTS`) into `GraphEngine` in a single pass.
//!
//! This crate compiles to a shared library (`cdylib`: `.dylib` / `.so` / `.dll`) and exports
//! its C-ABI plugin entrypoint via `hyperspace_pdk::hs_export_plugin!`.

#![allow(clippy::doc_markdown)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::if_not_else)]

use async_trait::async_trait;
use hyperspace_core::Durability;
use hyperspace_pdk::{
    hs_export_plugin, EngineContext, HsEmbedPlugin, HsPlugin, InsertPoint, PluginManifest,
    ProcedureSchema,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
use std::path::Path;

/// Relation ID: File/Module/Struct/Class contains a child symbol.
pub const REL_AST_CONTAINS: u16 = 10;
/// Relation ID: Function/Method calls another function/method.
pub const REL_AST_CALLS: u16 = 11;
/// Relation ID: File/Module imports another module or symbol.
pub const REL_AST_IMPORTS: u16 = 12;
/// Relation ID: Struct/Class implements a trait/interface or inherits a base class.
pub const REL_AST_IMPLEMENTS: u16 = 13;

/// A semantic AST symbol extracted from source code.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeSymbol {
    pub id: u32,
    pub file_path: String,
    pub language: String,
    pub kind: String,
    pub name: String,
    pub signature: String,
    pub start_line: usize,
    pub end_line: usize,
    pub code: String,
    pub calls: Vec<String>,
    pub imports: Vec<String>,
    pub implements: Option<String>,
}

/// A directed topological edge between two extracted code symbols.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeEdge {
    pub source_id: u32,
    pub target_id: u32,
    pub relation: u16,
    pub relation_name: &'static str,
    pub weight: f32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SourceFileInput {
    pub path: String,
    pub content: String,
}

#[derive(Debug, Clone, Deserialize)]
struct IngestFilesArgs {
    pub files: Vec<SourceFileInput>,
    #[serde(default)]
    pub base_id: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
struct IngestDirectoryArgs {
    pub path: String,
    #[serde(default)]
    pub max_files: Option<usize>,
    #[serde(default)]
    pub base_id: Option<u32>,
}

/// External `hs-git-ast` plugin implementation.
#[derive(Debug, Default)]
pub struct HsGitAstPlugin;

impl HsGitAstPlugin {
    pub fn new() -> Self {
        Self
    }

    /// Detects programming language from file extension.
    pub fn detect_language(path: &str) -> &'static str {
        let ext = Path::new(path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        match ext.as_str() {
            "rs" => "rust",
            "py" => "python",
            "ts" | "tsx" => "typescript",
            "js" | "jsx" | "mjs" => "javascript",
            "go" => "go",
            _ => "unknown",
        }
    }

    /// Parses a collection of source files into semantic AST symbols and resolves cross-file KAG edges.
    pub fn analyze_files(
        files: &[SourceFileInput],
        base_id: u32,
    ) -> (Vec<CodeSymbol>, Vec<CodeEdge>) {
        let mut symbols = Vec::new();
        let mut next_id = base_id;

        for file in files {
            let lang = Self::detect_language(&file.path).to_string();
            let file_root_id = next_id;
            next_id += 1;

            let lines: Vec<&str> = file.content.lines().collect();
            let mut file_imports = Vec::new();
            for line in &lines {
                let t = line.trim();
                if let Some(imp) = Self::extract_import(t, &lang) {
                    file_imports.push(imp);
                }
            }

            // Create file module root node
            let preview: String = lines
                .iter()
                .take(25)
                .copied()
                .collect::<Vec<_>>()
                .join("\n");
            symbols.push(CodeSymbol {
                id: file_root_id,
                file_path: file.path.clone(),
                language: lang.clone(),
                kind: "module".to_string(),
                name: file.path.clone(),
                signature: format!("module {}", file.path),
                start_line: 1,
                end_line: lines.len().max(1),
                code: preview,
                calls: Vec::new(),
                imports: file_imports,
                implements: None,
            });

            // Scan top-level and nested declarations
            let mut idx = 0usize;
            while idx < lines.len() {
                let trimmed = lines[idx].trim();
                if let Some((kind, name, impl_target)) = Self::match_declaration(trimmed, &lang) {
                    let start_line = idx + 1;
                    let end_idx = Self::find_block_end(&lines, idx, &lang);
                    let end_line = end_idx + 1;
                    let block_lines = &lines[idx..=end_idx];
                    let block_code = block_lines.join("\n");
                    let calls = Self::extract_callee_names(&block_code, &name);

                    let sym_id = next_id;
                    next_id += 1;

                    symbols.push(CodeSymbol {
                        id: sym_id,
                        file_path: file.path.clone(),
                        language: lang.clone(),
                        kind,
                        name,
                        signature: trimmed.trim_end_matches('{').trim().to_string(),
                        start_line,
                        end_line,
                        code: block_code,
                        calls,
                        imports: Vec::new(),
                        implements: impl_target,
                    });
                    idx = end_idx + 1;
                } else {
                    idx += 1;
                }
            }
        }

        // Build lookup indices by symbol name and module path
        let mut by_name: HashMap<String, Vec<u32>> = HashMap::new();
        let mut file_roots: HashMap<String, u32> = HashMap::new();
        for sym in &symbols {
            if sym.kind == "module" {
                file_roots.insert(sym.file_path.clone(), sym.id);
            } else {
                by_name.entry(sym.name.clone()).or_default().push(sym.id);
            }
        }

        let mut edges = Vec::new();
        for sym in &symbols {
            // 1. Module CONTAINS symbol
            if sym.kind != "module" {
                if let Some(&root_id) = file_roots.get(&sym.file_path) {
                    edges.push(CodeEdge {
                        source_id: root_id,
                        target_id: sym.id,
                        relation: REL_AST_CONTAINS,
                        relation_name: "CONTAINS",
                        weight: 1.0,
                    });
                }
            }

            // 2. Function/Method CALLS target symbol
            for callee in &sym.calls {
                if let Some(targets) = by_name.get(callee) {
                    for &target_id in targets {
                        if target_id != sym.id {
                            edges.push(CodeEdge {
                                source_id: sym.id,
                                target_id,
                                relation: REL_AST_CALLS,
                                relation_name: "CALLS",
                                weight: 0.9,
                            });
                        }
                    }
                }
            }

            // 3. Struct/Class IMPLEMENTS or INHERITS trait/interface/base
            if let Some(ref trait_name) = sym.implements {
                if let Some(targets) = by_name.get(trait_name) {
                    for &target_id in targets {
                        if target_id != sym.id {
                            edges.push(CodeEdge {
                                source_id: sym.id,
                                target_id,
                                relation: REL_AST_IMPLEMENTS,
                                relation_name: "IMPLEMENTS",
                                weight: 0.95,
                            });
                        }
                    }
                }
            }

            // 4. Module IMPORTS symbol or module
            for imp in &sym.imports {
                if let Some(targets) = by_name.get(imp) {
                    for &target_id in targets {
                        if target_id != sym.id {
                            edges.push(CodeEdge {
                                source_id: sym.id,
                                target_id,
                                relation: REL_AST_IMPORTS,
                                relation_name: "IMPORTS",
                                weight: 0.85,
                            });
                        }
                    }
                }
            }
        }

        (symbols, edges)
    }

    fn extract_import(trimmed: &str, lang: &str) -> Option<String> {
        match lang {
            "rust" if trimmed.starts_with("use ") => {
                let body = trimmed.trim_start_matches("use ").trim_end_matches(';');
                let last = body.split("::").last()?.trim();
                if !last.is_empty() && last != "*" {
                    Some(
                        last.trim_matches(|c: char| !c.is_alphanumeric() && c != '_')
                            .to_string(),
                    )
                } else {
                    None
                }
            }
            "python" if trimmed.starts_with("import ") || trimmed.starts_with("from ") => {
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                parts.last().map(|s| (*s).to_string())
            }
            "typescript" | "javascript" if trimmed.starts_with("import ") => {
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                parts.get(1).map(|s| {
                    s.trim_matches(|c: char| !c.is_alphanumeric() && c != '_')
                        .to_string()
                })
            }
            _ => None,
        }
    }

    fn match_declaration(trimmed: &str, lang: &str) -> Option<(String, String, Option<String>)> {
        if trimmed.starts_with("//") || trimmed.starts_with('#') || trimmed.starts_with("/*") {
            return None;
        }
        // Strip visibility/async modifiers
        let mut s = trimmed;
        for kw in [
            "pub(crate) ",
            "pub(super) ",
            "pub ",
            "export ",
            "default ",
            "async ",
            "unsafe ",
            "const ",
        ] {
            while let Some(rest) = s.strip_prefix(kw) {
                s = rest.trim_start();
            }
        }

        match lang {
            "rust" => {
                if let Some(rest) = s.strip_prefix("fn ") {
                    let name = Self::ident_prefix(rest)?;
                    return Some(("function".to_string(), name, None));
                }
                if let Some(rest) = s.strip_prefix("struct ") {
                    let name = Self::ident_prefix(rest)?;
                    return Some(("struct".to_string(), name, None));
                }
                if let Some(rest) = s.strip_prefix("enum ") {
                    let name = Self::ident_prefix(rest)?;
                    return Some(("enum".to_string(), name, None));
                }
                if let Some(rest) = s.strip_prefix("trait ") {
                    let name = Self::ident_prefix(rest)?;
                    return Some(("trait".to_string(), name, None));
                }
                if let Some(rest) = s.strip_prefix("impl") {
                    let after_impl = rest.trim_start();
                    // Handle `impl<T> Trait for Type` or `impl Type`
                    let clean = if after_impl.starts_with('<') {
                        after_impl
                            .split_once('>')
                            .map_or(after_impl, |(_, r)| r.trim_start())
                    } else {
                        after_impl
                    };
                    if let Some((trait_part, type_part)) = clean.split_once(" for ") {
                        let trait_name = Self::ident_prefix(trait_part.trim())?;
                        let type_name = Self::ident_prefix(type_part.trim())?;
                        return Some(("impl".to_string(), type_name, Some(trait_name)));
                    }
                    let type_name = Self::ident_prefix(clean)?;
                    return Some(("impl".to_string(), type_name, None));
                }
                None
            }
            "python" => {
                if let Some(rest) = s.strip_prefix("def ") {
                    let name = Self::ident_prefix(rest)?;
                    return Some(("function".to_string(), name, None));
                }
                if let Some(rest) = s.strip_prefix("class ") {
                    let name = Self::ident_prefix(rest)?;
                    let base = rest
                        .split_once('(')
                        .and_then(|(_, b)| b.split_once(')'))
                        .and_then(|(inside, _)| Self::ident_prefix(inside.trim()));
                    return Some(("class".to_string(), name, base));
                }
                None
            }
            "typescript" | "javascript" => {
                if let Some(rest) = s.strip_prefix("function ") {
                    let name = Self::ident_prefix(rest)?;
                    return Some(("function".to_string(), name, None));
                }
                if let Some(rest) = s.strip_prefix("class ") {
                    let name = Self::ident_prefix(rest)?;
                    let imp = rest
                        .split_once("implements ")
                        .or_else(|| rest.split_once("extends "))
                        .and_then(|(_, b)| Self::ident_prefix(b.trim()));
                    return Some(("class".to_string(), name, imp));
                }
                if let Some(rest) = s.strip_prefix("interface ") {
                    let name = Self::ident_prefix(rest)?;
                    return Some(("interface".to_string(), name, None));
                }
                None
            }
            "go" => {
                if let Some(rest) = s.strip_prefix("func ") {
                    let after_recv = if rest.starts_with('(') {
                        rest.split_once(')').map_or(rest, |(_, r)| r.trim_start())
                    } else {
                        rest
                    };
                    let name = Self::ident_prefix(after_recv)?;
                    return Some(("function".to_string(), name, None));
                }
                if let Some(rest) = s.strip_prefix("type ") {
                    let name = Self::ident_prefix(rest)?;
                    return Some(("struct".to_string(), name, None));
                }
                None
            }
            _ => None,
        }
    }

    fn ident_prefix(s: &str) -> Option<String> {
        let ident: String = s
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect();
        if ident.is_empty() {
            None
        } else {
            Some(ident)
        }
    }

    fn find_block_end(lines: &[&str], start_idx: usize, lang: &str) -> usize {
        if lang == "python" {
            let base_indent = lines[start_idx]
                .chars()
                .take_while(|c| c.is_whitespace())
                .count();
            let mut end = start_idx;
            for (i, line) in lines.iter().enumerate().skip(start_idx + 1) {
                let t = line.trim();
                if t.is_empty() || t.starts_with('#') {
                    continue;
                }
                let indent = line.chars().take_while(|c| c.is_whitespace()).count();
                if indent <= base_indent {
                    break;
                }
                end = i;
            }
            return end;
        }

        let mut brace_depth = 0i32;
        let mut seen_open = false;
        for (i, line) in lines.iter().enumerate().skip(start_idx) {
            for ch in line.chars() {
                if ch == '{' {
                    brace_depth += 1;
                    seen_open = true;
                } else if ch == '}' {
                    brace_depth -= 1;
                }
            }
            if seen_open && brace_depth <= 0 {
                return i;
            }
            if !seen_open && line.trim_end().ends_with(';') {
                return i;
            }
        }
        (start_idx + 25).min(lines.len().saturating_sub(1))
    }

    fn extract_callee_names(code: &str, self_name: &str) -> Vec<String> {
        const KEYWORDS: &[&str] = &[
            "if",
            "for",
            "while",
            "match",
            "loop",
            "return",
            "sizeof",
            "Some",
            "None",
            "Ok",
            "Err",
            "vec",
            "format",
            "println",
            "eprintln",
            "assert",
            "assert_eq",
            "fn",
            "def",
            "func",
            "function",
            "class",
        ];
        let mut set = HashSet::new();
        let bytes = code.as_bytes();
        let mut i = 0usize;
        while i < bytes.len() {
            if bytes[i] == b'(' && i > 0 {
                let mut start = i;
                while start > 0
                    && (bytes[start - 1].is_ascii_alphanumeric() || bytes[start - 1] == b'_')
                {
                    start -= 1;
                }
                if start < i {
                    if let Ok(ident) = std::str::from_utf8(&bytes[start..i]) {
                        if ident != self_name
                            && ident.len() >= 2
                            && !ident.chars().next().unwrap_or('0').is_ascii_digit()
                            && !KEYWORDS.contains(&ident)
                        {
                            set.insert(ident.to_string());
                        }
                    }
                }
            }
            i += 1;
        }
        let mut out: Vec<String> = set.into_iter().collect();
        out.sort();
        out
    }

    fn collect_source_files_from_dir(
        root: &Path,
        max_files: usize,
        out: &mut Vec<SourceFileInput>,
    ) -> Result<(), String> {
        if out.len() >= max_files {
            return Ok(());
        }
        let entries = std::fs::read_dir(root)
            .map_err(|e| format!("Failed to read dir {}: {e}", root.display()))?;
        for entry in entries.flatten() {
            if out.len() >= max_files {
                break;
            }
            let path = entry.path();
            let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if file_name.starts_with('.')
                || matches!(
                    file_name,
                    "target" | "node_modules" | "__pycache__" | "dist" | "build" | "vendor"
                )
            {
                continue;
            }
            if path.is_dir() {
                Self::collect_source_files_from_dir(&path, max_files, out)?;
            } else if path.is_file() {
                let path_str = path.to_string_lossy().to_string();
                if Self::detect_language(&path_str) != "unknown" {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        out.push(SourceFileInput {
                            path: path_str,
                            content,
                        });
                    }
                }
            }
        }
        Ok(())
    }

    async fn ingest_into_engine(
        files: &[SourceFileInput],
        base_id_opt: Option<u32>,
        ctx: &EngineContext,
    ) -> Result<Value, String> {
        let col = ctx.collection()?;
        let dim = col.dimension();
        let metric = col.metric_name();
        let base_id = base_id_opt
            .unwrap_or_else(|| (col.count() as u32).saturating_add(1).saturating_mul(1000));

        let (symbols, edges) = Self::analyze_files(files, base_id);
        let mut points = Vec::with_capacity(symbols.len());

        for sym in &symbols {
            let embed_input = format!(
                "{} {} {} {}\n{}",
                sym.language, sym.kind, sym.name, sym.signature, sym.code
            );
            let vec = HsEmbedPlugin::embed_for_manifold(&embed_input, dim, metric);
            let mut meta = HashMap::new();
            meta.insert("file_path".to_string(), sym.file_path.clone());
            meta.insert("language".to_string(), sym.language.clone());
            meta.insert("symbol_kind".to_string(), sym.kind.clone());
            meta.insert("symbol_name".to_string(), sym.name.clone());
            meta.insert("signature".to_string(), sym.signature.clone());
            meta.insert("start_line".to_string(), sym.start_line.to_string());
            meta.insert("end_line".to_string(), sym.end_line.to_string());

            points.push(InsertPoint {
                id: sym.id,
                vector: vec,
                metadata: meta,
                payload: Some(sym.code.clone().into_bytes()),
            });
        }

        let raw_edges: Vec<(u32, u32, u16, f32)> = edges
            .iter()
            .map(|e| (e.source_id, e.target_id, e.relation, e.weight))
            .collect();

        let symbols_inserted = ctx.insert_points(points, Durability::Default).await?;
        let edges_linked = if !raw_edges.is_empty() {
            ctx.link_batch(&raw_edges)?
        } else {
            0
        };

        Ok(json!({
            "files_processed": files.len(),
            "symbols_inserted": symbols_inserted,
            "edges_linked": edges_linked,
            "symbols": symbols.iter().map(|s| json!({
                "id": s.id,
                "name": s.name,
                "kind": s.kind,
                "file_path": s.file_path,
                "start_line": s.start_line,
                "end_line": s.end_line
            })).collect::<Vec<_>>(),
            "edges": edges
        }))
    }
}

#[async_trait]
impl HsPlugin for HsGitAstPlugin {
    fn manifest(&self) -> PluginManifest {
        PluginManifest {
            name: "git_ast".to_string(),
            version: "4.2.0".to_string(),
            author: "HyperspaceDB Core Team".to_string(),
            description: "Zero-cost AST codebase analyzer and KAG graph builder for Rust, Python, TypeScript, and Go.".to_string(),
            runtime: "native-rust-cdylib".to_string(),
            enabled: true,
            source: "external".to_string(),
            hooks: vec![],
            procedures: vec![
                ProcedureSchema {
                    name: "ingest_files".to_string(),
                    description: "Parses source files into AST symbols, embeds them into the collection, and links CALLS/CONTAINS/IMPORTS/IMPLEMENTS edges in GraphEngine.".to_string(),
                    requires_collection: true,
                    input_schema: json!({
                        "type": "object",
                        "required": ["files"],
                        "properties": {
                            "files": {
                                "type": "array",
                                "items": {
                                    "type": "object",
                                    "required": ["path", "content"],
                                    "properties": {
                                        "path": { "type": "string" },
                                        "content": { "type": "string" }
                                    }
                                }
                            },
                            "base_id": { "type": "integer" }
                        }
                    }),
                    output_schema: json!({
                        "type": "object",
                        "properties": {
                            "files_processed": { "type": "integer" },
                            "symbols_inserted": { "type": "integer" },
                            "edges_linked": { "type": "integer" },
                            "symbols": { "type": "array" },
                            "edges": { "type": "array" }
                        }
                    }),
                },
                ProcedureSchema {
                    name: "ingest_directory".to_string(),
                    description: "Walks a local repository directory, extracts AST symbols and KAG topology, and ingests everything into the collection and GraphEngine.".to_string(),
                    requires_collection: true,
                    input_schema: json!({
                        "type": "object",
                        "required": ["path"],
                        "properties": {
                            "path": { "type": "string" },
                            "max_files": { "type": "integer", "default": 500 },
                            "base_id": { "type": "integer" }
                        }
                    }),
                    output_schema: json!({
                        "type": "object",
                        "properties": {
                            "files_processed": { "type": "integer" },
                            "symbols_inserted": { "type": "integer" },
                            "edges_linked": { "type": "integer" }
                        }
                    }),
                },
                ProcedureSchema {
                    name: "parse_source".to_string(),
                    description: "Extracts AST symbols and topological KAG edges from source files without requiring a collection.".to_string(),
                    requires_collection: false,
                    input_schema: json!({
                        "type": "object",
                        "required": ["files"],
                        "properties": {
                            "files": { "type": "array" },
                            "base_id": { "type": "integer", "default": 1 }
                        }
                    }),
                    output_schema: json!({
                        "type": "object",
                        "properties": {
                            "symbols": { "type": "array" },
                            "edges": { "type": "array" }
                        }
                    }),
                },
            ],
            custom_metrics: vec![],
        }
    }

    async fn call_procedure(
        &self,
        method: &str,
        args: Value,
        ctx: &EngineContext,
    ) -> Result<Value, String> {
        match method {
            "ingest_files" | "ingest_code" => {
                let parsed: IngestFilesArgs = serde_json::from_value(args)
                    .map_err(|e| format!("Invalid arguments for git_ast.{method}: {e}"))?;
                Self::ingest_into_engine(&parsed.files, parsed.base_id, ctx).await
            }
            "ingest_directory" | "ingest_repo" => {
                let parsed: IngestDirectoryArgs = serde_json::from_value(args)
                    .map_err(|e| format!("Invalid arguments for git_ast.{method}: {e}"))?;
                let max_files = parsed.max_files.unwrap_or(500);
                let mut files = Vec::new();
                Self::collect_source_files_from_dir(
                    Path::new(&parsed.path),
                    max_files,
                    &mut files,
                )?;
                Self::ingest_into_engine(&files, parsed.base_id, ctx).await
            }
            "parse_source" | "analyze" => {
                let parsed: IngestFilesArgs = serde_json::from_value(args)
                    .map_err(|e| format!("Invalid arguments for git_ast.{method}: {e}"))?;
                let base_id = parsed.base_id.unwrap_or(1);
                let (symbols, edges) = Self::analyze_files(&parsed.files, base_id);
                Ok(json!({
                    "symbols": symbols,
                    "edges": edges
                }))
            }
            other => Err(format!("Unknown procedure 'git_ast.{other}'")),
        }
    }
}

// Export C-ABI entrypoints (`hs_pdk_api_version` & `hs_plugin_create`) for dynamic loading from `./plugins/`
hs_export_plugin!(HsGitAstPlugin::new);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_ast_manifest_and_detection() {
        let plugin = HsGitAstPlugin::new();
        let manifest = plugin.manifest();
        assert_eq!(manifest.name, "git_ast");
        assert_eq!(HsGitAstPlugin::detect_language("src/lib.rs"), "rust");
        assert_eq!(HsGitAstPlugin::detect_language("api/main.py"), "python");
        assert_eq!(HsGitAstPlugin::detect_language("web/app.tsx"), "typescript");
        assert_eq!(HsGitAstPlugin::detect_language("pkg/server.go"), "go");
    }

    #[tokio::test]
    async fn test_git_ast_parse_source_procedure() {
        let plugin = HsGitAstPlugin::new();
        let ctx = EngineContext::new("admin", None, None, 1);
        let args = json!({
            "files": [
                {
                    "path": "test.rs",
                    "content": "pub struct Engine {}\nimpl Engine {\n    pub fn run() {}\n}"
                }
            ]
        });
        let res = plugin
            .call_procedure("parse_source", args, &ctx)
            .await
            .unwrap();
        let symbols = res.get("symbols").unwrap().as_array().unwrap();
        assert!(!symbols.is_empty(), "Should extract at least one symbol");
    }
}
