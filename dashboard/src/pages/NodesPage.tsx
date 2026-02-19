import { useQuery } from "@tanstack/react-query"
import { api } from "@/lib/api"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Badge } from "@/components/ui/badge"
import { Network, ArrowDown, Activity, Clock, Box } from "lucide-react"

export function NodesPage() {
    const { data: cluster, isLoading } = useQuery({
        queryKey: ['cluster'],
        queryFn: () => api.get("/cluster/status").then(r => r.data),
        refetchInterval: 2000
    })

    if (isLoading && !cluster) {
        return <div className="p-8 text-center text-muted-foreground">Loading topology...</div>
    }

    return (
        <div className="space-y-6 fade-in h-[calc(100vh-100px)] flex flex-col">
            <div className="flex items-center justify-between shrink-0">
                <div>
                    <h1 className="text-3xl font-bold tracking-tight">Cluster Topology</h1>
                    <p className="text-muted-foreground mt-1">Visualize node relationships and replication status</p>
                </div>
                <div className="flex items-center gap-4">
                    <span className="text-xs text-muted-foreground font-mono">
                        Node ID: {cluster?.node_id}
                    </span>
                    <Badge variant={cluster?.role === 'Leader' ? "default" : "secondary"} className="text-lg px-4 py-1">
                        {cluster?.role?.toUpperCase() || "UNKNOWN"}
                    </Badge>
                </div>
            </div>

            {/* Top Info Zone - Horizontal Layout */}
            <div className="grid gap-4 md:grid-cols-3 shrink-0">
                <Card className="border-primary/10 shadow-sm">
                    <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                        <CardTitle className="text-sm font-medium">Role Status</CardTitle>
                        <Network className="h-4 w-4 text-muted-foreground" />
                    </CardHeader>
                    <CardContent>
                        <div className="text-2xl font-bold">{cluster?.role}</div>
                        <p className="text-xs text-muted-foreground">
                            {cluster?.role === 'Leader' ? 'Coordinating Write Operations' : 'Replicating from Leader'}
                        </p>
                    </CardContent>
                </Card>
                <Card className="border-primary/10 shadow-sm">
                    <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                        <CardTitle className="text-sm font-medium">Logical Clock</CardTitle>
                        <Clock className="h-4 w-4 text-muted-foreground" />
                    </CardHeader>
                    <CardContent>
                        <div className="text-2xl font-bold font-mono">{cluster?.logical_clock}</div>
                        <p className="text-xs text-muted-foreground">Lamport Timestamp</p>
                    </CardContent>
                </Card>
                <Card className="border-primary/10 shadow-sm">
                    <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                        <CardTitle className="text-sm font-medium">Connected Peers</CardTitle>
                        <Box className="h-4 w-4 text-muted-foreground" />
                    </CardHeader>
                    <CardContent>
                        <div className="text-2xl font-bold">{cluster?.downstream_peers?.length || 0}</div>
                        <p className="text-xs text-muted-foreground">Active Replicas</p>
                    </CardContent>
                </Card>
            </div>

            {/* Main Topology Visualizer - Full Width/Height */}
            <Card className="flex-1 flex flex-col items-center justify-center min-h-[400px] bg-muted/5 relative overflow-hidden border-2 border-dashed">
                {/* Background Grid - CSS Only */}
                <div
                    className="absolute inset-0 opacity-10 pointer-events-none"
                    style={{
                        backgroundImage: `linear-gradient(to right, currentColor 1px, transparent 1px), linear-gradient(to bottom, currentColor 1px, transparent 1px)`,
                        backgroundSize: '40px 40px'
                    }}
                ></div>

                <div className="z-10 flex flex-col items-center gap-12 w-full px-8 py-12 h-full overflow-y-auto">

                    {/* Upstream (Leader) */}
                    {cluster?.role === 'Follower' && (
                        <div className="flex flex-col items-center animate-in fade-in slide-in-from-top-4 shrink-0">
                            <div className="p-4 rounded-xl border border-border bg-card shadow-sm flex items-center gap-3 w-64 justify-center">
                                <div className="h-10 w-10 rounded-full bg-yellow-500/20 flex items-center justify-center">
                                    <Activity className="h-5 w-5 text-yellow-600" />
                                </div>
                                <div className="flex flex-col">
                                    <span className="font-bold text-sm">Leader Node</span>
                                    <span className="text-xs text-muted-foreground font-mono truncate max-w-[120px]">
                                        {cluster?.upstream_peer || "Connecting..."}
                                    </span>
                                </div>
                            </div>
                            <ArrowDown className="h-8 w-8 text-muted-foreground my-4 animate-bounce" />
                        </div>
                    )}

                    {/* Current Node (Center) */}
                    <div className="shrink-0 p-8 rounded-full border-4 border-primary bg-card shadow-[0_0_50px_rgba(124,58,237,0.15)] flex flex-col items-center justify-center gap-1 w-48 h-48 relative group z-20 hover:scale-105 transition-transform duration-300">
                        <div className="absolute -top-6 bg-primary text-primary-foreground text-sm px-4 py-1 rounded-full font-bold shadow-lg">
                            THIS NODE
                        </div>
                        <Network className="h-10 w-10 text-primary mb-2" />
                        <h3 className="font-bold text-xl">{cluster?.role}</h3>
                        <code className="text-[10px] bg-muted px-2 py-1 rounded text-muted-foreground font-mono">
                            {cluster?.node_id?.substring(0, 8)}...
                        </code>
                    </div>

                    {/* Downstream (Followers) */}
                    {cluster?.role === 'Leader' && cluster?.downstream_peers?.length > 0 && (
                        <div className="flex flex-col items-center w-full animate-in fade-in slide-in-from-bottom-4 flex-1">
                            <ArrowDown className="h-12 w-12 text-muted-foreground my-4 shrink-0" />

                            {/* Scrollable Container for many followers */}
                            <div className="w-full max-w-5xl overflow-y-auto max-h-[300px] p-4 rounded-xl bg-black/5 border border-black/5">
                                <div className="flex flex-wrap gap-6 justify-center">
                                    {cluster.downstream_peers.map((peer: string, i: number) => (
                                        <div key={i} className="p-4 rounded-lg border border-border bg-card shadow-md flex items-center gap-3 w-[200px] hover:shadow-lg transition-shadow">
                                            <div className="h-3 w-3 rounded-full bg-green-500 shrink-0 animate-pulse"></div>
                                            <div className="flex flex-col overflow-hidden">
                                                <span className="text-xs font-bold text-nowrap">Follower #{i + 1}</span>
                                                <span className="text-[10px] font-mono text-muted-foreground truncate" title={peer}>{peer}</span>
                                            </div>
                                        </div>
                                    ))}
                                </div>
                            </div>
                            <div className="text-xs text-muted-foreground mt-2">
                                {cluster.downstream_peers.length} active replicas connected
                            </div>
                        </div>
                    )}

                    {cluster?.role === 'Leader' && (!cluster?.downstream_peers || cluster?.downstream_peers.length === 0) && (
                        <div className="text-center text-muted-foreground text-sm mt-8 p-4 border border-dashed rounded-lg opacity-50">
                            No active followers connected. <br />
                            Start a follower node to see replication.
                        </div>
                    )}

                </div>
            </Card>
        </div>
    )
}
