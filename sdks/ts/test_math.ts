import * as process from 'process';
import {
    localEntropy,
    lyapunovConvergence,
    contextResonance,
    koopmanExtrapolate,
    mobiusAdd,
    logMap,
    expMap
} from './src/math';

function runTests() {
    let failed = 0;

    // 1. Local Entropy
    try {
        const candidate = [0.1, 0.1];
        const neighbors = [
            [0.11, 0.1],
            [0.1, 0.12],
            [0.09, 0.09]
        ];
        const entropy = localEntropy(candidate, neighbors, 1.0);
        if (entropy >= 0.1) throw new Error(`Entropy too high: ${entropy}`);
        console.log("localEntropy ok");
    } catch (e) {
        console.error("localEntropy failed:", e);
        failed++;
    }

    // 2. Lyapunov Convergence
    try {
        const trajectory = [
            [0.5, 0.5],
            [0.3, 0.3],
            [0.1, 0.1],
            [0.05, 0.05]
        ];
        const lya = lyapunovConvergence(trajectory, 1.0);
        if (lya >= 0) throw new Error(`Should converge (negative), got: ${lya}`);
        console.log("lyapunovConvergence ok");
    } catch (e) {
        console.error("lyapunovConvergence failed:", e);
        failed++;
    }

    // 3. Koopman Extrapolation
    try {
        const past = [0.1, 0.2];
        const current = [0.15, 0.25];
        const future = koopmanExtrapolate(past, current, 1.0, 1.0);
        if (future.length !== 2) throw new Error("Invalid output dimension");
        console.log("koopmanExtrapolate ok");
    } catch (e) {
        console.error("koopmanExtrapolate failed:", e);
        failed++;
    }

    // 4. Context Resonance
    try {
        const thought = [0.5, 0.0];
        const globalCtx = [0.0, 0.5];
        const pull = contextResonance(thought, globalCtx, 0.5, 1.0);
        if (pull.length !== 2) throw new Error("Invalid output dimension");
        console.log("contextResonance ok");
    } catch (e) {
        console.error("contextResonance failed:", e);
        failed++;
    }

    if (failed > 0) {
        process.exit(1);
    } else {
        console.log("All TS math tests passed!");
    }
}

runTests();
