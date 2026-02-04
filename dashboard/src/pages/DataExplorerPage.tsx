import { useState, useEffect } from "react"
import { useQuery, useMutation } from "@tanstack/react-query"
import { api } from "@/lib/api"
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table"
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Label } from "@/components/ui/label"
import { Skeleton } from "@/components/ui/skeleton"
import { useSearchParams } from "react-router-dom"
import { Code, Play, AlertCircle } from "lucide-react"

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
    const { data: items, isLoading } = useQuery({
        queryKey: ['peek', collection],
        queryFn: () => api.get(`/collections/${collection}/peek?limit=50`).then(r => r.data),
        enabled: !!collection
    })

    if (isLoading) return <div className="space-y-2"><Skeleton className="h-10 w-full" /><Skeleton className="h-10 w-full" /></div>

    return (
        <Card className="h-full flex flex-col">
            <CardHeader>
                <CardTitle>Recent Vectors (Last 50)</CardTitle>
                <CardDescription>Verify your data ingestion pipeline</CardDescription>
            </CardHeader>
            <CardContent className="flex-1 overflow-auto">
                <div className="rounded-md border">
                    <Table>
                        <TableHeader>
                            <TableRow>
                                <TableHead className="w-[80px]">ID</TableHead>
                                <TableHead>Vector (Prefix)</TableHead>
                                <TableHead>Metadata</TableHead>
                            </TableRow>
                        </TableHeader>
                        <TableBody>
                            {!items || items.length === 0 ? (
                                <TableRow><TableCell colSpan={3} className="text-center h-24 text-muted-foreground">No vectors found (or index empty)</TableCell></TableRow>
                            ) : (
                                items.map(([id, vec, meta]: any) => (
                                    <TableRow key={id}>
                                        <TableCell className="font-mono text-xs">{id}</TableCell>
                                        <TableCell className="font-mono text-xs text-muted-foreground">
                                            [{vec?.slice(0, 5).map((n: number) => n.toFixed(4)).join(", ")}...]
                                        </TableCell>
                                        <TableCell>
                                            <pre className="text-[10px] text-muted-foreground">{JSON.stringify(meta, null, 2)}</pre>
                                        </TableCell>
                                    </TableRow>
                                ))
                            )}
                        </TableBody>
                    </Table>
                </div>
            </CardContent>
        </Card>
    )
}

function SearchPlayground({ collection }: { collection: string }) {
    const [vectorInput, setVectorInput] = useState("[0.1, 0.2, 0.3]")
    const [res, setRes] = useState<any>(null)
    const [error, setError] = useState("")

    const searchMutation = useMutation({
        mutationFn: (vec: number[]) => api.post(`/collections/${collection}/search`, { vector: vec, top_k: 5 }),
        onSuccess: (data) => { setRes(data.data); setError("") },
        onError: (err: any) => { setError(err.message || "Search Failed"); setRes(null) }
    })

    const handleSearch = () => {
        try {
            const parsed = JSON.parse(vectorInput)
            if (!Array.isArray(parsed)) throw new Error("Input must be an array")
            searchMutation.mutate(parsed)
        } catch (e: any) {
            setError("Invalid JSON format: " + e.message)
        }
    }

    return (
        <div className="grid gap-6 md:grid-cols-2">
            <Card>
                <CardHeader>
                    <CardTitle>Query Vector</CardTitle>
                    <CardDescription>Enter a raw JSON vector array</CardDescription>
                </CardHeader>
                <CardContent className="space-y-4">
                    <div className="grid w-full gap-2">
                        <Label htmlFor="vector">Vector JSON</Label>
                        <textarea
                            className="flex min-h-[150px] w-full rounded-md border border-input bg-transparent px-3 py-2 text-sm shadow-sm placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50 font-mono"
                            value={vectorInput}
                            onChange={(e) => setVectorInput(e.target.value)}
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
                    <CardDescription>Nearest neighbors (HNSW)</CardDescription>
                </CardHeader>
                <CardContent>
                    {res ? (
                        <div className="rounded-md border">
                            <Table>
                                <TableHeader>
                                    <TableRow>
                                        <TableHead>ID</TableHead>
                                        <TableHead>Distance</TableHead>
                                    </TableRow>
                                </TableHeader>
                                <TableBody>
                                    {res.map((r: any) => (
                                        <TableRow key={r.id}>
                                            <TableCell className="font-mono">{r.id}</TableCell>
                                            <TableCell className="font-mono text-green-400 font-bold">{r.distance.toFixed(6)}</TableCell>
                                        </TableRow>
                                    ))}
                                </TableBody>
                            </Table>
                        </div>
                    ) : (
                        <div className="flex h-[200px] items-center justify-center text-muted-foreground text-sm flex-col gap-2">
                            <Code className="h-8 w-8 opacity-20" />
                            Run a search to see k-NN results
                        </div>
                    )}
                </CardContent>
            </Card>
        </div>
    )
}
