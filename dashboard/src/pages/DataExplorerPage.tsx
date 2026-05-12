import { useState, useEffect } from "react"
import { useQuery, useMutation } from "@tanstack/react-query"
import { api } from "@/lib/api"
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table"
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Label } from "@/components/ui/label"
import { Input } from "@/components/ui/input"
import { useSearchParams } from "react-router-dom"
import { Code, Play, AlertCircle, ChevronLeft, ChevronRight, Edit3, Hash, FileJson, Download, Plus, Database, Info } from "lucide-react"
import { scrollCollection, countFiltered, updatePayload, insertVector } from "@/lib/api"
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter, DialogDescription, DialogTrigger } from "@/components/ui/dialog"
import { Badge } from "@/components/ui/badge"
import { Switch } from "@/components/ui/switch"
import { useQueryClient } from "@tanstack/react-query"

export function DataExplorerPage() {
    const [searchParams, setSearchParams] = useSearchParams()
    const initialCol = searchParams.get("collection") || ""
    const [selectedCollection, setSelectedCollection] = useState(initialCol)

    const { data: collections } = useQuery({
        queryKey: ['collections'],
        queryFn: () => api.get("/collections").then(r => r.data)
    })

    const handleSelect = (val: string) => {
        setSelectedCollection(val)
        setSearchParams({ collection: val })
    }

    // Auto-select first if none selected
    useEffect(() => {
        if (!selectedCollection && collections && collections.length > 0) {
            const first = typeof collections[0] === 'string' ? collections[0] : collections[0].name
            handleSelect(first)
        }
    }, [collections, selectedCollection])

    return (
        <div className="space-y-6 fade-in h-full flex flex-col">
            <div className="flex items-center justify-between flex-none">
                <div>
                    <h1 className="text-3xl font-bold tracking-tight">Data Explorer</h1>
                    <p className="text-muted-foreground">Inspect vectors and validate search</p>
                </div>
                <div className="flex items-center gap-4">
                    <InsertVectorDialog collection={selectedCollection} />
                    <div className="w-[300px]">
                        <Select value={selectedCollection} onValueChange={handleSelect}>
                            <SelectTrigger>
                                <SelectValue placeholder="Select Collection" />
                            </SelectTrigger>
                            <SelectContent>
                                {collections?.map((col: any) => {
                                    const name = typeof col === 'string' ? col : col.name
                                    return <SelectItem key={name} value={name}>{name}</SelectItem>
                                })}
                            </SelectContent>
                        </Select>
                    </div>
                </div>
            </div>

            {selectedCollection ? (
                <Tabs defaultValue="raw" className="flex-1 flex flex-col space-y-4">
                    <TabsList>
                        <TabsTrigger value="raw">Raw Data Table</TabsTrigger>
                        <TabsTrigger value="playground">Search Playground</TabsTrigger>
                    </TabsList>

                    <TabsContent value="raw" className="flex-1 overflow-hidden">
                        <RawDataView collection={selectedCollection} />
                    </TabsContent>

                    <TabsContent value="playground" className="flex-1">
                        <SearchPlayground collection={selectedCollection} />
                    </TabsContent>
                </Tabs>
            ) : (
                <div className="flex h-[400px] items-center justify-center rounded-md border border-dashed text-muted-foreground">
                    Select a collection to view data
                </div>
            )}
        </div>
    )
}

