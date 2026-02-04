import { Outlet, NavLink } from "react-router-dom"
import { LayoutDashboard, Database, Search, Settings, Network } from "lucide-react"
import { cn } from "@/lib/utils"

export function DashboardLayout() {
    return (
        <div className="flex h-screen w-full bg-background text-foreground overflow-hidden">
            {/* Sidebar */}
            <aside className="w-64 border-r bg-card flex flex-col hidden md:flex">
                <div className="p-6">
                    <div className="flex items-center gap-3">
                        <div className="h-8 w-8 rounded border-2 border-primary flex items-center justify-center font-bold text-primary text-lg">
                            [H]
                        </div>
                        <span className="text-lg font-bold tracking-tight">HyperspaceDB</span>
                    </div>
                </div>

                <nav className="space-y-1 px-4 flex-1">
                    <NavItem to="/" icon={LayoutDashboard} label="Overview" />
                    <NavItem to="/collections" icon={Database} label="Collections" />
                    <NavItem to="/explorer" icon={Search} label="Data Explorer" />
                    <NavItem to="/graph" icon={Network} label="Graph Explorer" badge="Soon" disabled />
                    <NavItem to="/settings" icon={Settings} label="Settings" />
                </nav>

                <div className="p-4 border-t border-border/50">
                    <div className="text-xs text-muted-foreground">
                        <p>Version 1.2.0</p>
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
