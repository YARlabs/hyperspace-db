import { useQuery } from "@tanstack/react-query"
import { fetchStatus, getCacheStats, fetchMetrics, fetchCollections, triggerVacuum, fetchEcoMetrics, fetchEsgReportBlob } from "@/lib/api"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card"
import { Database, HardDrive, Server, Zap, FolderOpen, Layers, TrendingUp, Leaf } from "lucide-react"
import { Skeleton } from "@/components/ui/skeleton"
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"
import { useState } from "react"
import { Badge } from "@/components/ui/badge"
import { cn } from "@/lib/utils"

export function OverviewPage() {
    const { data: status, isLoading: sLoading } = useQuery({
        queryKey: ['status'],
        queryFn: fetchStatus,
        refetchInterval: 60000
    })
    const { data: metrics } = useQuery({
        queryKey: ['metrics'],
        queryFn: fetchMetrics,
        refetchInterval: 30000
    })
    const { data: collections } = useQuery({
        queryKey: ['collections'],
        queryFn: fetchCollections,
        refetchInterval: 60000,
        refetchOnWindowFocus: false
    })

    if (sLoading && !status) return <OverviewSkeleton />

    const formatDiskSize = (mb: number) => {
        if (mb >= 1024) {
            return `${(mb / 1024).toFixed(2)} GB`
        }
        return `${mb} MB`
    }

    const collectionNames: string[] = Array.isArray(collections)
        ? collections.map((c: any) => typeof c === "string" ? c : c.name)
        : []

    return (
        <div className="space-y-6 fade-in">
            <div className="flex items-center justify-between">
                <h1 className="text-3xl font-bold tracking-tight">System Overview</h1>
                <div className="flex items-center gap-2 text-sm text-muted-foreground">
                    <div className={`h-2 w-2 rounded-full ${status?.status === 'ONLINE' ? 'bg-green-500 shadow-[0_0_8px_#22c55e]' : 'bg-red-500'}`}></div>
                    {status?.status || "Connecting..."}
                </div>
            </div>

            <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-5">
                <StatCard title="Total Vectors" value={metrics?.total_vectors?.toLocaleString() || "0"} icon={Database} desc="Across all collections" />
                <StatCard
                    title="RAM Usage"
                    value={`${metrics?.ram_usage_mb || 0} MB`}
                    icon={HardDrive}
                    desc={status?.config?.max_ram_gb && status.config.max_ram_gb !== "0"
                        ? `${((metrics?.ram_usage_mb / (parseInt(status.config.max_ram_gb) * 1024)) * 100).toFixed(1)}% of ceiling`
                        : "Resident Set Size"}
                />
                <StatCard title="Disk Usage" value={formatDiskSize(metrics?.disk_usage_mb || 0)} icon={FolderOpen} desc="Data directory size" />
                <StatCard title="Collections" value={metrics?.total_collections || 0} icon={Server} desc="Active indices" />
                <StatCard title="CPU Load" value={`${metrics?.cpu_usage_percent || 0}%`} icon={Zap} desc="System Load (Est.)" />
            </div>

            {/* Hot Cache Summary — spans full width */}
            {collectionNames.length > 0 && (
                <HotCacheSummary collectionNames={collectionNames} />
            )}

            <div className="grid gap-4 md:grid-cols-2">

                <EsgCarbonCard />

                <Card>
                    <CardHeader><CardTitle>Configuration</CardTitle><CardDescription>Runtime parameters</CardDescription></CardHeader>
                    <CardContent>
                        <div className="space-y-4">
                            <ConfigRow label="Version" value={status?.version} />
                            <ConfigRow label="Optimization Mode" value={status?.config?.mode} />
                            <ConfigRow label="RAM Ceiling" value={status?.config?.max_ram_gb && status.config.max_ram_gb !== "0" ? `${status.config.max_ram_gb} GB` : "Unlimited"} />
                            {/* <ConfigRow label="Global Dimension" value={status?.config?.dimension} />
                            <ConfigRow label="Metric Space" value={
                                status?.config?.metric === 'cosine' ? 'Cosine Similarity' :
                                    status?.config?.metric === 'l2' || status?.config?.metric === 'euclidean' ? 'Euclidean (L2)' :
                                        status?.config?.metric === 'poincare' ? 'Hyperbolic (Poincaré)' :
                                            status?.config?.metric === 'lorentz' ? 'Lorentz (Hyperbolic)' :
                                                status?.config?.metric === 'hybrid' ? 'Hybrid (Lorentz + L2)' :
                                                    status?.config?.metric || 'Unknown'
                            } /> */}
                            <ConfigRow label="Quantization" value={status?.config?.quantization || "Scalar I8"} />
                            <ConfigRow label="Embedding Engine" value={status?.embedding?.enabled ? "Multi-Model Active" : "Disabled"} />
                            {status?.embedding?.enabled && status?.embedding?.models && (
                                <div className="mt-2 pl-4 border-l-2 border-primary/20 space-y-2">
                                    {Object.entries(status.embedding.models).map(([metric, info]: [string, any]) => (
                                        <div key={metric} className="flex flex-col">
                                            <span className="text-[10px] uppercase tracking-wider text-muted-foreground font-bold">
                                                {metric === 'l2' ? 'Euclidean (L2)' :
                                                    metric === 'cosine' ? 'Cosine' :
                                                        metric === 'poincare' ? 'Poincaré' :
                                                            metric === 'lorentz' ? 'Lorentz' :
                                                                metric === 'hybrid' ? 'Hybrid' :
                                                                    metric.charAt(0).toUpperCase() + metric.slice(1)}
                                            </span>
                                            <div className="flex items-center justify-between">
                                                <span className="text-xs">{info.enabled ? info.model : <span className="text-muted-foreground/50 italic">Disabled</span>}</span>
                                                <span className="text-[10px] font-mono bg-muted px-1 rounded">{info.enabled ? info.provider : "off"}</span>
                                            </div>
                                        </div>
                                    ))}
                                </div>
                            )}
                            <ConfigRow label="Uptime" value={status?.uptime} />
                        </div>
                    </CardContent>
                </Card>


                <IngestionStatusCard metrics={metrics} />

                <Card>
                    <CardHeader>
                        <CardTitle>Maintenance</CardTitle>
                        <CardDescription>System-level operations</CardDescription>
                    </CardHeader>
                    <CardContent>
                        <div className="space-y-4">
                            <div className="flex items-center justify-between">
                                <span className="text-sm font-medium">Memory Management</span>
                                <Button variant="outline" size="sm" onClick={() => {
                                    if (window.confirm("Trigger manual memory vacuum? This may cause temporary latency.")) {
                                        triggerVacuum()
                                            .then(() => alert("Memory cleanup triggered!"))
                                            .catch(e => alert("Failed: " + e.message))
                                    }
                                }}>
                                    Reset Memory
                                </Button>
                            </div>
                        </div>
                    </CardContent>
                </Card>


            </div>
        </div>
    )
}