function RawDataView({ collection }: { collection: string }) {
    const [page, setPage] = useState(0)
    const [limit] = useState(50)
    const [isEditing, setIsEditing] = useState<any>(null)
    const queryClient = useQueryClient()

    const { data: items, isLoading } = useQuery({
        queryKey: ['scroll', collection, page],
        queryFn: () => scrollCollection(collection, { limit, offset: page * limit }),
        enabled: !!collection
    })

    const { data: countData } = useQuery({
        queryKey: ['count', collection],
        queryFn: () => countFiltered(collection, {}),
        enabled: !!collection
    })

    const totalCount = countData?.count || 0

    return (
        <Card className="h-full flex flex-col bg-zinc-950/40 border-white/5">
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                <div>
                    <CardTitle>Vector Scanner</CardTitle>
                    <CardDescription>
                        Displaying {page * limit + 1}-{Math.min((page + 1) * limit, totalCount)} of {totalCount} vectors
                    </CardDescription>
                </div>
                <div className="flex gap-2">
                    <Button variant="outline" size="sm" onClick={() => setPage(p => Math.max(0, p - 1))} disabled={page === 0}>
                        <ChevronLeft className="h-4 w-4" />
                    </Button>
                    <Button variant="outline" size="sm" onClick={() => setPage(p => p + 1)} disabled={(page + 1) * limit >= totalCount}>
                        <ChevronRight className="h-4 w-4" />
                    </Button>
                </div>
            </CardHeader>
            <CardContent className="flex-1 overflow-auto">
                <div className="rounded-md border border-white/5">
                    <Table>
                        <TableHeader className="bg-zinc-900/50">
                            <TableRow className="border-white/5">
                                <TableHead className="w-[80px]">ID</TableHead>
                                <TableHead>Vector (Prefix)</TableHead>
                                <TableHead>Metadata</TableHead>
                                <TableHead>Payload (Disk)</TableHead>
                                <TableHead className="w-[50px]"></TableHead>
                            </TableRow>
                        </TableHeader>
                        <TableBody>
                            {isLoading ? (
                                <TableRow><TableCell colSpan={5} className="text-center h-24"><div className="animate-pulse">Loading data plane...</div></TableCell></TableRow>
                            ) : (!items || items.length === 0) ? (
                                <TableRow><TableCell colSpan={5} className="text-center h-24 text-muted-foreground">No vectors found</TableCell></TableRow>
                            ) : (
                                items.map((item: any) => {
                                    // Handle both [id, vec, meta] and {id, vector, metadata} formats
                                    const id = Array.isArray(item) ? item[0] : item.id;
                                    const vec = Array.isArray(item) ? item[1] : item.vector;
                                    const meta = Array.isArray(item) ? item[2] : (item.metadata || {});
                                    const typedMeta = item.typed_metadata || {};

                                    return (
                                        <TableRow key={id} className="border-white/5 hover:bg-white/5">
                                            <TableCell className="font-mono text-xs text-zinc-400">{id}</TableCell>
                                            <TableCell className="font-mono text-[10px] text-zinc-500">
                                                [{vec?.slice(0, 4).map((n: number) => n.toFixed(3)).join(", ")}...]
                                            </TableCell>
                                            <TableCell>
                                                <div className="flex flex-wrap gap-1">
                                                    {Object.entries(meta).map(([k, v]: any) => (
                                                        <Badge key={k} variant="secondary" className="text-[9px] bg-blue-500/10 text-blue-400 border-blue-500/20">
                                                            {k}: {v}
                                                        </Badge>
                                                    ))}
                                                    {Object.entries(typedMeta).map(([k, v]: any) => (
                                                        <Badge key={k} variant="secondary" className="text-[9px] bg-purple-500/10 text-purple-400 border-purple-500/20">
                                                            <Hash className="w-2 h-2 mr-1" /> {k}: {JSON.stringify(v)}
                                                        </Badge>
                                                    ))}
                                                </div>
                                            </TableCell>
                                            <TableCell>
                                                {item.payload ? (
                                                    <div className="flex items-center gap-2">
                                                        <Badge variant="outline" className="bg-emerald-500/10 text-emerald-400 border-emerald-500/20 text-[10px]">
                                                            <FileJson className="w-3 h-3 mr-1" /> Sidecar
                                                        </Badge>
                                                        <Button variant="ghost" size="icon" className="h-6 w-6" onClick={() => downloadPayload(id, item.payload)}>
                                                            <Download className="h-3 w-3" />
                                                        </Button>
                                                    </div>
                                                ) : (
                                                    <span className="text-[10px] text-zinc-600 italic">RAM Only</span>
                                                )}
                                            </TableCell>
                                            <TableCell>
                                                <Button variant="ghost" size="icon" className="h-7 w-7 text-zinc-500 hover:text-white" onClick={() => setIsEditing({ id, meta, typedMeta })}>
                                                    <Edit3 className="h-3 w-3" />
                                                </Button>
                                            </TableCell>
                                        </TableRow>
                                    );
                                })
                            )}
                        </TableBody>
                    </Table>
                </div>
            </CardContent>
            {isEditing && (
                <MetadataEditorDialog 
                    open={!!isEditing} 
                    onOpenChange={(open: boolean) => !open && setIsEditing(null)}
                    point={isEditing}
                    collection={collection}
                    onSuccess={() => {
                        queryClient.invalidateQueries({ queryKey: ['scroll', collection] })
                    }}
                />
            )}
        </Card>
    )
}

