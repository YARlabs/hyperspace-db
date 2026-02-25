import { useQuery } from "@tanstack/react-query"
import { api, fetchStatus } from "@/lib/api"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { Card, CardContent, CardHeader, CardTitle, CardDescription, CardFooter } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { ScrollArea } from "@/components/ui/scroll-area"
import { Copy, Download, Archive, RefreshCw, HardDrive, Settings, Code, Terminal, Cloud } from "lucide-react"

export function SettingsPage() {
    const { data: status } = useQuery({ queryKey: ['status'], queryFn: fetchStatus })
    const apiKey = localStorage.getItem("hyperspace_api_key") || "YOUR_KEY"

    return (
        <div className="space-y-6 fade-in">
            <div>
                <h1 className="text-3xl font-bold tracking-tight text-white mb-2">Control Plane</h1>
                <p className="text-muted-foreground">Manage system configuration, storage tiers, and SDK integrations.</p>
            </div>

            <Tabs defaultValue="integration" className="w-full">
                <TabsList className="grid w-full grid-cols-4 bg-zinc-950/50 border border-white/5 mb-6">
                    <TabsTrigger value="integration" className="data-[state=active]:bg-zinc-800"><Code className="w-4 h-4 mr-2" /> SDK & Integration</TabsTrigger>
                    <TabsTrigger value="config" className="data-[state=active]:bg-zinc-800"><Settings className="w-4 h-4 mr-2" /> Dynamic Config</TabsTrigger>
                    <TabsTrigger value="storage" className="data-[state=active]:bg-zinc-800"><HardDrive className="w-4 h-4 mr-2" /> Storage & S3</TabsTrigger>
                    <TabsTrigger value="logs" className="data-[state=active]:bg-zinc-800"><Terminal className="w-4 h-4 mr-2" /> Live Logs</TabsTrigger>
                </TabsList>

                <TabsContent value="config" className="space-y-6">
                    <div className="grid gap-6 md:grid-cols-2">
                        <Card className="bg-zinc-950/50 border-white/5 shadow-2xl backdrop-blur-sm">
                            <CardHeader>
                                <CardTitle className="text-white">Index Parameters (HNSW)</CardTitle>
                                <CardDescription>Tune vector search precision and speed on the fly.</CardDescription>
                            </CardHeader>
                            <CardContent className="space-y-4">
                                <div className="space-y-2">
                                    <label className="text-sm font-medium text-zinc-300">Default `ef_search`</label>
                                    <input type="number" defaultValue="200" className="flex h-10 w-full rounded-md border border-white/10 bg-zinc-900 px-3 py-2 text-sm text-white" />
                                </div>
                                <div className="space-y-2">
                                    <label className="text-sm font-medium text-zinc-300">Default `ef_construction`</label>
                                    <input type="number" defaultValue="200" className="flex h-10 w-full rounded-md border border-white/10 bg-zinc-900 px-3 py-2 text-sm text-white" />
                                </div>
                            </CardContent>
                            <CardFooter>
                                <Button className="bg-white text-black hover:bg-zinc-200">Apply Changes</Button>
                            </CardFooter>
                        </Card>

                        <Card className="bg-zinc-950/50 border-white/5 shadow-2xl backdrop-blur-sm">
                            <CardHeader>
                                <CardTitle className="text-white">Jemalloc Settings</CardTitle>
                                <CardDescription>Control RAM reclamation aggressiveness.</CardDescription>
                            </CardHeader>
                            <CardContent className="space-y-4">
                                <div className="space-y-2">
                                    <label className="text-sm font-medium text-zinc-300">MALLOC_CONF</label>
                                    <input type="text" defaultValue="background_thread:true,dirty_decay_ms:5000" className="flex h-10 w-full rounded-md border border-white/10 bg-zinc-900 px-3 py-2 text-sm text-white font-mono" />
                                </div>
                            </CardContent>
                            <CardFooter>
                                <Button className="bg-white text-black hover:bg-zinc-200">Reload Allocator</Button>
                            </CardFooter>
                        </Card>
                    </div>
                </TabsContent>

                <TabsContent value="storage" className="space-y-6">
                    <div className="grid gap-6 md:grid-cols-2">
                        <Card className="bg-zinc-950/50 border-white/5 shadow-2xl backdrop-blur-sm">
                            <CardHeader>
                                <CardTitle className="flex items-center text-white"><HardDrive className="mr-2" /> Global Storage Ops</CardTitle>
                                <CardDescription>Manage local NVMe WAL and HNSW segments.</CardDescription>
                            </CardHeader>
                            <CardContent className="space-y-4">
                                <Button variant="outline" className="w-full justify-start border-white/10 text-zinc-300 hover:text-white" onClick={() => api.post('/admin/vacuum').catch(() => alert('Vacuum requires implementation/admin access'))}>
                                    <RefreshCw className="mr-2 h-4 w-4" /> Trigger Hot Vacuum (Defrag)
                                </Button>
                                <Button variant="outline" className="w-full justify-start border-white/10 text-zinc-300 hover:text-white">
                                    <Download className="mr-2 h-4 w-4" /> Download Local Snapshot (.hyp)
                                </Button>
                                <Button variant="outline" className="w-full justify-start border-white/10 text-zinc-300 hover:text-white">
                                    <Archive className="mr-2 h-4 w-4" /> Restore from Snapshot
                                </Button>
                            </CardContent>
                        </Card>

                        <Card className="bg-zinc-950/50 border-white/5 shadow-2xl backdrop-blur-sm">
                            <CardHeader>
                                <CardTitle className="flex items-center text-white"><Cloud className="mr-2" /> S3 Tiering (Cold Storage)</CardTitle>
                                <CardDescription>Offload unused chunks to MinIO / AWS S3.</CardDescription>
                            </CardHeader>
                            <CardContent className="space-y-4">
                                <div className="rounded-lg border border-white/5 bg-zinc-900/50 p-4 mb-4">
                                    <div className="flex justify-between text-sm mb-2">
                                        <span className="text-zinc-400">S3 Integration</span>
                                        <span className="text-yellow-500 font-bold">Unconfigured</span>
                                    </div>
                                    <p className="text-xs text-zinc-500">Enable feature `s3-tiering` and set HS_STORAGE_BACKEND=s3</p>
                                </div>
                                <Button variant="default" disabled className="w-full bg-blue-600 text-white hover:bg-blue-700">
                                    Force Evict to S3
                                </Button>
                            </CardContent>
                        </Card>
                    </div>
                </TabsContent>

                <TabsContent value="integration" className="space-y-6">
                    <Card className="bg-zinc-950/50 border-white/5 shadow-2xl backdrop-blur-sm">
                        <CardHeader>
                            <CardTitle className="text-white">Interactive SDK Playground</CardTitle>
                            <CardDescription>Generated snippets automatically use your current API key and active collections.</CardDescription>
                        </CardHeader>
                        <CardContent>
                            <ClientSnippets apiKey={apiKey} dimension={status?.config?.dimension || 1024} />
                        </CardContent>
                    </Card>
                </TabsContent>

                <TabsContent value="logs" className="space-y-6">
                    <Card className="bg-zinc-950/50 border-white/5 shadow-2xl backdrop-blur-sm">
                        <CardHeader>
                            <CardTitle className="text-white">Streaming Event Log</CardTitle>
                            <CardDescription>Live output from the HyperspaceDB core.</CardDescription>
                        </CardHeader>
                        <CardContent>
                            <ScrollArea className="h-[500px] w-full rounded-md border border-white/5 bg-zinc-950 font-mono text-xs text-green-400 shadow-inner p-4">
                                <LogViewer />
                            </ScrollArea>
                        </CardContent>
                    </Card>
                </TabsContent>
            </Tabs>
        </div>
    )
}

