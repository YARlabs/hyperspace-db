import { type ClassValue, clsx } from "clsx"
import { twMerge } from "tailwind-merge"

export function cn(...inputs: ClassValue[]) {
    return twMerge(clsx(inputs))
}

export function getStatusColor(status?: string) {
    if (status === 'active') return 'bg-emerald-500/10 text-emerald-500 border-emerald-500/20 hover:bg-emerald-500/20 transition-all font-semibold capitalize'
    return 'bg-amber-500/10 text-amber-500 border-amber-500/20 hover:bg-amber-500/20 transition-all font-semibold capitalize'
}
