import { useState } from "react"
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query"
import { api, fetchStatus } from "@/lib/api"
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table"
import { Button } from "@/components/ui/button"
import { Plus, Trash2, MoreHorizontal, Database, Search, HardDrive, Cpu, Settings2, BarChart3, Moon, Zap, Layers, FlameKindling, TrendingUp, Clock } from "lucide-react"
import { getCollectionStats, updateCollectionConfig, freezeCollection, unfreezeCollection, getCacheStats, clearCache, updateCacheConfig } from "@/lib/api"
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogTrigger, DialogFooter, DialogDescription } from "@/components/ui/dialog"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { DropdownMenu, DropdownMenuTrigger, DropdownMenuContent, DropdownMenuItem, DropdownMenuLabel, DropdownMenuSeparator } from "@/components/ui/dropdown-menu"
import { Skeleton } from "@/components/ui/skeleton"
import { Badge } from "@/components/ui/badge"
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { useNavigate } from "react-router-dom"
import { useEffect } from "react"
import { getStatusColor } from "@/lib/utils"

export function CollectionsPage() {
    const queryClient = useQueryClient()
    const { data: collections, isLoading } = useQuery({
        queryKey: ['collections'],
        queryFn: () => api.get("/collections").then(r => r.data),
        refetchInterval: 60000,
        refetchOnWindowFocus: false
    })

    const isStringList = collections && collections.length > 0 && typeof collections[0] === 'string'

    const deleteMutation = useMutation({
        mutationFn: (name: string) => api.delete(`/collections/${name}`),
        onSuccess: () => queryClient.invalidateQueries({ queryKey: ['collections'] })
    })

    return (
        <div className="space-y-6 fade-in">
            <div className="flex items-center justify-between">
                <div>
                    <h1 className="text-3xl font-bold tracking-tight">Collections</h1>
                    <p className="text-muted-foreground">Manage your vector indices</p>
                </div>
                <CreateCollectionDialog />
            </div>

            <div className="rounded-md border bg-card">
                <Table>
                    <TableHeader>
                        <TableRow>
                            <TableHead className="w-[260px]">Name</TableHead>
                            <TableHead>Status</TableHead>
                            <TableHead>Dimension</TableHead>
                            <TableHead>Metric</TableHead>
                            <TableHead>Vectors</TableHead>
                            <TableHead>Queue</TableHead>
                            <TableHead className="text-right">Actions</TableHead>
                        </TableRow>
                    </TableHeader>
                    <TableBody>
                        {isLoading ? (
                            <TableSkeleton />
                        ) : (!collections || collections.length === 0) ? (
                            <TableRow><TableCell colSpan={7} className="text-center h-32 text-muted-foreground">No collections found. Create one to get started.</TableCell></TableRow>
                        ) : (
                            collections.map((col: any) => {
                                const name = isStringList ? col : col.name
                                return (
                                    <CollectionRow key={name} collection={col} isString={isStringList} onDelete={() => deleteMutation.mutate(name)} />
                                )
                            })
                        )}
                    </TableBody>
                </Table>
            </div>
        </div>
    )
}

