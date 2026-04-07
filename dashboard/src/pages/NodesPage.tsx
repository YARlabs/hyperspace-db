import { useQuery } from "@tanstack/react-query"
import { api } from "@/lib/api"
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card"
import { Badge } from "@/components/ui/badge"
import { Network, ArrowDown, Activity, Clock, Box, ShieldCheck, Globe, Database, Router, CheckCircle2, XCircle } from "lucide-react"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"

export function NodesPage() {
    const { data: cluster, isLoading: clusterLoading } = useQuery({
        queryKey: ['cluster'],
        queryFn: () => api.get("/cluster/status").then(r => r.data),
        refetchInterval: 2000
    })

    const { data: swarm, isLoading: swarmLoading } = useQuery({
        queryKey: ['swarm'],
        queryFn: () => api.get("/swarm/peers").then(r => r.data),
        refetchInterval: 2000
    })

    if ((clusterLoading && !cluster) || (swarmLoading && !swarm)) {
        return <div className="p-8 text-center text-muted-foreground animate-pulse">Scanning topological space...</div>
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

            {/* Tabs for Replication vs Swarm Gossip */}
            <Tabs defaultValue="swarm" className="flex-1 flex flex-col">
                <TabsList className="grid w-full max-w-[400px] grid-cols-2 mb-4 shrink-0">
                    <TabsTrigger value="swarm" className="flex items-center gap-2">
                        <Globe className="h-4 w-4" /> Gossip Swarm
                    </TabsTrigger>
                    <TabsTrigger value="replication" className="flex items-center gap-2">
                        <Router className="h-4 w-4" /> Replication Tree
                    </TabsTrigger>
                </TabsList>

                {/* Swarm Gossip P2P Tab */}
                <TabsContent value="swarm" className="flex-1 mt-0">
                    <div className="grid grid-cols-1 md:grid-cols-4 gap-6 h-full">
                        {/* Swarm Stats Sidebar */}
                        <div className="md:col-span-1 space-y-4">
                            <Card className="bg-card">
                                <CardHeader className="pb-2">
                                    <CardTitle className="text-sm">Gossip Protocol</CardTitle>
                                    <CardDescription>Decentralized UDP Heartbeats</CardDescription>
                                </CardHeader>
                                <CardContent>
                                    <div className="flex items-center gap-2 mb-4">
                                        {swarm?.gossip_enabled ? (
                                            <Badge variant="default" className="bg-green-500/10 text-green-500 hover:bg-green-500/20"><CheckCircle2 className="w-3 h-3 mr-1" /> Active</Badge>
                                        ) : (
                                            <Badge variant="destructive"><XCircle className="w-3 h-3 mr-1" /> Disabled</Badge>
                                        )}
                                    </div>
                                    <div className="space-y-2">
                                        <div className="flex justify-between text-sm">
                                            <span className="text-muted-foreground">Known Peers:</span>
                                            <span className="font-mono font-bold">{swarm?.peer_count || 0}</span>
                                        </div>
                                    </div>
                                </CardContent>
                            </Card>

                            {!swarm?.gossip_enabled && (
                                <div className="p-4 border border-dashed border-orange-500/50 bg-orange-500/10 rounded-lg text-orange-400 text-xs">
                                    Gossip is disabled. Set <code>HS_GOSSIP_PEERS</code> or <code>HS_GOSSIP_PORT</code> to enable the decentralized swarm.
                                </div>
                            )}
                        </div>

                        {/* Visual Swarm Grid */}
                        <Card className="md:col-span-3 h-[400px] md:h-full bg-muted/5 relative overflow-y-auto border-2 border-dashed p-6">
                            <div
                                className="absolute inset-0 opacity-5 pointer-events-none"
                                style={{
                                    backgroundImage: `linear-gradient(to right, currentColor 1px, transparent 1px), linear-gradient(to bottom, currentColor 1px, transparent 1px)`,
                                    backgroundSize: '40px 40px'
                                }}
                            ></div>

                            <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-6 relative z-10">
                                {/* Always show self */}
                                <div className="p-5 rounded-xl border-2 border-primary bg-card/80 backdrop-blur shadow-[0_0_30px_rgba(124,58,237,0.15)] flex flex-col gap-3 group relative hover:-translate-y-1 transition-transform">
                                    <div className="absolute -top-3 left-4 bg-primary text-primary-foreground text-[10px] px-2 py-0.5 rounded-full font-bold">
                                        THIS NODE
                                    </div>
                                    <div className="flex justify-between items-start mt-1">
                                        <Network className="h-6 w-6 text-primary" />
                                        <Badge variant="outline" className="text-[10px] font-mono">{cluster?.logical_clock} LC</Badge>
                                    </div>
                                    <div>
                                        <h4 className="font-bold font-mono text-sm truncate">{cluster?.node_id?.split('-')[0]}</h4>
                                        <div className="text-[10px] text-muted-foreground flex items-center gap-1 mt-1">
                                            <ShieldCheck className="w-3 h-3 text-green-500" /> Healthy (Local)
                                        </div>
                                    </div>
                                </div>

                                {/* Render remote peers */}
                                {swarm?.peers?.map((peer: any) => (
                                    <div key={peer.node_id} className={`p-5 rounded-xl border bg-card/50 backdrop-blur shadow-sm flex flex-col gap-3 transition-transform hover:-translate-y-1 ${!peer.healthy ? 'opacity-50 grayscale border-dashed' : 'border-border/50'}`}>
                                        <div className="flex justify-between items-start">
                                            <Database className={`h-6 w-6 ${peer.healthy ? 'text-blue-500' : 'text-muted-foreground'}`} />
                                            <Badge variant="secondary" className="text-[10px] font-mono bg-blue-500/10 text-blue-500">{peer.logical_clock} LC</Badge>
                                        </div>
                                        <div>
                                            <div className="flex items-center gap-2">
                                                <h4 className="font-bold font-mono text-sm truncate" title={peer.node_id}>{peer.node_id.split('-')[0]}</h4>
                                                <Badge variant="outline" className="text-[8px] px-1 h-4">{peer.role}</Badge>
                                            </div>
                                            <div className="text-[10px] text-muted-foreground flex items-center justify-between mt-2">
                                                <span className="truncate">{peer.addr}</span>
                                                {peer.healthy ? (
                                                    <span className="text-green-500 flex items-center gap-1"><Activity className="w-3 h-3" /> Live</span>
                                                ) : (
                                                    <span className="text-red-400">Offline</span>
                                                )}
                                            </div>
                                            {/* Collections quick list */}
                                            {peer.collections?.length > 0 && (
                                                <div className="mt-3 pt-3 border-t border-border/50 flex flex-wrap gap-1">
                                                    {peer.collections.map((c: any) => (
                                                        <span key={c.name} className="text-[9px] bg-muted px-1.5 py-0.5 rounded text-muted-foreground" title={`Vector Count: ${c.vector_count}`}>
                                                            {c.name} ({c.vector_count})
                                                        </span>
                                                    ))}
                                                </div>
                                            )}
                                        </div>
                                    </div>
                                ))}

                                {swarm?.peers?.length === 0 && swarm?.gossip_enabled && (
                                    <div className="col-span-full h-full min-h-[200px] flex items-center justify-center text-muted-foreground text-sm flex-col gap-2">
                                        <Network className="w-8 h-8 opacity-20 animate-pulse" />
                                        <span>Listening for incoming heartbeats...</span>
                                    </div>
                                )}
                            </div>
                        </Card>
                    </div>
                </TabsContent>

                {/* Legacy Replication Tree Tab */}
                <TabsContent value="replication" className="flex-1 mt-0">
                    <Card className="flex flex-col items-center justify-center min-h-[400px] bg-muted/5 relative overflow-hidden border-2 border-dashed h-full">
                        {/* Background Grid - CSS Only */}
                        <div
                            className="absolute inset-0 opacity-10 pointer-events-none"
                            style={{
                                backgroundImage: `linear-gradient(to right, currentColor 1px, transparent 1px), linear-gradient(to bottom, currentColor 1px, transparent 1px)`,
                                backgroundSize: '40px 40px'
                            }}
                        ></div>

                        <div className="z-10 flex flex-col items-center gap-12 w-full px-8 py-12 h-full overflow-y-auto">
                            {/* Status Header */}
                            <div className="flex flex-col items-center gap-4 text-center shrink-0">
                                <div className="flex items-center gap-3">
                                    <Router className="h-6 w-6 text-primary" />
                                    <h2 className="text-xl font-bold">Replication Distribution</h2>
                                </div>
                                <div className="flex items-center gap-2">
                                    {swarm?.replication_enabled ? (
                                        <Badge variant="default" className="bg-green-500/10 text-green-500 border-green-500/20">
                                            <CheckCircle2 className="w-3 h-3 mr-1" /> Active
                                        </Badge>
                                    ) : (
                                        <Badge variant="destructive" className="bg-red-500/10 text-red-500 border-red-500/20">
                                            <XCircle className="w-3 h-3 mr-1" /> Exports Disabled
                                        </Badge>
                                    )}
                                </div>
                                {!swarm?.replication_enabled && (
                                    <p className="text-xs text-red-400 max-w-md mx-auto bg-red-400/5 p-3 rounded-lg border border-red-400/10">
                                        Server-to-server replication exports are currently secured. 
                                        Followers cannot stream data from this node. 
                                        Set <code className="bg-red-400/10 px-1 rounded text-[10px]">HS_REPLICATION_ALLOWED=true</code> to re-enable.
                                    </p>
                                )}
                            </div>

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
                </TabsContent>
            </Tabs>
        </div>
    )
}
