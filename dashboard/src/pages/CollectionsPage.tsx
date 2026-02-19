import { useState } from "react"
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query"
import { api, fetchStatus } from "@/lib/api"
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table"
import { Button } from "@/components/ui/button"
import { Plus, Trash2, MoreHorizontal, Database, Search } from "lucide-react"
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
                            <Label htmlFor="dimension">Dimension</Label>
                            <Select value={dimension} onValueChange={setDimension}>
                                <SelectTrigger id="dimension">
                                    <SelectValue placeholder="Select" />
                                </SelectTrigger>
                                <SelectContent>
                                    {[8, 16, 32, 64, 128, 768, 1024, 1536, 2048].map(d => (
                                        <SelectItem key={d} value={d.toString()}>{d}</SelectItem>
                                    ))}
                                </SelectContent>
                            </Select>
                        </div>
                        <div className="space-y-2">
                            <Label htmlFor="metric">Metric</Label>
                            <Select value={metric} onValueChange={setMetric}>
                                <SelectTrigger id="metric">
                                    <SelectValue placeholder="Select" />
                                </SelectTrigger>
                                <SelectContent>
                                    <SelectItem value="l2">Euclidean (L2)</SelectItem>
                                    <SelectItem value="cosine">Cosine</SelectItem>
                                    <SelectItem value="poincare">Poincaré</SelectItem>
                                </SelectContent>
                            </Select>
                        </div>
                    </div>

                    <div className="p-3 rounded-lg bg-amber-500/10 border border-amber-500/20 text-[11px] text-amber-600 dark:text-amber-400">
                        <p className="font-bold mb-1">Architecture Warning:</p>
                        Current gRPC implementation requires dimensions and metrics to match pre-compiled templates for maximum performance.
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

function TableSkeleton() {
    return Array(3).fill(0).map((_, i) => (
        <TableRow key={i}><TableCell><Skeleton className="h-4 w-20" /></TableCell><TableCell><Skeleton className="h-4 w-10" /></TableCell><TableCell><Skeleton className="h-4 w-10" /></TableCell><TableCell><Skeleton className="h-4 w-10" /></TableCell><TableCell><Skeleton className="h-4 w-10" /></TableCell><TableCell><Skeleton className="h-4 w-8" /></TableCell></TableRow>
    ))
}
