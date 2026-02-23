/**
 * HyperspaceDB Spatial and Cognitive Math SDK
 * Provides hyperbolic math functions and Cognitive AI metrics for solving LLM hallucinations.
 */

export function dot(a: number[], b: number[]): number {
    let sum = 0;
    for (let i = 0; i < a.length; i++) sum += a[i] * b[i];
    return sum;
}

export function normSq(v: number[]): number {
    return dot(v, v);
}

export function norm(v: number[]): number {
    return Math.sqrt(Math.max(normSq(v), 0.0));
}

function projectToBall(x: number[], c: number): number[] {
    const n = norm(x);
    const maxN = (1.0 / Math.sqrt(c)) - 1e-9;
    if (n <= maxN || n <= 1e-15) return [...x];
    const s = maxN / n;
    return x.map(v => v * s);
}

export function mobiusAdd(x: number[], y: number[], c: number = 1.0): number[] {
    if (x.length !== y.length) throw new Error("Dimension mismatch");
    if (c <= 0.0) throw new Error("Curvature c must be > 0");
    const xy = dot(x, y);
    const x2 = normSq(x);
    const y2 = normSq(y);
    const numLeft = 1.0 + 2.0 * c * xy + c * y2;
    const numRight = 1.0 - c * x2;
    const den = 1.0 + 2.0 * c * xy + c * c * x2 * y2;
    if (Math.abs(den) < 1e-15) throw new Error("Möbius addition denominator too close to zero");
    return x.map((xi, i) => (numLeft * xi + numRight * y[i]) / den);
}

export function expMap(x: number[], v: number[], c: number = 1.0): number[] {
    if (x.length !== v.length) throw new Error("Dimension mismatch");
    if (c <= 0.0) throw new Error("Curvature c must be > 0");
    const x2 = normSq(x);
    const vNorm = Math.sqrt(Math.max(normSq(v), 0.0));
    if (vNorm < 1e-15) return [...x];
    const lambdaX = 2.0 / Math.max(1.0 - c * x2, 1e-15);
    const scale = Math.tanh(Math.sqrt(c) * lambdaX * vNorm / 2.0) / (Math.sqrt(c) * vNorm);
    const step = v.map(vi => scale * vi);
    return mobiusAdd(x, step, c);
}

export function logMap(x: number[], y: number[], c: number = 1.0): number[] {
    if (x.length !== y.length) throw new Error("Dimension mismatch");
    if (c <= 0.0) throw new Error("Curvature c must be > 0");
    const negX = x.map(xi => -xi);
    const delta = mobiusAdd(negX, y, c);
    const deltaNorm = Math.sqrt(Math.max(normSq(delta), 0.0));
    if (deltaNorm < 1e-15) return new Array(x.length).fill(0.0);
    const x2 = normSq(x);
    const lambdaX = 2.0 / Math.max(1.0 - c * x2, 1e-15);
    const factor = (2.0 / (lambdaX * Math.sqrt(c))) * Math.atanh(Math.min(Math.sqrt(c) * deltaNorm, 1.0 - 1e-15));
    return delta.map(di => factor * di / deltaNorm);
}

function gyro(u: number[], v: number[], w: number[], c: number = 1.0): number[] {
    const uv = mobiusAdd(u, v, c);
    const vw = mobiusAdd(v, w, c);
    const left = mobiusAdd(u, vw, c);
    const negUv = uv.map(z => -z);
    return mobiusAdd(negUv, left, c);
}

export function parallelTransport(x: number[], y: number[], v: number[], c: number = 1.0): number[] {
    if (x.length !== y.length || x.length !== v.length) throw new Error("Dimension mismatch");
    if (c <= 0.0) throw new Error("Curvature c must be > 0");
    const negX = x.map(xi => -xi);
    const gyr = gyro(y, negX, v, c);
    const lambdaX = 2.0 / Math.max(1.0 - c * normSq(x), 1e-15);
    const lambdaY = 2.0 / Math.max(1.0 - c * normSq(y), 1e-15);
    const scale = lambdaX / lambdaY;
    return gyr.map(gi => scale * gi);
}