function MetadataEditorDialog({ open, onOpenChange, point, collection, onSuccess }: any) {
    const [metadata, setMetadata] = useState<string>(JSON.stringify(point.meta, null, 2))

    const mutation = useMutation({
        mutationFn: (data: any) => updatePayload(collection, point.id, data),
        onSuccess: () => {
            onOpenChange(false)
            onSuccess()
        }
    })

    return (
        <Dialog open={open} onOpenChange={onOpenChange}>
            <DialogContent className="sm:max-w-[425px] bg-zinc-950 border-white/10">
                <DialogHeader>
                    <DialogTitle>Edit Metadata: Point {point.id}</DialogTitle>
                    <DialogDescription>Update metadata for this vector node. Currently supports legacy string-map format.</DialogDescription>
                </DialogHeader>
                <div className="grid gap-4 py-4">
                    <div className="space-y-2">
                        <Label>Metadata (JSON Object)</Label>
                        <textarea
                            className="flex min-h-[200px] w-full rounded-md border border-white/10 bg-zinc-900 px-3 py-2 text-sm font-mono text-zinc-300"
                            value={metadata}
                            onChange={(e) => setMetadata(e.target.value)}
                        />
                    </div>
                </div>
                <DialogFooter>
                    <Button variant="outline" onClick={() => onOpenChange(false)} className="border-white/10">Cancel</Button>
                    <Button onClick={() => {
                        try {
                            mutation.mutate(JSON.parse(metadata))
                        } catch (e) {
                            alert("Invalid JSON")
                        }
                    }} disabled={mutation.isPending}>
                        {mutation.isPending ? "Updating..." : "Save Changes"}
                    </Button>
                </DialogFooter>
            </DialogContent>
        </Dialog>
    )
}