// ─── Hot Cache Summary ─────────────────────────────────────────────────────────

function HotCacheSummary({ collectionNames }: { collectionNames: string[] }) {
    // Fetch stats for all collections in parallel
    const queries = collectionNames.map(name =>
        // eslint-disable-next-line react-hooks/rules-of-hooks
        useQuery({
            queryKey: ['cache-stats', name],
            queryFn: () => getCacheStats(name),
            refetchInterval: 10000,
            retry: false,
        })
    )

    const allStats = queries.map(q => q.data).filter(Boolean)

    if (allStats.length === 0) {
        // Cache might be disabled or all collections cold
        return (
            <Card className="border-dashed border-zinc-700/50 bg-zinc-950/40">
                <CardContent className="py-4 text-center text-xs text-zinc-500">
                    Hot Cache — no data yet (collections may still be warming up)
                </CardContent>
            </Card>
        )
    }

    // Aggregate across all collections
    const totalL1 = allStats.reduce((s, d) => s + (d.l1_size ?? 0), 0)
    const totalL2 = allStats.reduce((s, d) => s + (d.l2_index_size ?? 0), 0)
    const totalMem = allStats.reduce((s, d) => s + (d.estimated_memory_bytes ?? 0), 0)
    const totalTombstone = allStats.reduce((s, d) => s + (d.tombstone_count ?? 0), 0)
    const totalPending = allStats.reduce((s, d) => s + (d.pending_rebuild ?? 0), 0)

    const avgL1HitRate = allStats.length
        ? allStats.reduce((s, d) => s + (d.l1_hit_rate ?? 0), 0) / allStats.length
        : 0
    const avgL2HitRate = allStats.length
        ? allStats.reduce((s, d) => s + (d.l2_hit_rate ?? 0), 0) / allStats.length
        : 0

    const formatBytes = (b: number) => {
        if (b >= 1024 ** 3) return `${(b / 1024 ** 3).toFixed(1)} GB`
        if (b >= 1024 ** 2) return `${(b / 1024 ** 2).toFixed(0)} MB`
        if (b >= 1024) return `${(b / 1024).toFixed(0)} KB`
        return `${b} B`
    }

    const hitRateColor = (r: number) =>
        r >= 0.8 ? "text-emerald-400" : r >= 0.5 ? "text-amber-400" : "text-red-400"

    return (
        <Card className="border-blue-500/20 bg-gradient-to-br from-blue-950/30 via-zinc-950 to-zinc-950">
            <CardHeader className="pb-3">
                <div className="flex items-center gap-2">
                    <div className="p-1.5 rounded-md bg-blue-500/15 text-blue-400">
                        <Layers className="h-4 w-4" />
                    </div>
                    <div>
                        <CardTitle className="text-base">L0 Hot Tier Cache</CardTitle>
                        <CardDescription className="text-xs">
                            Aggregated across {allStats.length} collection{allStats.length !== 1 ? "s" : ""}
                        </CardDescription>
                    </div>
                    <div className="ml-auto flex items-center gap-1.5 text-[10px] text-emerald-400 font-medium">
                        <div className="h-1.5 w-1.5 rounded-full bg-emerald-400 animate-pulse" />
                        ACTIVE
                    </div>
                </div>
            </CardHeader>
            <CardContent>
                <div className="grid grid-cols-2 gap-3 sm:grid-cols-4 lg:grid-cols-6">

                    {/* L1 Hit Rate */}
                    <div className="rounded-lg bg-zinc-900/60 border border-white/5 p-3 space-y-1">
                        <div className="flex items-center gap-1.5 text-zinc-400">
                            <TrendingUp className="h-3 w-3" />
                            <span className="text-[10px] uppercase font-bold tracking-wider">L1 Hit Rate</span>
                        </div>
                        <div className={`text-2xl font-mono font-bold ${hitRateColor(avgL1HitRate)}`}>
                            {(avgL1HitRate * 100).toFixed(1)}%
                        </div>
                        <div className="text-[10px] text-zinc-500">Exact ID lookup</div>
                        {/* Mini bar */}
                        <div className="h-1 w-full rounded-full bg-zinc-800 overflow-hidden">
                            <div
                                className={`h-full rounded-full transition-all ${avgL1HitRate >= 0.8 ? "bg-emerald-500" : avgL1HitRate >= 0.5 ? "bg-amber-500" : "bg-red-500"}`}
                                style={{ width: `${Math.min(avgL1HitRate * 100, 100)}%` }}
                            />
                        </div>
                    </div>

                    {/* L2 Hit Rate */}
                    <div className="rounded-lg bg-zinc-900/60 border border-white/5 p-3 space-y-1">
                        <div className="flex items-center gap-1.5 text-zinc-400">
                            <TrendingUp className="h-3 w-3" />
                            <span className="text-[10px] uppercase font-bold tracking-wider">L2 Hit Rate</span>
                        </div>
                        <div className={`text-2xl font-mono font-bold ${hitRateColor(avgL2HitRate)}`}>
                            {(avgL2HitRate * 100).toFixed(1)}%
                        </div>
                        <div className="text-[10px] text-zinc-500">ANN in-memory</div>
                        <div className="h-1 w-full rounded-full bg-zinc-800 overflow-hidden">
                            <div
                                className={`h-full rounded-full transition-all ${avgL2HitRate >= 0.8 ? "bg-emerald-500" : avgL2HitRate >= 0.5 ? "bg-amber-500" : "bg-red-500"}`}
                                style={{ width: `${Math.min(avgL2HitRate * 100, 100)}%` }}
                            />
                        </div>
                    </div>

                    {/* L1 Vectors */}
                    <div className="rounded-lg bg-zinc-900/60 border border-white/5 p-3 space-y-1">
                        <div className="text-[10px] uppercase font-bold tracking-wider text-zinc-400">L1 Vectors</div>
                        <div className="text-2xl font-mono font-bold text-blue-400">{totalL1.toLocaleString()}</div>
                        <div className="text-[10px] text-zinc-500">L2 graph: {totalL2.toLocaleString()}</div>
                    </div>

                    {/* Memory */}
                    <div className="rounded-lg bg-zinc-900/60 border border-white/5 p-3 space-y-1">
                        <div className="flex items-center gap-1.5 text-zinc-400">
                            <HardDrive className="h-3 w-3" />
                            <span className="text-[10px] uppercase font-bold tracking-wider">Cache RAM</span>
                        </div>
                        <div className="text-2xl font-mono font-bold text-purple-400">{formatBytes(totalMem)}</div>
                        <div className="text-[10px] text-zinc-500">Est. hot tier</div>
                    </div>

                    {/* Tombstones */}
                    <div className="rounded-lg bg-zinc-900/60 border border-white/5 p-3 space-y-1">
                        <div className="text-[10px] uppercase font-bold tracking-wider text-zinc-400">Tombstones</div>
                        <div className={`text-2xl font-mono font-bold ${totalTombstone > 0 ? "text-amber-400" : "text-zinc-500"}`}>
                            {totalTombstone.toLocaleString()}
                        </div>
                        <div className="text-[10px] text-zinc-500">Pending cleanup</div>
                    </div>

                    {/* Pending Rebuild */}
                    <div className="rounded-lg bg-zinc-900/60 border border-white/5 p-3 space-y-1">
                        <div className="text-[10px] uppercase font-bold tracking-wider text-zinc-400">Pending Rebuild</div>
                        <div className={`text-2xl font-mono font-bold ${totalPending > 0 ? "text-sky-400" : "text-zinc-500"}`}>
                            {totalPending.toLocaleString()}
                        </div>
                        <div className="text-[10px] text-zinc-500">L2 queue</div>
                    </div>
                </div>
            </CardContent>
        </Card>
    )
}