function CollectionRow({ collection, isString, onDelete }: any) {
    const name = isString ? collection : collection.name
    // If backend only returns strings, we can fetch detailed stats here individually if needed, 
    // but better to fix backend. For now show placeholder if string.
    const status = isString ? "active" : (collection.status || "active")
    const count = isString ? "-" : (status === "idle" ? "0 (Idle)" : collection.count)
    const schema = collection.schema
    const dim = schema ? schema.components.map((c: any) => c.full_dimension).reduce((a: number, b: number) => a + b, 0) : collection.dimension
    const metric = schema ? schema.components.map((c: any) => c.metric).join('+') : collection.metric

    const navigate = useNavigate()
    const queryClient = useQueryClient()

    const freezeMutation = useMutation({
        mutationFn: (name: string) => freezeCollection(name),
        onSuccess: () => queryClient.invalidateQueries({ queryKey: ['collections'] })
    })

    const unfreezeMutation = useMutation({
        mutationFn: (name: string) => unfreezeCollection(name),
        onSuccess: () => queryClient.invalidateQueries({ queryKey: ['collections'] })
    })

    return (
        <TableRow>
            <TableCell className="font-medium flex items-center gap-2">
                <div className="p-1.5 rounded bg-primary/10 text-primary">
                    <Database className="h-4 w-4" />
                </div>
                {name}
            </TableCell>
            <TableCell>
                <div className="flex items-center gap-2">
                    <Badge variant="outline" className={getStatusColor(status)}>
                        {status}
                    </Badge>
                    {status === "active" ? (
                        <Button 
                            variant="ghost" 
                            size="icon" 
                            className="h-6 w-6 text-zinc-400 hover:text-amber-500 hover:bg-amber-500/10 rounded" 
                            onClick={() => freezeMutation.mutate(name)}
                            disabled={freezeMutation.isPending}
                            title="Put to Sleep (Freeze)"
                        >
                            <Moon className="h-3.5 w-3.5" />
                        </Button>
                    ) : (
                        <Button 
                            variant="ghost" 
                            size="icon" 
                            className="h-6 w-6 text-zinc-400 hover:text-emerald-500 hover:bg-emerald-500/10 rounded animate-pulse" 
                            onClick={() => unfreezeMutation.mutate(name)}
                            disabled={unfreezeMutation.isPending}
                            title="Wake Up (Activate)"
                        >
                            <Zap className="h-3.5 w-3.5 animate-bounce" style={{ animationDuration: '3s' }} />
                        </Button>
                    )}
                </div>
            </TableCell>
            <TableCell><Badge variant="outline" className="font-mono">{dim}</Badge></TableCell>
            <TableCell className="capitalize">{metric}</TableCell>
            <TableCell className="font-mono">{count}</TableCell>
            <TableCell className="font-mono">{collection.indexing_queue || 0}</TableCell>
            <TableCell className="text-right">
                <DropdownMenu>
                    <DropdownMenuTrigger asChild><Button variant="ghost" className="h-8 w-8 p-0"><MoreHorizontal className="h-4 w-4" /></Button></DropdownMenuTrigger>
                    <DropdownMenuContent align="end">
                        <DropdownMenuLabel>Actions</DropdownMenuLabel>
                        <DropdownMenuItem onClick={() => navigate(`/explorer?collection=${name}`)}>
                            <Search className="mr-2 h-4 w-4" /> Inspect Data
                        </DropdownMenuItem>
                        <CollectionStatsDialog collectionName={name} />
                        <DropdownMenuItem disabled>
                            Export Snapshot
                        </DropdownMenuItem>
                        <DropdownMenuSeparator />
                        {status === "active" ? (
                            <DropdownMenuItem onClick={() => freezeMutation.mutate(name)} disabled={freezeMutation.isPending}>
                                <Moon className="mr-2 h-4 w-4 text-amber-500" /> Put to Sleep (Freeze)
                            </DropdownMenuItem>
                        ) : (
                            <DropdownMenuItem onClick={() => unfreezeMutation.mutate(name)} disabled={unfreezeMutation.isPending}>
                                <Zap className="mr-2 h-4 w-4 text-emerald-500" /> Wake Up (Activate)
                            </DropdownMenuItem>
                        )}
                        <DropdownMenuSeparator />
                        <DropdownMenuItem onClick={() => {
                            if (window.confirm(`Are you sure you want to rebuild index for '${name}'? This is a heavy operation.`)) {
                                api.post(`/collections/${name}/rebuild`)
                                    .then(() => alert("Index rebuild started!"))
                                    .catch(e => alert("Failed: " + e.message))
                            }
                        }}>
                            <Database className="mr-2 h-4 w-4" /> Rebuild Index
                        </DropdownMenuItem>
                        <DropdownMenuItem onClick={() => {
                            if (window.confirm(`Flush hot cache for '${name}'? Next requests will go to disk until cache warms up.`)) {
                                clearCache(name)
                                    .then(() => alert("Cache flushed!"))
                                    .catch(e => alert("Failed: " + e.message))
                            }
                        }}>
                            <FlameKindling className="mr-2 h-4 w-4 text-orange-400" /> Flush Cache
                        </DropdownMenuItem>
                        <DropdownMenuSeparator />
                        <DropdownMenuItem className="text-destructive focus:text-destructive" onClick={onDelete}>
                            <Trash2 className="mr-2 h-4 w-4" /> Delete
                        </DropdownMenuItem>
                    </DropdownMenuContent>
                </DropdownMenu>
            </TableCell>
        </TableRow>
    )
}

