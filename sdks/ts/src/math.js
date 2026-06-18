"use strict";
/**
 * HyperspaceDB Spatial and Cognitive Math SDK
 * Provides hyperbolic math functions and Cognitive AI metrics for solving LLM hallucinations.
 */
Object.defineProperty(exports, "__esModule", { value: true });
exports.dot = dot;
exports.normSq = normSq;
exports.norm = norm;
exports.mobiusAdd = mobiusAdd;
exports.expMap = expMap;
exports.logMap = logMap;
exports.parallelTransport = parallelTransport;
exports.frechetMean = frechetMean;
exports.lorentzProduct = lorentzProduct;
exports.lorentzDist = lorentzDist;
exports.lorentzToPoincare = lorentzToPoincare;
exports.poincareToLorentz = poincareToLorentz;
exports.projectToHyperboloid = projectToHyperboloid;
exports.localEntropy = localEntropy;
exports.lyapunovConvergence = lyapunovConvergence;
exports.koopmanExtrapolate = koopmanExtrapolate;
exports.contextResonance = contextResonance;
function dot(a, b) {
    let sum = 0;
    for (let i = 0; i < a.length; i++)
        sum += a[i] * b[i];
    return sum;
}
function normSq(v) {
    return dot(v, v);
}
function norm(v) {
    return Math.sqrt(Math.max(normSq(v), 0.0));
}
function projectToBall(x, c) {
    const n = norm(x);
    const maxN = (1.0 / Math.sqrt(c)) - 1e-9;
    if (n <= maxN || n <= 1e-15)
        return [...x];
    const s = maxN / n;
    return x.map(v => v * s);
}
function mobiusAdd(x, y, c = 1.0) {
    if (x.length !== y.length)
        throw new Error("Dimension mismatch");
    if (c <= 0.0)
        throw new Error("Curvature c must be > 0");
    const xy = dot(x, y);
    const x2 = normSq(x);
    const y2 = normSq(y);
    const numLeft = 1.0 + 2.0 * c * xy + c * y2;
    const numRight = 1.0 - c * x2;
    const den = 1.0 + 2.0 * c * xy + c * c * x2 * y2;
    if (Math.abs(den) < 1e-15)
        throw new Error("Möbius addition denominator too close to zero");
    return x.map((xi, i) => (numLeft * xi + numRight * y[i]) / den);
}
function expMap(x, v, c = 1.0) {
    if (x.length !== v.length)
        throw new Error("Dimension mismatch");
    if (c <= 0.0)
        throw new Error("Curvature c must be > 0");
    const x2 = normSq(x);
    const vNorm = Math.sqrt(Math.max(normSq(v), 0.0));
    if (vNorm < 1e-15)
        return [...x];
    const lambdaX = 2.0 / Math.max(1.0 - c * x2, 1e-15);
    const scale = Math.tanh(Math.sqrt(c) * lambdaX * vNorm / 2.0) / (Math.sqrt(c) * vNorm);
    const step = v.map(vi => scale * vi);
    return mobiusAdd(x, step, c);
}
function logMap(x, y, c = 1.0) {
    if (x.length !== y.length)
        throw new Error("Dimension mismatch");
    if (c <= 0.0)
        throw new Error("Curvature c must be > 0");
    const negX = x.map(xi => -xi);
    const delta = mobiusAdd(negX, y, c);
    const deltaNorm = Math.sqrt(Math.max(normSq(delta), 0.0));
    if (deltaNorm < 1e-15)
        return new Array(x.length).fill(0.0);
    const x2 = normSq(x);
    const lambdaX = 2.0 / Math.max(1.0 - c * x2, 1e-15);
    const factor = (2.0 / (lambdaX * Math.sqrt(c))) * Math.atanh(Math.min(Math.sqrt(c) * deltaNorm, 1.0 - 1e-15));
    return delta.map(di => factor * di / deltaNorm);
}
function gyro(u, v, w, c = 1.0) {
    const uv = mobiusAdd(u, v, c);
    const vw = mobiusAdd(v, w, c);
    const left = mobiusAdd(u, vw, c);
    const negUv = uv.map(z => -z);
    return mobiusAdd(negUv, left, c);
}
function parallelTransport(x, y, v, c = 1.0) {
    if (x.length !== y.length || x.length !== v.length)
        throw new Error("Dimension mismatch");
    if (c <= 0.0)
        throw new Error("Curvature c must be > 0");
    const negX = x.map(xi => -xi);
    const gyr = gyro(y, negX, v, c);
    const lambdaX = 2.0 / Math.max(1.0 - c * normSq(x), 1e-15);
    const lambdaY = 2.0 / Math.max(1.0 - c * normSq(y), 1e-15);
    const scale = lambdaX / lambdaY;
    return gyr.map(gi => scale * gi);
}
function frechetMean(points, c = 1.0, maxIter = 32, tol = 1e-6) {
    if (points.length === 0)
        throw new Error("Points set cannot be empty");
    if (c <= 0.0)
        throw new Error("Curvature c must be > 0");
    const dim = points[0].length;
    let mu = projectToBall(points[0], c);
    for (let iter = 0; iter < Math.max(1, maxIter); iter++) {
        let grad = new Array(dim).fill(0.0);
        for (const p of points) {
            const lg = logMap(mu, p, c);
            for (let i = 0; i < dim; i++)
                grad[i] += lg[i];
        }
        const inv = 1.0 / points.length;
        for (let i = 0; i < dim; i++)
            grad[i] *= inv;
        const gNorm = norm(grad);
        if (gNorm <= Math.max(tol, 1e-15))
            break;
        mu = expMap(mu, grad, c);
        mu = projectToBall(mu, c);
    }
    return mu;
}
// ==========================================
// Lorentz Model Math (Hyperboloid)
// ==========================================
/** Computes the Minkowski inner product (Lorentz product) between two vectors. */
function lorentzProduct(u, v) {
    if (u.length === 0 || v.length === 0)
        return 0.0;
    let product = -u[0] * v[0];
    for (let i = 1; i < u.length; i++)
        product += u[i] * v[i];
    return product;
}
/** Computes the Lorentz distance between two points on the hyperboloid. */
function lorentzDist(u, v) {
    const inner = -lorentzProduct(u, v);
    return Math.acosh(Math.max(inner, 1.0));
}
/** Converts a point from the Lorentz model (Hyperboloid) to the Poincaré Ball model (129 -> 128). */
function lorentzToPoincare(x) {
    if (x.length === 0)
        return [];
    const denom = Math.max(1.0 + x[0], 1e-12);
    const proj = [];
    for (let i = 1; i < x.length; i++)
        proj.push(x[i] / denom);
    return proj;
}
/** Converts a point from the Poincaré Ball model to the Lorentz model (128 -> 129). */
function poincareToLorentz(p) {
    const pSq = normSq(p);
    const denom = Math.max(1.0 - pSq, 1e-12);
    const x = [(1.0 + pSq) / denom];
    for (const pi of p)
        x.push((2.0 * pi) / denom);
    return x;
}
/** Ensures a vector satisfies the Lorentz constraint -x0^2 + |x|^2 = -1 (stabilization). */
function projectToHyperboloid(v) {
    if (v.length === 0)
        return [];
    const res = [...v];
    let spatialNormSq = 0;
    for (let i = 1; i < res.length; i++)
        spatialNormSq += res[i] * res[i];
    res[0] = Math.sqrt(1.0 + spatialNormSq);
    return res;
}
// ==========================================
// Cognitive Math SDK (Spatial AI Engine)
// ==========================================
/**
 * Calculates the spatial entropy (dispersion) of a `candidate` vector relative to its `neighbors`.
 * Used to track LLM hallucinations (Task 2.3.1).
 * Returns a value in [0, 1) where values approaching 1 imply high chaos (hallucination).
 */
function localEntropy(candidate, neighbors, c = 1.0) {
    if (neighbors.length === 0)
        return 1.0;
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
function lyapunovConvergence(trajectory, c = 1.0) {
    if (trajectory.length < 3)
        throw new Error("Need at least 3 points");
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
function koopmanExtrapolate(past, current, steps, c = 1.0) {
    const velocityAtPast = logMap(past, current, c);
    const velocityAtCurrent = parallelTransport(past, current, velocityAtPast, c);
    const futureVelocity = velocityAtCurrent.map(v => v * steps);
    return expMap(current, futureVelocity, c);
}
/**
 * Resonates a thought vector towards a global context vector (Phase-Locked Loop context synchronization).
 * Pulls the thought towards the context along the geodesic by `resonanceFactor` [0, 1].
 */
function contextResonance(thought, globalContext, resonanceFactor, c = 1.0) {
    const pullDir = logMap(thought, globalContext, c);
    const factor = Math.max(0.0, Math.min(1.0, resonanceFactor));
    const appliedPull = pullDir.map(v => v * factor);
    return expMap(thought, appliedPull, c);
}