function ConfigRow({ label, value }: any) {
    return (
        <div className="flex items-center justify-between py-1 border-b border-border/40 last:border-0">
            <span className="text-sm font-medium text-muted-foreground">{label}</span>
            <span className="font-mono text-sm">{value || "-"}</span>
        </div>
    )
}

function IngestionStatusCard({ metrics }: any) {
    const [refreshInterval, setRefreshInterval] = useState("30")

    const { data: liveMetrics } = useQuery({
        queryKey: ['live-metrics'],
        queryFn: fetchMetrics,
        refetchInterval: parseInt(refreshInterval) * 1000
    })

    const currentMetrics = liveMetrics || metrics

    return (
        <Card>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                <div>
                    <CardTitle>Ingestion Status</CardTitle>
                    <CardDescription>Auto-refresh monitoring</CardDescription>
                </div>
                <Select value={refreshInterval} onValueChange={setRefreshInterval}>
                    <SelectTrigger className="w-[110px]">
                        <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                        <SelectItem value="5">5 sec</SelectItem>
                        <SelectItem value="10">10 sec</SelectItem>
                        <SelectItem value="30">30 sec</SelectItem>
                        <SelectItem value="60">60 sec</SelectItem>
                    </SelectContent>
                </Select>
            </CardHeader>
            <CardContent>
                <div className="space-y-4">
                    <div className="flex items-center justify-between py-2 border-b">
                        <span className="text-sm text-muted-foreground">Total Vectors</span>
                        <span className="font-mono font-bold text-lg">{currentMetrics?.total_vectors?.toLocaleString() || "0"}</span>
                    </div>
                    <div className="flex items-center justify-between py-2 border-b">
                        <span className="text-sm text-muted-foreground">Active Collections</span>
                        <span className="font-mono font-bold text-lg">{currentMetrics?.total_collections || 0}</span>
                    </div>
                    <div className="flex items-center justify-between py-2 border-b">
                        <span className="text-sm text-muted-foreground">RAM Usage</span>
                        <span className="font-mono font-bold text-lg">{currentMetrics?.ram_usage_mb || 0} MB</span>
                    </div>
                    <div className="flex items-center justify-between py-2">
                        <span className="text-sm text-muted-foreground">Disk Usage</span>
                        <span className="font-mono font-bold text-lg">
                            {currentMetrics?.disk_usage_mb >= 1024
                                ? `${(currentMetrics.disk_usage_mb / 1024).toFixed(2)} GB`
                                : `${currentMetrics?.disk_usage_mb || 0} MB`}
                        </span>
                    </div>
                </div>
            </CardContent>
        </Card>
    )
}

