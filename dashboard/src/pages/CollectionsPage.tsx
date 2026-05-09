import { useState } from "react"
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query"
import { api, fetchStatus } from "@/lib/api"
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table"
import { Button } from "@/components/ui/button"
import { Plus, Trash2, MoreHorizontal, Database, Search, HardDrive, Cpu, Settings2, BarChart3 } from "lucide-react"
import { getCollectionStats, updateCollectionConfig } from "@/lib/api"
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogTrigger, DialogFooter, DialogDescription } from "@/components/ui/dialog"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { DropdownMenu, DropdownMenuTrigger, DropdownMenuContent, DropdownMenuItem, DropdownMenuLabel, DropdownMenuSeparator } from "@/components/ui/dropdown-menu"
import { Skeleton } from "@/components/ui/skeleton"
import { Badge } from "@/components/ui/badge"
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"
import { useNavigate } from "react-router-dom"
import { useEffect } from "react"

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
                            <TableHead className="w-[300px]">Name</TableHead>
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
                            <TableRow><TableCell colSpan={6} className="text-center h-32 text-muted-foreground">No collections found. Create one to get started.</TableCell></TableRow>
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
    const count = isString ? "-" : collection.count
    const dim = isString ? "-" : collection.dimension
    const metric = isString ? "-" : collection.metric

    const navigate = useNavigate()

    return (
        <TableRow>
            <TableCell className="font-medium flex items-center gap-2">
                <div className="p-1.5 rounded bg-primary/10 text-primary">
                    <Database className="h-4 w-4" />
                </div>
                {name}
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
                        <DropdownMenuItem onClick={() => {
                            if (window.confirm(`Are you sure you want to rebuild index for '${name}'? This is a heavy operation.`)) {
                                api.post(`/collections/${name}/rebuild`)
                                    .then(() => alert("Index rebuild started!"))
                                    .catch(e => alert("Failed: " + e.message))
                            }
                        }}>
                            <Database className="mr-2 h-4 w-4" /> Rebuild Index
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
        mutation.mutate({
            name,
            dimension: parseInt(dimension) || 1024,
            metric: metric || "l2"
        })
    }

    const getDimensions = () => {
        const large_base = [8, 16, 32, 64, 128, 768, 1024, 1536, 2048, 3072, 4096, 8192]
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

    return (
        <Dialog open={open} onOpenChange={setOpen}>
            <DialogTrigger asChild>
                <Button><Plus className="mr-2 h-4 w-4" /> New Collection</Button>
            </DialogTrigger>
            <DialogContent className="sm:max-w-[425px]">
                <DialogHeader>
                    <DialogTitle>Create Collection</DialogTitle>
                    <DialogDescription>Add a new vector index to the system.</DialogDescription>
                </DialogHeader>
                <div className="grid gap-6 py-4">
                    <div className="space-y-2">
                        <Label htmlFor="name">Collection Name</Label>
                        <Input id="name" value={name} onChange={e => setName(e.target.value)} placeholder="e.g. user_embeddings" />
                    </div>

                    <div className="grid grid-cols-2 gap-4">
                        <div className="space-y-2">
                            <Label htmlFor="metric">Metric</Label>
                            <Select value={metric} onValueChange={(v) => {
                                setMetric(v)
                                // Auto-adjust dimension to N+1 if switching to lorentz
                                const current = parseInt(dimension)
                                if (v === "lorentz") {
                                    if (current % 128 === 0 || [4, 8, 16, 32, 64].includes(current)) {
                                        if (current <= 128) setDimension((current + 1).toString())
                                        else setDimension("129")
                                    }
                                } else if (metric === "lorentz") {
                                    if (current % 128 === 1 || [5, 9, 17, 33, 65].includes(current)) {
                                        setDimension((current - 1).toString())
                                    }
                                } else if (v === "poincare") {
                                    if (current > 128) setDimension("128")
                                }
                            }}>
                                <SelectTrigger id="metric">
                                    <SelectValue placeholder="Select" />
                                </SelectTrigger>
                                <SelectContent>
                                    <SelectItem value="l2">Euclidean (L2)</SelectItem>
                                    <SelectItem value="cosine">Cosine</SelectItem>
                                    <SelectItem value="poincare">Poincaré</SelectItem>
                                    <SelectItem value="lorentz">Lorentz (Hyperbolic)</SelectItem>
                                </SelectContent>
                            </Select>
                        </div>
                        <div className="space-y-2">
                            <Label htmlFor="dimension">Dimension</Label>
                            <Select value={dimension} onValueChange={setDimension}>
                                <SelectTrigger id="dimension">
                                    <SelectValue placeholder="Select" />
                                </SelectTrigger>
                                <SelectContent>
                                    {getDimensions().map(d => (
                                        <SelectItem key={d} value={d.toString()}>{d}</SelectItem>
                                    ))}
                                </SelectContent>
                            </Select>
                        </div>
                    </div>

                    <div className="p-3 rounded-lg bg-amber-500/10 border border-amber-500/20 text-[11px] text-amber-600 dark:text-amber-400">
                        <p className="font-bold mb-1">Architecture Warning:</p>
                        Selected dimensions and metrics must match the capabilities of your embedding model (e.g. 129d for YAR Lorentz).
                    </div>
                </div>
                <DialogFooter>
                    <Button variant="outline" onClick={() => setOpen(false)}>Cancel</Button>
                    <Button onClick={handleCreate} disabled={!name || mutation.isPending}>
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
            <DialogContent className="sm:max-w-[500px] bg-zinc-950 border-white/10">
                <DialogHeader>
                    <DialogTitle className="flex items-center gap-2">
                        <Database className="h-5 w-5 text-primary" />
                        {collectionName} — Performance Stats
                    </DialogTitle>
                    <DialogDescription>Real-time resource usage and HNSW tuning.</DialogDescription>
                </DialogHeader>

                {isLoading ? (
                    <div className="py-10 flex flex-col items-center justify-center gap-3">
                        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
                        <span className="text-xs text-muted-foreground">Gathering telemetry...</span>
                    </div>
                ) : stats && (
                    <div className="space-y-6 py-4">
                        {/* Resource Grid */}
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

                        {/* Config Editor */}
                        <div className="space-y-4">
                            <div className="flex items-center gap-2 text-zinc-400 border-b border-white/5 pb-2">
                                <Settings2 className="h-4 w-4" />
                                <h4 className="text-sm font-semibold">HNSW Configuration (Dynamic)</h4>
                            </div>

                            <ConfigField 
                                label="EF Search" 
                                description="Search depth (Hot-patchable)" 
                                value={stats.indexing_queue > 0 ? "Indexing..." : "Current: " + (stats.ef_search || 100)}
                                onSave={(val) => configMutation.mutate({ ef_search: parseInt(val) })}
                            />
                            
                            <ConfigField 
                                label="EF Construction" 
                                description="Build quality for NEW points" 
                                value="Current: " + (stats.ef_construction || 100)
                                onSave={(val) => configMutation.mutate({ ef_construction: parseInt(val) })}
                            />

                            <ConfigField 
                                label="M (Max Connections)" 
                                description="Graph degree (Requires Rebuild if changed)" 
                                value="Current: " + (stats.m || 16)
                                onSave={(val) => configMutation.mutate({ m: parseInt(val) })}
                            />
                        </div>

                        <div className="p-3 rounded bg-blue-500/5 border border-blue-500/20 text-[10px] text-blue-400 leading-relaxed">
                            <strong>Tip:</strong> Increasing EF Search improves recall but adds latency. 
                            Changes to M and EF Construction only affect vectors inserted AFTER the change.
                        </div>
                    </div>
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

function TableSkeleton() {
    return Array(3).fill(0).map((_, i) => (
        <TableRow key={i}><TableCell><Skeleton className="h-4 w-20" /></TableCell><TableCell><Skeleton className="h-4 w-10" /></TableCell><TableCell><Skeleton className="h-4 w-10" /></TableCell><TableCell><Skeleton className="h-4 w-10" /></TableCell><TableCell><Skeleton className="h-4 w-10" /></TableCell><TableCell><Skeleton className="h-4 w-8" /></TableCell></TableRow>
    ))
}