function SearchPlayground({ collection }: { collection: string }) {
    const [vectorInput, setVectorInput] = useState("[0.1, 0.2, 0.3]")
    const [topK, setTopK] = useState("5")
    const [exactFilterJson, setExactFilterJson] = useState("{}")
    const [complexFiltersJson, setComplexFiltersJson] = useState("[]")
    const [includePayload, setIncludePayload] = useState(false)
    const [res, setRes] = useState<any>(null)
    const [graphRes, setGraphRes] = useState<any>(null)
    const [error, setError] = useState("")
    const [startId, setStartId] = useState("0")
    const [graphLayer, setGraphLayer] = useState("0")
    const [graphDepth, setGraphDepth] = useState("2")
    const [graphNodes, setGraphNodes] = useState("128")
    const [isMultiSearch, setIsMultiSearch] = useState(false)
    const [multiCollections, setMultiCollections] = useState<string[]>([collection])

    const { data: allCollections } = useQuery({
        queryKey: ['collections'],
        queryFn: () => api.get("/collections").then(r => r.data)
    })

    const searchMutation = useMutation({
        mutationFn: (payload: any) => {
            if (isMultiSearch) {
                return api.post(`/search/multi`, { ...payload, collections: multiCollections })
            }
            return api.post(`/collections/${collection}/search`, payload)
        },
        onSuccess: (data) => {
            const payload = data.data
            setRes(payload)
            setError("")
        },
        onError: (err: any) => { setError(err.message || "Search Failed"); setRes(null) }
    })

    const traverseMutation = useMutation({
        mutationFn: (payload: any) => api.post(`/collections/${collection}/graph/traverse`, payload),
        onSuccess: (data) => {
            setGraphRes(data.data)
            setError("")
        },
        onError: (err: any) => {
            setError(err.message || "Traverse failed")
            setGraphRes(null)
        }
    })

    const handleSearch = () => {
        try {
            const parsed = JSON.parse(vectorInput)
            const parsedExact = JSON.parse(exactFilterJson)
            const parsedComplex = JSON.parse(complexFiltersJson)
            if (!Array.isArray(parsed)) throw new Error("Input must be an array")
            if (typeof parsedExact !== "object" || parsedExact === null || Array.isArray(parsedExact)) {
                throw new Error("Filter must be an object")
            }
            if (!Array.isArray(parsedComplex)) {
                throw new Error("Filters must be an array")
            }
            searchMutation.mutate({
                vector: parsed,
                top_k: Math.max(1, Number(topK) || 5),
                filter: parsedExact,
                filters: parsedComplex,
                include_payload: includePayload
            })
        } catch (e: any) {
            setError("Invalid JSON format: " + e.message)
        }
    }

    const handleTraverse = () => {
        const sid = Number(startId)
        const layer = Number(graphLayer)
        const depth = Number(graphDepth)
        const nodes = Number(graphNodes)
        if (Number.isNaN(sid) || Number.isNaN(layer) || Number.isNaN(depth) || Number.isNaN(nodes)) {
            setError("Graph inputs must be valid numbers")
            return
        }
        try {
            const parsedExact = JSON.parse(exactFilterJson)
            const parsedComplex = JSON.parse(complexFiltersJson)
            traverseMutation.mutate({
                start_id: sid,
                layer: Math.max(0, layer),
                max_depth: Math.max(0, depth),
                max_nodes: Math.max(1, nodes),
                filter: parsedExact,
                filters: parsedComplex,
            })
        } catch (e: any) {
            setError("Invalid filter JSON format: " + e.message)
        }
    }

    return (
        <Tabs defaultValue="vector-search" className="space-y-4">
            <TabsList>
                <TabsTrigger value="vector-search">Vector Search</TabsTrigger>
                <TabsTrigger value="graph-traverse">Graph Traverse</TabsTrigger>
            </TabsList>

            <TabsContent value="vector-search" className="grid gap-6 md:grid-cols-2">
                <Card>
                    <CardHeader>
                        <CardTitle>Query Vector</CardTitle>
                        <CardDescription>Use filters and inspect typed metadata</CardDescription>
                    </CardHeader>
                    <CardContent className="space-y-4">
                        <div className="grid grid-cols-2 gap-3">
                            <div className="grid gap-2">
                                <Label htmlFor="topk">Top K</Label>
                                <Input id="topk" value={topK} onChange={(e) => setTopK(e.target.value)} />
                            </div>
                            <div className="grid gap-2">
                                <Label htmlFor="collection">Collection Context</Label>
                                <div className="flex gap-2">
                                    <Input id="collection" value={collection} disabled className="flex-1" />
                                    <Button variant="outline" size="sm" onClick={() => setIsMultiSearch(!isMultiSearch)}>
                                        {isMultiSearch ? "Single" : "Multi"}
                                    </Button>
                                </div>
                            </div>
                        </div>
                        {isMultiSearch && (
                            <div className="p-3 rounded border border-blue-500/20 bg-blue-500/5 space-y-2">
                                <Label className="text-[10px] uppercase text-blue-400">Target Collections (Multi-Search)</Label>
                                <div className="flex flex-wrap gap-2">
                                    {allCollections?.map((c: any) => {
                                        const name = typeof c === 'string' ? c : c.name
                                        const active = multiCollections.includes(name)
                                        return (
                                            <Badge 
                                                key={name} 
                                                variant={active ? "default" : "outline"} 
                                                className="cursor-pointer"
                                                onClick={() => {
                                                    if (active) setMultiCollections(multiCollections.filter(n => n !== name))
                                                    else setMultiCollections([...multiCollections, name])
                                                }}
                                            >
                                                {name}
                                            </Badge>
                                        )
                                    })}
                                </div>
                            </div>
                        )}
                        <div className="flex items-center justify-between p-3 rounded-lg bg-zinc-900 border border-white/5">
                            <div className="space-y-0.5">
                                <Label className="text-sm">Include Sidecar Payload</Label>
                                <p className="text-[10px] text-muted-foreground">Perform lazy disk I/O to fetch heavy documents</p>
                            </div>
                            <Switch checked={includePayload} onCheckedChange={setIncludePayload} />
                        </div>
                        <div className="grid w-full gap-2">
                            <Label htmlFor="vector">Vector JSON</Label>
                            <textarea
                                className="flex min-h-[120px] w-full rounded-md border border-input bg-transparent px-3 py-2 text-sm shadow-sm placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50 font-mono"
                                value={vectorInput}
                                onChange={(e) => setVectorInput(e.target.value)}
                            />
                        </div>
                        <div className="grid w-full gap-2">
                            <Label htmlFor="exact-filter">Exact Filter JSON (map)</Label>
                            <textarea
                                className="flex min-h-[70px] w-full rounded-md border border-input bg-transparent px-3 py-2 text-sm font-mono"
                                value={exactFilterJson}
                                onChange={(e) => setExactFilterJson(e.target.value)}
                            />
                        </div>
                        <div className="grid w-full gap-2">
                            <Label htmlFor="complex-filters">Complex Filters JSON (array)</Label>
                            <textarea
                                className="flex min-h-[90px] w-full rounded-md border border-input bg-transparent px-3 py-2 text-sm font-mono"
                                value={complexFiltersJson}
                                onChange={(e) => setComplexFiltersJson(e.target.value)}
                            />
                        </div>
                        {error && <div className="text-sm text-destructive flex gap-2 items-center"><AlertCircle className="h-4 w-4" /> {error}</div>}
                        <Button onClick={handleSearch} disabled={searchMutation.isPending} className="w-full">
                            {searchMutation.isPending ? "Searching..." : "Execute Search"}
                            {!searchMutation.isPending && <Play className="ml-2 h-4 w-4" />}
                        </Button>
                    </CardContent>
                </Card>

                <Card>
                    <CardHeader>
                        <CardTitle>Results</CardTitle>
                        <CardDescription>Nearest neighbors (with metadata + typed metadata)</CardDescription>
                    </CardHeader>
                    <CardContent>
                        {res ? (
                            <div className="rounded-md border border-white/5 overflow-hidden">
                                {isMultiSearch ? (
                                    <div className="p-4 space-y-6">
                                        {Object.entries(res).map(([colName, results]: any) => (
                                            <div key={colName} className="space-y-2">
                                                <h4 className="text-xs font-bold text-blue-400 uppercase tracking-widest px-2">{colName}</h4>
                                                <div className="rounded border border-white/5 bg-white/5">
                                                    <ResultsTable results={results} />
                                                </div>
                                            </div>
                                        ))}
                                    </div>
                                ) : (
                                    <ResultsTable results={Array.isArray(res) ? res : (res.results || [])} />
                                )}
                            </div>
                        ) : (
                            <div className="flex h-[200px] items-center justify-center text-muted-foreground text-sm flex-col gap-2">
                                <Code className="h-8 w-8 opacity-20" />
                                Run a search to see k-NN results
                            </div>
                        )}
                    </CardContent>
                </Card>
            </TabsContent>

            <TabsContent value="graph-traverse" className="grid gap-6 md:grid-cols-2">
                <Card>
                    <CardHeader>
                        <CardTitle>Graph Traverse</CardTitle>
                        <CardDescription>Debug HNSW topology through traversal API</CardDescription>
                    </CardHeader>
                    <CardContent className="space-y-4">
                        <div className="grid grid-cols-2 gap-3">
                            <div className="grid gap-2">
                                <Label>Start ID</Label>
                                <Input value={startId} onChange={(e) => setStartId(e.target.value)} />
                            </div>
                            <div className="grid gap-2">
                                <Label>Layer</Label>
                                <Input value={graphLayer} onChange={(e) => setGraphLayer(e.target.value)} />
                            </div>
                            <div className="grid gap-2">
                                <Label>Max Depth</Label>
                                <Input value={graphDepth} onChange={(e) => setGraphDepth(e.target.value)} />
                            </div>
                            <div className="grid gap-2">
                                <Label>Max Nodes</Label>
                                <Input value={graphNodes} onChange={(e) => setGraphNodes(e.target.value)} />
                            </div>
                        </div>
                        {error && <div className="text-sm text-destructive flex gap-2 items-center"><AlertCircle className="h-4 w-4" /> {error}</div>}
                        <Button onClick={handleTraverse} disabled={traverseMutation.isPending} className="w-full">
                            {traverseMutation.isPending ? "Traversing..." : "Run Traverse"}
                            {!traverseMutation.isPending && <Play className="ml-2 h-4 w-4" />}
                        </Button>
                    </CardContent>
                </Card>

                <Card>
                    <CardHeader>
                        <CardTitle>Traverse Result</CardTitle>
                        <CardDescription>Nodes reached with adjacency snapshot</CardDescription>
                    </CardHeader>
                    <CardContent>
                        {graphRes ? (
                            <div className="rounded-md border">
                                <Table>
                                    <TableHeader>
                                        <TableRow>
                                            <TableHead>ID</TableHead>
                                            <TableHead>Layer</TableHead>
                                            <TableHead>Neighbors</TableHead>
                                        </TableRow>
                                    </TableHeader>
                                    <TableBody>
                                        {graphRes.map((n: any) => (
                                            <TableRow key={n.id}>
                                                <TableCell className="font-mono">{n.id}</TableCell>
                                                <TableCell className="font-mono">{n.layer}</TableCell>
                                                <TableCell className="font-mono text-xs text-muted-foreground">
                                                    [{(n.neighbors || []).slice(0, 12).join(", ")}{(n.neighbors || []).length > 12 ? ", ..." : ""}]
                                                </TableCell>
                                            </TableRow>
                                        ))}
                                    </TableBody>
                                </Table>
                            </div>
                        ) : (
                            <div className="flex h-[200px] items-center justify-center text-muted-foreground text-sm flex-col gap-2">
                                <Code className="h-8 w-8 opacity-20" />
                                Run traversal to inspect graph nodes
                            </div>
                        )}
                    </CardContent>
                </Card>
            </TabsContent>
        </Tabs>
    )
}

