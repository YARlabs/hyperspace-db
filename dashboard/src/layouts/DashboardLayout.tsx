import { Outlet, NavLink } from "react-router-dom"
import { LayoutDashboard, Database, Search, Settings, Network, ArrowUpRight, Waves, Activity } from "lucide-react"
import { cn } from "@/lib/utils"
import { useQuery } from "@tanstack/react-query"
import { api } from "@/lib/api"

export function DashboardLayout() {
    return (
        <div className="flex h-screen w-full bg-background text-foreground overflow-hidden">
            {/* Sidebar */}
            <aside className="w-64 border-r bg-card flex flex-col hidden md:flex">
                <div className="p-6">
                    <div className="flex items-center gap-3">
                        <div className="h-8 w-8 flex items-center justify-center font-bold text-primary text-lg">
                            [H]
                        </div>
                        <span className="text-lg font-bold tracking-tight">HyperspaceDB</span>
                    </div>
                </div>

                <nav className="space-y-1 px-4 flex-1">
                    <NavItem to="/" icon={LayoutDashboard} label="Overview" />
                    <NavItem to="/collections" icon={Database} label="Collections" />
                    <NavItem to="/nodes" icon={Network} label="Cluster Nodes" />
                    <NavItem to="/explorer" icon={Search} label="Data Explorer" />
                    <NavItem to="/graph" icon={Network} label="Graph Explorer" />
                    <NavItem to="/migration" icon={ArrowUpRight} label="Migration" />
                    <NavItem to="/trajectory" icon={Waves} label="Trajectories" />
                    <NavItem to="/settings" icon={Settings} label="Settings" />
                </nav>

                <div className="p-4 border-t border-border/50">
                    <HealthIndicator />
                    <div className="text-xs text-muted-foreground mt-4">
                        <p>Version 3.1.0</p>
                        <p className="opacity-50">Local Control Plane</p>
                    </div>
                </div>
            </aside>

            {/* Main Content */}
            <main className="flex-1 overflow-auto bg-background/50">
                <div className="container mx-auto p-6 md:p-8 max-w-7xl">
                    <Outlet />
                </div>
            </main>
        </div>
    )
}

function NavItem({ to, icon: Icon, label, badge, disabled }: any) {
    if (disabled) {
        return (
            <div className="flex items-center gap-3 rounded-md px-3 py-2 text-sm font-medium text-muted-foreground/50 cursor-not-allowed">
                <Icon className="h-4 w-4" />
                <span>{label}</span>
                {badge && <span className="ml-auto text-[10px] border border-muted-foreground/30 px-1.5 rounded text-muted-foreground">{badge}</span>}
            </div>
        )
    }
    return (
        <NavLink
            to={to}
            className={({ isActive }) =>
                cn(
                    "flex items-center gap-3 rounded-md px-3 py-2 text-sm font-medium transition-colors hover:bg-accent hover:text-accent-foreground",
                    isActive ? "bg-accent text-accent-foreground" : "text-muted-foreground"
                )
            }
        >
            <Icon className="h-4 w-4" />
            <span>{label}</span>
            {badge && <span className="ml-auto text-[10px] bg-primary/10 text-primary px-1.5 py-0.5 rounded font-semibold">{badge}</span>}
        </NavLink>
    )
}
function HealthIndicator() {
    const { data, isError } = useQuery({
        queryKey: ['health'],
        queryFn: () => api.get("/health").then(r => r.data),
        refetchInterval: 10000,
    })

    const status = data?.status || (isError ? "OFFLINE" : "CONNECTING")
    const isOnline = status === "ONLINE"

    return (
        <div className="flex items-center gap-2 px-2 py-1.5 rounded bg-zinc-950/50 border border-white/5">
            <div className={cn(
                "h-2 w-2 rounded-full animate-pulse",
                isOnline ? "bg-green-500 shadow-[0_0_8px_rgba(34,197,94,0.6)]" : "bg-red-500 shadow-[0_0_8px_rgba(239,68,68,0.6)]"
            )} />
            <div className="flex flex-col">
                <span className="text-[10px] font-bold uppercase tracking-wider text-zinc-400">System Status</span>
                <span className={cn("text-xs font-medium", isOnline ? "text-green-400" : "text-red-400")}>
                    {status}
                </span>
            </div>
            <Activity className="ml-auto h-3 w-3 text-zinc-600" />
        </div>
    )
}
