import { useQuery } from "@tanstack/react-query"
import { api, fetchStatus } from "@/lib/api"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card"
import { Database, HardDrive, Server, Zap, FolderOpen } from "lucide-react"
import { Skeleton } from "@/components/ui/skeleton"
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"
import { useState } from "react"

export function OverviewPage() {
    const { data: status, isLoading: sLoading } = useQuery({
        queryKey: ['status'],
        queryFn: fetchStatus,
        refetchInterval: 60000
    })
    const { data: metrics } = useQuery({
        queryKey: ['metrics'],
        queryFn: () => api.get("/metrics").then(r => r.data),
        refetchInterval: 30000
    })

    if (sLoading && !status) return <OverviewSkeleton />

    const formatDiskSize = (mb: number) => {
        if (mb >= 1024) {
            return `${(mb / 1024).toFixed(2)} GB`
        }
        return `${mb} MB`
    }

    return (
        <div className="space-y-6 fade-in">
            <div className="flex items-center justify-between">
                <h1 className="text-3xl font-bold tracking-tight">System Overview</h1>
                <div className="flex items-center gap-2 text-sm text-muted-foreground">
                    <div className={`h-2 w-2 rounded-full ${status?.status === 'ONLINE' ? 'bg-green-500 shadow-[0_0_8px_#22c55e]' : 'bg-red-500'}`}></div>
                    {status?.status || "Connecting..."}
                </div>
            </div>

            <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-6">
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
                <StatCard title="System Mode" value={status?.config?.mode?.toUpperCase() || "PERFORMANCE"} icon={Zap} desc="Optimization profile" />
            </div>

            <div className="grid gap-4 md:grid-cols-2">
                <Card>
                    <CardHeader><CardTitle>Configuration</CardTitle><CardDescription>Runtime parameters</CardDescription></CardHeader>
                    <CardContent>
                        <div className="space-y-4">
                            <ConfigRow label="Version" value={status?.version} />
                            <ConfigRow label="Optimization Mode" value={status?.config?.mode} />
                            <ConfigRow label="RAM Ceiling" value={status?.config?.max_ram_gb && status.config.max_ram_gb !== "0" ? `${status.config.max_ram_gb} GB` : "Unlimited"} />
                            <ConfigRow label="Global Dimension" value={status?.config?.dimension} />
                            <ConfigRow label="Metric Space" value={
                                status?.config?.metric === 'cosine' ? 'Cosine Similarity' :
                                    status?.config?.metric === 'l2' || status?.config?.metric === 'euclidean' ? 'Euclidean (L2)' :
                                        status?.config?.metric === 'poincare' ? 'Hyperbolic (Poincaré)' :
                                            status?.config?.metric || 'Unknown'
                            } />
                            <ConfigRow label="Quantization" value={status?.config?.quantization || "Scalar I8"} />
                            <ConfigRow label="Embedding Engine" value={status?.embedding?.enabled ? "Multi-Model Active" : "Disabled"} />
                            {status?.embedding?.enabled && status?.embedding?.models && (
                                <div className="mt-2 pl-4 border-l-2 border-primary/20 space-y-2">
                                    {Object.entries(status.embedding.models).map(([metric, info]: [string, any]) => (
                                        <div key={metric} className="flex flex-col">
                                            <span className="text-[10px] uppercase tracking-wider text-muted-foreground font-bold">
                                                {metric === 'l2' ? 'Euclidean (L2)' : 
                                                 metric === 'cosine' ? 'Cosine' : 
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
                                        api.post("/admin/vacuum")
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
        queryFn: () => api.get("/metrics").then(r => r.data),
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
