import { useState } from "react"
import { useMutation, useQuery } from "@tanstack/react-query"
import { api } from "@/lib/api"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table"
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"

export function GraphExplorerPage() {
    const [collection, setCollection] = useState("")
    const [nodeId, setNodeId] = useState("1")
    const [layer, setLayer] = useState("0")
    const [limit, setLimit] = useState("32")

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

    const parents = useMutation({
        mutationFn: () =>
            api.get(`/collections/${collection}/graph/parents`, {
                params: {
                    id: Number(nodeId),
                    layer: Number(layer),
                    limit: Number(limit),
                },
            }),
    })

    const runNeighbors = () => {
        if (collection) neighbors.mutate()
    }
    const runParents = () => {
        if (collection) parents.mutate()
    }

    const neighborPayload = neighbors.data?.data as { neighbors?: any[], edge_weights?: number[] } | undefined
    const parentRows = (parents.data?.data || []) as any[]
    const rows = neighborPayload
        ? (neighborPayload.neighbors || []).map((node, idx) => ({
            ...node,
            edge_weight: neighborPayload.edge_weights?.[idx],
        }))
        : parentRows
    const showWeights = Boolean(neighborPayload)

    return (
        <div className="space-y-6 fade-in">
            <div>
                <h1 className="text-3xl font-bold tracking-tight">Graph Explorer</h1>
                <p className="text-muted-foreground">Inspect HNSW adjacency, neighbors and parent-like links.</p>
            </div>

            <Card>
                <CardHeader>
                    <CardTitle>Graph Query</CardTitle>
                    <CardDescription>Use HTTP graph endpoints added in v2.2.x</CardDescription>
                </CardHeader>
                <CardContent className="grid gap-4 md:grid-cols-4">
                    <div className="space-y-2">
                        <Label>Collection</Label>
                        <Select value={collection} onValueChange={setCollection}>
                            <SelectTrigger>
                                <SelectValue placeholder="Select collection" />
                            </SelectTrigger>
                            <SelectContent>
                                {(collections || []).map((c: any) => {
                                    const name = typeof c === "string" ? c : c.name
                                    return <SelectItem key={name} value={name}>{name}</SelectItem>
                                })}
                            </SelectContent>
                        </Select>
                    </div>
                    <div className="space-y-2">
                        <Label>Node ID</Label>
                        <Input value={nodeId} onChange={(e) => setNodeId(e.target.value)} />
                    </div>
                    <div className="space-y-2">
                        <Label>Layer</Label>
                        <Input value={layer} onChange={(e) => setLayer(e.target.value)} />
                    </div>
                    <div className="space-y-2">
                        <Label>Limit</Label>
                        <Input value={limit} onChange={(e) => setLimit(e.target.value)} />
                    </div>
                    <Button onClick={runNeighbors} disabled={!collection || neighbors.isPending}>
                        {neighbors.isPending ? "Loading..." : "Get Neighbors"}
                    </Button>
                    <Button variant="secondary" onClick={runParents} disabled={!collection || parents.isPending}>
                        {parents.isPending ? "Loading..." : "Get Concept Parents"}
                    </Button>
                </CardContent>
            </Card>

            <Card>
                <CardHeader>
                    <CardTitle>Graph Nodes</CardTitle>
                </CardHeader>
                <CardContent>
                    <div className="rounded-md border">
                        <Table>
                            <TableHeader>
                                <TableRow>
                                    <TableHead>ID</TableHead>
                                    <TableHead>Layer</TableHead>
                                    <TableHead>Weight</TableHead>
                                    <TableHead>Neighbors</TableHead>
                                </TableRow>
                            </TableHeader>
                            <TableBody>
                                {rows.length === 0 ? (
                                    <TableRow>
                                        <TableCell colSpan={4} className="h-24 text-center text-muted-foreground">
                                            Run a graph query to see nodes
                                        </TableCell>
                                    </TableRow>
                                ) : rows.map((n) => (
                                    <TableRow key={n.id}>
                                        <TableCell className="font-mono">{n.id}</TableCell>
                                        <TableCell className="font-mono">{n.layer}</TableCell>
                                        <TableCell className="font-mono text-xs">
                                            {showWeights && typeof n.edge_weight === "number" ? n.edge_weight.toFixed(6) : "-"}
                                        </TableCell>
                                        <TableCell className="font-mono text-xs text-muted-foreground">
                                            [{(n.neighbors || []).slice(0, 16).join(", ")}{(n.neighbors || []).length > 16 ? ", ..." : ""}]
                                        </TableCell>
                                    </TableRow>
                                ))}
                            </TableBody>
                        </Table>
                    </div>
                </CardContent>
            </Card>
        </div>
    )
}