function CreateCollectionDialog() {
    const [name, setName] = useState("")
    const [open, setOpen] = useState(false)
    const [dimension, setDimension] = useState<string>("1024")
    const [metric, setMetric] = useState<string>("l2")
    const [enableMRL, setEnableMRL] = useState<boolean>(false)
    const [mrlCutoff, setMrlCutoff] = useState<string>("128")
    const [mrlRerankTopK, setMrlRerankTopK] = useState<string>("100")
    const [isCustomDimension, setIsCustomDimension] = useState<boolean>(false)
    const [isCustomMrlCutoff, setIsCustomMrlCutoff] = useState<boolean>(false)
    const queryClient = useQueryClient()

    // Get global config to show default values
    const { data: status } = useQuery({
        queryKey: ['status'],
        queryFn: fetchStatus
    })

    // Sync from global config once loaded
    useEffect(() => {
        if (status?.config) {
            setDimension(status.config.dimension.toString())
            setMetric(status.config.metric)
        }
    }, [status])

    const mutation = useMutation({
        mutationFn: (data: any) => api.post("/collections", data),
        onSuccess: () => {
            setOpen(false)
            setName("")
            queryClient.invalidateQueries({ queryKey: ['collections'] })
        }
    })

    const handleCreate = () => {
        const fullDim = metric === "hybrid" ? 801 : (parseInt(dimension) || 1024);
        mutation.mutate({
            name,
            dimension: fullDim,
            metric: metric || "l2",
            mrl_cutoff_dimension: enableMRL ? (parseInt(mrlCutoff) || Math.max(Math.floor(fullDim / 8), 64)) : undefined,
            mrl_rerank_top_k: enableMRL ? (parseInt(mrlRerankTopK) || 100) : undefined,
        })
    }

    const getDimensions = () => {
        const large_base = [8, 16, 32, 64, 128, 512, 768, 1024, 1536, 2048, 3072, 4096, 8192]
        const small_base_hyper = [4, 8, 16, 32, 64, 128]

        if (metric === "lorentz") {
            // Lorentz restricted to 129 (128 spatial + 1 time)
            return small_base_hyper.flatMap(d => [d, d + 1])
        }
        if (metric === "poincare") {
            // Poincare restricted to 128
            return small_base_hyper
        }
        // L2 and Cosine support full range up to 8192
        return large_base
    }

    const handleMetricChange = (v: string) => {
        setMetric(v)
        setIsCustomDimension(false)
        setIsCustomMrlCutoff(false)
        if (v === "hybrid") {
            setDimension("801")
            setMrlCutoff("161")
        } else if (v === "lorentz") {
            setDimension("129")
            setMrlCutoff("17")
        } else if (v === "poincare") {
            setDimension("128")
            setMrlCutoff("16")
        } else {
            setDimension("1024")
            setMrlCutoff("128")
        }
    }

    const isValid = () => {
        if (!name) return false;
        if (metric !== "hybrid") {
            const dim = parseInt(dimension);
            if (isNaN(dim) || dim <= 0) return false;
            if (enableMRL) {
                const cutoff = parseInt(mrlCutoff);
                if (isNaN(cutoff) || cutoff <= 0 || cutoff >= dim) return false;
            }
        } else {
            if (enableMRL) {
                const cutoff = parseInt(mrlCutoff);
                if (isNaN(cutoff) || cutoff <= 0 || cutoff >= 801) return false;
            }
        }
        return true;
    }

    return (
        <Dialog open={open} onOpenChange={setOpen}>
            <DialogTrigger asChild>
                <Button><Plus className="mr-2 h-4 w-4" /> New Collection</Button>
            </DialogTrigger>
            <DialogContent className="sm:max-w-[450px]">
                <DialogHeader>
                    <DialogTitle>Create Collection</DialogTitle>
                    <DialogDescription>Add a new vector index to the system.</DialogDescription>
                </DialogHeader>
                <div className="grid gap-5 py-4">
                    <div className="space-y-2">
                        <Label htmlFor="name">Collection Name</Label>
                        <Input id="name" value={name} onChange={e => setName(e.target.value)} placeholder="e.g. user_embeddings" />
                    </div>

                    <div className="space-y-2">
                        <Label htmlFor="metric">Metric Space</Label>
                        <Select value={metric} onValueChange={handleMetricChange}>
                            <SelectTrigger id="metric">
                                <SelectValue placeholder="Select Metric" />
                            </SelectTrigger>
                            <SelectContent>
                                <SelectItem value="l2">Euclidean (L2)</SelectItem>
                                <SelectItem value="cosine">Cosine Similarity</SelectItem>
                                <SelectItem value="poincare">Poincaré Ball (Hyperbolic)</SelectItem>
                                <SelectItem value="lorentz">Lorentz / Hyperboloid (Hyperbolic)</SelectItem>
                                <SelectItem value="hybrid">Hybrid (Lorentz + L2)</SelectItem>
                            </SelectContent>
                        </Select>
                    </div>

                    {metric === "hybrid" ? (
                        <div className="space-y-2 bg-blue-500/5 border border-blue-500/10 p-3 rounded-lg">
                            <Label className="text-blue-400 font-semibold">Dimension</Label>
                            <div className="text-2xl font-bold text-white tracking-wide">801</div>
                            <p className="text-[10px] text-zinc-400">
                                Fixed dimension optimized for hybrid Lorentz-Euclidean embeddings (33 spatial-time + 768 Euclidean dimensions).
                            </p>
                        </div>
                    ) : (
                        <div className="space-y-2">
                            <Label htmlFor="dimension-select">Dimension</Label>
                            <div className="flex flex-col gap-2">
                                <Select 
                                    value={isCustomDimension ? "custom" : dimension} 
                                    onValueChange={(v) => {
                                        if (v === "custom") {
                                            setIsCustomDimension(true)
                                        } else {
                                            setIsCustomDimension(false)
                                            setDimension(v)
                                        }
                                    }}
                                >
                                    <SelectTrigger id="dimension-select" className="bg-zinc-900 border-white/10">
                                        <SelectValue placeholder="Select Dimension" />
                                    </SelectTrigger>
                                    <SelectContent>
                                        {getDimensions().map(d => (
                                            <SelectItem key={d} value={d.toString()}>{d} Dimensions</SelectItem>
                                        ))}
                                        <SelectItem value="custom">Custom Dimension...</SelectItem>
                                    </SelectContent>
                                </Select>

                                {isCustomDimension && (
                                    <div className="space-y-1.5 animate-in fade-in slide-in-from-top-1 duration-200">
                                        <Label htmlFor="custom-dimension-input" className="text-xs text-zinc-400">Enter Custom Dimension</Label>
                                        <Input
                                            id="custom-dimension-input"
                                            type="number"
                                            min="1"
                                            max="8192"
                                            value={dimension}
                                            onChange={(e) => setDimension(e.target.value)}
                                            placeholder="e.g. 512"
                                            className="bg-zinc-900 border-white/10"
                                        />
                                    </div>
                                )}
                            </div>
                        </div>
                    )}

                    <div className="space-y-3 pt-2 border-t border-white/5">
                        <div className="flex items-center space-x-2">
                            <input 
                                type="checkbox" 
                                id="mrl" 
                                checked={enableMRL} 
                                onChange={(e) => {
                                    setEnableMRL(e.target.checked)
                                    if (e.target.checked) {
                                        const dimVal = metric === "hybrid" ? 801 : (parseInt(dimension) || 1024)
                                        setMrlCutoff(metric === "hybrid" ? "161" : Math.max(Math.floor(dimVal / 8), 64).toString())
                                    }
                                }} 
                                className="w-4 h-4 rounded border-gray-300 text-blue-600 focus:ring-blue-500 bg-zinc-900"
                            />
                            <Label htmlFor="mrl" className="font-normal cursor-pointer text-sm">
                                Enable Matryoshka Representation Learning (MRL)
                            </Label>
                        </div>

                        {enableMRL && metric !== "hybrid" && (
                            <div className="grid grid-cols-2 gap-4 p-3 rounded-lg bg-blue-500/5 border border-blue-500/10 animate-in fade-in slide-in-from-top-1 duration-200">
                                <div className="space-y-2 col-span-2">
                                    <Label htmlFor="mrl-cutoff-select" className="text-xs">MRL Cutoff Dimension</Label>
                                    <div className="flex flex-col gap-1.5">
                                        <Select 
                                            value={isCustomMrlCutoff ? "custom" : mrlCutoff} 
                                            onValueChange={(v) => {
                                                if (v === "custom") {
                                                    setIsCustomMrlCutoff(true)
                                                } else {
                                                    setIsCustomMrlCutoff(false)
                                                    setMrlCutoff(v)
                                                }
                                            }}
                                        >
                                            <SelectTrigger id="mrl-cutoff-select" className="h-8 bg-zinc-900 border-white/10 text-xs">
                                                <SelectValue placeholder="Select Cutoff" />
                                            </SelectTrigger>
                                            <SelectContent>
                                                {[8, 16, 32, 64, 128, 256, 512].filter(c => c < (parseInt(dimension) || 1024)).map(c => (
                                                    <SelectItem key={c} value={c.toString()} className="text-xs">{c} Dimensions</SelectItem>
                                                ))}
                                                <SelectItem value="custom" className="text-xs">Custom Cutoff...</SelectItem>
                                            </SelectContent>
                                        </Select>

                                        {isCustomMrlCutoff && (
                                            <Input
                                                id="mrlCutoff"
                                                type="number"
                                                min="1"
                                                max={parseInt(dimension) - 1 || 1023}
                                                value={mrlCutoff}
                                                onChange={(e) => setMrlCutoff(e.target.value)}
                                                placeholder="e.g. 128"
                                                className="h-8 bg-zinc-900 border-white/10 text-xs"
                                            />
                                        )}
                                    </div>
                                    <p className="text-[9px] text-zinc-400">Truncated dimension for fast initial search phase.</p>
                                </div>
                                <div className="space-y-2 col-span-2">
                                    <Label htmlFor="mrlRerankTopK" className="text-xs">Rerank Top K</Label>
                                    <Input
                                        id="mrlRerankTopK"
                                        type="number"
                                        min="1"
                                        value={mrlRerankTopK}
                                        onChange={(e) => setMrlRerankTopK(e.target.value)}
                                        className="h-8 bg-zinc-900 border-white/10 text-xs"
                                    />
                                    <p className="text-[9px] text-zinc-400">Number of candidate items to rerank from Fast phase.</p>
                                </div>
                            </div>
                        )}

                        {enableMRL && metric === "hybrid" && (
                            <div className="grid grid-cols-2 gap-4 p-3 rounded-lg bg-blue-500/5 border border-blue-500/10 animate-in fade-in slide-in-from-top-1 duration-200">
                                <div className="space-y-2 col-span-2">
                                    <Label htmlFor="hybrid-mrl-cutoff-select" className="text-xs">Hybrid Cutoff Dimension</Label>
                                    <div className="flex flex-col gap-1.5">
                                        <Select 
                                            value={isCustomMrlCutoff ? "custom" : mrlCutoff} 
                                            onValueChange={(v) => {
                                                if (v === "custom") {
                                                    setIsCustomMrlCutoff(true)
                                                } else {
                                                    setIsCustomMrlCutoff(false)
                                                    setMrlCutoff(v)
                                                }
                                            }}
                                        >
                                            <SelectTrigger id="hybrid-mrl-cutoff-select" className="h-8 bg-zinc-900 border-white/10 text-xs">
                                                <SelectValue placeholder="Select Hybrid Cascade Cutoff" />
                                            </SelectTrigger>
                                            <SelectContent>
                                                <SelectItem value="33" className="text-xs">Hyperbolic Only (33d) — Extreme speed</SelectItem>
                                                <SelectItem value="161" className="text-xs">Balanced Hybrid (161d) — 33 Lorentz + 128 L2</SelectItem>
                                                <SelectItem value="321" className="text-xs">Deep Hybrid (321d) — 33 Lorentz + 288 L2</SelectItem>
                                                <SelectItem value="custom" className="text-xs">Custom Hybrid Cutoff...</SelectItem>
                                            </SelectContent>
                                        </Select>

                                        {isCustomMrlCutoff && (
                                            <Input
                                                id="mrlCutoff"
                                                type="number"
                                                min="1"
                                                max="800"
                                                value={mrlCutoff}
                                                onChange={(e) => setMrlCutoff(e.target.value)}
                                                placeholder="Enter custom cutoff (e.g. 96)"
                                                className="h-8 bg-zinc-900 border-white/10 text-xs"
                                            />
                                        )}
                                    </div>
                                    <p className="text-[9px] text-zinc-400">
                                        Select the cascade prefix boundary optimized for YAR v5 hybrid architecture.
                                    </p>
                                </div>
                                <div className="space-y-2 col-span-2">
                                    <Label htmlFor="mrlRerankTopK" className="text-xs">Rerank Top K</Label>
                                    <Input
                                        id="mrlRerankTopK"
                                        type="number"
                                        min="1"
                                        value={mrlRerankTopK}
                                        onChange={(e) => setMrlRerankTopK(e.target.value)}
                                        className="h-8 bg-zinc-900 border-white/10 text-xs"
                                    />
                                    <p className="text-[9px] text-zinc-400">Number of candidate items to rerank from Fast phase.</p>
                                </div>
                            </div>
                        )}
                    </div>

                    <div className="p-3 rounded-lg bg-amber-500/10 border border-amber-500/20 text-[11px] text-amber-600 dark:text-amber-400">
                        <p className="font-bold mb-1">Architecture Warning:</p>
                        Selected dimensions and metrics must match the capabilities of your embedding model (e.g. 129d for YAR Lorentz).
                    </div>
                </div>
                <DialogFooter>
                    <Button variant="outline" onClick={() => setOpen(false)}>Cancel</Button>
                    <Button onClick={handleCreate} disabled={!isValid() || mutation.isPending}>
                        {mutation.isPending ? "Creating..." : "Create Collection"}
                    </Button>
                </DialogFooter>
            </DialogContent>
        </Dialog>
    )
}

