import { Button } from "@/components/ui/button"
import { Network } from "lucide-react"

export function GraphExplorerPage() {
    return (
        <div className="flex flex-col items-center justify-center h-full text-center space-y-6 fade-in">
            <div className="relative">
                <div className="absolute inset-0 bg-primary/20 blur-xl rounded-full"></div>
                <div className="rounded-full bg-background border p-8 relative z-10">
                    <Network className="h-16 w-16 text-primary" />
                </div>
            </div>
            <h1 className="text-4xl font-bold tracking-tight bg-gradient-to-br from-white to-white/50 bg-clip-text text-transparent">Graph Explorer</h1>
            <p className="text-muted-foreground text-lg max-w-[600px]">
                Visualize the HNSW graph structure in 3D Hyperbolic Space. Inspect node connections and nearest neighbors visually.
            </p>
            <div className="pt-4 space-y-4">
                <p className="text-xs uppercase tracking-widest text-muted-foreground">Release Target: v1.4 (Alpha)</p>
                <Button disabled>Join Waitlist</Button>
            </div>
        </div>
    )
}