function ClientSnippets({ apiKey, dimension }: any) {
    const snippet_py = `import hyperspace

client = hyperspace.HyperspaceClient("localhost:50051", api_key="${apiKey}")

# 1. Delta Sync Handshake (Edge to Cloud)
resp = client.sync_handshake(
    collection="default", 
    client_buckets=[0]*256
)
print(f"Differing Buckets: {len(resp['diff_buckets'])}")

# 2. Insert vector (Cognitive Math support)
client.insert(
    id=1, 
    vector=[0.5] * ${dimension}, 
    collection="default", 
    metadata={"type": "agent_memory"}
)

# 3. Hybrid Search
res = client.search(
    vector=[0.5] * ${dimension}, 
    top_k=5, 
    collection="default"
)
print([hit.id for hit in res])`

    const snippet_ts_sdk = `import { DatabaseClient } from "hyperspace-sdk-ts";

const client = new DatabaseClient("localhost:50051", "${apiKey}");

// Subscribe to CDC Events continuously
const stream = client.subscribeToEvents({ types: ['insert'] }, (event) => {
    console.log("Vector inserted with ID:", event.id);
});

// Perform Search
const results = await client.search(
    new Array(${dimension}).fill(0.1),
    10, // topK
    "default"
);
console.log(results);`

    const snippet_rust = `use hyperspace_sdk::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = Client::connect(
        "http://localhost:50051".to_string(),
        Some("${apiKey}".to_string()),
        None
    ).await?;

    // Upsert a vector node
    client.insert(
        42,
        vec![0.1; ${dimension}],
        std::collections::HashMap::new(),
        Some("default".to_string())
    ).await?;

    Ok(())
}`

    const copyToClipboard = (text: string) => {
        navigator.clipboard.writeText(text)
    }

    return (
        <Tabs defaultValue="python" className="w-full">
            <TabsList className="w-full justify-start bg-zinc-900 border border-white/5 mb-4">
                <TabsTrigger value="python" className="data-[state=active]:bg-zinc-800">Python</TabsTrigger>
                <TabsTrigger value="typescript" className="data-[state=active]:bg-zinc-800">TypeScript</TabsTrigger>
                <TabsTrigger value="rust" className="data-[state=active]:bg-zinc-800">Rust Core</TabsTrigger>
            </TabsList>
            <div className="relative group">
                <TabsContent value="python" className="mt-0">
                    <CodeBlock code={snippet_py} language="python" onCopy={() => copyToClipboard(snippet_py)} />
                </TabsContent>
                <TabsContent value="typescript" className="mt-0">
                    <CodeBlock code={snippet_ts_sdk} language="typescript" onCopy={() => copyToClipboard(snippet_ts_sdk)} />
                </TabsContent>
                <TabsContent value="rust" className="mt-0">
                    <CodeBlock code={snippet_rust} language="rust" onCopy={() => copyToClipboard(snippet_rust)} />
                </TabsContent>
            </div>
        </Tabs>
    )
}