export function frechetMean(points: number[][], c: number = 1.0, maxIter: number = 32, tol: number = 1e-6): number[] {
    if (points.length === 0) throw new Error("Points set cannot be empty");
    if (c <= 0.0) throw new Error("Curvature c must be > 0");
    const dim = points[0].length;
    let mu = projectToBall(points[0], c);
    for (let iter = 0; iter < Math.max(1, maxIter); iter++) {
        let grad = new Array(dim).fill(0.0);
        for (const p of points) {
            const lg = logMap(mu, p, c);
            for (let i = 0; i < dim; i++) grad[i] += lg[i];
        }
        const inv = 1.0 / points.length;
        for (let i = 0; i < dim; i++) grad[i] *= inv;
        const gNorm = norm(grad);
        if (gNorm <= Math.max(tol, 1e-15)) break;
        mu = expMap(mu, grad, c);
        mu = projectToBall(mu, c);
    }
    return mu;
}

// ==========================================
// Cognitive Math SDK (Spatial AI Engine)
// ==========================================

/**
 * Calculates the spatial entropy (dispersion) of a `candidate` vector relative to its `neighbors`.
 * Used to track LLM hallucinations (Task 2.3.1).
 * Returns a value in [0, 1) where values approaching 1 imply high chaos (hallucination).
 */
export function localEntropy(candidate: number[], neighbors: number[][], c: number = 1.0): number {
    if (neighbors.length === 0) return 1.0;
    let totalDeviation = 0.0;
    for (const neighbor of neighbors) {
        const diff = logMap(candidate, neighbor, c);
        totalDeviation += norm(diff);
    }
    const meanDeviation = totalDeviation / neighbors.length;
    return 1.0 - Math.exp(-meanDeviation);
}

/**
 * Evaluates if a trajectory of vectors (e.g. Chain of Thought) converges to an attractor.
 * Calculates the average energy derivative (Lyapunov function derivative).
 * Negative values indicate convergence (stable), positive indicate divergence (chaos/hallucination).
 */
export function lyapunovConvergence(trajectory: number[][], c: number = 1.0): number {
    if (trajectory.length < 3) throw new Error("Need at least 3 points");
    const attractor = frechetMean(trajectory, c, 32, 1e-6);
    let vDiffSum = 0.0;
    for (let i = 0; i < trajectory.length - 1; i++) {
        const vt0 = norm(logMap(attractor, trajectory[i], c));
        const vt1 = norm(logMap(attractor, trajectory[i + 1], c));
        vDiffSum += (vt1 - vt0);
    }
    return vDiffSum / (trajectory.length - 1);
}

/**
 * Extrapolates the trajectory in linear space (Koopman linearization) by tracking the 
 * shift vector from `past` to `current` and projecting it forward.
 */
export function koopmanExtrapolate(past: number[], current: number[], steps: number, c: number = 1.0): number[] {
    const velocityAtPast = logMap(past, current, c);
    const velocityAtCurrent = parallelTransport(past, current, velocityAtPast, c);
    const futureVelocity = velocityAtCurrent.map(v => v * steps);
    return expMap(current, futureVelocity, c);
}

/**
 * Resonates a thought vector towards a global context vector (Phase-Locked Loop context synchronization).
 * Pulls the thought towards the context along the geodesic by `resonanceFactor` [0, 1].
 */
export function contextResonance(thought: number[], globalContext: number[], resonanceFactor: number, c: number = 1.0): number[] {
    const pullDir = logMap(thought, globalContext, c);
    const factor = Math.max(0.0, Math.min(1.0, resonanceFactor));
    const appliedPull = pullDir.map(v => v * factor);
    return expMap(thought, appliedPull, c);
}
