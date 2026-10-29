import { useState, useEffect } from "react"
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query"
import {
    fetchExtensionsCatalog,
    enableExtension,
    disableExtension,
    reloadExtensions,
    callExtensionProcedure,
    fetchCollections,
    type ExtensionManifest,
    type ExtensionProcedureSchema,
} from "@/lib/api"
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import {
    Blocks,
    RefreshCw,
    Power,
    Play,
    Check,
    Copy,
    Shield,
    Sparkles,
    GitBranch,
    FolderOpen,
    Code2,
    Terminal,
    CheckCircle2,
    XCircle,
    Cpu,
    Layers,
    AlertCircle,
} from "lucide-react"
import { cn } from "@/lib/utils"

const SAMPLE_PAYLOADS: Record<string, Record<string, any>> = {
    "embed.chunk_text": {
        text: "# HyperspaceDB v4.2.0\nSpatial & Graph Runtime with pgrx-inspired PDK extensions.\n\n## Multi-Metric Engine\nSupports 12 hardware-accelerated Euclidean, Spherical, Hyperbolic, and Probabilistic metrics.",
        chunk_size: 160,
        chunk_overlap: 24,
    },
    "embed.embed_text": {
        text: "Zero-Python in-engine Markdown chunker and manifold vectorizer",
        dimension: 32,
        metric: "poincare",
    },
    "embed.ingest_text": {
        text: "# Service Level Agreement\n99.99% availability across all DePIN and Enterprise shards.\n\n## Security\nProtected by hs-shield orthogonal projection.",
        chunk_size: 256,
        link_sequential: true,
        link_hierarchy: true,
    },
    "git_ast.parse_source": {
        files: [
            {
                path: "src/engine.rs",
                content:
                    "pub trait StorageEngine {\n    fn flush(&self);\n}\npub struct MmapEngine;\nimpl StorageEngine for MmapEngine {\n    fn flush(&self) {\n        sync_disk();\n    }\n}\npub fn sync_disk() {}\n",
            },
        ],
        base_id: 1,
    },
    "git_ast.ingest_files": {
        files: [
            {
                path: "src/lib.rs",
                content:
                    "pub struct GraphIndex;\npub fn build_kag() {\n    link_nodes();\n}\npub fn link_nodes() {}\n",
            },
        ],
    },
    "git_ast.ingest_directory": {
        path: "./crates/hyperspace-pdk/src",
        max_files: 20,
    },
    "shield.status": {},
    "shield.configure": {
        collection: "confidential_docs",
        enabled: true,
        secret_key: "kms-tenant-master-key-2026",
        key_source: "inline",
    },
    "shield.transform": {
        secret_key: "kms-tenant-master-key-2026",
        metric: "cosine",
        inverse: false,
        vector: [0.25, -0.5, 0.75, 0.125, -0.25, 0.5, -0.125, 0.375],
    },
    "eco.metrics": {
        range: "all",
    },
    "eco.esg_report": {
        range: "30d",
        format: "json",
    },
    "eco.collection_tier": {
        vector_count: 100000,
        full_dimension: 1024,
        mrl_dimension: 1024,
        quantization: "medium_plus",
    },
}

function getDefaultPayload(pluginName: string, proc: ExtensionProcedureSchema): string {
    const key = `${pluginName}.${proc.name}`
    if (SAMPLE_PAYLOADS[key]) {
        return JSON.stringify(SAMPLE_PAYLOADS[key], null, 2)
    }
    const props = proc.input_schema?.properties || {}
    const skeleton: Record<string, any> = {}
    for (const [k, v] of Object.entries<any>(props)) {
        if (v.default !== undefined) {
            skeleton[k] = v.default
        } else if (v.type === "string") {
            skeleton[k] = ""
        } else if (v.type === "integer" || v.type === "number") {
            skeleton[k] = 0
        } else if (v.type === "boolean") {
            skeleton[k] = true
        } else if (v.type === "array") {
            skeleton[k] = []
        } else if (v.type === "object") {
            skeleton[k] = {}
        }
    }
    return JSON.stringify(skeleton, null, 2)
}

function getPluginIcon(name: string) {
    switch (name) {
        case "embed":
            return Sparkles
        case "shield":
            return Shield
        case "git_ast":
            return GitBranch
        default:
            return Blocks
    }
}