function CodeBlock({ code, onCopy }: any) {
    return (
        <div className="relative">
            <pre className="p-4 rounded-lg bg-[#0d1117] text-[#c9d1d9] overflow-x-auto text-sm font-mono border border-white/10 shadow-xl">
                {code}
            </pre>
            <Button variant="ghost" size="icon" className="absolute top-2 right-2 text-zinc-400 hover:text-white bg-white/5 backdrop-blur-sm hover:bg-white/10" onClick={onCopy}>
                <Copy className="h-4 w-4" />
            </Button>
        </div>
    )
}

function LogViewer() {
    const { data: logs } = useQuery({
        queryKey: ['logs'],
        queryFn: async () => {
            try {
                const r = await api.get("/logs");
                return r.data;
            } catch (e) {
                return ["[WARN] Log stream unreachable. Ensure backend is running."];
            }
        },
        refetchInterval: 3000
    })

    if (!logs) return <div className="text-zinc-500 animate-pulse">Establishing secure connection...</div>

    return (
        <div className="space-y-1 opacity-90">
            {Array.isArray(logs) && logs.map((l: string, i: number) => (
                <div key={i} className="hover:bg-zinc-900/50 px-2 py-0.5 rounded transition-colors opacity-90"><span className="text-zinc-500 mr-2">{new Date().toISOString().split('T')[1].split('.')[0]}</span> {l}</div>
            ))}
        </div>
    )
}