function ResultsTable({ results }: { results: any[] }) {
    return (
        <Table>
            <TableHeader>
                <TableRow className="border-white/5 bg-zinc-900/50">
                    <TableHead>ID</TableHead>
                    <TableHead>Score</TableHead>
                    <TableHead>Metadata</TableHead>
                    <TableHead>Payload</TableHead>
                </TableRow>
            </TableHeader>
            <TableBody>
                {results.map((r: any) => (
                    <TableRow key={r.id} className="border-white/5 hover:bg-white/5">
                        <TableCell className="font-mono text-xs">{r.id}</TableCell>
                        <TableCell className="font-mono text-green-400 font-bold text-xs">{(1 - Number(r.distance)).toFixed(4)}</TableCell>
                        <TableCell className="align-top">
                            <div className="flex flex-wrap gap-1 max-w-[200px]">
                                {Object.entries(r.metadata || {}).map(([k, v]: any) => (
                                    <Badge key={k} variant="outline" className="text-[9px] border-white/10 text-zinc-400">{k}: {v}</Badge>
                                ))}
                                {Object.entries(r.typed_metadata || {}).map(([k]: any) => (
                                    <Badge key={k} variant="outline" className="text-[9px] border-purple-500/30 text-purple-400">
                                        <Hash className="w-2 h-2 mr-1" /> {k}
                                    </Badge>
                                ))}
                            </div>
                        </TableCell>
                        <TableCell>
                            {r.payload ? (
                                <Button variant="outline" size="sm" className="h-7 text-[10px] gap-1" onClick={() => downloadPayload(r.id, r.payload)}>
                                    <Download className="w-3 h-3" /> View
                                </Button>
                            ) : (
                                <span className="text-[10px] text-zinc-600">-</span>
                            )}
                        </TableCell>
                    </TableRow>
                ))}
            </TableBody>
        </Table>
    )
}