function getPluginAccent(name: string) {
    switch (name) {
        case "embed":
            return {
                iconColor: "text-blue-400",
                badgeBg: "bg-blue-500/10 text-blue-400 border-blue-500/20",
            }
        case "shield":
            return {
                iconColor: "text-emerald-400",
                badgeBg: "bg-emerald-500/10 text-emerald-400 border-emerald-500/20",
            }
        case "git_ast":
            return {
                iconColor: "text-violet-400",
                badgeBg: "bg-violet-500/10 text-violet-400 border-violet-500/20",
            }
        default:
            return {
                iconColor: "text-amber-400",
                badgeBg: "bg-amber-500/10 text-amber-400 border-amber-500/20",
            }
    }
}

export function PluginsPage() {
    const queryClient = useQueryClient()
    const [selectedPluginName, setSelectedPluginName] = useState<string>("embed")
    const [selectedProcedureName, setSelectedProcedureName] = useState<string>("")
    const [selectedCollection, setSelectedCollection] = useState<string>("")
    const [payloadJson, setPayloadJson] = useState<string>("{}")
    const [execResult, setExecResult] = useState<any>(null)
    const [execError, setExecError] = useState<string | null>(null)
    const [isExecuting, setIsExecuting] = useState<boolean>(false)
    const [reloadBanner, setReloadBanner] = useState<string | null>(null)
    const [copiedSnippet, setCopiedSnippet] = useState<string | null>(null)

    const { data: catalog, isLoading, isError, refetch } = useQuery({
        queryKey: ["extensions-catalog"],
        queryFn: fetchExtensionsCatalog,
    })

    const { data: collectionsData } = useQuery({
        queryKey: ["collections"],
        queryFn: fetchCollections,
    })

    const collections: string[] = Array.isArray(collectionsData)
        ? collectionsData.map((c: any) => (typeof c === "string" ? c : c.name))
        : collectionsData?.collections?.map((c: any) => (typeof c === "string" ? c : c.name)) || []

    const extensions: ExtensionManifest[] = catalog?.extensions || []
    const selectedPlugin =
        extensions.find((e) => e.name === selectedPluginName) || extensions[0] || null

    useEffect(() => {
        if (selectedPlugin && selectedPlugin.procedures.length > 0) {
            const exists = selectedPlugin.procedures.some((p) => p.name === selectedProcedureName)
            const targetProc = exists
                ? selectedPlugin.procedures.find((p) => p.name === selectedProcedureName)!
                : selectedPlugin.procedures[0]
            setSelectedProcedureName(targetProc.name)
            setPayloadJson(getDefaultPayload(selectedPlugin.name, targetProc))
            setExecResult(null)
            setExecError(null)
        }
    }, [selectedPlugin?.name])

    const handleSelectProcedure = (proc: ExtensionProcedureSchema) => {
        if (!selectedPlugin) return
        setSelectedProcedureName(proc.name)
        setPayloadJson(getDefaultPayload(selectedPlugin.name, proc))
        setExecResult(null)
        setExecError(null)
    }

    const toggleMutation = useMutation({
        mutationFn: async ({ name, enable }: { name: string; enable: boolean }) => {
            return enable ? enableExtension(name) : disableExtension(name)
        },
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ["extensions-catalog"] })
        },
    })

    const reloadMutation = useMutation({
        mutationFn: reloadExtensions,
        onSuccess: (data) => {
            queryClient.invalidateQueries({ queryKey: ["extensions-catalog"] })
            const count = Array.isArray(data?.loaded) ? data.loaded.length : 0
            setReloadBanner(
                `Hot-reloaded plugins directory (${catalog?.plugins_dir || "./plugins"}): ${count} external cdylib plugin(s) scanned.`
            )
            setTimeout(() => setReloadBanner(null), 5000)
        },
    })

    const handleRunProcedure = async () => {
        if (!selectedPlugin || !selectedProcedureName) return
        setExecError(null)
        setExecResult(null)
        let parsedArgs: Record<string, any> = {}
        try {
            parsedArgs = payloadJson.trim() ? JSON.parse(payloadJson) : {}
        } catch (e: any) {
            setExecError(`Invalid JSON payload: ${e.message}`)
            return
        }

        setIsExecuting(true)
        try {
            const res = await callExtensionProcedure(
                selectedPlugin.name,
                selectedProcedureName,
                parsedArgs,
                selectedCollection
            )
            setExecResult(res)
        } catch (err: any) {
            const msg =
                err?.response?.data?.error ||
                err?.response?.data?.message ||
                err?.message ||
                "Procedure call failed"
            setExecError(String(msg))
        } finally {
            setIsExecuting(false)
        }
    }

    const copyText = (text: string, id: string) => {
        navigator.clipboard.writeText(text)
        setCopiedSnippet(id)
        setTimeout(() => setCopiedSnippet(null), 2000)
    }

    const activeCount = extensions.filter((e) => e.enabled).length
    const externalCount = extensions.filter((e) => e.source?.startsWith("external")).length
    const builtinCount = extensions.length - externalCount
    const activeProc = selectedPlugin?.procedures.find((p) => p.name === selectedProcedureName)

    return (
        <div className="space-y-6">
            {/* Header */}
            <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
                <div>
                    <div className="flex items-center gap-2.5">
                        <Blocks className="h-6 w-6 text-primary" />
                        <h1 className="text-2xl font-bold tracking-tight">
                            PDK Extensions & Plugins
                        </h1>
                        <span className="text-xs font-mono px-2 py-0.5 rounded bg-primary/10 text-primary border border-primary/20">
                            v{catalog?.pdk_version || "4.2.0"} (ABI {catalog?.pdk_api_version || 40200})
                        </span>
                    </div>
                    <p className="text-sm text-muted-foreground mt-1">
                        Manage built-in core extensions and dynamically loaded external{" "}
                        <code className="text-xs bg-muted px-1.5 py-0.5 rounded">.dylib / .so</code>{" "}
                        plugins without restarting the server.
                    </p>
                </div>

                <div className="flex items-center gap-2">
                    <Button
                        variant="outline"
                        size="sm"
                        onClick={() => refetch()}
                        disabled={isLoading}
                        className="gap-2"
                    >
                        <RefreshCw className={cn("h-4 w-4", isLoading && "animate-spin")} />
                        Refresh
                    </Button>
                    <Button
                        size="sm"
                        onClick={() => reloadMutation.mutate()}
                        disabled={reloadMutation.isPending}
                        className="gap-2"
                    >
                        <FolderOpen
                            className={cn("h-4 w-4", reloadMutation.isPending && "animate-pulse")}
                        />
                        Hot-Reload ./plugins
                    </Button>
                </div>
            </div>

            {reloadBanner && (
                <div className="flex items-center gap-2 px-4 py-3 rounded-lg bg-emerald-500/10 border border-emerald-500/30 text-emerald-300 text-sm">
                    <CheckCircle2 className="h-4 w-4 shrink-0" />
                    <span>{reloadBanner}</span>
                </div>
            )}

            {/* Summary KPI Cards */}
            <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
                <Card className="bg-card/60 border-border/60">
                    <CardContent className="p-4 flex items-center justify-between">
                        <div>
                            <p className="text-xs uppercase tracking-wider text-muted-foreground">
                                Active Plugins
                            </p>
                            <p className="text-2xl font-bold mt-1">
                                {activeCount}{" "}
                                <span className="text-sm font-normal text-muted-foreground">
                                    / {extensions.length}
                                </span>
                            </p>
                        </div>
                        <Power className="h-8 w-8 text-emerald-400 opacity-80" />
                    </CardContent>
                </Card>

                <Card className="bg-card/60 border-border/60">
                    <CardContent className="p-4 flex items-center justify-between">
                        <div>
                            <p className="text-xs uppercase tracking-wider text-muted-foreground">
                                Built-in Core
                            </p>
                            <p className="text-2xl font-bold mt-1">{builtinCount}</p>
                            <p className="text-[11px] text-muted-foreground mt-0.5">
                                hs-embed, hs-shield
                            </p>
                        </div>
                        <Cpu className="h-8 w-8 text-blue-400 opacity-80" />
                    </CardContent>
                </Card>

                <Card className="bg-card/60 border-border/60">
                    <CardContent className="p-4 flex items-center justify-between">
                        <div>
                            <p className="text-xs uppercase tracking-wider text-muted-foreground">
                                External (cdylib)
                            </p>
                            <p className="text-2xl font-bold mt-1">{externalCount}</p>
                            <p className="text-[11px] text-muted-foreground mt-0.5 font-mono truncate max-w-[180px]">
                                dir: {catalog?.plugins_dir || "./plugins"}
                            </p>
                        </div>
                        <FolderOpen className="h-8 w-8 text-violet-400 opacity-80" />
                    </CardContent>
                </Card>

                <Card className="bg-card/60 border-border/60">
                    <CardContent className="p-4 flex items-center justify-between">
                        <div>
                            <p className="text-xs uppercase tracking-wider text-muted-foreground">
                                Native SIMD Metrics
                            </p>
                            <p className="text-2xl font-bold mt-1">
                                {catalog?.supported_metrics?.length || 12}
                            </p>
                            <p className="text-[11px] text-muted-foreground mt-0.5">
                                Euclidean, Hyperbolic, JSD...
                            </p>
                        </div>
                        <Layers className="h-8 w-8 text-amber-400 opacity-80" />
                    </CardContent>
                </Card>
            </div>

            {isError && (
                <Card className="border-red-500/30 bg-red-500/5">
                    <CardContent className="p-4 flex items-center gap-3 text-red-400 text-sm">
                        <AlertCircle className="h-5 w-5 shrink-0" />
                        <span>
                            Failed to load extensions catalog from{" "}
                            <code className="font-mono">/api/extensions</code>. Verify the server is
                            running v4.2.0+.
                        </span>
                    </CardContent>
                </Card>
            )}

            {/* Main Grid: Left = Installed Plugins List, Right = Inspector & Live Runner */}
            <div className="grid grid-cols-1 lg:grid-cols-12 gap-6">
                {/* Left Column: Installed Plugins */}
                <div className="lg:col-span-5 space-y-4">
                    <h2 className="text-sm font-semibold uppercase tracking-wider text-muted-foreground px-1">
                        Installed Extensions ({extensions.length})
                    </h2>

                    {extensions.map((ext) => {
                        const Icon = getPluginIcon(ext.name)
                        const accent = getPluginAccent(ext.name)
                        const isSelected = selectedPlugin?.name === ext.name
                        const isExternal = ext.source?.startsWith("external")

                        return (
                            <Card
                                key={ext.name}
                                onClick={() => setSelectedPluginName(ext.name)}
                                className={cn(
                                    "cursor-pointer transition-all border",
                                    isSelected
                                        ? "border-primary bg-accent/30 shadow-md"
                                        : "border-border/60 bg-card/50 hover:border-border"
                                )}
                            >
                                <CardContent className="p-5 space-y-3">
                                    <div className="flex items-start justify-between gap-3">
                                        <div className="flex items-center gap-3">
                                            <div
                                                className={cn(
                                                    "p-2.5 rounded-lg border",
                                                    accent.badgeBg
                                                )}
                                            >
                                                <Icon className={cn("h-5 w-5", accent.iconColor)} />
                                            </div>
                                            <div>
                                                <div className="flex items-center gap-2 flex-wrap">
                                                    <span className="font-bold text-base">
                                                        hs-{ext.name.replace("_", "-")}
                                                    </span>
                                                    <span className="text-xs font-mono text-muted-foreground">
                                                        ({ext.name})
                                                    </span>
                                                    <span className="text-[10px] font-mono px-1.5 py-0.5 rounded bg-muted text-muted-foreground">
                                                        v{ext.version}
                                                    </span>
                                                </div>
                                                <div className="flex items-center gap-2 mt-1">
                                                    <span
                                                        className={cn(
                                                            "text-[10px] font-semibold uppercase px-2 py-0.5 rounded border",
                                                            isExternal
                                                                ? "bg-violet-500/10 text-violet-300 border-violet-500/30"
                                                                : "bg-blue-500/10 text-blue-300 border-blue-500/30"
                                                        )}
                                                    >
                                                        {isExternal
                                                            ? "External cdylib"
                                                            : "Built-in Core"}
                                                    </span>
                                                    <span
                                                        className={cn(
                                                            "text-[10px] font-semibold uppercase px-2 py-0.5 rounded border flex items-center gap-1",
                                                            ext.enabled
                                                                ? "bg-emerald-500/10 text-emerald-400 border-emerald-500/30"
                                                                : "bg-zinc-500/10 text-zinc-400 border-zinc-500/30"
                                                        )}
                                                    >
                                                        {ext.enabled ? (
                                                            <>
                                                                <CheckCircle2 className="h-3 w-3" />
                                                                Active
                                                            </>
                                                        ) : (
                                                            <>
                                                                <XCircle className="h-3 w-3" />
                                                                Disabled
                                                            </>
                                                        )}
                                                    </span>
                                                </div>
                                            </div>
                                        </div>

                                        <Button
                                            size="sm"
                                            variant={ext.enabled ? "outline" : "default"}
                                            disabled={toggleMutation.isPending}
                                            onClick={(e) => {
                                                e.stopPropagation()
                                                toggleMutation.mutate({
                                                    name: ext.name,
                                                    enable: !ext.enabled,
                                                })
                                            }}
                                            className={cn(
                                                "shrink-0 text-xs h-8",
                                                ext.enabled
                                                    ? "border-red-500/30 text-red-400 hover:bg-red-500/10"
                                                    : "bg-emerald-600 hover:bg-emerald-500 text-white"
                                            )}
                                        >
                                            <Power className="h-3.5 w-3.5 mr-1.5" />
                                            {ext.enabled ? "Disable" : "Enable"}
                                        </Button>
                                    </div>

                                    <p className="text-xs text-muted-foreground leading-relaxed">
                                        {ext.description}
                                    </p>

                                    <div className="flex flex-wrap items-center gap-1.5 pt-1">
                                        {ext.hooks.map((h) => (
                                            <span
                                                key={h}
                                                className="text-[10px] font-mono px-2 py-0.5 rounded bg-amber-500/10 text-amber-300 border border-amber-500/20"
                                            >
                                                hook:{h}
                                            </span>
                                        ))}
                                        {ext.procedures.map((p) => (
                                            <span
                                                key={p.name}
                                                className="text-[10px] font-mono px-2 py-0.5 rounded bg-zinc-800 text-zinc-300 border border-white/5"
                                            >
                                                fn:{p.name}()
                                            </span>
                                        ))}
                                    </div>

                                    {isExternal && (
                                        <p className="text-[11px] font-mono text-muted-foreground/80 truncate pt-1 border-t border-border/40">
                                            {ext.source}
                                        </p>
                                    )}
                                </CardContent>
                            </Card>
                        )
                    })}
                </div>

                {/* Right Column: Selected Plugin Workbench & Developer Guide */}
                <div className="lg:col-span-7">
                    {selectedPlugin ? (
                        <Card className="border-border/60 bg-card/60">
                            <CardHeader className="border-b border-border/50 pb-4">
                                <div className="flex items-center justify-between">
                                    <div>
                                        <CardTitle className="text-lg flex items-center gap-2">
                                            <span>
                                                hs-{selectedPlugin.name.replace("_", "-")} Workbench
                                            </span>
                                            <span className="text-xs font-mono text-muted-foreground">
                                                client.ext.{selectedPlugin.name}.*
                                            </span>
                                        </CardTitle>
                                        <CardDescription className="mt-1">
                                            {selectedPlugin.description}
                                        </CardDescription>
                                    </div>
                                </div>
                            </CardHeader>

                            <CardContent className="p-6">
                                <Tabs defaultValue="playground" className="space-y-4">
                                    <TabsList className="grid grid-cols-3 w-full">
                                        <TabsTrigger value="playground" className="gap-1.5 text-xs">
                                            <Play className="h-3.5 w-3.5" />
                                            Procedure Playground
                                        </TabsTrigger>
                                        <TabsTrigger value="sdk" className="gap-1.5 text-xs">
                                            <Terminal className="h-3.5 w-3.5" />
                                            SDK & REST Usage
                                        </TabsTrigger>
                                        <TabsTrigger value="authoring" className="gap-1.5 text-xs">
                                            <Code2 className="h-3.5 w-3.5" />
                                            Build External Plugin
                                        </TabsTrigger>
                                    </TabsList>

                                    {/* TAB 1: Interactive Playground */}
                                    <TabsContent value="playground" className="space-y-4">
                                        {!selectedPlugin.enabled && (
                                            <div className="p-3 rounded-lg bg-amber-500/10 border border-amber-500/30 text-amber-300 text-xs flex items-center justify-between">
                                                <span>
                                                    Plugin <b>{selectedPlugin.name}</b> is currently
                                                    disabled. Enable it to execute procedures.
                                                </span>
                                                <Button
                                                    size="sm"
                                                    variant="outline"
                                                    className="h-7 text-xs border-amber-500/40 text-amber-300"
                                                    onClick={() =>
                                                        toggleMutation.mutate({
                                                            name: selectedPlugin.name,
                                                            enable: true,
                                                        })
                                                    }
                                                >
                                                    Enable Now
                                                </Button>
                                            </div>
                                        )}

                                        {/* Procedure Selector Pills */}
                                        <div className="space-y-1.5">
                                            <label className="text-xs font-medium text-muted-foreground">
                                                Select Procedure
                                            </label>
                                            <div className="flex flex-wrap gap-2">
                                                {selectedPlugin.procedures.map((proc) => (
                                                    <Button
                                                        key={proc.name}
                                                        type="button"
                                                        size="sm"
                                                        variant={
                                                            selectedProcedureName === proc.name
                                                                ? "default"
                                                                : "outline"
                                                        }
                                                        onClick={() => handleSelectProcedure(proc)}
                                                        className="font-mono text-xs h-8"
                                                    >
                                                        {proc.name}()
                                                        {proc.requires_collection && (
                                                            <span className="ml-1.5 text-[10px] opacity-75">
                                                                [collection]
                                                            </span>
                                                        )}
                                                    </Button>
                                                ))}
                                            </div>
                                        </div>

                                        {activeProc && (
                                            <div className="p-3 rounded-md bg-muted/40 border border-border/50 text-xs text-muted-foreground">
                                                <span className="font-semibold text-foreground">
                                                    {selectedPlugin.name}.{activeProc.name}:
                                                </span>{" "}
                                                {activeProc.description}
                                            </div>
                                        )}

                                        {/* Target Collection Input/Selector */}
                                        <div className="space-y-1.5">
                                            <label className="text-xs font-medium text-muted-foreground flex items-center justify-between">
                                                <span>
                                                    Target Collection{" "}
                                                    {activeProc?.requires_collection ? (
                                                        <span className="text-amber-400">
                                                            (Required by this procedure)
                                                        </span>
                                                    ) : (
                                                        <span className="opacity-60">
                                                            (Optional)
                                                        </span>
                                                    )}
                                                </span>
                                            </label>
                                            <div className="flex gap-2">
                                                <input
                                                    type="text"
                                                    value={selectedCollection}
                                                    onChange={(e) =>
                                                        setSelectedCollection(e.target.value)
                                                    }
                                                    placeholder="e.g. docs, code_kag, confidential_docs"
                                                    className="flex-1 h-9 rounded-md border border-input bg-background px-3 py-1 text-xs font-mono"
                                                />
                                                {collections.length > 0 && (
                                                    <select
                                                        value={selectedCollection}
                                                        onChange={(e) =>
                                                            setSelectedCollection(e.target.value)
                                                        }
                                                        className="h-9 rounded-md border border-input bg-background px-2 text-xs font-mono"
                                                    >
                                                        <option value="">-- Choose existing --</option>
                                                        {collections.map((c) => (
                                                            <option key={c} value={c}>
                                                                {c}
                                                            </option>
                                                        ))}
                                                    </select>
                                                )}
                                            </div>
                                        </div>

                                        {/* JSON Arguments Editor */}
                                        <div className="space-y-1.5">
                                            <label className="text-xs font-medium text-muted-foreground">
                                                JSON Arguments Payload
                                            </label>
                                            <textarea
                                                rows={8}
                                                value={payloadJson}
                                                onChange={(e) => setPayloadJson(e.target.value)}
                                                className="w-full rounded-md border border-input bg-[#0d1117] text-[#c9d1d9] p-3 text-xs font-mono focus:outline-none focus:ring-1 focus:ring-primary"
                                            />
                                        </div>

                                        <div className="flex items-center justify-between">
                                            <span className="text-[11px] font-mono text-muted-foreground">
                                                POST{" "}
                                                {selectedCollection
                                                    ? `/api/collections/${selectedCollection}/ext/${selectedPlugin.name}/${selectedProcedureName}`
                                                    : `/api/ext/${selectedPlugin.name}/${selectedProcedureName}`}
                                            </span>
                                            <Button
                                                onClick={handleRunProcedure}
                                                disabled={isExecuting || !selectedPlugin.enabled}
                                                className="gap-2"
                                            >
                                                <Play className="h-4 w-4" />
                                                {isExecuting
                                                    ? "Executing..."
                                                    : `Run ${selectedPlugin.name}.${selectedProcedureName}`}
                                            </Button>
                                        </div>

                                        {execError && (
                                            <div className="p-3 rounded-lg bg-red-500/10 border border-red-500/30 text-red-300 text-xs font-mono whitespace-pre-wrap">
                                                {execError}
                                            </div>
                                        )}

                                        {execResult && (
                                            <div className="space-y-1.5">
                                                <label className="text-xs font-medium text-emerald-400">
                                                    Execution Output (JSON)
                                                </label>
                                                <pre className="p-3 rounded-lg bg-[#0d1117] text-emerald-300 border border-emerald-500/20 text-xs font-mono overflow-x-auto max-h-72">
                                                    {JSON.stringify(execResult, null, 2)}
                                                </pre>
                                            </div>
                                        )}
                                    </TabsContent>

                                    {/* TAB 2: SDK & REST Usage */}
                                    <TabsContent value="sdk" className="space-y-4">
                                        {[
                                            {
                                                id: "python",
                                                title: "Python SDK (Adaptive Extension Proxy)",
                                                code: `from hyperspace import HyperspaceClient

client = HyperspaceClient("localhost:50051")

# Call ${selectedPlugin.name}.${selectedProcedureName || "procedure"} dynamically:
res = client.ext.${selectedPlugin.name}.${selectedProcedureName || "call"}(
${
    selectedCollection
        ? `    collection="${selectedCollection}",\n`
        : ""
}    **${payloadJson.replace(/\n/g, "\n    ")}
)
print(res)`,
                                            },
                                            {
                                                id: "ts",
                                                title: "TypeScript / Node.js SDK",
                                                code: `import { HyperspaceClient } from 'hyperspace-sdk-ts';

const client = new HyperspaceClient('localhost:50051');
const res = await client.ext.${selectedPlugin.name}.${selectedProcedureName || "call"}(${payloadJson});
console.log(res);`,
                                            },
                                            {
                                                id: "curl",
                                                title: "HTTP REST API (Zero-Downtime Control)",
                                                code: `# 1. Call procedure
curl -X POST http://localhost:8080/api/ext/${selectedPlugin.name}/${selectedProcedureName || "status"} \\
  -H "x-api-key: I_LOVE_HYPERSPACEDB" \\
  -H "Content-Type: application/json" \\
  -d '${payloadJson.replace(/\s+/g, " ")}'

# 2. Enable / Disable plugin at runtime without server restart
curl -X POST http://localhost:8080/v1/extensions/${selectedPlugin.name}/enable -H "x-api-key: I_LOVE_HYPERSPACEDB"
curl -X POST http://localhost:8080/v1/extensions/${selectedPlugin.name}/disable -H "x-api-key: I_LOVE_HYPERSPACEDB"

# 3. Hot-reload external .dylib/.so plugins from ./plugins
curl -X POST http://localhost:8080/v1/extensions/reload -H "x-api-key: I_LOVE_HYPERSPACEDB"`,
                                            },
                                        ].map((snip) => (
                                            <div key={snip.id} className="relative">
                                                <p className="text-xs font-medium text-muted-foreground mb-1.5">
                                                    {snip.title}
                                                </p>
                                                <pre className="p-3 rounded-lg bg-[#0d1117] text-[#c9d1d9] border border-white/10 text-xs font-mono overflow-x-auto">
                                                    {snip.code}
                                                </pre>
                                                <Button
                                                    size="icon"
                                                    variant="ghost"
                                                    className="absolute top-7 right-2 h-7 w-7 bg-white/5 hover:bg-white/10"
                                                    onClick={() => copyText(snip.code, snip.id)}
                                                >
                                                    {copiedSnippet === snip.id ? (
                                                        <Check className="h-3.5 w-3.5 text-emerald-400" />
                                                    ) : (
                                                        <Copy className="h-3.5 w-3.5 text-zinc-400" />
                                                    )}
                                                </Button>
                                            </div>
                                        ))}
                                    </TabsContent>

                                    {/* TAB 3: Build External Plugin Guide */}
                                    <TabsContent value="authoring" className="space-y-4 text-xs">
                                        <div className="p-3 rounded-lg bg-violet-500/10 border border-violet-500/20 text-violet-200 space-y-1">
                                            <p className="font-semibold">
                                                How External Plugins Work (like{" "}
                                                <code>plugins/hs-git-ast</code>)
                                            </p>
                                            <p className="text-violet-300/90">
                                                External plugins are standard Rust crates compiled as{" "}
                                                <code>cdylib</code> (<code>.dylib</code> on macOS,{" "}
                                                <code>.so</code> on Linux, <code>.dll</code> on
                                                Windows). Drop the compiled library into{" "}
                                                <code>{catalog?.plugins_dir || "./plugins"}</code>{" "}
                                                and click <b>Hot-Reload ./plugins</b> above.
                                            </p>
                                        </div>

                                        <div className="space-y-1.5">
                                            <p className="font-medium text-muted-foreground">
                                                1. <code>plugins/hs-my-plugin/Cargo.toml</code>
                                            </p>
                                            <pre className="p-3 rounded-lg bg-[#0d1117] text-[#c9d1d9] border border-white/10 font-mono overflow-x-auto">
{`[package]
name = "hs-my-plugin"
version = "0.1.0"
edition = "2021"

[lib]
name = "hs_my_plugin"
crate-type = ["cdylib", "rlib"]

[dependencies]
hyperspace-pdk = { path = "../../crates/hyperspace-pdk" }
async-trait = "0.1"
serde_json = "1.0"`}
                                            </pre>
                                        </div>

                                        <div className="space-y-1.5">
                                            <p className="font-medium text-muted-foreground">
                                                2. <code>plugins/hs-my-plugin/src/lib.rs</code>
                                            </p>
                                            <pre className="p-3 rounded-lg bg-[#0d1117] text-[#c9d1d9] border border-white/10 font-mono overflow-x-auto">
{`use hyperspace_pdk::prelude::*;

#[derive(Default)]
pub struct MyPlugin;

impl MyPlugin {
    pub fn new() -> Self { Self }
}

#[async_trait]
impl HsPlugin for MyPlugin {
    fn manifest(&self) -> PluginManifest {
        hs_plugin_manifest!(
            name: "my_plugin",
            version: "0.1.0",
            author: "Your Team",
            description: "Custom external cdylib plugin",
            hooks: [HookStage::BeforeInsert],
            procedures: []
        )
    }

    async fn call_procedure(&self, method: &str, args: Value, _ctx: &EngineContext) -> Result<Value, String> {
        Ok(json!({ "method": method, "echo": args }))
    }
}

// Export C-ABI entrypoints (hs_pdk_api_version & hs_plugin_create)
hs_export_plugin!(MyPlugin::new);`}
                                            </pre>
                                        </div>

                                        <div className="space-y-1.5">
                                            <p className="font-medium text-muted-foreground">
                                                3. Build & Hot-Load into HyperspaceDB
                                            </p>
                                            <pre className="p-3 rounded-lg bg-[#0d1117] text-[#c9d1d9] border border-white/10 font-mono overflow-x-auto">
{`cargo build -p hs-my-plugin --release
cp target/release/libhs_my_plugin.dylib ./plugins/
# Then click "Hot-Reload ./plugins" in the Dashboard or call POST /v1/extensions/reload`}
                                            </pre>
                                        </div>
                                    </TabsContent>
                                </Tabs>
                            </CardContent>
                        </Card>
                    ) : null}
                </div>
            </div>
        </div>
    )
}
