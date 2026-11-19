// Quantization catalog shared by the dashboard.
// The server publishes the authoritative list in `/api/status` → `quantization_modes`;
// this static copy is the fallback for older servers.

export interface QuantizationMode {
    value: string
    label: string
    /** Approximate stored bytes per dimension (codes only). */
    bytes_per_dim: number
    /** Metrics the mode is valid for (others fall back to ScalarI8 server-side). */
    metrics: string[]
    /** Needs codebook / centroid training via index rebuild (vacuum) after ingest. */
    calibration: boolean
    description: string
}

export const FALLBACK_QUANTIZATION_MODES: QuantizationMode[] = [
    { value: "none", label: "None", bytes_per_dim: 4, metrics: ["l2", "cosine", "poincare", "lorentz", "hybrid"], calibration: false, description: "Full precision (f32 storage, f64 legacy)" },
    { value: "medium", label: "Medium (ScalarI8)", bytes_per_dim: 1, metrics: ["l2", "cosine", "poincare", "lorentz", "hybrid"], calibration: false, description: "8-bit scalar (AsymmetricHybrid801 for 801-D hybrid)" },
    { value: "medium_plus", label: "Medium Plus (ScalarI4)", bytes_per_dim: 0.75, metrics: ["l2", "cosine", "poincare", "lorentz", "hybrid"], calibration: false, description: "4-bit block-wise scalar" },
    { value: "turbo", label: "Turbo", bytes_per_dim: 0.5, metrics: ["l2", "cosine", "hybrid"], calibration: false, description: "4-bit Lloyd-Max spherical with FWHT rotation" },
    { value: "ternary", label: "Ternary (BitNet 1.58)", bytes_per_dim: 0.25, metrics: ["l2", "cosine", "poincare", "lorentz", "hybrid"], calibration: false, description: "{-1, 0, +1} dual bitmask with SIMD popcount" },
    { value: "pq4", label: "PQ4 Fast-Scan", bytes_per_dim: 0.125, metrics: ["l2", "cosine"], calibration: true, description: "4-bit PQ over 4-D sub-spaces, L1-resident u8 LUT" },
    { value: "rabitq", label: "RaBitQ", bytes_per_dim: 0.125, metrics: ["l2", "cosine"], calibration: true, description: "1-bit code + unbiased estimator (popcount bit-planes)" },
    { value: "extreme", label: "Extreme (Binary)", bytes_per_dim: 0.125, metrics: ["l2", "cosine", "poincare", "lorentz", "hybrid"], calibration: false, description: "1-bit sign code with ADC" },
    { value: "opq", label: "Optimized PQ", bytes_per_dim: 0.0625, metrics: ["l2", "cosine"], calibration: true, description: "8-bit PQ (16-D sub-spaces) with FWHT rotation" },
    { value: "pq", label: "Product Quantization", bytes_per_dim: 0.0625, metrics: ["l2", "cosine"], calibration: true, description: "8-bit PQ (16-D sub-spaces)" },
]

/** Approximate bytes per stored vector (codes + per-vector scalars). */
export function bytesPerVector(mode: string, dim: number): number {
    switch (mode) {
        case "none": return dim * 4 + 4
        case "medium": return dim + 4
        case "medium_plus": return Math.ceil(dim / 16) * 12 + 4
        case "turbo": return Math.ceil(dim / 2) + 8
        case "ternary": return 2 * Math.ceil(dim / 8) + 8
        case "pq4": return Math.ceil(Math.ceil(dim / 4) / 2)
        case "rabitq": return Math.ceil(dim / 64) * 8 + 8
        case "extreme": return Math.ceil(dim / 8) + 4
        case "opq":
        case "pq": return Math.ceil(dim / 16)
        default: return dim + 4
    }
}

/** Human label for the server's `QuantizationMode` debug name (collection stats). */
export function quantizationModeLabel(q: string | undefined): string {
    switch (q ?? "") {
        case "None": return "None — Full precision"
        case "ScalarI8": return "Medium — ScalarI8 (8-bit)"
        case "ScalarI4": return "Medium Plus — 4-bit block-wise"
        case "Ternary": case "ternary": return "Ternary — 1.58-bit BitNet {-1, 0, +1}"
        case "Turbo": return "Turbo — 4-bit spherical"
        case "OPQ": case "opq": return "Optimized PQ — 8-bit, 16-D sub-spaces"
        case "ProductQuantization": case "pq": case "PQ": return "Product Quantization — 8-bit, 16-D sub-spaces"
        case "PQ4": case "pq4": return "PQ4 Fast-Scan — 4-bit, L1 LUT"
        case "RaBitQ": case "rabitq": return "RaBitQ — 1-bit + unbiased estimator"
        case "AsymmetricHybrid801": return "Medium — Hybrid 801 (Lorentz-aware)"
        case "AsymmetricHybridLowBit": return "Medium Plus — Hybrid 801 low-bit"
        case "AsymmetricHybridExtreme": return "Extreme — Hybrid 801"
        case "Binary": return "Extreme — Binary (1-bit)"
        default: return q || "—"
    }
}

/** Server debug names whose quantizer must be trained by an index rebuild. */
export function needsCalibration(q: string | undefined): boolean {
    return ["PQ4", "RaBitQ", "OPQ", "ProductQuantization"].includes(q ?? "")
}
