import { useState } from "react"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Badge } from "@/components/ui/badge"
import {
    Copy, Check, Bot, Cpu, Brain, GitBranch, Coins, Wrench,
    ChevronRight, ExternalLink, Terminal, Package
} from "lucide-react"
import { cn } from "@/lib/utils"

const SKILLS = [
    {
        id: "core",
        name: "hyperspacedb-core",
        icon: Cpu,
        color: "text-blue-400",
        bg: "bg-blue-500/10 border-blue-500/20",
        description: "Collections, insert, search, point operations",
        triggers: ["create collection", "insert vector", "semantic search", "HyperspaceDB"],
    },
    {
        id: "graph",
        name: "hyperspacedb-graph",
        icon: GitBranch,
        color: "text-emerald-400",
        bg: "bg-emerald-500/10 border-emerald-500/20",
        description: "Graph traversal, Lorentz hierarchy, concept parents, clustering",
        triggers: ["knowledge graph", "traverse", "hierarchy", "ontology", "subsumption"],
    },
    {
        id: "cognitive",
        name: "hyperspacedb-cognitive",
        icon: Brain,
        color: "text-violet-400",
        bg: "bg-violet-500/10 border-violet-500/20",
        description: "CoT stability, Koopman momentum, trust score, Lyapunov analysis",
        triggers: ["thought stability", "CoT", "hallucination", "momentum", "trust score"],
    },
    {
        id: "depin",
        name: "hyperspacedb-depin",
        icon: Coins,
        color: "text-amber-400",
        bg: "bg-amber-500/10 border-amber-500/20",
        description: "DePIN nodes, billing, storage economics, replication",
        triggers: ["DePIN", "node", "miner", "billing", "storage fee"],
    },
    {
        id: "mcp",
        name: "hyperspacedb-mcp",
        icon: Wrench,
        color: "text-cyan-400",
        bg: "bg-cyan-500/10 border-cyan-500/20",
        description: "Full 30+ MCP tool reference, config, return types",
        triggers: ["MCP", "Claude Desktop", "Cursor MCP", "hyperspace_search"],
    },
]

const AGENTS = [
    { id: "cursor", label: "Cursor", emoji: "⚡" },
    { id: "claude", label: "Claude Code", emoji: "🤖" },
    { id: "windsurf", label: "Windsurf", emoji: "🏄" },
    { id: "custom", label: "Custom Agent", emoji: "🔧" },
]

function useCopy() {
    const [copiedId, setCopiedId] = useState<string | null>(null)
    const copy = (text: string, id: string) => {
        navigator.clipboard.writeText(text)
        setCopiedId(id)
        setTimeout(() => setCopiedId(null), 2000)
    }
    return { copy, copiedId }
}

function CopyBlock({ code, id, label }: { code: string; id: string; label?: string }) {
    const { copy, copiedId } = useCopy()
    const isCopied = copiedId === id
    return (
        <div className="relative group">
            {label && <p className="text-xs text-zinc-500 mb-1.5 font-mono">{label}</p>}
            <pre className="p-4 rounded-lg bg-[#0d1117] text-[#c9d1d9] overflow-x-auto text-sm font-mono border border-white/10 shadow-xl whitespace-pre-wrap break-all">
                {code}
            </pre>
            <Button
                variant="ghost" size="icon"
                className={cn(
                    "absolute top-2 right-2 h-7 w-7 text-zinc-400 hover:text-white bg-white/5 backdrop-blur-sm hover:bg-white/10 transition-all",
                    isCopied && "text-emerald-400 hover:text-emerald-400"
                )}
                onClick={() => copy(code, id)}
            >
                {isCopied ? <Check className="h-3.5 w-3.5" /> : <Copy className="h-3.5 w-3.5" />}
            </Button>
        </div>
    )
}

function Step({ number, title, children }: { number: number; title: string; children: React.ReactNode }) {
    return (
        <div className="flex gap-4">
            <div className="flex-shrink-0 h-7 w-7 rounded-full bg-white/10 border border-white/10 flex items-center justify-center text-sm font-bold text-white">
                {number}
            </div>
            <div className="flex-1 space-y-3">
                <h3 className="text-sm font-semibold text-white leading-7">{title}</h3>
                {children}
            </div>
        </div>
    )
}

