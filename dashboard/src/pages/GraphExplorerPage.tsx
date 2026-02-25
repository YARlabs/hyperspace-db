import { useState, useEffect, useRef } from "react"
import { useMutation, useQuery } from "@tanstack/react-query"
import { api } from "@/lib/api"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"
import { Network, Orbit, RefreshCw } from "lucide-react"

export function GraphExplorerPage() {
    const [collection, setCollection] = useState("")
    const [nodeId, setNodeId] = useState("1")
    const [layer, setLayer] = useState("0")
    const [limit, setLimit] = useState("100")
    const [vizMode, setVizMode] = useState<"poincare" | "euclidean">("poincare")

    const canvasRef = useRef<HTMLCanvasElement>(null)

    const { data: collections } = useQuery({
        queryKey: ["collections", "graph"],
        queryFn: () => api.get("/collections").then((r) => r.data),
    })

    const neighbors = useMutation({
        mutationFn: () =>
            api.get(`/collections/${collection}/graph/neighbors`, {
                params: {
                    id: Number(nodeId),
                    layer: Number(layer),
                    limit: Number(limit),
                    offset: 0,
                },
            }),
    })

    const runNeighbors = () => {
        if (collection) neighbors.mutate()
    }

    // Render Canvas Logic
    useEffect(() => {
        const canvas = canvasRef.current
        if (!canvas) return
        const ctx = canvas.getContext("2d")
        if (!ctx) return

        ctx.clearRect(0, 0, canvas.width, canvas.height)

        const nodesData = neighbors.data?.data?.neighbors || []
        const count = nodesData.length

        if (count === 0) {
            ctx.fillStyle = "#3f3f46" // zinc-700
            ctx.font = "14px monospace"
            ctx.textAlign = "center"
            ctx.fillText("No vectors to display. Run a query first.", canvas.width / 2, canvas.height / 2)
            return
        }

        const cx = canvas.width / 2
        const cy = canvas.height / 2
        const r = Math.min(cx, cy) - 20 // boundary radius

        if (vizMode === "poincare") {
            // Draw Poincare Disk boundary
            ctx.beginPath()
            ctx.arc(cx, cy, r, 0, 2 * Math.PI)
            ctx.strokeStyle = "rgba(255, 255, 255, 0.2)"
            ctx.lineWidth = 2
            ctx.stroke()

            // Draw Origin
            ctx.beginPath()
            ctx.arc(cx, cy, 3, 0, 2 * Math.PI)
            ctx.fillStyle = "#10b981" // emerald-500
            ctx.fill()

            // Draw simulated neighbors (since we don't have full vectors for graphing from just neighbor API, we mock spatial layout based on edge weights)
            nodesData.forEach((node: any, i: number) => {
                const angle = (Math.PI * 2 * i) / count
                const weight = neighbors.data?.data?.edge_weights?.[i] || 0.5
                // The further the distance/weight, the closer to the boundary in Poincare
                const distRadius = Math.min(r * 0.9, Math.max(r * 0.1, r * (1 - Math.exp(-weight))))

                const nx = cx + Math.cos(angle) * distRadius
                const ny = cy + Math.sin(angle) * distRadius

                // Draw node
                ctx.beginPath()
                ctx.arc(nx, ny, 4, 0, 2 * Math.PI)
                ctx.fillStyle = "#3b82f6" // blue-500
                ctx.fill()

                // Draw Edge to central node
                ctx.beginPath()
                ctx.moveTo(cx, cy)

                // Poincare geodesic arc mock (bezier)
                const ctrlX = cx + Math.cos(angle + 0.5) * distRadius * 0.5
                const ctrlY = cy + Math.sin(angle + 0.5) * distRadius * 0.5

                ctx.quadraticCurveTo(ctrlX, ctrlY, nx, ny)
                ctx.strokeStyle = "rgba(59, 130, 246, 0.3)"
                ctx.lineWidth = 1
                ctx.stroke()

                // Text label
                ctx.fillStyle = "rgba(255, 255, 255, 0.5)"
                ctx.font = "10px monospace"
                ctx.fillText(node.id.toString(), nx + 6, ny + 3)
            })
        } else {
            // Euclidean projection (t-SNE/UMAP Mock simulation based on weights)
            ctx.fillStyle = "rgba(255,255,255,0.02)"
            // Grid
            for (let i = 0; i < canvas.width; i += 40) {
                ctx.fillRect(i, 0, 1, canvas.height)
                ctx.fillRect(0, i, canvas.width, 1)
            }

            nodesData.forEach((node: any, i: number) => {
                const angle = (Math.PI * 2 * i) / count
                const weight = (neighbors.data?.data?.edge_weights?.[i] || 0.5) * 10
                const nx = cx + Math.cos(angle * 3.1) * (r * Math.random())
                const ny = cy + Math.sin(angle * 2.7) * (r * Math.random())

                // Draw node
                ctx.beginPath()
                // Use weight to determine size, within bounds
                const nodeRadius = Math.max(3, Math.min(10, weight))
                ctx.arc(nx, ny, nodeRadius, 0, 2 * Math.PI)
                ctx.fillStyle = "#f43f5e" // rose-500
                ctx.fill()

                // Connecting lines indicating Euclidean neighborhood cluster
                if (i > 0) {
                    ctx.beginPath()
                    ctx.moveTo(cx, cy)
                    ctx.lineTo(nx, ny)
                    ctx.strokeStyle = "rgba(244, 63, 94, 0.1)"
                    ctx.lineWidth = 1
                    ctx.stroke()
                }

                // Text label
                ctx.fillStyle = "rgba(255, 255, 255, 0.5)"
                ctx.font = "10px monospace"
                ctx.fillText(node.id.toString(), nx + 6, ny + 3)
            })

            // Draw Origin (Query Vector Proxy)
            ctx.beginPath()
            ctx.arc(cx, cy, 6, 0, 2 * Math.PI)
            ctx.fillStyle = "#10b981"
            ctx.fill()
            ctx.fillText("Q", cx + 8, cy - 8)
        }

    }, [neighbors.data, vizMode])

    return (
        <div className="space-y-6 fade-in h-screen flex flex-col pb-10">
            <div>
                <h1 className="text-3xl font-bold tracking-tight text-white mb-2">Spatial Concept Visualizer</h1>
                <p className="text-muted-foreground">Interact with high-dimensional Euclidean bounds and Hyperbolic structures.</p>
            </div>

            <div className="grid grid-cols-1 lg:grid-cols-4 gap-6 flex-1">
                <Card className="col-span-1 bg-zinc-950/50 border-white/5 backdrop-blur-sm shadow-2xl h-fit">
                    <CardHeader>
                        <CardTitle className="text-white">Rendering Engine</CardTitle>
                        <CardDescription>Select metric space to inspect.</CardDescription>
                    </CardHeader>
                    <CardContent className="space-y-6">
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
                                <Label className="text-zinc-300">Graph Node ID (Origin Pivot)</Label>
                                <Input className="bg-zinc-900 border-white/10 text-white" value={nodeId} onChange={(e) => setNodeId(e.target.value)} />
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
                            <Button className="w-full bg-white text-black hover:bg-zinc-200" onClick={runNeighbors} disabled={!collection || neighbors.isPending}>
                                {neighbors.isPending ? <RefreshCw className="mr-2 h-4 w-4 animate-spin" /> : null}
                                {neighbors.isPending ? "Computing..." : "Render Vectors"}
                            </Button>
                        </div>
                    </CardContent>
                </Card>

                <Card className="col-span-1 lg:col-span-3 bg-zinc-950 border-white/5 shadow-2xl overflow-hidden relative min-h-[500px]">
                    <div className="absolute top-4 left-4 z-10 flex gap-2">
                        <span className="px-2 py-1 bg-black/50 border border-white/10 rounded text-xs font-mono text-zinc-400 backdrop-blur-md">
                            Mode: {vizMode.toUpperCase()}
                        </span>
                        <span className="px-2 py-1 bg-black/50 border border-white/10 rounded text-xs font-mono text-zinc-400 backdrop-blur-md">
                            Nodes: {neighbors.data?.data?.neighbors?.length || 0}
                        </span>
                    </div>
                    {/* Render Canvas */}
                    <div className="absolute inset-0 flex items-center justify-center bg-[radial-gradient(circle_at_center,_var(--tw-gradient-stops))] from-zinc-900/20 to-zinc-950">
                        <canvas
                            ref={canvasRef}
                            width={800}
                            height={800}
                            style={{ width: '100%', height: '100%', objectFit: 'contain' }}
                            className="bg-transparent"
                        />
                    </div>
                </Card>
            </div>
        </div>
    )
}