function StatCard({ title, value, icon: Icon, desc }: any) {
    return (
        <Card>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                <CardTitle className="text-sm font-medium">{title}</CardTitle>
                <Icon className="h-4 w-4 text-muted-foreground" />
            </CardHeader>
            <CardContent>
                <div className="text-2xl font-bold">{value}</div>
                <p className="text-xs text-muted-foreground">{desc}</p>
            </CardContent>
        </Card>
    )
}

function OverviewSkeleton() {
    return <div className="space-y-6"><Skeleton className="h-10 w-48" /><div className="grid gap-4 md:grid-cols-2 lg:grid-cols-5"><Skeleton className="h-32" /><Skeleton className="h-32" /><Skeleton className="h-32" /><Skeleton className="h-32" /><Skeleton className="h-32" /></div></div>
}

function formatCarbonMass(kg: number): string {
    if (kg === 0) return "0.0000 kg"
    if (kg < 0.000001) {
        return `${(kg * 1_000_000).toFixed(3)} mg`
    }
    if (kg < 0.001) {
        return `${(kg * 1_000_000).toFixed(1)} mg`
    }
    if (kg < 1.0) {
        return `${(kg * 1_000).toFixed(2)} g`
    }
    return `${kg.toFixed(4)} kg`
}

function formatEnergy(kwh: number): string {
    if (kwh === 0) return "0.0000 kWh"
    if (kwh < 0.000001) {
        return `${(kwh * 1_000_000_000).toFixed(1)} µWh`
    }
    if (kwh < 0.001) {
        return `${(kwh * 1_000_000).toFixed(1)} mWh`
    }
    if (kwh < 1.0) {
        return `${(kwh * 1_000).toFixed(1)} Wh`
    }
    return `${kwh.toFixed(4)} kWh`
}

