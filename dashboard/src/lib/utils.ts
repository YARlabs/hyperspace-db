import { type ClassValue, clsx } from "clsx"
import { twMerge } from "tailwind-merge"

export function cn(...inputs: ClassValue[]) {
    return twMerge(clsx(inputs))
}

export function getStatusColor(status?: string) {
    if (status === 'active') return 'bg-emerald-500/10 text-emerald-500 border-emerald-500/20 hover:bg-emerald-500/20 transition-all font-semibold capitalize'
    return 'bg-amber-500/10 text-amber-500 border-amber-500/20 hover:bg-amber-500/20 transition-all font-semibold capitalize'
}

export function getActionColor(action: string): string {
    const act = action.toUpperCase();
    if (act.includes("CREATE") || act.includes("INSERT")) return "text-emerald-400 font-semibold";
    if (act.includes("DELETE") || act.includes("DROP") || act.includes("VACUUM")) return "text-rose-400 font-semibold";
    if (act.includes("FREEZE")) return "text-cyan-400 font-semibold";
    if (act.includes("SEARCH") || act.includes("COUNT") || act.includes("SCROLL")) return "text-blue-400 font-semibold";
    return "text-zinc-300";
}
