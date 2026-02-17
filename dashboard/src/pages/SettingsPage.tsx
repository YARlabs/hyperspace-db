import { useQuery } from "@tanstack/react-query"
import { api, fetchStatus } from "@/lib/api"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { ScrollArea } from "@/components/ui/scroll-area"
import { Copy, Download, Archive } from "lucide-react"

export function SettingsPage() {
    const { data: status } = useQuery({ queryKey: ['status'], queryFn: fetchStatus })
    const apiKey = localStorage.getItem("hyperspace_api_key") || "YOUR_KEY"

    return (
        <div className="space-y-6 fade-in">
            <h1 className="text-3xl font-bold tracking-tight">Settings & Integration</h1>

            <div className="grid gap-6 md:grid-cols-2">
                <div className="space-y-6">
                    <Card className="h-full">
                        <CardHeader>
                            <CardTitle>Integration Snippets</CardTitle>
                            <CardDescription>Connect your agents to Hyperspace</CardDescription>
                        </CardHeader>
                        <CardContent>
                            <ClientSnippets apiKey={apiKey} dimension={status?.config?.dimension || 1024} />
                        </CardContent>
                    </Card>
                </div>

                <div className="space-y-6">
                    <Card>
                        <CardHeader><CardTitle>Live Server Logs</CardTitle><CardDescription>Real-time events</CardDescription></CardHeader>
                        <CardContent>
                            <ScrollArea className="h-[300px] w-full rounded-md border p-4 bg-zinc-950 font-mono text-xs text-green-500 shadow-inner">
                                <LogViewer />
                            </ScrollArea>
                        </CardContent>
                    </Card>

                    <Card>
                        <CardHeader><CardTitle>Backup & Persistence</CardTitle><CardDescription>Manage local snapshots</CardDescription></CardHeader>
                        <CardContent className="flex gap-4">
                            <Button variant="outline"><Download className="mr-2 h-4 w-4" /> Download WAL</Button>
                            <Button variant="outline" disabled><Archive className="mr-2 h-4 w-4" /> Restore Snapshot</Button>
                        </CardContent>
                    </Card>
                </div>
            </div>
        </div>
    )
}

function ClientSnippets({ apiKey, dimension }: any) {
    const snippet_py = `import requests

API_URL = "http://localhost:50050/api"
HEADERS = {"x-api-key": "${apiKey}"}

# 1. Create Collection
requests.post(f"{API_URL}/collections", json={
    "name": "agents_memory",
    "dimension": ${dimension},
    "metric": "l2"
}, headers=HEADERS)

# 2. Insert Vector (Not exposed in HTTP yet, use gRPC for high Perf)
# ...`

    const snippet_curl = `curl -X GET http://localhost:50050/api/cluster/status \\
  -H "x-api-key: ${apiKey}"`

    const snippet_js = `import axios from "axios";

const client = axios.create({
  baseURL: "http://localhost:50050/api",
  headers: { "x-api-key": "${apiKey}" },
});

const status = await client.get("/cluster/status");
console.log(status.data);`

    const copyToClipboard = (text: string) => {
        navigator.clipboard.writeText(text)
    }

    return (
        <Tabs defaultValue="python" className="w-full">
            <TabsList className="w-full justify-start">
                <TabsTrigger value="python">Python</TabsTrigger>
                <TabsTrigger value="curl">cURL</TabsTrigger>
                <TabsTrigger value="js">Node.js</TabsTrigger>
            </TabsList>
            <div className="mt-4 relative group">
                <TabsContent value="python">
                    <CodeBlock code={snippet_py} language="python" onCopy={() => copyToClipboard(snippet_py)} />
                </TabsContent>
                <TabsContent value="curl">
                    <CodeBlock code={snippet_curl} language="bash" onCopy={() => copyToClipboard(snippet_curl)} />
                </TabsContent>
                <TabsContent value="js">
                    <CodeBlock code={snippet_js} language="javascript" onCopy={() => copyToClipboard(snippet_js)} />
                </TabsContent>
            </div>
        </Tabs>
    )
}

function CodeBlock({ code, onCopy }: any) {
    return (
        <div className="relative">
            <pre className="p-4 rounded-lg bg-secondary/50 overflow-x-auto text-sm font-mono border border-border/50">
                {code}
            </pre>
            <Button variant="ghost" size="icon" className="absolute top-2 right-2 text-muted-foreground hover:text-foreground bg-background/50 backdrop-blur-sm" onClick={onCopy}>
                <Copy className="h-4 w-4" />
            </Button>
        </div>
    )
}

function LogViewer() {
    const { data: logs } = useQuery({
        queryKey: ['logs'],
        queryFn: async () => {
            // Basic fallback enabled to prevent white screen if backend not updated yet
            try {
                const r = await api.get("/logs");
                return r.data;
            } catch (e) {
                return ["[WARN] Log stream unreachable"];
            }
        },
        refetchInterval: 3000
    })

    if (!logs) return <div>Connecting...</div>

    return (
        <div className="space-y-1">
            {Array.isArray(logs) && logs.map((l: string, i: number) => <div key={i}>{l}</div>)}
        </div>
    )
}
