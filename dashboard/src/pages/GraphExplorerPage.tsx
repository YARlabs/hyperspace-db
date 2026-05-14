import { useState, useEffect, useRef, useMemo } from "react"
import { useMutation, useQuery } from "@tanstack/react-query"
import { api } from "@/lib/api"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"
import { Badge } from "@/components/ui/badge"
import { Network, Orbit, RefreshCw, MousePointer2, Info, Search, ArrowRight, Activity, Layers } from "lucide-react"

interface GraphNode {
    id: number;
    vector?: number[];
    metadata?: any;
    neighbors?: number[];
}

interface PositionedNode {
    lx: number;
    ly: number;
    id: number;
    data: GraphNode;
}

interface NodePosition {
    x: number;
    y: number;
    id: number;
    data: GraphNode;
}

interface GeometryAnalysis {
    delta: number;
    recommendation: string;
}

export function GraphExplorerPage() {
    const [collection, setCollection] = useState("")
    const [nodeId, setNodeId] = useState("1")
    const [layer, setLayer] = useState("0")
    const [limit, setLimit] = useState("100")
    const [vizMode, setVizMode] = useState<"poincare" | "euclidean">("poincare")
    const [showMomentum, setShowMomentum] = useState(true)
    const [showSubsumption, setShowSubsumption] = useState(false)

    const [hoveredNode, setHoveredNode] = useState<GraphNode | null>(null)
    const [selectedNode, setSelectedNode] = useState<GraphNode | null>(null)
    const [analysis, setAnalysis] = useState<GeometryAnalysis | null>(null)
    const [trustScore, setTrustScore] = useState<number | null>(null)

    const nodePositions = useRef<NodePosition[]>([])
    const momentumPath = useRef<{ lx: number; ly: number }[]>([])
    const canvasRef = useRef<HTMLCanvasElement>(null)

    const { data: collections } = useQuery({
        queryKey: ["collections", "graph"],
        queryFn: () => api.get("/collections").then((r) => r.data),
    })

    const explore = useMutation({
        mutationFn: (targetId?: number) => {
            const endpoint = showSubsumption ? "subsumption" : "explore";
            return api.get(`/collections/${collection}/graph/${endpoint}`, {
                params: {
                    [showSubsumption ? "root_id" : "start_id"]: targetId !== undefined ? targetId : Number(nodeId),
                    max_depth: 3,
                    max_nodes: Number(limit),
                },
            });
        },
        onSuccess: (res: any) => {
            const nodes = (Array.isArray(res.data) ? res.data : (res.data?.nodes || [])) as GraphNode[]
            if (nodes.length > 0) {
                // Calculate pseudo-Lyapunov stability based on average connectivity
                const avgDegree = nodes.reduce((acc: number, n: GraphNode) => acc + (n.neighbors?.length || 0), 0) / nodes.length;
                const stability = Math.min(0.2, 1.0 / (avgDegree + 1));
                setTrustScore(stability);
            }
        }
    })

    const analyze = useMutation({
        mutationFn: () => api.get(`/collections/${collection}/analyze/geometry`),
        onSuccess: (res) => setAnalysis(res.data)
    })

    const runNeighbors = (id?: number) => {
        const targetId = id !== undefined ? id : Number(nodeId)
        if (collection) {
            if (id !== undefined) setNodeId(id.toString())
            explore.mutate(targetId)
        }
    }

    // Stable Layout Memo
    const nodesWithPositions = useMemo<PositionedNode[]>(() => {
        const nodesData = (Array.isArray(explore.data?.data) ? explore.data.data : (explore.data?.data?.nodes || [])) as GraphNode[]
        const count = nodesData.length
        if (count === 0) return []

        return nodesData.map((node: GraphNode, i: number) => {
            let lx, ly;
            const angle = (Math.PI * 2 * i) / count

            // Use actual vector data for projection if available (PCA-like 2D projection)
            if (node.vector && node.vector.length >= 2) {
                if (vizMode === "poincare") {
                    // Hyperbolic projection (simplified: use first 2 dims as polar)
                    const mag = Math.tanh(Math.sqrt(node.vector[0]**2 + node.vector[1]**2) / 10)
                    const ang = Math.atan2(node.vector[1], node.vector[0])
                    lx = Math.cos(ang) * mag
                    ly = Math.sin(ang) * mag
                } else {
                    // Euclidean projection
                    lx = node.vector[0] / 10
                    ly = node.vector[1] / 10
                }
            } else {
                // Fallback to circular layout
                if (vizMode === "poincare") {
                    const weight = (1 - (i / count)) * 0.8 + 0.1
                    const mag = 1 - Math.exp(-weight)
                    lx = Math.cos(angle) * mag
                    ly = Math.sin(angle) * mag
                } else {
                    const hash = Math.abs(Math.sin(i * 123.456 + 78.91))
                    const spread = 0.4 + hash * 0.6
                    lx = Math.cos(angle * 1.3 + i) * spread
                    ly = Math.sin(angle * 0.7 + i) * spread
                }
            }
            return { lx: lx || 0, ly: ly || 0, id: node.id, data: node }
        })
    }, [explore.data, vizMode])

    // Update internal ref for hit-testing based on current canvas size
    useEffect(() => {
        const canvas = canvasRef.current
        if (!canvas || !nodesWithPositions.length) return
        const rect = canvas.getBoundingClientRect()
        const width = rect.width
        const height = rect.height
        const cx = width / 2
        const cy = height / 2
        const r = Math.min(width, height) / 2 * 0.8

        nodePositions.current = nodesWithPositions.map((p: PositionedNode) => ({
            x: cx + p.lx * r,
            y: cy + p.ly * r,
            id: p.id,
            data: p.data
        }))

        // Update momentum path based on selected node and its neighbors (normalized coordinates)
        if (selectedNode && showMomentum) {
            const startNode = nodesWithPositions.find((p: PositionedNode) => p.id === selectedNode.id)
            if (startNode) {
                const neighbors = nodesWithPositions.filter((p: PositionedNode) => startNode.data.neighbors?.includes(p.id))
                if (neighbors.length > 0) {
                    // Average direction in manifold space
                    const dlx = neighbors.reduce((acc: number, n: PositionedNode) => acc + (n.lx - startNode.lx), 0) / neighbors.length
                    const dly = neighbors.reduce((acc: number, n: PositionedNode) => acc + (n.ly - startNode.ly), 0) / neighbors.length
                    
                    momentumPath.current = [
                        { lx: startNode.lx, ly: startNode.ly },
                        { lx: startNode.lx + dlx * 1.5, ly: startNode.ly + dly * 1.5 },
                        { lx: startNode.lx + dlx * 3.0, ly: startNode.ly + dly * 3.0 }
                    ]
                }
            }
        } else {
            momentumPath.current = []
        }
    }, [nodesWithPositions, selectedNode, showMomentum])

    // Render Canvas Logic
    useEffect(() => {
        const canvas = canvasRef.current
        if (!canvas) return
        const ctx = canvas.getContext("2d")
        if (!ctx) return

        // High DPI Support - only update size if changed to avoid flicker
        const dpr = window.devicePixelRatio || 1
        const rect = canvas.getBoundingClientRect()
        const desiredW = Math.round(rect.width * dpr)
        const desiredH = Math.round(rect.height * dpr)

        if (canvas.width !== desiredW || canvas.height !== desiredH) {
            canvas.width = desiredW
            canvas.height = desiredH
        }

        ctx.setTransform(1, 0, 0, 1, 0, 0) // Reset scale
        ctx.scale(dpr, dpr)

        const width = rect.width
        const height = rect.height
        const cx = width / 2
        const cy = height / 2
        const r = Math.min(width, height) / 2 * 0.8

        ctx.clearRect(0, 0, width, height)

        if (nodesWithPositions.length === 0) {
            ctx.fillStyle = "#3f3f46"
            ctx.font = "12px monospace"
            ctx.textAlign = "center"
            ctx.fillText("No vectors in view", cx, cy)
            return
        }

        const positions = nodePositions.current

        // Poincaré Boundary
        if (vizMode === "poincare") {
            ctx.beginPath()
            ctx.arc(cx, cy, r, 0, 2 * Math.PI)
            ctx.strokeStyle = "rgba(16, 185, 129, 0.1)"
            ctx.lineWidth = 1
            ctx.stroke()
        }

        // Momentum Path Tracer (Ghost Path)
        if (showMomentum && momentumPath.current.length > 1) {
            ctx.beginPath()
            ctx.setLineDash([5, 5])
            ctx.moveTo(cx + momentumPath.current[0].lx * r, cy + momentumPath.current[0].ly * r)
            
            for (let i = 1; i < momentumPath.current.length; i++) {
                ctx.lineTo(cx + momentumPath.current[i].lx * r, cy + momentumPath.current[i].ly * r)
            }
            
            ctx.strokeStyle = "rgba(0, 243, 255, 0.4)"
            ctx.lineWidth = 1.5
            ctx.stroke()
            ctx.setLineDash([])
            
            // Prediction Glow
            const end = momentumPath.current[momentumPath.current.length - 1]
            ctx.shadowBlur = 15
            ctx.shadowColor = "#00f3ff"
            ctx.fillStyle = "#00f3ff"
            ctx.beginPath()
            ctx.arc(cx + end.lx * r, cy + end.ly * r, 3, 0, Math.PI * 2)
            ctx.fill()
            ctx.shadowBlur = 0
        }

        // Connections
        positions.forEach(pos => {
            const nodeNeighbors = pos.data.neighbors || []
            nodeNeighbors.forEach((nbId: number) => {
                const target = positions.find(p => p.id === nbId)
                if (target) {
                    ctx.beginPath()
                    ctx.moveTo(pos.x, pos.y)
                    ctx.lineTo(target.x, target.y)
                    
                    if (showSubsumption) {
                        ctx.strokeStyle = "rgba(112, 0, 255, 0.4)"
                        ctx.lineWidth = 0.5
                    } else {
                        ctx.strokeStyle = "rgba(59, 130, 246, 0.1)"
                        ctx.lineWidth = 1
                    }
                    ctx.stroke()
                }
            })
            if (vizMode === "poincare" && !showSubsumption) {
                ctx.beginPath()
                ctx.moveTo(cx, cy)
                ctx.lineTo(pos.x, pos.y)
                ctx.strokeStyle = "rgba(255, 255, 255, 0.02)"
                ctx.stroke()
            }
        })

        // Nodes
        positions.forEach(pos => {
            const isHovered = hoveredNode?.id === pos.id || (pos.data && hoveredNode?.id === pos.data.id)
            const isSelected = selectedNode?.id === pos.id || (pos.data && selectedNode?.id === pos.data.id)

            ctx.beginPath()
            ctx.arc(pos.x, pos.y, isSelected ? 6 : (isHovered ? 5 : 3), 0, 2 * Math.PI)

            if (isSelected) ctx.fillStyle = "#f59e0b"
            else if (isHovered) ctx.fillStyle = "#f43f5e"
            else ctx.fillStyle = "rgba(59, 130, 246, 0.5)"

            ctx.fill()
            if (isSelected || isHovered) {
                ctx.strokeStyle = "white"
                ctx.lineWidth = 1
                ctx.stroke()
            }
        })

        // Origin
        ctx.beginPath()
        ctx.arc(cx, cy, 3, 0, 2 * Math.PI)
        ctx.fillStyle = "#10b981"
        ctx.fill()

    }, [nodesWithPositions, vizMode, hoveredNode, selectedNode])

    const getHitAt = (clientX: number, clientY: number) => {
        const canvas = canvasRef.current
        if (!canvas) return null
        const rect = canvas.getBoundingClientRect()
        const mouseX = clientX - rect.left
        const mouseY = clientY - rect.top

        return nodePositions.current.find(pos => {
            const dx = pos.x - mouseX
            const dy = pos.y - mouseY
            return Math.sqrt(dx * dx + dy * dy) < 14 // Increased hit radius for easier selection
        })
    }

    const handleMouseMove = (e: React.MouseEvent<HTMLCanvasElement>) => {
        const hit = getHitAt(e.clientX, e.clientY)
        setHoveredNode(hit ? hit.data : null)
    }

    const handleCanvasClick = (e: React.MouseEvent<HTMLCanvasElement>) => {
        const hit = getHitAt(e.clientX, e.clientY)
        setSelectedNode(hit ? hit.data : null)
    }

    return (
        <div className="space-y-6 fade-in h-screen flex flex-col pb-10">
            <div className="flex justify-between items-center">
                <div>
                    <h1 className="text-3xl font-bold tracking-tight text-white mb-2">Graph Explorer</h1>
                    <p className="text-muted-foreground">Spatial manifold traversal and geometric analysis.</p>
                </div>
                <div className="flex items-center gap-4">
                    <Tabs value={vizMode} onValueChange={(v: any) => setVizMode(v)} className="w-[300px]">
                        <TabsList className="grid w-full grid-cols-2 bg-zinc-900 border border-white/5">
                            <TabsTrigger value="poincare"><Orbit className="w-4 h-4 mr-2" /> Poincaré</TabsTrigger>
                            <TabsTrigger value="euclidean"><Network className="w-4 h-4 mr-2" /> Euclidean</TabsTrigger>
                        </TabsList>
                    </Tabs>
                </div>
            </div>

            <div className="grid grid-cols-1 lg:grid-cols-12 gap-6 flex-1 min-h-0">
                <Card className="lg:col-span-3 bg-zinc-950/50 border-white/5 backdrop-blur-sm shadow-2xl h-fit">
                    <CardHeader className="bg-white/5 border-b border-white/5 py-4">
                        <CardTitle className="text-white text-sm uppercase tracking-widest opacity-70 flex items-center gap-2">
                            <Activity className="w-4 h-4" /> Navigator
                        </CardTitle>
                    </CardHeader>
                    <CardContent className="space-y-6 pt-6">
                        <div className="space-y-4">
                            <div className="space-y-2">
                                <Label className="text-zinc-500 text-[10px] font-bold uppercase">Collection</Label>
                                <Select value={collection} onValueChange={setCollection}>
                                    <SelectTrigger className="bg-zinc-900 border-white/10 text-white rounded-xl">
                                        <SelectValue placeholder="Select" />
                                    </SelectTrigger>
                                    <SelectContent className="bg-zinc-900 border-white/10 text-white">
                                        {(collections || []).map((c: any) => {
                                            const name = typeof c === "string" ? c : c.name
                                            return <SelectItem key={name} value={name}>{name}</SelectItem>
                                        })}
                                    </SelectContent>
                                </Select>
                            </div>
                            <div className="space-y-2">
                                <Label className="text-zinc-500 text-[10px] font-bold uppercase">Root Node ID</Label>
                                <Input className="bg-zinc-900 border-white/10 text-white font-mono rounded-xl text-center" value={nodeId} onChange={(e) => setNodeId(e.target.value)} />
                            </div>
                            <div className="grid grid-cols-2 gap-4">
                                <div className="space-y-2">
                                    <Label className="text-zinc-500 text-[10px] font-bold uppercase">Layer</Label>
                                    <Input className="bg-zinc-900 border-white/10 text-white rounded-xl text-center" value={layer} onChange={(e) => setLayer(e.target.value)} />
                                </div>
                                <div className="space-y-2">
                                    <Label className="text-zinc-500 text-[10px] font-bold uppercase">Limit</Label>
                                    <Input className="bg-zinc-900 border-white/10 text-white rounded-xl text-center" value={limit} onChange={(e) => setLimit(e.target.value)} />
                                </div>
                            </div>
                            <div className="flex items-center justify-between p-3 bg-white/5 rounded-xl border border-white/5">
                                <Label className="text-xs text-zinc-300">Momentum Path</Label>
                                <input type="checkbox" checked={showMomentum} onChange={e => setShowMomentum(e.target.checked)} className="accent-primary" />
                            </div>
                            <div className="flex items-center justify-between p-3 bg-white/5 rounded-xl border border-white/5">
                                <Label className="text-xs text-zinc-300">Lorentz Hierarchy</Label>
                                <input type="checkbox" checked={showSubsumption} onChange={e => setShowSubsumption(e.target.checked)} className="accent-primary" />
                            </div>
                            <Button className="w-full bg-primary hover:bg-primary/90 rounded-xl" onClick={() => runNeighbors()} disabled={!collection || explore.isPending}>
                                {explore.isPending ? <RefreshCw className="mr-2 h-4 w-4 animate-spin" /> : <Search className="w-4 h-4 mr-2" />}
                                Sync Manifold
                            </Button>
                        </div>
                    </CardContent>
                </Card>

                <Card className="lg:col-span-6 bg-zinc-950 border-white/5 shadow-2xl overflow-hidden relative min-h-[500px]">
                    <div className="absolute top-4 left-4 z-10 flex gap-2">
                        <Badge className="bg-primary/20 text-primary border-primary/30 backdrop-blur-md">
                            {vizMode.toUpperCase()} SPACE
                        </Badge>
                        {trustScore !== null && (
                            <Badge className="bg-emerald-500/20 text-emerald-400 border-emerald-500/30 backdrop-blur-md">
                                TRUST: {(100 - trustScore * 100).toFixed(1)}%
                            </Badge>
                        )}
                    </div>
                    <div className="absolute inset-0">
                        <canvas
                            ref={canvasRef}
                            onMouseMove={handleMouseMove}
                            onClick={handleCanvasClick}
                            className="w-full h-full cursor-crosshair"
                        />
                    </div>
                    {explore.isPending && (
                        <div className="absolute inset-0 flex items-center justify-center bg-black/20 backdrop-blur-sm">
                            <RefreshCw className="w-10 h-10 animate-spin text-primary" />
                        </div>
                    )}
                    {trustScore !== null && (
                        <div className="absolute bottom-6 right-6 p-4 bg-zinc-900/80 backdrop-blur-xl rounded-2xl border border-white/5 shadow-2xl w-48 animate-in slide-in-from-bottom-4">
                            <p className="text-[10px] font-bold text-zinc-500 uppercase mb-3 flex items-center gap-2">
                                <Activity className="w-3 h-3 text-primary" /> Stability Radar
                            </p>
                            <div className="h-2 w-full bg-white/5 rounded-full overflow-hidden mb-2">
                                <div className="h-full bg-primary" style={{ width: `${(1.0 - trustScore) * 100}%` }} />
                            </div>
                            <div className="flex justify-between text-[9px] font-mono text-zinc-400">
                                <span>LYAPUNOV</span>
                                <span className={trustScore < 0.1 ? "text-emerald-400" : "text-amber-400"}>
                                    -{trustScore.toFixed(4)} λ
                                </span>
                            </div>
                        </div>
                    )}
                </Card>

                <div className="lg:col-span-3 flex flex-col gap-6 overflow-y-auto">
                    <Card className="bg-zinc-950/50 border-white/5 backdrop-blur-sm shadow-2xl overflow-hidden flex-none">
                        <CardHeader className="bg-white/5 border-b border-white/5 py-3">
                            <CardTitle className="text-white text-sm uppercase tracking-widest opacity-70 flex items-center gap-2">
                                <Info className="w-4 h-4" /> Inspector
                            </CardTitle>
                        </CardHeader>
                        <CardContent className="p-0">
                            {selectedNode ? (
                                <div className="p-6 space-y-5">
                                    <div className="bg-primary/5 border border-primary/20 rounded-2xl p-4">
                                        <p className="text-[10px] font-bold uppercase text-zinc-500 mb-1">Node Identity</p>
                                        <p className="text-2xl font-black text-primary font-mono"># {selectedNode.id}</p>
                                    </div>
                                    <Button className="w-full bg-white text-black hover:bg-zinc-200 rounded-xl" onClick={() => runNeighbors(selectedNode.id)}>
                                        <ArrowRight className="w-4 h-4 mr-2" /> Pivot Center
                                    </Button>
                                    <div className="space-y-2">
                                        <Label className="text-[10px] font-bold uppercase text-zinc-500">Metadata Preview</Label>
                                        <div className="bg-black/60 border border-white/5 rounded-xl p-3 text-xs text-zinc-300 max-h-48 overflow-y-auto font-mono scrollbar-hide">
                                            {JSON.stringify(selectedNode.metadata, null, 2)}
                                        </div>
                                    </div>
                                </div>
                            ) : (
                                <div className="p-12 text-center opacity-30">
                                    <MousePointer2 className="w-8 h-8 mx-auto mb-4 animate-bounce" />
                                    <p className="text-xs uppercase font-bold">Select a node</p>
                                </div>
                            )}
                        </CardContent>
                    </Card>

                    <Card className="bg-zinc-950/50 border-white/5 backdrop-blur-sm shadow-2xl flex-none">
                        <CardHeader className="bg-white/5 border-b border-white/5 py-3">
                            <CardTitle className="text-white text-sm uppercase tracking-widest opacity-70 flex items-center gap-2">
                                <Layers className="w-4 h-4" /> Gromov Advisor
                            </CardTitle>
                        </CardHeader>
                        <CardContent className="p-6">
                            {!analysis ? (
                                <Button
                                    variant="outline"
                                    className="w-full border-primary/20 text-primary hover:bg-primary/5 rounded-xl"
                                    onClick={() => analyze.mutate()}
                                    disabled={!collection || analyze.isPending}
                                >
                                    {analyze.isPending ? <RefreshCw className="mr-2 h-4 w-4 animate-spin" /> : "Analyze Geometry"}
                                </Button>
                            ) : (
                                <div className="space-y-3">
                                    <div className="flex justify-between items-center text-xs">
                                        <span className="text-zinc-500 uppercase font-bold text-[10px]">Delta</span>
                                        <span className="font-mono text-primary font-bold">{analysis.delta.toFixed(4)}</span>
                                    </div>
                                    <div className="flex justify-between items-center text-xs">
                                        <span className="text-zinc-500 uppercase font-bold text-[10px]">Rec. Metric</span>
                                        <Badge className="bg-primary/10 text-primary border-0">{analysis.recommendation.toUpperCase()}</Badge>
                                    </div>
                                    <p className="text-[10px] text-zinc-500 italic mt-2 italic">
                                        Data exhibits {analysis.delta < 0.2 ? "strong hyperbolic" : "flat Euclidean"} curvature hints.
                                    </p>
                                    <Button variant="ghost" size="sm" className="w-full text-[10px] text-zinc-500" onClick={() => setAnalysis(null)}>RESET</Button>
                                </div>
                            )}
                        </CardContent>
                    </Card>
                </div>
            </div>
        </div>
    )
}
