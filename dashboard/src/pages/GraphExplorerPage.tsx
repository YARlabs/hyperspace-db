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

export function GraphExplorerPage() {
    const [collection, setCollection] = useState("")
    const [nodeId, setNodeId] = useState("1")
    const [layer, setLayer] = useState("0")
    const [limit, setLimit] = useState("100")
    const [vizMode, setVizMode] = useState<"poincare" | "euclidean">("poincare")

    const [hoveredNode, setHoveredNode] = useState<any>(null)
    const [selectedNode, setSelectedNode] = useState<any>(null)
    const [analysis, setAnalysis] = useState<any>(null)

    const nodePositions = useRef<any[]>([])
    const canvasRef = useRef<HTMLCanvasElement>(null)

    const { data: collections } = useQuery({
        queryKey: ["collections", "graph"],
        queryFn: () => api.get("/collections").then((r) => r.data),
    })

    const neighbors = useMutation<any, any, number | undefined>({
        mutationFn: (targetId?: number) =>
            api.get(`/collections/${collection}/graph/neighbors`, {
                params: {
                    id: targetId !== undefined ? targetId : Number(nodeId),
                    layer: Number(layer),
                    limit: Number(limit),
                    offset: 0,
                },
            }),
    })

    const analyze = useMutation({
        mutationFn: () => api.get(`/collections/${collection}/analyze/geometry`),
        onSuccess: (res) => setAnalysis(res.data)
    })

    const runNeighbors = (id?: number) => {
        const targetId = id !== undefined ? id : Number(nodeId)
        if (collection) {
            if (id !== undefined) setNodeId(id.toString())
            neighbors.mutate(targetId)
        }
    }

    // Stable Layout Memo
    const nodesWithPositions = useMemo(() => {
        const nodesData = Array.isArray(neighbors.data?.data) ? neighbors.data.data : (neighbors.data?.data?.neighbors || [])
        const count = nodesData.length
        if (count === 0) return []

        return nodesData.map((node: any, i: number) => {
            let lx, ly;
            const angle = (Math.PI * 2 * i) / count

            if (vizMode === "poincare") {
                const weight = (1 - (i / count)) * 0.8 + 0.1
                const mag = 1 - Math.exp(-weight)
                lx = Math.cos(angle) * mag
                ly = Math.sin(angle) * mag
            } else {
                // Stable deterministic distribution instead of Math.random()
                const hash = Math.abs(Math.sin(i * 123.456 + 78.91))
                const spread = 0.4 + hash * 0.6
                lx = Math.cos(angle * 1.3 + i) * spread
                ly = Math.sin(angle * 0.7 + i) * spread
            }
            return { lx, ly, id: node.id, data: node }
        })
    }, [neighbors.data, vizMode])

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

        nodePositions.current = nodesWithPositions.map((p: any) => ({
            x: cx + p.lx * r,
            y: cy + p.ly * r,
            id: p.id,
            data: p.data
        }))
    }, [nodesWithPositions])

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

        // Connections
        positions.forEach(pos => {
            pos.data.neighbors?.forEach((nbId: number) => {
                const target = positions.find(p => p.id === nbId)
                if (target) {
                    ctx.beginPath()
                    ctx.moveTo(pos.x, pos.y)
                    ctx.lineTo(target.x, target.y)
                    ctx.strokeStyle = "rgba(59, 130, 246, 0.05)"
                    ctx.stroke()
                }
            })
            if (vizMode === "poincare") {
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
                            <Button className="w-full bg-primary hover:bg-primary/90 rounded-xl" onClick={() => runNeighbors()} disabled={!collection || neighbors.isPending}>
                                {neighbors.isPending ? <RefreshCw className="mr-2 h-4 w-4 animate-spin" /> : <Search className="w-4 h-4 mr-2" />}
                                Sync Manifold
                            </Button>
                        </div>
                    </CardContent>
                </Card>

                <Card className="lg:col-span-6 bg-zinc-950 border-white/5 shadow-2xl overflow-hidden relative min-h-[500px]">
                    <div className="absolute inset-0">
                        <canvas
                            ref={canvasRef}
                            onMouseMove={handleMouseMove}
                            onClick={handleCanvasClick}
                            className="w-full h-full cursor-crosshair"
                        />
                    </div>
                    {neighbors.isPending && (
                        <div className="absolute inset-0 flex items-center justify-center bg-black/20 backdrop-blur-sm">
                            <RefreshCw className="w-10 h-10 animate-spin text-primary" />
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
