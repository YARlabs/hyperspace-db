import { useEffect, useState, useRef } from "react"
import { 
    Activity,
    Clock, 
    Maximize2, 
    Play, 
    Square,
    Waves,
    Terminal,
    ArrowRight
} from "lucide-react"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Badge } from "@/components/ui/badge"
import { ScrollArea } from "@/components/ui/scroll-area"
import { fetchTrajectoryHistory, fetchAgentRuns, fetchAgentRunById } from "@/lib/api"

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

interface RunStep {
    x: number;
    y: number;
    timestamp: number;
    metadata: any;
}

interface AgentRun {
    session_id: string;
    task_description: string;
    status: string;
    steps: RunStep[];
    created_at: number;
    completed_at?: number;
    total_latency_ms: number;
    lyapunov_stability: number;
    trust_score: number;
}

export function TrajectoryPage() {
    const [allPoints, setAllPoints] = useState<TrajectoryPoint[]>([]);
    const [_ripples, setRipples] = useState<Ripple[]>([]);
    const [visiblePointsCount, setVisiblePointsCount] = useState(0);
    const [isStreaming, setIsStreaming] = useState(false);
    const [runs, setRuns] = useState<AgentRun[]>([]);
    const [selectedRun, setSelectedRun] = useState<AgentRun | null>(null);
    
    const canvasRef = useRef<HTMLCanvasElement>(null);
    const eventSourceRef = useRef<EventSource | null>(null);

    useEffect(() => {
        loadRuns();
        fetchHistory();
    }, []);

    const loadRuns = async () => {
        try {
            const data = await fetchAgentRuns();
            setRuns(data);
        } catch (err) {
            console.error("Failed to load agent runs", err);
        }
    };

    const selectRun = async (sessionId: string) => {
        setIsStreaming(false);
        try {
            const run = await fetchAgentRunById(sessionId);
            setSelectedRun(run);
            
            // Map run steps to trajectory points
            const points = (run.steps || []).map((step: any, idx: number) => ({
                id: `${run.session_id}_step_${idx}`,
                x: step.x,
                y: step.y,
                timestamp: step.timestamp,
                metadata: step.metadata || {}
            }));
            
            setAllPoints(points);
            setVisiblePointsCount(points.length);
        } catch (err) {
            console.error("Failed to load run detail", err);
        }
    };

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
            setSelectedRun(null);
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
        setSelectedRun(null);
        const es = new EventSource("/api/admin/trajectory/stream");
        es.onmessage = (event) => {
            const msg = JSON.parse(event.data);
            const newPoint: TrajectoryPoint = {
                id: msg.id || Math.random().toString(),
                x: msg.x || (Math.random() * 2 - 1),
                y: msg.y || (Math.random() * 2 - 1),
                timestamp: Date.now(),
                metadata: msg.metadata || {},
                resonance: msg.metadata?.resonance || (Math.random() * 0.5)
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
                            onClick={() => {
                                loadRuns();
                                fetchHistory();
                            }}
                            className="gap-2"
                        >
                            <Clock className="h-4 w-4" />
                            Refresh History
                        </Button>
                        <Button 
                            variant={isStreaming ? "destructive" : "default"}
                            onClick={() => setIsStreaming(!isStreaming)}
                            className="gap-2"
                        >
                            {isStreaming ? <Square className="h-4 w-4" /> : <Play className="h-4 w-4" />}
                            {isStreaming ? "Stop Monitoring" : "Start Live Feed"}
                        </Button>
                    </div>
                </div>
                <p className="text-muted-foreground text-lg max-w-2xl">
                    Observing agent "Chain of Thought" through Klein-Gordon diffusion patterns in Lorentz space.
                </p>
            </div>

            <div className="grid grid-cols-1 lg:grid-cols-4 gap-8">
                {/* Runs Selector Sidebar */}
                <Card className="bg-black/40 border-white/5 shadow-2xl backdrop-blur-xl h-[700px] flex flex-col">
                    <CardHeader className="pb-3">
                        <CardTitle className="text-sm uppercase tracking-wider text-muted-foreground font-mono">Agent Runs</CardTitle>
                        <CardDescription>Select historical tracing session</CardDescription>
                    </CardHeader>
                    <CardContent className="flex-1 overflow-hidden p-0">
                        <ScrollArea className="h-full px-4 pb-4">
                            <div className="space-y-2">
                                {runs.map(run => (
                                    <div 
                                        key={run.session_id}
                                        onClick={() => selectRun(run.session_id)}
                                        className={`p-3 rounded-lg border cursor-pointer transition-all ${
                                            selectedRun?.session_id === run.session_id 
                                            ? 'bg-primary/10 border-primary text-white' 
                                            : 'bg-white/5 border-white/5 text-white/70 hover:bg-white/10'
                                        }`}
                                    >
                                        <div className="flex justify-between items-start mb-1">
                                            <span className="font-mono text-xs font-semibold truncate max-w-[120px]">{run.session_id}</span>
                                            <Badge 
                                                variant="outline" 
                                                className={
                                                    run.status === 'SUCCESS' 
                                                    ? 'bg-emerald-500/10 text-emerald-400 border-emerald-500/20'
                                                    : run.status === 'FAILED'
                                                    ? 'bg-rose-500/10 text-rose-400 border-rose-500/20'
                                                    : 'bg-amber-500/10 text-amber-400 border-amber-500/20'
                                                }
                                            >
                                                {run.status}
                                            </Badge>
                                        </div>
                                        <p className="text-[10px] text-muted-foreground line-clamp-2">{run.task_description}</p>
                                        <div className="flex justify-between items-center mt-2 text-[9px] font-mono text-muted-foreground">
                                            <span>{(run.total_latency_ms / 1000).toFixed(2)}s</span>
                                            <span>Steps: {run.steps?.length || 0}</span>
                                        </div>
                                    </div>
                                ))}
                            </div>
                        </ScrollArea>
                    </CardContent>
                </Card>

                {/* Poincaré Disk canvas */}
                <Card className="lg:col-span-2 bg-black/40 border-white/5 shadow-2xl relative overflow-hidden backdrop-blur-xl h-[700px] flex flex-col justify-between">
                    <div className="absolute top-0 inset-x-0 h-px bg-gradient-to-r from-transparent via-primary/50 to-transparent" />
                    <CardHeader className="flex flex-row items-center justify-between pb-2">
                        <div>
                            <CardTitle className="text-xl flex items-center gap-2">
                                <Maximize2 className="h-4 w-4 text-primary" />
                                Poincaré Projection
                            </CardTitle>
                            <CardDescription>33D Lorentz ➜ 2D Hyperbolic Disk</CardDescription>
                        </div>
                        <div className="flex items-center gap-4">
                            <Badge variant="outline" className="bg-primary/5 text-primary border-primary/20">
                                {selectedRun ? "Telemetry Replay" : isStreaming ? "Live Feed Active" : "Static History"}
                            </Badge>
                        </div>
                    </CardHeader>
                    <CardContent className="flex-1 flex flex-col items-center justify-center p-0">
                        <canvas 
                            ref={canvasRef} 
                            width={480} 
                            height={480} 
                            className="max-w-full h-auto drop-shadow-[0_0_50px_rgba(0,243,255,0.1)]"
                        />
                        
                        {/* Time Slider Overlay */}
                        <div className="w-full px-12 mt-4 space-y-2 pb-6">
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

                {/* Metrics and Trace Logs */}
                <div className="space-y-6 h-[700px] flex flex-col justify-between">
                    <Card className="bg-card/50 backdrop-blur-md">
                        <CardHeader className="py-3">
                            <CardTitle className="text-sm flex items-center gap-2">
                                <Activity className="h-4 w-4 text-primary" />
                                Run Analytics
                            </CardTitle>
                        </CardHeader>
                        <CardContent className="space-y-3 pb-4">
                            <div className="flex justify-between items-center p-2 rounded-lg bg-white/5 border border-white/5">
                                <span className="text-xs text-muted-foreground">Stability (Lyapunov)</span>
                                <span className={`font-mono text-xs ${
                                    selectedRun 
                                    ? selectedRun.lyapunov_stability < 0 ? 'text-emerald-400' : 'text-rose-400'
                                    : 'text-amber-400'
                                }`}>
                                    {selectedRun ? `${selectedRun.lyapunov_stability.toFixed(3)} (${selectedRun.lyapunov_stability < 0 ? 'STABLE' : 'CHAOTIC'})` : 'N/A'}
                                </span>
                            </div>
                            <div className="flex justify-between items-center p-2 rounded-lg bg-white/5 border border-white/5">
                                <span className="text-xs text-muted-foreground">Trust Score</span>
                                <span className="font-mono text-xs text-emerald-400">
                                    {selectedRun ? `${(selectedRun.trust_score * 100).toFixed(1)}%` : 'N/A'}
                                </span>
                            </div>
                            <div className="flex justify-between items-center p-2 rounded-lg bg-white/5 border border-white/5">
                                <span className="text-xs text-muted-foreground">Execution Latency</span>
                                <span className="font-mono text-xs text-primary">
                                    {selectedRun ? `${(selectedRun.total_latency_ms / 1000).toFixed(2)}s` : 'N/A'}
                                </span>
                            </div>
                        </CardContent>
                    </Card>

                    <Card className="bg-black/20 border-white/5 flex-1 flex flex-col overflow-hidden">
                        <CardHeader className="pb-2 pt-3">
                            <CardTitle className="text-xs font-mono uppercase text-muted-foreground flex items-center gap-2">
                                <Terminal className="h-3 w-3" />
                                Execution Trace Logs
                            </CardTitle>
                        </CardHeader>
                        <CardContent className="p-0 flex-1 overflow-hidden">
                            <ScrollArea className="h-full px-4 pb-4">
                                <div className="space-y-4">
                                    {visiblePoints.slice().reverse().map((p, i) => (
                                        <div key={p.id} className="relative pl-4 border-l border-white/10 py-1 animate-in slide-in-from-left-2 duration-300">
                                            <div className="absolute left-[-5px] top-2 h-2 w-2 rounded-full bg-primary" />
                                            <div className="flex justify-between text-[9px] text-muted-foreground">
                                                <span>Step {visiblePoints.length - i}</span>
                                                <span>{new Date(p.timestamp).toLocaleTimeString()}</span>
                                            </div>
                                            <p className="text-xs font-semibold text-white/90 truncate">
                                                {p.metadata?.operation || "Vector Operation"}
                                            </p>
                                            <p className="text-[10px] text-muted-foreground font-mono leading-tight truncate">
                                                {p.metadata?.query || `Coord: (${p.x.toFixed(3)}, ${p.y.toFixed(3)})`}
                                            </p>
                                            {p.metadata?.latency_ms && (
                                                <div className="flex gap-2 text-[9px] font-mono text-primary/60 mt-0.5">
                                                    <span>{p.metadata.latency_ms}ms</span>
                                                    {p.metadata.provider && (
                                                        <>
                                                            <ArrowRight className="h-2 w-2 self-center" />
                                                            <span>{p.metadata.provider}</span>
                                                        </>
                                                    )}
                                                </div>
                                            )}
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
