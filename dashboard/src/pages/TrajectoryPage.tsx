import React, { useEffect, useState, useRef } from "react"
import { 
    Activity, 
    Share2, 
    Zap, 
    Clock, 
    Maximize2, 
    Play, 
    Square,
    Waves
} from "lucide-react"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Badge } from "@/components/ui/badge"
import { ScrollArea } from "@/components/ui/scroll-area"
import { cn } from "@/lib/utils"
import { fetchTrajectoryHistory } from "@/lib/api"

interface TrajectoryPoint {
    id: string;
    x: number;
    y: number;
    timestamp: number;
    metadata: any;
    resonance?: number;
}

interface Ripple {
    id: string;
    x: number;
    y: number;
    radius: number;
    opacity: number;
    createdAt: number;
    resonance?: number;
}

export function TrajectoryPage() {
    const [allPoints, setAllPoints] = useState<TrajectoryPoint[]>([]);
    const [ripples, setRipples] = useState<Ripple[]>([]);
    const [visiblePointsCount, setVisiblePointsCount] = useState(0);
    const [isStreaming, setIsStreaming] = useState(false);
    const canvasRef = useRef<HTMLCanvasElement>(null);
    const eventSourceRef = useRef<EventSource | null>(null);

    useEffect(() => {
        fetchHistory();
    }, []);

    const fetchHistory = async () => {
        try {
            const data = await fetchTrajectoryHistory();
            const history = data.map((msg: any) => ({
                id: msg.id || Math.random().toString(),
                x: msg.x || (Math.random() * 2 - 1),
                y: msg.y || (Math.random() * 2 - 1),
                timestamp: Date.now(),
                metadata: msg.metadata || {}
            }));
            setAllPoints(history);
            setVisiblePointsCount(history.length);
        } catch (err) {
            console.error("Failed to fetch history", err);
        }
    };

    useEffect(() => {
        if (isStreaming) {
            startStreaming();
        } else {
            stopStreaming();
        }
        return () => stopStreaming();
    }, [isStreaming]);

    const startStreaming = () => {
        const es = new EventSource("/api/admin/trajectory/stream");
        es.onmessage = (event) => {
            const msg = JSON.parse(event.data);
            const newPoint: TrajectoryPoint = {
                id: msg.id || Math.random().toString(),
                x: msg.x || (Math.random() * 2 - 1),
                y: msg.y || (Math.random() * 2 - 1),
                timestamp: Date.now(),
                metadata: msg.metadata || {},
                resonance: msg.metadata?.resonance || (Math.random() * 0.5) // Fallback for demo
            };

            setRipples(prev => [...prev, {
                id: Math.random().toString(),
                x: newPoint.x,
                y: newPoint.y,
                radius: 0,
                opacity: 0.8,
                createdAt: Date.now(),
                resonance: newPoint.resonance
            }]);

            setAllPoints(prev => {
                const updated = [...prev.slice(-1000), newPoint];
                // If we were at the end, keep following the stream
                if (visiblePointsCount === prev.length) {
                    setVisiblePointsCount(updated.length);
                }
                return updated;
            });
        };
        eventSourceRef.current = es;
    };

    const stopStreaming = () => {
        if (eventSourceRef.current) {
            eventSourceRef.current.close();
            eventSourceRef.current = null;
        }
    };

    const visiblePoints = allPoints.slice(0, visiblePointsCount);

    // Rendering logic for Poincaré Disk
    useEffect(() => {
        const canvas = canvasRef.current;
        if (!canvas) return;
        const ctx = canvas.getContext('2d');
        if (!ctx) return;

        const render = () => {
            const w = canvas.width;
            const h = canvas.height;
            const centerX = w / 2;
            const centerY = h / 2;
            const radius = Math.min(w, h) / 2 - 20;

            ctx.clearRect(0, 0, w, h);

            // Draw Poincaré Disk Border
            ctx.beginPath();
            ctx.arc(centerX, centerY, radius, 0, Math.PI * 2);
            ctx.strokeStyle = 'rgba(0, 243, 255, 0.2)';
            ctx.lineWidth = 2;
            ctx.stroke();
            
            // Draw Grid Lines
            ctx.strokeStyle = 'rgba(255, 255, 255, 0.05)';
            ctx.lineWidth = 1;
            for (let r = 0.2; r < 1; r += 0.2) {
                ctx.beginPath();
                ctx.arc(centerX, centerY, radius * r, 0, Math.PI * 2);
                ctx.stroke();
            }

            // Draw Path
            if (visiblePoints.length > 1) {
                ctx.beginPath();
                ctx.moveTo(
                    centerX + visiblePoints[0].x * radius,
                    centerY + visiblePoints[0].y * radius
                );
                
                for (let i = 1; i < visiblePoints.length; i++) {
                    const p = visiblePoints[i];
                    ctx.lineTo(
                        centerX + p.x * radius,
                        centerY + p.y * radius
                    );
                }
                
                const gradient = ctx.createLinearGradient(0, 0, w, h);
                gradient.addColorStop(0, '#00f3ff');
                gradient.addColorStop(1, '#7000ff');
                ctx.strokeStyle = gradient;
                ctx.lineWidth = 3;
                ctx.lineJoin = 'round';
                ctx.stroke();
            }

            // Draw current point (Glow)
            if (visiblePoints.length > 0) {
                const last = visiblePoints[visiblePoints.length - 1];
                const px = centerX + last.x * radius;
                const py = centerY + last.y * radius;

                ctx.shadowBlur = 15;
                ctx.shadowColor = '#00f3ff';
                ctx.fillStyle = '#fff';
                ctx.beginPath();
                ctx.arc(px, py, 4, 0, Math.PI * 2);
                ctx.fill();
                ctx.shadowBlur = 0;
            }

            // Draw Ripples (Wave Diffusion)
            const now = Date.now();
            setRipples(prev => {
                const stillActive = prev.filter(r => now - r.createdAt < 2000);
                stillActive.forEach(r => {
                    const elapsed = now - r.createdAt;
                    const rippleRadius = (elapsed / 2000) * radius * 0.5;
                    const rippleOpacity = Math.max(0, 0.6 * (1 - elapsed / 2000));
                    
                    ctx.beginPath();
                    ctx.arc(
                        centerX + r.x * radius,
                        centerY + r.y * radius,
                        rippleRadius,
                        0, Math.PI * 2
                    );
                    ctx.strokeStyle = `rgba(0, 243, 255, ${rippleOpacity})`;
                    ctx.lineWidth = 1;
                    ctx.stroke();

                    // Inner resonant ring
                    ctx.beginPath();
                    ctx.arc(
                        centerX + r.x * radius,
                        centerY + r.y * radius,
                        rippleRadius * 0.7,
                        0, Math.PI * 2
                    );
                    ctx.strokeStyle = `rgba(112, 0, 255, ${rippleOpacity * 0.5})`;
                    ctx.stroke();

                    // Task 10: Hybrid Resonance Visualization
                    if (r.resonance && r.resonance > 0.6) {
                        ctx.beginPath();
                        ctx.arc(
                            centerX + r.x * radius,
                            centerY + r.y * radius,
                            rippleRadius * (1.0 + r.resonance * 0.5),
                            0, Math.PI * 2
                        );
                        ctx.strokeStyle = `rgba(255, 255, 0, ${rippleOpacity * r.resonance})`;
                        ctx.setLineDash([5, 5]);
                        ctx.stroke();
                        ctx.setLineDash([]);
                    }
                });
                return stillActive;
            });

            requestAnimationFrame(render);
        };

        const animId = requestAnimationFrame(render);
        return () => cancelAnimationFrame(animId);
    }, [visiblePoints]);

    return (
        <div className="space-y-8 animate-in fade-in duration-700 pb-20">
            <div className="flex flex-col gap-2">
                <div className="flex items-center gap-2 text-primary">
                    <Waves className="h-5 w-5" />
                    <span className="text-sm font-semibold uppercase tracking-wider">Cognitive Engine</span>
                </div>
                <div className="flex justify-between items-center">
                    <h1 className="text-4xl font-bold tracking-tight">Trajectory Visualizer</h1>
                    <div className="flex gap-2">
                        <Button 
                            variant="outline"
                            onClick={fetchHistory}
                            className="gap-2"
                        >
                            <Clock className="h-4 w-4" />
                            Load History
                        </Button>
                        <Button 
                            variant={isStreaming ? "destructive" : "default"}
                            onClick={() => setIsStreaming(!isStreaming)}
                            className="gap-2"
                        >
                            {isStreaming ? <Square className="h-4 w-4" /> : <Play className="h-4 w-4" />}
                            {isStreaming ? "Stop Monitoring" : "Start Real-time Sync"}
                        </Button>
                    </div>
                </div>
                <p className="text-muted-foreground text-lg max-w-2xl">
                    Observing agent "Chain of Thought" through Klein-Gordon diffusion patterns in Lorentz space.
                </p>
            </div>

            <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
                <Card className="lg:col-span-2 bg-black/40 border-white/5 shadow-2xl relative overflow-hidden backdrop-blur-xl">
                    <div className="absolute top-0 inset-x-0 h-px bg-gradient-to-r from-transparent via-primary/50 to-transparent" />
                    <CardHeader className="flex flex-row items-center justify-between">
                        <div>
                            <CardTitle className="text-xl flex items-center gap-2">
                                <Maximize2 className="h-4 w-4 text-primary" />
                                Poincaré Projection
                            </CardTitle>
                            <CardDescription>33D Lorentz ➜ 2D Hyperbolic Disk</CardDescription>
                        </div>
                        <div className="flex items-center gap-4">
                            <Badge variant="outline" className="bg-primary/5 text-primary border-primary/20">
                                {isStreaming ? "Live Feed Active" : "Time Travel Mode"}
                            </Badge>
                        </div>
                    </CardHeader>
                    <CardContent className="flex flex-col items-center justify-center p-0 pb-8">
                        <canvas 
                            ref={canvasRef} 
                            width={600} 
                            height={600} 
                            className="max-w-full h-auto drop-shadow-[0_0_50px_rgba(0,243,255,0.1)]"
                        />
                        
                        {/* Time Slider Overlay */}
                        <div className="w-full px-12 mt-4 space-y-2">
                            <div className="flex justify-between text-[10px] uppercase tracking-widest text-muted-foreground font-mono">
                                <span>Past</span>
                                <span>Present</span>
                            </div>
                            <input 
                                type="range" 
                                min={0} 
                                max={allPoints.length} 
                                value={visiblePointsCount}
                                onChange={(e) => {
                                    setIsStreaming(false);
                                    setVisiblePointsCount(parseInt(e.target.value));
                                }}
                                className="w-full h-1.5 bg-white/10 rounded-lg appearance-none cursor-pointer accent-primary"
                            />
                            <div className="text-center text-[10px] text-primary/50 font-mono">
                                Step {visiblePointsCount} of {allPoints.length}
                            </div>
                        </div>
                    </CardContent>
                </Card>

                <div className="space-y-6">
                    <Card className="bg-card/50 backdrop-blur-md">
                        <CardHeader>
                            <CardTitle className="text-sm flex items-center gap-2">
                                <Activity className="h-4 w-4 text-primary" />
                                Semantic Resonance
                            </CardTitle>
                        </CardHeader>
                        <CardContent className="space-y-4">
                            <div className="flex justify-between items-center p-3 rounded-lg bg-white/5 border border-white/5">
                                <span className="text-xs text-muted-foreground">Momentum</span>
                                <span className="font-mono text-emerald-400">0.942 λ</span>
                            </div>
                            <div className="flex justify-between items-center p-3 rounded-lg bg-white/5 border border-white/5">
                                <span className="text-xs text-muted-foreground">Entropy</span>
                                <span className="font-mono text-amber-400">0.021 Δ</span>
                            </div>
                            <div className="flex justify-between items-center p-3 rounded-lg bg-white/5 border border-white/5">
                                <span className="text-xs text-muted-foreground">Inertia</span>
                                <span className="font-mono text-primary">High</span>
                            </div>
                        </CardContent>
                    </Card>

                    <Card className="bg-black/20 border-white/5 h-[400px]">
                        <CardHeader className="pb-2">
                            <CardTitle className="text-xs font-mono uppercase text-muted-foreground flex items-center gap-2">
                                <Clock className="h-3 w-3" />
                                Event Timeline
                            </CardTitle>
                        </CardHeader>
                        <CardContent className="p-0">
                            <ScrollArea className="h-[320px] px-4">
                                <div className="space-y-4">
                                    {visiblePoints.slice().reverse().map((p, i) => (
                                        <div key={p.id} className="relative pl-4 border-l border-white/10 py-1 animate-in slide-in-from-left-2 duration-300">
                                            <div className="absolute left-[-5px] top-2 h-2 w-2 rounded-full bg-primary" />
                                            <p className="text-[10px] text-muted-foreground">
                                                {new Date(p.timestamp).toLocaleTimeString()}
                                            </p>
                                            <p className="text-sm font-medium text-white/90">
                                                Step {visiblePoints.length - i}: Concept Hop
                                            </p>
                                            <p className="text-[10px] text-primary/70 font-mono">
                                                Coord: ({p.x.toFixed(3)}, {p.y.toFixed(3)})
                                            </p>
                                        </div>
                                    ))}
                                </div>
                            </ScrollArea>
                        </CardContent>
                    </Card>
                </div>
            </div>
        </div>
    )
}