function CollectionStatsDialog({ collectionName }: { collectionName: string }) {
    const [open, setOpen] = useState(false)
    const queryClient = useQueryClient()

    const { data: stats, isLoading } = useQuery({
        queryKey: ['stats', collectionName],
        queryFn: () => getCollectionStats(collectionName),
        enabled: open,
        refetchInterval: 5000
    })

    const configMutation = useMutation({
        mutationFn: (config: any) => updateCollectionConfig(collectionName, config),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['stats', collectionName] })
            alert("Configuration updated!")
        }
    })

    const formatBytes = (bytes: number) => {
        if (!bytes) return "0 B"
        const k = 1024
        const sizes = ['B', 'KB', 'MB', 'GB', 'TB']
        const i = Math.floor(Math.log(bytes) / Math.log(k))
        return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
    }

    return (
        <Dialog open={open} onOpenChange={setOpen}>
            <DialogTrigger asChild>
                <DropdownMenuItem onSelect={(e) => { e.preventDefault(); setOpen(true); }}>
                    <BarChart3 className="mr-2 h-4 w-4" /> Resource & Config
                </DropdownMenuItem>
            </DialogTrigger>
            <DialogContent className="sm:max-w-[580px] bg-zinc-950 border-white/10">
                <DialogHeader>
                    <DialogTitle className="flex items-center gap-2">
                        <Database className="h-5 w-5 text-primary" />
                        {collectionName}
                    </DialogTitle>
                    <DialogDescription>Performance stats, HNSW tuning, and Cache control.</DialogDescription>
                </DialogHeader>

                {isLoading ? (
                    <div className="py-10 flex flex-col items-center justify-center gap-3">
                        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
                        <span className="text-xs text-muted-foreground">Gathering telemetry...</span>
                    </div>
                ) : stats && (
                    <Tabs defaultValue="hnsw" className="w-full">
                        <TabsList className="w-full mb-4 bg-zinc-900">
                            <TabsTrigger value="hnsw" className="flex-1 text-xs"><Settings2 className="h-3 w-3 mr-1.5" />HNSW</TabsTrigger>
                            <TabsTrigger value="cache" className="flex-1 text-xs"><Layers className="h-3 w-3 mr-1.5" />Hot Cache</TabsTrigger>
                            <TabsTrigger value="resources" className="flex-1 text-xs"><HardDrive className="h-3 w-3 mr-1.5" />Resources</TabsTrigger>
                        </TabsList>

                        {/* ── HNSW Tab ─────────────────────────────────── */}
                        <TabsContent value="hnsw" className="space-y-4 py-1">
                            {/* Live values badge row */}
                            <div className="flex items-center gap-3 p-2 rounded-lg bg-zinc-900/60 border border-white/5 mb-2">
                                <div className="text-[10px] text-zinc-400 uppercase font-bold tracking-wider">Live values:</div>
                                <div className="flex gap-3 text-xs font-mono">
                                    <span className="text-emerald-400">EF Search: <strong>{stats.ef_search ?? '…'}</strong></span>
                                    <span className="text-sky-400">EF Const: <strong>{stats.ef_construction ?? '…'}</strong></span>
                                    <span className="text-purple-400">M: <strong>{stats.m ?? '…'}</strong></span>
                                </div>
                            </div>
                            <div className="flex items-center gap-2 text-zinc-400 border-b border-white/5 pb-2">
                                <Settings2 className="h-4 w-4" />
                                <h4 className="text-sm font-semibold">HNSW Configuration (Dynamic)</h4>
                            </div>
                            <ConfigField
                                label="EF Search"
                                description="Search depth — higher = better recall, higher latency (Hot-patchable)"
                                value={stats.indexing_queue > 0 ? "⏳ Indexing in progress..." : `Current: ${stats.ef_search ?? '?'}`}
                                onSave={(val: string) => configMutation.mutate({ ef_search: parseInt(val) })}
                            />
                            <ConfigField
                                label="EF Construction"
                                description="Build quality — affects NEW points only (not existing graph)"
                                value={`Current: ${stats.ef_construction ?? '?'}`}
                                onSave={(val: string) => configMutation.mutate({ ef_construction: parseInt(val) })}
                            />
                            <ConfigField
                                label="M (Max Connections)"
                                description="Graph degree — requires index Rebuild if changed (restart unsafe)"
                                value={`Current: ${stats.m ?? '?'}`}
                                onSave={(val: string) => configMutation.mutate({ m: parseInt(val) })}
                            />
                            <div className="p-3 rounded bg-blue-500/5 border border-blue-500/20 text-[10px] text-blue-400 leading-relaxed">
                                <strong>EF Search</strong> takes effect immediately — no restart needed.<br />
                                <strong>EF Construction &amp; M</strong> only affect vectors inserted after the change.
                                Changing M without a full rebuild leaves the existing graph at the old degree.
                            </div>
                        </TabsContent>

                        {/* ── Cache Tab ────────────────────────────────── */}
                        <TabsContent value="cache">
                            <CacheControlPanel collectionName={collectionName} formatBytes={formatBytes} />
                        </TabsContent>

                        {/* ── Resources Tab ────────────────────────────── */}
                        <TabsContent value="resources" className="space-y-4 py-1">
                            <div className="grid grid-cols-2 gap-4">
                                <div className="p-4 rounded-lg bg-zinc-900 border border-white/5 space-y-1">
                                    <div className="flex items-center gap-2 text-zinc-400">
                                        <HardDrive className="h-3 w-3" />
                                        <span className="text-[10px] uppercase font-bold tracking-wider">Disk Usage</span>
                                    </div>
                                    <div className="text-xl font-mono font-bold text-blue-400">
                                        {formatBytes(stats.usage?.disk_bytes)}
                                    </div>
                                </div>
                                <div className="p-4 rounded-lg bg-zinc-900 border border-white/5 space-y-1">
                                    <div className="flex items-center gap-2 text-zinc-400">
                                        <Cpu className="h-3 w-3" />
                                        <span className="text-[10px] uppercase font-bold tracking-wider">RAM (Est.)</span>
                                    </div>
                                    <div className="text-xl font-mono font-bold text-purple-400">
                                        {formatBytes(stats.usage?.ram_bytes)}
                                    </div>
                                </div>
                            </div>
                            <div className="grid grid-cols-2 gap-4">
                                <div className="p-3 rounded bg-zinc-900 border border-white/5 space-y-1.5">
                                    <div className="text-[10px] uppercase font-bold tracking-wider text-zinc-400">HNSW Indexing Queue</div>
                                    <div className="text-2xl font-mono font-bold text-white">{stats.indexing_queue ?? 0}</div>
                                    <div className="text-[10px] text-zinc-500">Vectors awaiting graph integration</div>
                                </div>
                                <div className="p-3 rounded bg-zinc-900 border border-white/5 space-y-1.5">
                                    <div className="text-[10px] uppercase font-bold tracking-wider text-zinc-400">Write Buffer Status</div>
                                    <div className="text-2xl font-mono font-bold text-emerald-400">{stats.write_buffer_size ?? 0}</div>
                                    <div className="text-[10px] text-zinc-500">Unindexed vectors available for searching</div>
                                </div>
                            </div>
                            <div className="p-3 rounded bg-zinc-900 border border-white/5 space-y-1">
                                <div className="text-[10px] uppercase font-bold tracking-wider text-zinc-400 mb-1">Quantization Mode</div>
                                <div className="flex items-center gap-2">
                                    <span className="text-sm font-mono font-bold text-amber-400">
                                        {(() => {
                                            const q = stats.quantization ?? ''
                                            if (q === 'None') return 'None — Full f64 precision'
                                            if (q === 'ScalarI8') return 'Medium — ScalarI8 (8-bit)'
                                            if (q === 'AsymmetricHybrid801') return 'Medium — Hybrid 801 (Lorentz-aware)'
                                            if (q === 'Binary') return 'Extreme — Binary (1-bit)'
                                            return q || '—'
                                        })()}
                                    </span>
                                </div>
                                <div className="text-[10px] text-zinc-500">
                                    {stats.dimension}d · {stats.metric}
                                </div>
                            </div>
                        </TabsContent>
                    </Tabs>
                )}
            </DialogContent>
        </Dialog>
    )
}

