import { useQuery } from "@tanstack/react-query"
import { api } from "@/lib/api"
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card"
import { Database, HardDrive, Server, Zap } from "lucide-react"
import { Skeleton } from "@/components/ui/skeleton"

export function OverviewPage() {
    const { data: status, isLoading: sLoading } = useQuery({
        queryKey: ['status'],
        queryFn: () => api.get("/status").then(r => r.data),
        refetchInterval: 5000
    })
    const { data: metrics } = useQuery({
        queryKey: ['metrics'],
        queryFn: () => api.get("/metrics").then(r => r.data),
        refetchInterval: 2000
    })

    if (sLoading && !status) return <OverviewSkeleton />

    return (
        <div className="space-y-6 fade-in">
            <div className="flex items-center justify-between">
                <h1 className="text-3xl font-bold tracking-tight">System Overview</h1>
                <div className="flex items-center gap-2 text-sm text-muted-foreground">
                    <div className={`h-2 w-2 rounded-full ${status?.status === 'ONLINE' ? 'bg-green-500 shadow-[0_0_8px_#22c55e]' : 'bg-red-500'}`}></div>
                    {status?.status || "Connecting..."}
                </div>
            </div>

            <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
                <StatCard title="Total Vectors" value={metrics?.total_vectors?.toLocaleString() || "0"} icon={Database} desc="Across all collections" />
                <StatCard title="RAM Usage" value={`${metrics?.ram_usage_mb || 0} MB`} icon={HardDrive} desc="Resident Set Size" />
                <StatCard title="Collections" value={metrics?.total_collections || 0} icon={Server} desc="Active indices" />
                <StatCard title="CPU Load" value={`${metrics?.cpu_usage_percent || 0}%`} icon={Zap} desc="System Load (Est.)" />
            </div>

            <div className="grid gap-4 md:grid-cols-2">
                <Card>
                    <CardHeader><CardTitle>Configuration</CardTitle><CardDescription>Runtime parameters</CardDescription></CardHeader>
                    <CardContent>
                        <div className="space-y-4">
                            <ConfigRow label="Version" value={status?.version} />
                            <ConfigRow label="Global Dimension" value={status?.config?.dimension} />
                            <ConfigRow label="Metric Space" value={status?.config?.metric === 'l2' ? 'Euclidean (L2)' : 'Hyperbolic (Poincaré)'} />
                            <ConfigRow label="Quantization" value={status?.config?.quantization || "Scalar I8"} />
                            <ConfigRow label="Uptime" value={status?.uptime} />
                        </div>
                    </CardContent>
                </Card>

                <Card>
                    <CardHeader><CardTitle>Ingestion Status</CardTitle><CardDescription>Real-time throughput monitoring</CardDescription></CardHeader>
                    <CardContent className="flex items-center justify-center h-[240px] text-muted-foreground bg-muted/10 rounded-md border border-dashed">
                        Real-time metrics unavailable in MVP.
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
    return <div className="space-y-6"><Skeleton className="h-10 w-48" /><div className="grid gap-4 md:grid-cols-4"><Skeleton className="h-32" /><Skeleton className="h-32" /><Skeleton className="h-32" /><Skeleton className="h-32" /></div></div>
}
