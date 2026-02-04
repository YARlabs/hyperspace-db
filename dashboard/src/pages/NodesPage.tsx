import { useQuery } from "@tanstack/react-query"
import { api } from "@/lib/api"
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card"
import { Badge } from "@/components/ui/badge"
import { Network, Server, ArrowDown, Activity } from "lucide-react"

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
        <div className="space-y-6 fade-in">
            <div className="flex items-center justify-between">
                <div>
                    <h1 className="text-3xl font-bold tracking-tight">Cluster Topology</h1>
                    <p className="text-muted-foreground mt-1">Visualize node relationships and replication status</p>
                </div>
                <Badge variant={cluster?.role === 'Leader' ? "default" : "secondary"} className="text-lg px-4 py-1">
                    {cluster?.role?.toUpperCase() || "UNKNOWN"}
                </Badge>
            </div>

            <div className="grid gap-6 md:grid-cols-3">
                {/* Current Node Card */}
                <Card className="md:col-span-1 border-primary/20 shadow-lg bg-primary/5">
                    <CardHeader className="pb-2">
                        <CardTitle className="flex items-center gap-2">
                            <Server className="h-5 w-5 text-primary" />
                            Current Node (You)
                        </CardTitle>
                        <CardDescription>Node ID: {cluster?.node_id}</CardDescription>
                    </CardHeader>
                    <CardContent>
                        <div className="space-y-4">
                            <div className="flex justify-between items-center py-2 border-b border-primary/10">
                                <span className="text-sm font-medium">Role</span>
                                <span className="font-mono font-bold">{cluster?.role}</span>
                            </div>
                            <div className="flex justify-between items-center py-2 border-b border-primary/10">
                                <span className="text-sm font-medium">Logical Clock</span>
                                <span className="font-mono">{cluster?.logical_clock}</span>
                            </div>
                            <div className="flex justify-between items-center py-2">
                                <span className="text-sm font-medium">Peers Connected</span>
                                <span className="font-mono">{cluster?.downstream_peers?.length || 0}</span>
                            </div>
                        </div>
                    </CardContent>
                </Card>

                {/* Topology Visualization Area */}
                <Card className="md:col-span-2 flex flex-col items-center justify-center min-h-[300px] bg-muted/5 relative overflow-hidden">
                    {/* Background Grid */}
                    <div className="absolute inset-0 bg-[url('/grid.svg')] opacity-10 pointer-events-none"></div>

                    <div className="z-10 flex flex-col items-center gap-8 w-full max-w-2xl px-8">

                        {/* Upstream (Leader) */}
                        {cluster?.role === 'Follower' && (
                            <div className="flex flex-col items-center animate-in fade-in slide-in-from-top-4">
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
                                <ArrowDown className="h-8 w-8 text-muted-foreground my-2 animate-bounce" />
                            </div>
                        )}

                        {/* Current Node */}
                        <div className="p-6 rounded-2xl border-2 border-primary bg-card shadow-[0_0_30px_rgba(124,58,237,0.15)] flex flex-col items-center gap-2 w-full max-w-sm relative group">
                            <div className="absolute -top-3 bg-primary text-primary-foreground text-xs px-3 py-1 rounded-full font-bold shadow-sm">
                                THIS NODE
                            </div>
                            <div className="h-16 w-16 rounded-full bg-primary/10 flex items-center justify-center mb-2 group-hover:bg-primary/20 transition-colors">
                                <Network className="h-8 w-8 text-primary" />
                            </div>
                            <h3 className="font-bold text-xl">{cluster?.role}</h3>
                            <code className="text-xs bg-muted px-2 py-1 rounded text-muted-foreground">
                                {cluster?.node_id}
                            </code>
                        </div>

                        {/* Downstream (Followers) */}
                        {cluster?.role === 'Leader' && cluster?.downstream_peers?.length > 0 && (
                            <div className="flex flex-col items-center w-full animate-in fade-in slide-in-from-bottom-4">
                                <ArrowDown className="h-8 w-8 text-muted-foreground my-2" />
                                <div className="flex flex-wrap gap-4 justify-center">
                                    {cluster.downstream_peers.map((peer: string, i: number) => (
                                        <div key={i} className="p-3 rounded-lg border border-border bg-card shadow-sm flex items-center gap-2">
                                            <div className="h-2 w-2 rounded-full bg-green-500"></div>
                                            <div className="flex flex-col">
                                                <span className="text-xs font-bold">Follower #{i + 1}</span>
                                                <span className="text-[10px] font-mono text-muted-foreground">{peer}</span>
                                            </div>
                                        </div>
                                    ))}
                                </div>
                            </div>
                        )}

                        {cluster?.role === 'Leader' && (!cluster?.downstream_peers || cluster?.downstream_peers.length === 0) && (
                            <div className="text-center text-muted-foreground text-sm mt-4">
                                No active followers connected.
                            </div>
                        )}

                    </div>
                </Card>
            </div>
        </div>
    )
}
