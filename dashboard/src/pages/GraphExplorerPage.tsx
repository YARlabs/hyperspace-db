import { useState, useEffect, useRef } from "react"
import { useMutation, useQuery } from "@tanstack/react-query"
import { api } from "@/lib/api"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"
import { Network, Orbit, RefreshCw, MousePointer2, Info, Search, ArrowRight, Activity } from "lucide-react"

export function GraphExplorerPage() {
    const [collection, setCollection] = useState("")
    const [nodeId, setNodeId] = useState("1")
    const [layer, setLayer] = useState("0")
    const [limit, setLimit] = useState("100")
    const [vizMode, setVizMode] = useState<"poincare" | "euclidean">("poincare")

    const [hoveredNode, setHoveredNode] = useState<any>(null)
    const [selectedNode, setSelectedNode] = useState<any>(null)
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

    const runNeighbors = (id?: number) => {
        const targetId = id !== undefined ? id : Number(nodeId)
        if (collection) {
            if (id !== undefined) setNodeId(id.toString())
            neighbors.mutate(targetId)
        }
    }

    const handleCanvasClick = () => {
        if (hoveredNode) {
            setSelectedNode(hoveredNode)
        }
    }

    const handleMouseMove = (e: React.MouseEvent<HTMLCanvasElement>) => {
        const canvas = canvasRef.current
        if (!canvas) return
        const rect = canvas.getBoundingClientRect()
        const x = (e.clientX - rect.left) * (canvas.width / rect.width)
        const y = (e.clientY - rect.top) * (canvas.height / rect.height)

        const hit = nodePositions.current.find(pos => {
            const dx = pos.x - x
            const dy = pos.y - y
            return Math.sqrt(dx*dx + dy*dy) < 10
        })
        setHoveredNode(hit ? hit.data : null)
    }

    // Render Canvas Logic
    useEffect(() => {
        const canvas = canvasRef.current
        if (!canvas) return
        const ctx = canvas.getContext("2d")
        if (!ctx) return

        ctx.clearRect(0, 0, canvas.width, canvas.height)

        const nodesData = Array.isArray(neighbors.data?.data) ? neighbors.data.data : (neighbors.data?.data?.neighbors || [])
        const count = nodesData.length
        
        const positions: any[] = []

        if (count === 0) {
            ctx.fillStyle = "#3f3f46" 
            ctx.font = "14px monospace"
            ctx.textAlign = "center"
            ctx.fillText("No vectors to display. Run a query first.", canvas.width / 2, canvas.height / 2)
            nodePositions.current = []
            return
        }

        const cx = canvas.width / 2
        const cy = canvas.height / 2
        const r = Math.min(cx, cy) - 40 

        // 1. Calculate Positions
        nodesData.forEach((node: any, i: number) => {
            let nx, ny;
            const angle = (Math.PI * 2 * i) / count
            
            if (vizMode === "poincare") {
                const weight = (1 - (i / count)) * 0.8 + 0.1
                const distRadius = Math.min(r * 0.95, Math.max(r * 0.1, r * (1 - Math.exp(-weight))))
                nx = cx + Math.cos(angle) * distRadius
                ny = cy + Math.sin(angle) * distRadius
            } else {
                const spread = r * (0.4 + Math.random() * 0.6)
                nx = cx + Math.cos(angle * 1.3 + i) * spread
                ny = cy + Math.sin(angle * 0.7 + i) * spread
            }
            positions.push({ x: nx, y: ny, id: node.id, data: node })
        })
        nodePositions.current = positions

        // 2. Draw Connections
        if (vizMode === "poincare") {
            ctx.beginPath()
            ctx.arc(cx, cy, r, 0, 2 * Math.PI)
            ctx.strokeStyle = "rgba(16, 185, 129, 0.1)"
            ctx.lineWidth = 1
            ctx.stroke()
        }

        positions.forEach(pos => {
            // Lines to neighbors within the current view
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

            // Line to Pivot
            ctx.beginPath()
            ctx.moveTo(cx, cy)
            ctx.lineTo(pos.x, pos.y)
            ctx.strokeStyle = "rgba(255, 255, 255, 0.03)"
            ctx.stroke()
        })

        // 3. Draw Nodes
        positions.forEach(pos => {
            const isHovered = hoveredNode?.id === pos.id
            const isSelected = selectedNode?.id === pos.id
            
            ctx.beginPath()
            ctx.arc(pos.x, pos.y, isSelected ? 8 : (isHovered ? 6 : 4), 0, 2 * Math.PI)
            
            if (isSelected) ctx.fillStyle = "#f59e0b" // amber
            else if (isHovered) ctx.fillStyle = "#f43f5e" // rose
            else ctx.fillStyle = "#3b82f6" // blue
            
            ctx.fill()
            
            if (isSelected || isHovered) {
                ctx.strokeStyle = "white"
                ctx.lineWidth = 2
                ctx.stroke()
                
                ctx.fillStyle = "white"
                ctx.font = "bold 12px monospace"
                ctx.textAlign = "left"
                ctx.fillText(pos.id.toString(), pos.x + 10, pos.y + 4)
            }
        })

        // Draw Origin
        ctx.beginPath()
        ctx.arc(cx, cy, 6, 0, 2 * Math.PI)
        ctx.fillStyle = "#10b981"
        ctx.fill()
        ctx.strokeStyle = "white"
        ctx.lineWidth = 1
        ctx.stroke()

    }, [neighbors.data, vizMode, hoveredNode, selectedNode])

    return (
        <div className="space-y-6 fade-in h-screen flex flex-col pb-10">
            <div>
                <h1 className="text-3xl font-bold tracking-tight text-white mb-2">Spatial Concept Visualizer</h1>
                <p className="text-muted-foreground">Interact with high-dimensional Euclidean bounds and Hyperbolic structures.</p>
            </div>

            <div className="grid grid-cols-1 lg:grid-cols-12 gap-6 flex-1">
                <Card className="lg:col-span-3 bg-zinc-950/50 border-white/5 backdrop-blur-sm shadow-2xl h-fit overflow-hidden">
                    <CardHeader className="bg-white/5 border-b border-white/5 py-4">
                        <CardTitle className="text-white flex items-center gap-2"><Activity className="w-4 h-4" /> Navigator</CardTitle>
                    </CardHeader>
                    <CardContent className="space-y-6 pt-6">
                        <div className="space-y-4">
                            <Tabs value={vizMode} onValueChange={(v: any) => setVizMode(v)} className="w-full">
                                <TabsList className="grid w-full grid-cols-2 bg-zinc-900 border border-white/5">
                                    <TabsTrigger value="poincare" className="data-[state=active]:bg-zinc-800"><Orbit className="w-4 h-4 mr-2" /> Poincaré</TabsTrigger>
                                    <TabsTrigger value="euclidean" className="data-[state=active]:bg-zinc-800"><Network className="w-4 h-4 mr-2" /> Euclidean</TabsTrigger>
                                </TabsList>
                            </Tabs>
                        </div>

                        <div className="space-y-4 pt-4 border-t border-white/5">
                            <div className="space-y-2">
                                <Label className="text-zinc-300">Target Collection</Label>
                                <Select value={collection} onValueChange={setCollection}>
                                    <SelectTrigger className="bg-zinc-900 border-white/10 text-white">
                                        <SelectValue placeholder="Select collection" />
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
                                <Label className="text-zinc-300">Root Node ID</Label>
                                <Input className="bg-zinc-900 border-white/10 text-white font-mono" value={nodeId} onChange={(e) => setNodeId(e.target.value)} />
                            </div>
                            <div className="grid grid-cols-2 gap-4">
                                <div className="space-y-2">
                                    <Label className="text-zinc-300">Layer</Label>
                                    <Input className="bg-zinc-900 border-white/10 text-white" value={layer} onChange={(e) => setLayer(e.target.value)} />
                                </div>
                                <div className="space-y-2">
                                    <Label className="text-zinc-300">Limit</Label>
                                    <Input className="bg-zinc-900 border-white/10 text-white" value={limit} onChange={(e) => setLimit(e.target.value)} />
                                </div>
                            </div>
                            <Button className="w-full bg-emerald-500 hover:bg-emerald-600 text-white border-0 shadow-lg shadow-emerald-500/20" onClick={() => runNeighbors()} disabled={!collection || neighbors.isPending}>
                                {neighbors.isPending ? <RefreshCw className="mr-2 h-4 w-4 animate-spin" /> : <Search className="w-4 h-4 mr-2" />}
                                {neighbors.isPending ? "Computing..." : "Sync Manifold"}
                            </Button>
                        </div>
                    </CardContent>
                </Card>

                <Card className="lg:col-span-6 bg-zinc-950 border-white/5 shadow-2xl overflow-hidden relative min-h-[500px] group">
                    <div className="absolute top-4 left-4 z-10 flex gap-2">
                        <span className="px-2 py-1 bg-black/50 border border-white/10 rounded text-xs font-mono text-zinc-400 backdrop-blur-md">
                            Mode: {vizMode.toUpperCase()}
                        </span>
                        <span className="px-2 py-1 bg-black/50 border border-white/10 rounded text-xs font-mono text-zinc-400 backdrop-blur-md">
                            Nodes: {Array.isArray(neighbors.data?.data) ? neighbors.data.data.length : (neighbors.data?.data?.neighbors?.length || 0)}
                        </span>
                    </div>
                    
                    {hoveredNode && (
                        <div className="absolute bottom-4 left-4 z-10 px-3 py-2 bg-emerald-500/10 border border-emerald-500/20 rounded-lg backdrop-blur-xl animate-in fade-in slide-in-from-bottom-2">
                           <p className="text-[10px] text-emerald-400 font-bold uppercase tracking-widest mb-1">Hovering Node</p>
                           <p className="text-white font-mono text-sm">ID: {hoveredNode.id}</p>
                        </div>
                    )}

                    {/* Render Canvas */}
                    <div className="absolute inset-0 flex items-center justify-center bg-[radial-gradient(circle_at_center,_var(--tw-gradient-stops))] from-zinc-900/20 to-zinc-950 cursor-crosshair">
                        <canvas
                            ref={canvasRef}
                            width={800}
                            height={800}
                            onMouseMove={handleMouseMove}
                            onClick={handleCanvasClick}
                            style={{ width: '100%', height: '100%', objectFit: 'contain' }}
                            className="bg-transparent"
                        />
                    </div>
                </Card>

                <Card className="lg:col-span-3 bg-zinc-950/50 border-white/5 backdrop-blur-sm shadow-2xl h-fit overflow-hidden">
                    <CardHeader className="bg-white/5 border-b border-white/5 py-4">
                        <CardTitle className="text-white flex items-center gap-2"><Info className="w-4 h-4" /> Inspector</CardTitle>
                    </CardHeader>
                    <CardContent className="p-0">
                        {selectedNode ? (
                            <div className="divide-y divide-white/5">
                                <div className="p-6 bg-emerald-500/5">
                                    <div className="flex justify-between items-start mb-4">
                                        <div>
                                            <h3 className="text-white font-bold text-lg leading-tight">Node #{selectedNode.id}</h3>
                                            <p className="text-emerald-400 text-xs font-mono">Layer {selectedNode.layer}</p>
                                        </div>
                                        <Button size="icon" variant="ghost" className="text-zinc-500 hover:text-white" onClick={() => setSelectedNode(null)}>
                                            <RefreshCw className="w-4 h-4 rotate-45" />
                                        </Button>
                                    </div>
                                    <Button className="w-full bg-white text-black hover:bg-zinc-200" onClick={() => runNeighbors(selectedNode.id)}>
                                        <ArrowRight className="w-4 h-4 mr-2" /> Pivot to this Node
                                    </Button>
                                </div>
                                <div className="p-6 space-y-4">
                                    <div>
                                        <Label className="text-[10px] uppercase tracking-wider text-zinc-500 mb-2 block">Content Preview</Label>
                                        <div className="bg-black/40 border border-white/5 rounded-lg p-3 text-sm text-zinc-300 leading-relaxed max-h-48 overflow-y-auto font-sans">
                                            {selectedNode.metadata?.text || selectedNode.metadata?.clean_fact || selectedNode.metadata?.pos || "No text content available for this vector."}
                                        </div>
                                    </div>
                                    <div>
                                        <Label className="text-[10px] uppercase tracking-wider text-zinc-500 mb-2 block">Available Metadata</Label>
                                        <div className="space-y-2">
                                            {Object.entries(selectedNode.metadata || {}).filter(([k]) => k !== 'text').map(([k, v]) => (
                                                <div key={k} className="flex justify-between text-xs border-b border-white/5 pb-1">
                                                    <span className="text-zinc-500 font-mono">{k}</span>
                                                    <span className="text-zinc-300 truncate ml-4" title={v as string}>{v as string}</span>
                                                </div>
                                            ))}
                                            {Object.keys(selectedNode.metadata || {}).length === 0 && (
                                                <p className="text-xs text-zinc-600 italic">No structured metadata</p>
                                            )}
                                        </div>
                                    </div>
                                </div>
                            </div>
                        ) : (
                            <div className="p-12 text-center space-y-4">
                                <div className="w-12 h-12 rounded-full bg-white/5 flex items-center justify-center mx-auto">
                                    <MousePointer2 className="w-6 h-6 text-zinc-700" />
                                </div>
                                <div>
                                    <p className="text-white font-medium">Select a node</p>
                                    <p className="text-xs text-zinc-500 mt-1">Click any point on the manifold to inspect its properties and navigate.</p>
                                </div>
                            </div>
                        )}
                    </CardContent>
                </Card>
            </div>
        </div>
    )
}
