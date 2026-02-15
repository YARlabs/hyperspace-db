import { useState } from "react"
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query"
import { api } from "@/lib/api"
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table"
import { Button } from "@/components/ui/button"
import { Plus, Trash2, MoreHorizontal, Database, Search } from "lucide-react"
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogTrigger, DialogFooter, DialogDescription } from "@/components/ui/dialog"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { DropdownMenu, DropdownMenuTrigger, DropdownMenuContent, DropdownMenuItem, DropdownMenuLabel, DropdownMenuSeparator } from "@/components/ui/dropdown-menu"
import { Skeleton } from "@/components/ui/skeleton"
import { Badge } from "@/components/ui/badge"
import { useNavigate } from "react-router-dom"

export function CollectionsPage() {
    const queryClient = useQueryClient()
    const { data: collections, isLoading } = useQuery({
        queryKey: ['collections'],
        queryFn: () => api.get("/collections").then(r => r.data)
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
                            if (confirm(`Are you sure you want to rebuild index for '${name}'? This is a heavy operation.`)) {
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
    const queryClient = useQueryClient()

    // Get global config to show locked values
    const { data: status } = useQuery({ queryKey: ['status'], queryFn: () => api.get("/status").then(r => r.data) })

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
            dimension: parseInt(status?.config?.dimension) || 1024,
            metric: status?.config?.metric || "l2"
        })
    }

    return (
        <Dialog open={open} onOpenChange={setOpen}>
            <DialogTrigger asChild>
                <Button><Plus className="mr-2 h-4 w-4" /> New Collection</Button>
            </DialogTrigger>
            <DialogContent>
                <DialogHeader>
                    <DialogTitle>Create Collection</DialogTitle>
                    <DialogDescription>Add a new vector index to the system.</DialogDescription>
                </DialogHeader>
                <div className="grid gap-4 py-4">
                    <div className="grid grid-cols-4 items-center gap-4">
                        <Label htmlFor="name" className="text-right">Name</Label>
                        <Input id="name" value={name} onChange={e => setName(e.target.value)} className="col-span-3" placeholder="my_vectors" />
                    </div>
                    {status && (
                        <>
                            <div className="grid grid-cols-4 items-center gap-4 text-sm">
                                <span className="text-right text-muted-foreground">Dimension</span>
                                <span className="col-span-3 font-mono bg-muted px-2 py-1 rounded w-fit text-xs text-muted-foreground">{status.config.dimension} (Locked)</span>
                            </div>
                            <div className="grid grid-cols-4 items-center gap-4 text-sm">
                                <span className="text-right text-muted-foreground">Metric</span>
                                <span className="col-span-3 font-mono bg-muted px-2 py-1 rounded w-fit text-xs text-muted-foreground">{status.config.metric} (Locked)</span>
                            </div>
                        </>
                    )}
                </div>
                <DialogFooter>
                    <Button variant="outline" onClick={() => setOpen(false)}>Cancel</Button>
                    <Button onClick={handleCreate} disabled={!name}>Create</Button>
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
