import React, { useState, useEffect, useRef } from "react"
import { 
    Database, 
    ArrowRight, 
    CheckCircle2, 
    Loader2, 
    Terminal, 
    History,
    ArrowUpRight,
    Server,
    Shield,
    Activity
} from "lucide-react"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from "@/components/ui/card"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { ScrollArea } from "@/components/ui/scroll-area"
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"
import { cn } from "@/lib/utils"
import { 
    getMigrationEngineStatus, 
    startMigrationService, 
    startMigrationTask, 
    getMigrationTaskStatus
} from "@/lib/api"

// --- Types ---

type SourceType = 'qdrant' | 'chroma' | 'pinecone' | 'weaviate' | 'milvus' | 'hyperspace';

interface MigrationConfig {
    sourceType: SourceType;
    url: string;
    apiKey: string;
    sourceCollection: string;
    targetCollection: string;
    batchSize: number;
}

interface LogEntry {
    timestamp: Date;
    level: 'info' | 'warn' | 'error' | 'success';
    message: string;
}


// --- Icons for sources ---
const SourceIcons: Record<SourceType, any> = {
    qdrant: () => <div className="bg-blue-500/10 p-2 rounded"><Database className="h-6 w-6 text-blue-500" /></div>,
    chroma: () => <div className="bg-emerald-500/10 p-2 rounded"><Database className="h-6 w-6 text-emerald-500" /></div>,
    pinecone: () => <div className="bg-purple-500/10 p-2 rounded"><Database className="h-6 w-6 text-purple-500" /></div>,
    weaviate: () => <div className="bg-orange-500/10 p-2 rounded"><Database className="h-6 w-6 text-orange-500" /></div>,
    milvus: () => <div className="bg-cyan-500/10 p-2 rounded"><Database className="h-6 w-6 text-cyan-500" /></div>,
    hyperspace: () => <div className="bg-primary/10 p-2 rounded"><div className="h-6 w-6 text-primary font-bold flex items-center justify-center">[H]</div></div>,
};