function formatTreeDays(days: number): string {
    if (days === 0) return "0.0 days"
    if (days < 1.0 / 1440.0) {
        return `${(days * 86400).toFixed(0)} secs`
    }
    if (days < 1.0 / 24.0) {
        return `${(days * 1440).toFixed(1)} mins`
    }
    if (days < 1.0) {
        return `${(days * 24).toFixed(1)} hours`
    }
    return `${days.toFixed(1)} days`
}

function formatCarMiles(miles: number): string {
    if (miles === 0) return "0.0 miles"
    if (miles < 0.1) {
        const feet = miles * 5280;
        if (feet < 1.0) {
            return `${(feet * 12).toFixed(1)} inches`
        }
        return `${feet.toFixed(0)} feet`
    }
    return `${miles.toFixed(1)} miles`
}

function EsgCarbonCard() {
    const [range, setRange] = useState("all")
    const { data: ecoData } = useQuery({
        queryKey: ['eco-metrics', range],
        queryFn: () => fetchEcoMetrics(range),
        refetchInterval: 30000,
        retry: false
    })

    if (!ecoData || ecoData.status !== "success") {
        return null // Hide if feature is disabled or backend not configured
    }

    const metrics = ecoData.metrics;

    const handleDownloadEsg = (format: "json" | "csv") => {
        fetchEsgReportBlob(range, format)
            .then(blobData => {
                const blob = new Blob([blobData], { type: format === 'csv' ? 'text/csv' : 'application/json' })
                const url = window.URL.createObjectURL(blob)
                const a = document.createElement('a')
                a.href = url
                a.download = `hyperspacedb_esg_report_${range}.${format}`
                document.body.appendChild(a)
                a.click()
                window.URL.revokeObjectURL(url)
                document.body.removeChild(a)
            })
            .catch(err => alert("Failed to generate ESG report: " + err.message))
    }

    return (
        <Card className="border-emerald-500/20 bg-gradient-to-br from-emerald-950/20 via-zinc-950 to-zinc-950">
            <CardHeader className="flex flex-row items-center justify-between pb-2 space-y-0">
                <div>
                    <CardTitle className="text-base flex items-center gap-2">
                        <Leaf className="h-4 w-4 text-emerald-400 animate-pulse" />
                        ESG &amp; Carbon Footprint
                    </CardTitle>
                    <CardDescription className="text-xs">Matryoshka MRL Efficiency</CardDescription>
                </div>
                <Select value={range} onValueChange={setRange}>
                    <SelectTrigger className="w-[100px] h-8 text-xs bg-zinc-900 border-white/5">
                        <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                        <SelectItem value="24h">Last 24h</SelectItem>
                        <SelectItem value="30d">Last 30d</SelectItem>
                        <SelectItem value="1y">Last Year</SelectItem>
                        <SelectItem value="all">All Time</SelectItem>
                    </SelectContent>
                </Select>
            </CardHeader>
            <CardContent className="space-y-4">
                <div className="grid grid-cols-2 gap-4">
                    <div className="p-3 rounded-lg bg-zinc-900/60 border border-white/5 space-y-1">
                        <span className="text-[10px] text-zinc-400 uppercase font-bold tracking-wider">CO2 Saved</span>
                        <div className="text-xl font-bold font-mono text-emerald-400">
                            {formatCarbonMass(metrics.total_co2_saved_kg)}
                        </div>
                        <div className="text-[10px] text-zinc-500">Scope 3 Emissions Avoided</div>
                    </div>
                    <div className="p-3 rounded-lg bg-zinc-900/60 border border-white/5 space-y-1">
                        <span className="text-[10px] text-zinc-400 uppercase font-bold tracking-wider">CO2 Emitted</span>
                        <div className="text-xl font-bold font-mono text-zinc-500">
                            {formatCarbonMass(metrics.total_co2_emitted_kg)}
                        </div>
                        <div className="text-[10px] text-zinc-500">Actual Footprint</div>
                    </div>
                </div>

                <div className="space-y-2">
                    <div className="flex items-center justify-between py-1 border-b border-border/40 text-xs">
                        <span className="text-muted-foreground">Eco Tier Status</span>
                        <Badge
                            className={cn(
                                "border capitalize cursor-help px-1.5 py-0.5 font-semibold",
                                metrics.eco_tier === "Platinum" || metrics.eco_tier === "Gold"
                                    ? "bg-emerald-950/50 text-emerald-400 border-emerald-500/30"
                                    : "bg-zinc-900 text-zinc-400 border-white/5"
                            )}
                            title={`Overall Eco Status: ${metrics.eco_tier}`}
                        >
                            {metrics.eco_tier === "Bronze" && "🌱 "}
                            {metrics.eco_tier === "Silver" && "🌿 "}
                            {metrics.eco_tier === "Gold" && "🌳 "}
                            {metrics.eco_tier === "Platinum" && "🍀 "}
                            {metrics.eco_tier}
                        </Badge>
                    </div>
                    <div className="flex items-center justify-between py-1 border-b border-border/40 text-xs">
                        <span className="text-muted-foreground">Energy Conserved</span>
                        <span className="font-mono">{formatEnergy(metrics.power_saved_kwh)}</span>
                    </div>
                    <div className="flex items-center justify-between py-1 border-b border-border/40 text-xs">
                        <span className="text-muted-foreground">Tree Days Saved</span>
                        <span className="font-mono">🌳 {formatTreeDays(metrics.equivalents.tree_days)}</span>
                    </div>
                    <div className="flex items-center justify-between py-1 border-b border-border/40 text-xs">
                        <span className="text-muted-foreground">Car Miles Avoided</span>
                        <span className="font-mono">🚗 {formatCarMiles(metrics.equivalents.car_miles)}</span>
                    </div>
                </div>

                <div className="flex items-center gap-2 pt-2">
                    <Button variant="outline" size="sm" className="flex-1 text-xs border-emerald-500/20 text-emerald-400 hover:bg-emerald-500/10" onClick={() => handleDownloadEsg("csv")}>
                        Download ESG (CSV)
                    </Button>
                    <Button variant="outline" size="sm" className="flex-1 text-xs border-emerald-500/20 text-emerald-400 hover:bg-emerald-500/10" onClick={() => handleDownloadEsg("json")}>
                        JSON Report
                    </Button>
                </div>
            </CardContent>
        </Card>
    )
}

