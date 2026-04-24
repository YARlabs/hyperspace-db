export interface VectorData {
    id: string | number;
    vector: number[];
    metadata: Record<string, any>;
}

export interface CollectionSchema {
    dimension: number;
    metric: 'cosine' | 'l2' | 'ip' | 'lorentz' | 'poincare';
    count?: number;
}

export interface MigrationProgress {
    totalVectors: number;
    migratedVectors: number;
    batchesProcessed: number;
    status: 'idle' | 'running' | 'completed' | 'failed';
    error?: string;
    startTime?: Date;
    endTime?: Date;
}

export type ProgressCallback = (progress: MigrationProgress) => void;