const downloadPayload = (id: number, payload: string) => {
    try {
        const blob = new Blob([atob(payload)], { type: 'application/octet-stream' });
        const url = window.URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = `payload_${id}.bin`;
        document.body.appendChild(a);
        a.click();
        window.URL.revokeObjectURL(url);
    } catch (e) {
        // Fallback: if not base64, show in alert
        alert("Payload (Plain text): " + payload);
    }
}

function InsertVectorDialog({ collection }: { collection: string }) {
    const [open, setOpen] = useState(false)
    const [id, setId] = useState("")
    const [vector, setVector] = useState("[]")
    const [metadata, setMetadata] = useState("{}")
    const [payload, setPayload] = useState("")
    const queryClient = useQueryClient()

    const mutation = useMutation({
        mutationFn: (data: any) => insertVector(collection, data),
        onSuccess: () => {
            setOpen(false)
            setId("")
            setVector("[]")
            setMetadata("{}")
            setPayload("")
            queryClient.invalidateQueries({ queryKey: ['scroll', collection] })
            queryClient.invalidateQueries({ queryKey: ['count', collection] })
        }
    })

    const handleInsert = () => {
        try {
            mutation.mutate({
                id: parseInt(id),
                vector: JSON.parse(vector),
                metadata: JSON.parse(metadata),
                payload: payload || undefined
            })
        } catch (e: any) {
            alert("Invalid JSON: " + e.message)
        }
    }

    return (
        <Dialog open={open} onOpenChange={setOpen}>
            <DialogTrigger asChild>
                <Button variant="outline" className="gap-2 border-emerald-500/20 text-emerald-400 hover:bg-emerald-500/10">
                    <Plus className="h-4 w-4" /> Insert Vector
                </Button>
            </DialogTrigger>
            <DialogContent className="sm:max-w-[600px] bg-zinc-950 border-white/10">
                <DialogHeader>
                    <DialogTitle className="flex items-center gap-2">
                        <Database className="h-5 w-5 text-emerald-400" />
                        Insert into {collection}
                    </DialogTitle>
                    <DialogDescription>Store high-dimensional vectors with optional sidecar payloads.</DialogDescription>
                </DialogHeader>
                <div className="grid gap-4 py-4">
                    <div className="grid grid-cols-2 gap-4">
                        <div className="space-y-2">
                            <Label>Vector ID (Integer)</Label>
                            <Input value={id} onChange={e => setId(e.target.value)} placeholder="e.g. 1001" />
                        </div>
                    </div>
                    <div className="space-y-2">
                        <Label>Vector Data (JSON Array)</Label>
                        <textarea
                            className="flex min-h-[80px] w-full rounded-md border border-white/10 bg-zinc-900 px-3 py-2 text-sm font-mono"
                            value={vector}
                            onChange={e => setVector(e.target.value)}
                        />
                    </div>
                    <div className="space-y-2">
                        <Label>Metadata (JSON Object)</Label>
                        <textarea
                            className="flex min-h-[80px] w-full rounded-md border border-white/10 bg-zinc-900 px-3 py-2 text-sm font-mono"
                            value={metadata}
                            onChange={e => setMetadata(e.target.value)}
                        />
                    </div>
                    <div className="space-y-2">
                        <div className="flex items-center justify-between">
                            <Label className="text-emerald-400 flex items-center gap-1">
                                <FileJson className="w-3 h-3" /> Sidecar Payload (Disk-Stored)
                            </Label>
                            <Info className="w-3 h-3 text-zinc-500" />
                        </div>
                        <textarea
                            className="flex min-h-[120px] w-full rounded-md border border-emerald-500/20 bg-emerald-500/5 px-3 py-2 text-sm font-mono text-emerald-100"
                            placeholder="Raw text or base64 blob. This will be stored on disk and fetched lazily."
                            value={payload}
                            onChange={e => setPayload(e.target.value)}
                        />
                    </div>
                </div>
                <DialogFooter>
                    <Button variant="ghost" onClick={() => setOpen(false)}>Cancel</Button>
                    <Button onClick={handleInsert} disabled={!id || mutation.isPending} className="bg-emerald-600 hover:bg-emerald-700">
                        {mutation.isPending ? "Inserting..." : "Insert Point"}
                    </Button>
                </DialogFooter>
            </DialogContent>
        </Dialog>
    )
}