export function AgentSkillsPage() {
    const host = window.location.hostname === "localhost"
        ? "localhost:50051"
        : `${window.location.hostname}:50051`
    const apiKey = localStorage.getItem("hyperspace_api_key") || "YOUR_API_KEY"

    const mcpConfig = JSON.stringify({
        mcpServers: {
            hyperspacedb: {
                command: "npx",
                args: ["-y", "mcp-hyperspacedb"],
                env: {
                    HYPERSPACE_HOST: host,
                    HYPERSPACE_API_KEY: apiKey,
                }
            }
        }
    }, null, 2)

    const cursorRulesSnippet =
        `cat node_modules/hyperspacedb-skills/agents/cursor-rules.md >> .cursorrules`

    const claudeSnippet =
        `cat node_modules/hyperspacedb-skills/agents/claude-code-instructions.md >> CLAUDE.md`

    const copySkillsSnippet =
        `mkdir -p .agents/skills\ncp -r node_modules/hyperspacedb-skills/skills/* .agents/skills/`

    const configLocations: Record<string, { path: string; extra?: string }> = {
        cursor: { path: ".cursor/mcp.json" },
        claude: { path: "~/.config/claude/mcp.json" },
        windsurf: { path: "~/.windsurf/mcp.json" },
        custom: { path: "your-agent-config.json" },
    }

    return (
        <div className="space-y-8 fade-in">
            {/* Header */}
            <div className="flex items-start justify-between">
                <div>
                    <div className="flex items-center gap-2 mb-2">
                        <Bot className="h-7 w-7 text-violet-400" />
                        <h1 className="text-3xl font-bold tracking-tight text-white">AI Agent Skills</h1>
                        <Badge variant="outline" className="border-violet-500/40 text-violet-400 text-xs">
                            v3.1.2
                        </Badge>
                    </div>
                    <p className="text-muted-foreground max-w-2xl">
                        Install <code className="text-violet-300 bg-violet-500/10 px-1.5 py-0.5 rounded text-xs font-mono">hyperspacedb-skills</code> to
                        teach Cursor, Claude Code, Windsurf, and custom AI agents how to correctly use this database.
                        Configs are auto-filled with your current host and API key.
                    </p>
                </div>
                <a
                    href="https://www.npmjs.com/package/hyperspacedb-skills"
                    target="_blank" rel="noopener noreferrer"
                    className="flex items-center gap-1.5 text-xs text-zinc-400 hover:text-white border border-white/10 rounded-md px-3 py-2 hover:bg-white/5 transition-colors"
                >
                    <Package className="h-3.5 w-3.5" />
                    npm
                    <ExternalLink className="h-3 w-3" />
                </a>
            </div>

            {/* Skills grid */}
            <div>
                <h2 className="text-sm font-semibold text-zinc-400 uppercase tracking-wider mb-4">Skills Included</h2>
                <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3">
                    {SKILLS.map(skill => (
                        <Card key={skill.id} className={cn("bg-zinc-950/50 border transition-colors", skill.bg)}>
                            <CardHeader className="pb-2 pt-4 px-4">
                                <div className="flex items-center gap-2">
                                    <skill.icon className={cn("h-4 w-4", skill.color)} />
                                    <CardTitle className="text-sm font-mono text-white">{skill.name}</CardTitle>
                                </div>
                                <CardDescription className="text-xs">{skill.description}</CardDescription>
                            </CardHeader>
                            <CardContent className="px-4 pb-4">
                                <div className="flex flex-wrap gap-1">
                                    {skill.triggers.map(t => (
                                        <span key={t} className="text-[10px] bg-white/5 text-zinc-400 px-1.5 py-0.5 rounded border border-white/5 font-mono">
                                            {t}
                                        </span>
                                    ))}
                                </div>
                            </CardContent>
                        </Card>
                    ))}
                </div>
            </div>

            {/* Main installer */}
            <Card className="bg-zinc-950/50 border-white/5 shadow-2xl">
                <CardHeader>
                    <CardTitle className="text-white flex items-center gap-2">
                        <Terminal className="h-4 w-4 text-cyan-400" />
                        Quick Installer
                    </CardTitle>
                    <CardDescription>
                        Select your AI agent. Configs are auto-filled with{" "}
                        <code className="text-cyan-300 text-xs">{host}</code>.
                    </CardDescription>
                </CardHeader>
                <CardContent>
                    <Tabs defaultValue="cursor" className="w-full">
                        <TabsList className="bg-zinc-900 border border-white/5 mb-6 w-full justify-start">
                            {AGENTS.map(a => (
                                <TabsTrigger key={a.id} value={a.id} className="data-[state=active]:bg-zinc-800 gap-1.5">
                                    <span>{a.emoji}</span> {a.label}
                                </TabsTrigger>
                            ))}
                        </TabsList>

                        {AGENTS.map(agent => (
                            <TabsContent key={agent.id} value={agent.id} className="space-y-6 mt-0">
                                {/* Step 1 */}
                                <Step number={1} title="Install the skills package">
                                    <CopyBlock
                                        code="npm install hyperspacedb-skills"
                                        id={`install-${agent.id}`}
                                    />
                                </Step>

                                {/* Step 2: agent-specific */}
                                <Step number={2} title={
                                    agent.id === "cursor" ? "Copy skills rules" :
                                    agent.id === "claude" ? "Append to CLAUDE.md" :
                                    agent.id === "windsurf" ? "Copy skills to .agents/" :
                                    "Copy skills to your project"
                                }>
                                    {agent.id === "cursor" && (
                                        <div className="space-y-3">
                                            <CopyBlock
                                                code={cursorRulesSnippet}
                                                id={`rules-${agent.id}`}
                                                label="Option A: .cursorrules (quick)"
                                            />
                                            <CopyBlock
                                                code={copySkillsSnippet}
                                                id={`skills-${agent.id}`}
                                                label="Option B: .agents/skills/ (full)"
                                            />
                                        </div>
                                    )}
                                    {agent.id === "claude" && (
                                        <CopyBlock
                                            code={claudeSnippet}
                                            id={`claude-md-${agent.id}`}
                                        />
                                    )}
                                    {(agent.id === "windsurf" || agent.id === "custom") && (
                                        <CopyBlock
                                            code={copySkillsSnippet}
                                            id={`skills-dir-${agent.id}`}
                                        />
                                    )}
                                </Step>

                                {/* Step 3: MCP config */}
                                <Step number={3} title={`Add MCP config → ${configLocations[agent.id].path}`}>
                                    <CopyBlock
                                        code={mcpConfig}
                                        id={`mcp-${agent.id}`}
                                    />
                                    <p className="text-xs text-zinc-500">
                                        <ChevronRight className="inline h-3 w-3" />
                                        {" "}Host <code className="text-cyan-300">{host}</code> and API key are pre-filled from your current session.
                                    </p>
                                </Step>

                                {/* Done */}
                                <div className="rounded-lg border border-emerald-500/20 bg-emerald-500/5 p-4">
                                    <p className="text-sm text-emerald-400 font-medium">
                                        ✅ Done! Restart your agent/IDE and try:
                                    </p>
                                    <p className="text-xs text-zinc-400 mt-1 font-mono">
                                        "Create a HyperspaceDB collection for my research papers with cosine metric"
                                    </p>
                                    <p className="text-xs text-zinc-500 mt-0.5 font-mono">
                                        "Check if my reasoning chain [1,2,3,4,5] is stable or hallucinating"
                                    </p>
                                </div>
                            </TabsContent>
                        ))}
                    </Tabs>
                </CardContent>
            </Card>

            {/* MCP tools reference */}
            <Card className="bg-zinc-950/50 border-white/5">
                <CardHeader>
                    <CardTitle className="text-white text-base">Available MCP Tools (30+)</CardTitle>
                    <CardDescription>All tools are accessible once MCP is connected</CardDescription>
                </CardHeader>
                <CardContent>
                    <div className="grid grid-cols-1 md:grid-cols-2 gap-2 text-xs font-mono">
                        {[
                            ["Collections", "hyperspace_list_collections, hyperspace_create_collection, hyperspace_delete_collection, hyperspace_freeze_collection, hyperspace_unfreeze_collection"],
                            ["Data", "hyperspace_insert_text, hyperspace_delete_points, hyperspace_get_points"],
                            ["Search", "hyperspace_search_text (+ hybridAlpha), hyperspace_search_wasserstein"],
                            ["Graph", "hyperspace_get_neighbors, hyperspace_graph_traverse, hyperspace_explore_graph, hyperspace_get_subsumption_tree, hyperspace_get_concept_parents, hyperspace_find_clusters"],
                            ["Cognitive AI", "hyperspace_analyze_thought_stability, hyperspace_predict_momentum, hyperspace_get_trust_score, hyperspace_analyze_geometry"],
                            ["System", "hyperspace_get_stats, hyperspace_rebuild_index, hyperspace_vacuum, hyperspace_trigger_reconsolidation, hyperspace_cache_stats, hyperspace_cache_clear, hyperspace_cache_config"],
                        ].map(([category, tools]) => (
                            <div key={category} className="space-y-1">
                                <p className="text-zinc-400 font-semibold text-[10px] uppercase tracking-wider">{category}</p>
                                <p className="text-zinc-500 leading-relaxed">{tools}</p>
                            </div>
                        ))}
                    </div>
                </CardContent>
            </Card>
        </div>
    )
}