export function MigrationPage() {
    const [isServiceActive, setIsServiceActive] = useState<boolean | null>(null);
    const [step, setStep] = useState(1);
    
    useEffect(() => {
        checkServiceStatus();
    }, []);

    const checkServiceStatus = async () => {
        try {
            const data = await getMigrationEngineStatus();
            setIsServiceActive(data.active);
        } catch {
            setIsServiceActive(false);
        }
    };

    const [isActivating, setIsActivating] = useState(false);

    const activateService = async () => {
        setIsActivating(true);
        try {
            await startMigrationService();
            let attempts = 0;
            const check = setInterval(async () => {
                attempts++;
                try {
                    const data = await getMigrationEngineStatus();
                    if (data.active) {
                        setIsServiceActive(true);
                        setIsActivating(false);
                        clearInterval(check);
                    }
                } catch (e) {}
                
                if (attempts > 20) { 
                    setIsActivating(false);
                    clearInterval(check);
                }
            }, 3000);
        } catch (err) {
            console.error("Failed to start service", err);
            setIsActivating(false);
        }
    };

    if (isServiceActive === null) {
        return (
            <div className="flex h-[400px] items-center justify-center">
                <Loader2 className="h-8 w-8 animate-spin text-primary" />
            </div>
        );
    }

    if (!isServiceActive) {
        return (
            <div className="flex flex-col items-center justify-center h-[500px] space-y-6 animate-in fade-in duration-700">
                <div className="h-24 w-24 bg-primary/10 rounded-full flex items-center justify-center">
                    <History className="h-12 w-12 text-primary" />
                </div>
                <div className="text-center space-y-2">
                    <h2 className="text-3xl font-bold">Migration Engine Offline</h2>
                    <p className="text-muted-foreground max-w-md">
                        The background migration service is currently inactive. To begin transferring data, you must first initialize the migration engine.
                    </p>
                </div>
                <Button 
                    size="lg" 
                    onClick={activateService} 
                    className="gap-2 min-w-[240px]"
                    disabled={isActivating}
                >
                    {isActivating ? (
                        <>
                            <Loader2 className="h-4 w-4 animate-spin" />
                            Installing Engine...
                        </>
                    ) : (
                        <>
                            <Activity className="h-4 w-4" />
                            Activate Migration Service
                        </>
                    )}
                </Button>
                <div className="flex items-center gap-4 text-xs text-muted-foreground">
                    <div className="flex items-center gap-1"><Shield className="h-3 w-3" /> Secure Auth</div>
                    <div className="flex items-center gap-1"><Terminal className="h-3 w-3" /> Background Execution</div>
                </div>
            </div>
        );
    }
    const [config, setConfig] = useState<MigrationConfig>({
        sourceType: 'qdrant',
        url: '',
        apiKey: '',
        sourceCollection: '',
        targetCollection: '',
        batchSize: 500
    });
    
    const [isRunning, setIsRunning] = useState(false);
    const [progress, setProgress] = useState(0);
    const [stats, setStats] = useState({ current: 0, total: 0 });
    const [logs, setLogs] = useState<LogEntry[]>([]);
    const scrollRef = useRef<HTMLDivElement>(null);

    const addLog = (message: string, level: LogEntry['level'] = 'info') => {
        setLogs(prev => [...prev, { timestamp: new Date(), level, message }]);
    };

    useEffect(() => {
        if (scrollRef.current) {
            scrollRef.current.scrollIntoView({ behavior: 'smooth' });
        }
    }, [logs]);

    const [migrationId, setMigrationId] = useState<string | null>(null);

    const handleStartMigration = async () => {
        setIsRunning(true);
        setStep(4);
        addLog(`Initiating migration on server...`, 'info');
        
        try {
            const data = await startMigrationTask({
                ...config,
                hyperspaceUrl: window.location.origin.replace('5173', '50051') // Heuristic for local setup
            });
            
            setMigrationId(data.migrationId);
            addLog(`Migration task ${data.migrationId} started in background.`, 'success');
        } catch (err: any) {
            addLog(`Failed to start migration: ${err.message}`, 'error');
            setIsRunning(false);
        }
    };

    // Polling for progress
    useEffect(() => {
        if (!migrationId || !isRunning) return;

        const interval = setInterval(async () => {
            try {
                const data = await getMigrationTaskStatus(migrationId);
                
                setProgress(Math.round((data.migratedVectors / (data.totalVectors || 1)) * 100));
                setStats({ current: data.migratedVectors, total: data.totalVectors });
                
                if (data.status === 'completed') {
                    addLog('Migration completed successfully!', 'success');
                    setIsRunning(false);
                    clearInterval(interval);
                } else if (data.status === 'failed') {
                    addLog(`Migration failed: ${data.error}`, 'error');
                    setIsRunning(false);
                    clearInterval(interval);
                }
            } catch (err) {
                console.error('Polling error:', err);
            }
        }, 2000);

        return () => clearInterval(interval);
    }, [migrationId, isRunning]);

    return (
        <div className="space-y-8 animate-in fade-in duration-500">
            {/* Header */}
            <div className="flex flex-col gap-2">
                <div className="flex items-center gap-2 text-primary">
                    <ArrowUpRight className="h-5 w-5" />
                    <span className="text-sm font-semibold uppercase tracking-wider">Infrastructure</span>
                </div>
                <h1 className="text-4xl font-bold tracking-tight">Migration Wizard</h1>
                <p className="text-muted-foreground text-lg max-w-2xl">
                    Seamlessly move your vector data from legacy or third-party databases to HyperspaceDB with zero downtime.
                </p>
            </div>

            {/* Stepper */}
            <div className="flex items-center gap-4 py-4">
                {[1, 2, 3, 4].map((s) => (
                    <React.Fragment key={s}>
                        <div className={cn(
                            "h-10 w-10 rounded-full border-2 flex items-center justify-center font-bold transition-all",
                            step === s ? "border-primary bg-primary text-primary-foreground shadow-[0_0_15px_rgba(var(--primary),0.3)]" : 
                            step > s ? "border-primary bg-primary/20 text-primary" : "border-muted text-muted-foreground"
                        )}>
                            {step > s ? <CheckCircle2 className="h-5 w-5" /> : s}
                        </div>
                        {s < 4 && <div className={cn("h-0.5 w-12", step > s ? "bg-primary" : "bg-muted")} />}
                    </React.Fragment>
                ))}
            </div>

            {/* Step 1: Select Source */}
            {step === 1 && (
                <div className="grid grid-cols-1 md:grid-cols-3 gap-6 animate-in slide-in-from-bottom-4 duration-500">
                    {(Object.keys(SourceIcons) as SourceType[]).map((type) => (
                        <Card 
                            key={type}
                            className={cn(
                                "cursor-pointer transition-all hover:ring-2 hover:ring-primary/50 group relative overflow-hidden",
                                config.sourceType === type ? "ring-2 ring-primary bg-primary/5 shadow-lg shadow-primary/10" : "bg-card"
                            )}
                            onClick={() => {
                                setConfig({ ...config, sourceType: type });
                                setStep(2);
                            }}
                        >
                            <div className="absolute top-0 right-0 p-4 opacity-0 group-hover:opacity-100 transition-opacity">
                                <ArrowRight className="h-5 w-5 text-primary" />
                            </div>
                            <CardHeader className="flex flex-row items-center gap-4 pb-2">
                                {SourceIcons[type]()}
                                <div>
                                    <CardTitle className="capitalize">{type}</CardTitle>
                                    <CardDescription>Vector Database</CardDescription>
                                </div>
                            </CardHeader>
                            <CardContent>
                                <p className="text-sm text-muted-foreground">
                                    Migrate vectors, payloads, and schema definitions from {type} instance.
                                </p>
                            </CardContent>
                        </Card>
                    ))}
                </div>
            )}

            {/* Step 2: Configuration */}
            {step === 2 && (
                <Card className="max-w-2xl animate-in slide-in-from-right-4 duration-500">
                    <CardHeader>
                        <CardTitle className="flex items-center gap-2">
                            {SourceIcons[config.sourceType]()}
                            Configure {config.sourceType} Connection
                        </CardTitle>
                        <CardDescription>Enter the credentials for your source database.</CardDescription>
                    </CardHeader>
                    <CardContent className="space-y-4">
                        <div className="grid gap-2">
                            <Label htmlFor="url">API Endpoint / Host URL</Label>
                            <div className="relative">
                                <Server className="absolute left-3 top-3 h-4 w-4 text-muted-foreground" />
                                <Input 
                                    id="url" 
                                    placeholder="https://your-instance.cloud.qdrant.io" 
                                    className="pl-10"
                                    value={config.url}
                                    onChange={(e) => setConfig({ ...config, url: e.target.value })}
                                />
                            </div>
                        </div>
                        <div className="grid gap-2">
                            <Label htmlFor="key">API Key (Optional)</Label>
                            <div className="relative">
                                <Shield className="absolute left-3 top-3 h-4 w-4 text-muted-foreground" />
                                <Input 
                                    id="key" 
                                    type="password" 
                                    placeholder="••••••••••••••••" 
                                    className="pl-10"
                                    value={config.apiKey}
                                    onChange={(e) => setConfig({ ...config, apiKey: e.target.value })}
                                />
                            </div>
                        </div>
                        <div className="grid gap-2">
                            <Label htmlFor="sourceCol">Source Collection Name</Label>
                            <Input 
                                id="sourceCol" 
                                placeholder="production_vectors" 
                                value={config.sourceCollection}
                                onChange={(e) => setConfig({ ...config, sourceCollection: e.target.value })}
                            />
                        </div>
                    </CardContent>
                    <CardFooter className="flex justify-between">
                        <Button variant="ghost" onClick={() => setStep(1)}>Back</Button>
                        <Button 
                            disabled={!config.url || !config.sourceCollection} 
                            onClick={() => setStep(3)}
                        >
                            Continue <ArrowRight className="ml-2 h-4 w-4" />
                        </Button>
                    </CardFooter>
                </Card>
            )}

            {/* Step 3: Target & Batching */}
            {step === 3 && (
                <Card className="max-w-2xl animate-in slide-in-from-right-4 duration-500">
                    <CardHeader>
                        <CardTitle className="flex items-center gap-2">
                            <div className="bg-primary/10 p-2 rounded"><Database className="h-6 w-6 text-primary" /></div>
                            Destination Settings
                        </CardTitle>
                        <CardDescription>How should the data be ingested into HyperspaceDB?</CardDescription>
                    </CardHeader>
                    <CardContent className="space-y-6">
                        <div className="grid gap-2">
                            <Label htmlFor="targetCol">Target Collection Name</Label>
                            <Input 
                                id="targetCol" 
                                placeholder="new_collection_v3" 
                                value={config.targetCollection || config.sourceCollection}
                                onChange={(e) => setConfig({ ...config, targetCollection: e.target.value })}
                            />
                            <p className="text-[10px] text-muted-foreground">If the collection doesn't exist, it will be created automatically using the source schema.</p>
                        </div>
                        <div className="grid gap-4 sm:grid-cols-2">
                            <div className="grid gap-2">
                                <Label>Batch Size</Label>
                                <Select 
                                    value={config.batchSize.toString()} 
                                    onValueChange={(v) => setConfig({ ...config, batchSize: parseInt(v) })}
                                >
                                    <SelectTrigger>
                                        <SelectValue />
                                    </SelectTrigger>
                                    <SelectContent>
                                        <SelectItem value="100">100 vectors</SelectItem>
                                        <SelectItem value="500">500 vectors</SelectItem>
                                        <SelectItem value="1000">1000 vectors</SelectItem>
                                        <SelectItem value="2000">2000 vectors</SelectItem>
                                    </SelectContent>
                                </Select>
                            </div>
                            <div className="grid gap-2">
                                <Label>Deduplication</Label>
                                <Select defaultValue="overwrite">
                                    <SelectTrigger>
                                        <SelectValue />
                                    </SelectTrigger>
                                    <SelectContent>
                                        <SelectItem value="overwrite">Overwrite existing IDs</SelectItem>
                                        <SelectItem value="skip">Skip existing IDs</SelectItem>
                                    </SelectContent>
                                </Select>
                            </div>
                        </div>

                        <div className="rounded-lg border bg-muted/30 p-4 space-y-3">
                            <div className="flex items-center gap-2 text-xs font-semibold uppercase text-muted-foreground">
                                <Activity className="h-3 w-3" />
                                Migration Summary
                            </div>
                            <div className="grid grid-cols-2 gap-4 text-sm">
                                <div>
                                    <p className="text-muted-foreground">Source</p>
                                    <p className="font-medium capitalize">{config.sourceType} ({config.sourceCollection})</p>
                                </div>
                                <div>
                                    <p className="text-muted-foreground">Target</p>
                                    <p className="font-medium">HyperspaceDB ({config.targetCollection || config.sourceCollection})</p>
                                </div>
                            </div>
                        </div>
                    </CardContent>
                    <CardFooter className="flex justify-between">
                        <Button variant="ghost" onClick={() => setStep(2)}>Back</Button>
                        <Button className="bg-primary hover:bg-primary/90" onClick={handleStartMigration}>
                            Start Migration <ArrowUpRight className="ml-2 h-4 w-4" />
                        </Button>
                    </CardFooter>
                </Card>
            )}

            {/* Step 4: Progress & Logs */}
            {step === 4 && (
                <div className="grid grid-cols-1 lg:grid-cols-3 gap-6 animate-in zoom-in-95 duration-500">
                    <Card className="lg:col-span-2">
                        <CardHeader>
                            <CardTitle>Migration Progress</CardTitle>
                            <CardDescription>Streaming data from {config.sourceType}...</CardDescription>
                        </CardHeader>
                        <CardContent className="space-y-8">
                            <div className="space-y-2">
                                <div className="flex justify-between text-sm font-medium">
                                    <span>Ingesting Vectors</span>
                                    <span>{progress}%</span>
                                </div>
                                <div className="h-4 w-full bg-secondary rounded-full overflow-hidden relative">
                                    <div 
                                        className="h-full bg-primary transition-all duration-500 shadow-[0_0_15px_rgba(var(--primary),0.5)]"
                                        style={{ width: `${progress}%` }}
                                    />
                                    {/* Glass reflection effect */}
                                    <div className="absolute inset-0 bg-gradient-to-r from-transparent via-white/10 to-transparent skew-x-[-20deg]" />
                                </div>
                                <div className="flex justify-between text-[10px] text-muted-foreground">
                                    <span>{stats.current.toLocaleString()} vectors migrated</span>
                                    <span>{stats.total.toLocaleString()} total</span>
                                </div>
                            </div>

                            <div className="grid grid-cols-3 gap-4">
                                <div className="p-4 rounded-xl bg-card border shadow-sm space-y-1">
                                    <p className="text-xs text-muted-foreground font-medium uppercase">Batch Size</p>
                                    <p className="text-2xl font-bold">{config.batchSize}</p>
                                </div>
                                <div className="p-4 rounded-xl bg-card border shadow-sm space-y-1">
                                    <p className="text-xs text-muted-foreground font-medium uppercase">Speed</p>
                                    <p className="text-2xl font-bold">~1.2k/s</p>
                                </div>
                                <div className="p-4 rounded-xl bg-card border shadow-sm space-y-1">
                                    <p className="text-xs text-muted-foreground font-medium uppercase">Retries</p>
                                    <p className="text-2xl font-bold">0</p>
                                </div>
                            </div>

                            <div className="flex items-center justify-center p-8">
                                {isRunning ? (
                                    <div className="flex flex-col items-center gap-4">
                                        <Loader2 className="h-12 w-12 text-primary animate-spin" />
                                        <p className="text-muted-foreground animate-pulse">Syncing data structures...</p>
                                    </div>
                                ) : (
                                    <div className="flex flex-col items-center gap-4 text-center">
                                        <div className="h-16 w-16 bg-emerald-500/10 rounded-full flex items-center justify-center">
                                            <CheckCircle2 className="h-10 w-10 text-emerald-500" />
                                        </div>
                                        <div>
                                            <h3 className="text-xl font-bold">Migration Complete</h3>
                                            <p className="text-muted-foreground">All data has been successfully synchronized.</p>
                                        </div>
                                        <Button onClick={() => setStep(1)}>New Migration</Button>
                                    </div>
                                )}
                            </div>
                        </CardContent>
                    </Card>

                    <Card className="bg-[#0c0c0c] border-white/5 shadow-2xl relative overflow-hidden">
                        <div className="absolute top-0 inset-x-0 h-px bg-gradient-to-r from-transparent via-primary/50 to-transparent" />
                        <CardHeader className="border-b border-white/5 pb-4">
                            <CardTitle className="text-sm font-mono flex items-center gap-2 text-white/90">
                                <Terminal className="h-4 w-4 text-primary" />
                                Migration Logs
                            </CardTitle>
                        </CardHeader>
                        <CardContent className="p-0">
                            <ScrollArea className="h-[500px] p-4">
                                <div className="space-y-1 font-mono text-[11px]">
                                    {logs.map((log, i) => (
                                        <div key={i} className="flex gap-2">
                                            <span className="text-white/20 whitespace-nowrap">
                                                {log.timestamp.toLocaleTimeString([], { hour12: false })}
                                            </span>
                                            <span className={cn(
                                                "font-bold uppercase w-16",
                                                log.level === 'info' && "text-blue-400",
                                                log.level === 'warn' && "text-yellow-400",
                                                log.level === 'error' && "text-red-400",
                                                log.level === 'success' && "text-emerald-400",
                                            )}>
                                                [{log.level}]
                                            </span>
                                            <span className="text-white/70">{log.message}</span>
                                        </div>
                                    ))}
                                    <div ref={scrollRef} />
                                </div>
                            </ScrollArea>
                        </CardContent>
                    </Card>
                </div>
            )}
        </div>
    )
}