function ConfigField({ label, description, value, onSave }: any) {
    const [inputValue, setInputValue] = useState("")

    return (
        <div className="flex items-center justify-between gap-4">
            <div className="flex-1">
                <div className="text-sm font-medium">{label}</div>
                <div className="text-[11px] text-muted-foreground">{description}</div>
                <div className="text-[10px] font-mono text-zinc-500 mt-0.5">{value}</div>
            </div>
            <div className="flex items-center gap-2">
                <Input
                    className="h-8 w-20 text-xs bg-zinc-900 border-white/10"
                    placeholder="New val"
                    value={inputValue}
                    onChange={(e) => setInputValue(e.target.value)}
                />
                <Button variant="outline" size="sm" className="h-8 px-2 text-[10px]" onClick={() => onSave(inputValue)}>Set</Button>
            </div>
        </div>
    )
}

// ─── Cache Control Panel ────────────────────────────────────────────────────

function CacheControlPanel({ collectionName, formatBytes }: { collectionName: string; formatBytes: (b: number) => string }) {
    const queryClient = useQueryClient()
    const [policy, setPolicy] = useState("lru")
    const [annThreshold, setAnnThreshold] = useState("")
    const [ttlSeconds] = useState("300")
    const [copied, setCopied] = useState(false)

    const { data: cache, isLoading } = useQuery({
        queryKey: ['cache-stats', collectionName],
        queryFn: () => getCacheStats(collectionName),
        refetchInterval: 3000,
        retry: false,
    })

    const clearMutation = useMutation({
        mutationFn: () => clearCache(collectionName),
        onSuccess: () => queryClient.invalidateQueries({ queryKey: ['cache-stats', collectionName] })
    })

    const configMutation = useMutation({
        mutationFn: ({ p, t }: { p: string; t: string }) =>
            updateCacheConfig(collectionName, p, t ? parseFloat(t) : null),
        onSuccess: () => queryClient.invalidateQueries({ queryKey: ['cache-stats', collectionName] })
    })

    const hitRateBar = (r: number) => (
        <div className="flex items-center gap-2">
            <div className="flex-1 h-1.5 rounded-full bg-zinc-800 overflow-hidden">
                <div
                    className={`h-full rounded-full transition-all duration-500 ${
                        r >= 0.8 ? "bg-emerald-500" : r >= 0.5 ? "bg-amber-500" : "bg-red-500"
                    }`}
                    style={{ width: `${Math.min(r * 100, 100)}%` }}
                />
            </div>
            <span className={`text-xs font-mono font-bold ${
                r >= 0.8 ? "text-emerald-400" : r >= 0.5 ? "text-amber-400" : "text-red-400"
            }`}>{(r * 100).toFixed(1)}%</span>
        </div>
    )

    if (isLoading) return <div className="py-8 text-center text-xs text-zinc-500">Loading cache stats...</div>

    if (!cache) return (
        <div className="py-6 rounded-lg border border-dashed border-zinc-700 text-center text-xs text-zinc-500 space-y-1">
            <Layers className="h-6 w-6 mx-auto mb-2 opacity-40" />
            <p>Cache disabled or collection not active.</p>
            <p className="text-zinc-600">Set <code className="text-zinc-500">HYPERSPACE_CACHE_ENABLED=true</code> to enable.</p>
        </div>
    )

    return (
        <div className="space-y-4 py-1">
            {/* Live Stats */}
            <div className="grid grid-cols-2 gap-3">
                <div className="rounded-lg bg-zinc-900 border border-white/5 p-3 space-y-2">
                    <div className="flex items-center gap-1.5 text-zinc-400">
                        <TrendingUp className="h-3 w-3" />
                        <span className="text-[10px] uppercase font-bold tracking-wider">L1 Hit Rate</span>
                    </div>
                    {hitRateBar(cache.l1_hit_rate ?? 0)}
                    <div className="text-[10px] text-zinc-500">{(cache.l1_size ?? 0).toLocaleString()} vectors in L1</div>
                </div>
                <div className="rounded-lg bg-zinc-900 border border-white/5 p-3 space-y-2">
                    <div className="flex items-center gap-1.5 text-zinc-400">
                        <TrendingUp className="h-3 w-3" />
                        <span className="text-[10px] uppercase font-bold tracking-wider">L2 Hit Rate</span>
                    </div>
                    {hitRateBar(cache.l2_hit_rate ?? 0)}
                    <div className="text-[10px] text-zinc-500">{(cache.l2_index_size ?? 0).toLocaleString()} in ANN graph</div>
                </div>
                <div className="rounded-lg bg-zinc-900 border border-white/5 p-3">
                    <div className="text-[10px] uppercase font-bold tracking-wider text-zinc-400 mb-1">Cache RAM</div>
                    <div className="text-lg font-mono font-bold text-purple-400">{formatBytes(cache.estimated_memory_bytes ?? 0)}</div>
                    <div className="text-[10px] text-zinc-500">L1 hot tier</div>
                </div>
                <div className="rounded-lg bg-zinc-900 border border-white/5 p-3">
                    <div className="text-[10px] uppercase font-bold tracking-wider text-zinc-400 mb-1">Tombstones / Pending</div>
                    <div className="text-lg font-mono font-bold">
                        <span className={cache.tombstone_count > 0 ? "text-amber-400" : "text-zinc-500"}>{cache.tombstone_count ?? 0}</span>
                        <span className="text-zinc-600 mx-1">/</span>
                        <span className={cache.pending_rebuild > 0 ? "text-sky-400" : "text-zinc-500"}>{cache.pending_rebuild ?? 0}</span>
                    </div>
                    <div className="text-[10px] text-zinc-500">cleanup / rebuild queue</div>
                </div>
            </div>

            {/* Hot Controls */}
            <div className="border-t border-white/5 pt-3 space-y-3">
                <div className="text-[10px] uppercase font-bold tracking-wider text-zinc-400 flex items-center gap-1.5">
                    <Settings2 className="h-3 w-3" /> Hot Reconfiguration
                </div>

                <div className="grid grid-cols-2 gap-3">
                    <div className="space-y-1.5">
                        <Label className="text-[11px] text-zinc-400">Eviction Policy</Label>
                        <Select value={policy} onValueChange={setPolicy}>
                            <SelectTrigger className="h-8 text-xs bg-zinc-900 border-white/10">
                                <SelectValue />
                            </SelectTrigger>
                            <SelectContent>
                                <SelectItem value="lru" className="text-xs">LRU — Least Recently Used</SelectItem>
                                <SelectItem value="lfu" className="text-xs">LFU — Least Frequently Used</SelectItem>
                                <SelectItem value="ttl" className="text-xs">TTL — Time-based expiry</SelectItem>
                            </SelectContent>
                        </Select>
                    </div>
                    <div className="space-y-1.5">
                        <Label className="text-[11px] text-zinc-400">ANN Threshold <span className="text-zinc-600">(blank = disable L2)</span></Label>
                        <Input
                            className="h-8 text-xs bg-zinc-900 border-white/10"
                            placeholder="e.g. 0.15"
                            value={annThreshold}
                            onChange={e => setAnnThreshold(e.target.value)}
                        />
                    </div>
                </div>

                <div className="flex gap-2">
                    <Button
                        size="sm"
                        className="flex-1 h-8 text-xs"
                        onClick={() => configMutation.mutate({ p: policy, t: annThreshold })}
                        disabled={configMutation.isPending}
                    >
                        Apply Config
                    </Button>
                    <Button
                        size="sm"
                        variant="destructive"
                        className="h-8 text-xs px-3"
                        onClick={() => {
                            if (window.confirm(`Flush hot cache for '${collectionName}'? Next requests will hit disk until warmed.`)) {
                                clearMutation.mutate()
                            }
                        }}
                        disabled={clearMutation.isPending}
                    >
                        <FlameKindling className="h-3.5 w-3.5 mr-1.5" />
                        Flush Cache
                    </Button>
                </div>
            </div>

            {/* TTL Guide */}
            <div className="rounded-lg bg-amber-500/5 border border-amber-500/15 p-3 space-y-2">
                <div className="flex items-center gap-1.5 text-amber-400 text-[10px] uppercase font-bold tracking-wider">
                    <Clock className="h-3 w-3" /> Per-Vector TTL
                </div>
                <p className="text-[10px] text-zinc-400 leading-relaxed">
                    TTL is set per-vector at insert time via the <code className="text-amber-300/80">__ttl</code> metadata key
                    (value in seconds). The cache automatically evicts expired vectors every 100 ms.
                </p>
                <div className="rounded bg-zinc-900 border border-white/5 p-2">
                    <code className="text-[10px] text-emerald-400 font-mono block">
                        {`insert(id=42, vector=[...], metadata={"__ttl": "300"})`}
                    </code>
                    <div className="text-[9px] text-zinc-500 mt-1">→ expires in 5 minutes</div>
                </div>
                <Button
                    variant="ghost"
                    size="sm"
                    className="h-6 px-2 text-[10px] text-zinc-500 hover:text-zinc-300"
                    onClick={() => {
                        navigator.clipboard.writeText(`insert(id=42, vector=[...], metadata={"__ttl": "${ttlSeconds || 300}"})`)
                        setCopied(true)
                        setTimeout(() => setCopied(false), 1500)
                    }}
                >
                    {copied ? "✓ Copied!" : "Copy snippet"}
                </Button>
            </div>
        </div>
    )
}

function TableSkeleton() {
    return Array(3).fill(0).map((_, i) => (
        <TableRow key={i}><TableCell><Skeleton className="h-4 w-20" /></TableCell><TableCell><Skeleton className="h-4 w-12" /></TableCell><TableCell><Skeleton className="h-4 w-10" /></TableCell><TableCell><Skeleton className="h-4 w-10" /></TableCell><TableCell><Skeleton className="h-4 w-10" /></TableCell><TableCell><Skeleton className="h-4 w-10" /></TableCell><TableCell><Skeleton className="h-4 w-8" /></TableCell></TableRow>
    ))
}
