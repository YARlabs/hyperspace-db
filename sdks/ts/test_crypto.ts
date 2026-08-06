import * as assert from 'assert';
import { HyperspaceClient } from './src/client';
import * as hsMath from './src/math';
import * as crypto from 'crypto';

async function runCryptoTests() {
    console.log("=== STARTING TS SDK CRYPTO AND PROJECTION TESTS ===");
    let failed = 0;

    // 1. Test Orthogonal matrix generation and projection distance preservation
    try {
        const dim = 64;
        const seed = crypto.randomBytes(32);
        const matrix = hsMath.generateOrthogonalMatrix(dim, seed);
        
        // Orthogonality check: O^T O = I
        for (let i = 0; i < dim; i++) {
            const rNorm = Math.sqrt(matrix[i].reduce((s, val) => s + val * val, 0));
            console.log(`Row ${i} norm: ${rNorm}`);
        }
        for (let i = 0; i < dim; i++) {
            for (let j = 0; j < dim; j++) {
                let sum = 0;
                for (let k = 0; k < dim; k++) {
                    sum += matrix[i][k] * matrix[j][k];
                }
                const expected = i === j ? 1.0 : 0.0;
                if (Math.abs(sum - expected) > 1e-9) {
                    throw new Error(`Row orthonormality failed at ${i}, ${j}: ${sum}`);
                }
            }
        }
        
        // Distance preservation check
        const u = Array.from({ length: dim }, () => Math.random());
        const v = Array.from({ length: dim }, () => Math.random());
        
        const uProj = hsMath.projectVector(u, matrix);
        const vProj = hsMath.projectVector(v, matrix);
        
        const distOrig = Math.sqrt(u.reduce((s, val, idx) => s + (val - v[idx]) ** 2, 0));
        const distProj = Math.sqrt(uProj.reduce((s, val, idx) => s + (val - vProj[idx]) ** 2, 0));
        
        if (Math.abs(distOrig - distProj) > 1e-9) {
            throw new Error(`Distance not preserved: orig=${distOrig}, proj=${distProj}`);
        }
        console.log("Orthogonal matrix distance preservation ok");
    } catch (e) {
        console.error("Orthogonal matrix test failed:", e);
        failed++;
    }

    // 2. Test Poincaré projection and Poincaré distance preservation
    try {
        const dim = 32;
        const seed = crypto.randomBytes(32);
        const matrix = hsMath.generateLorentzMatrix(dim + 1, seed);
        
        // Poincaré ball points (norm < 1)
        const u = Array.from({ length: dim }, () => Math.random() * 0.1);
        const v = Array.from({ length: dim }, () => Math.random() * 0.1);
        
        const uLorentz = hsMath.poincareToLorentz(u);
        const uLorentzProj = hsMath.projectVector(uLorentz, matrix);
        const uProj = hsMath.lorentzToPoincare(uLorentzProj);
        
        const vLorentz = hsMath.poincareToLorentz(v);
        const vLorentzProj = hsMath.projectVector(vLorentz, matrix);
        const vProj = hsMath.lorentzToPoincare(vLorentzProj);
        
        // Norms must be < 1
        const uProjNorm = Math.sqrt(uProj.reduce((s, z) => s + z * z, 0));
        const vProjNorm = Math.sqrt(vProj.reduce((s, z) => s + z * z, 0));
        if (uProjNorm >= 1.0 || vProjNorm >= 1.0) {
            throw new Error("Projected Poincare points left the unit ball!");
        }
        
        // Poincaré distance preservation check
        const poincareDist = (x: number[], y: number[]) => {
            const xSq = x.reduce((s, val) => s + val * val, 0);
            const ySq = y.reduce((s, val) => s + val * val, 0);
            const diffSq = x.reduce((s, val, idx) => s + (val - y[idx]) ** 2, 0);
            const val = 1.0 + 2.0 * diffSq / ((1.0 - xSq) * (1.0 - ySq));
            return Math.acosh(val);
        };
        
        const distOrig = poincareDist(u, v);
        const distProj = poincareDist(uProj, vProj);
        if (Math.abs(distOrig - distProj) > 1e-9) {
            throw new Error(`Poincare distance not preserved: orig=${distOrig}, proj=${distProj}`);
        }
        console.log("Poincaré matrix distance preservation ok");
    } catch (e) {
        console.error("Poincaré test failed:", e);
        failed++;
    }

    // 3. Test MRL Block-diagonal projection
    try {
        const client = new HyperspaceClient('dummy_host');
        const schema = {
            components: [
                { name: "primary", metric: "cosine", fullDimension: 128, weight: 1.0 }
            ],
            cascadePipeline: [
                { componentName: "primary", cutoffDimension: 32, storeInRam: true, rerankTopK: 100 }
            ]
        };
        client.registerCollectionKey("mrl_ts", "secret_ts", "cosine", 0.0, schema);
        
        // Force context build
        await (client as any)._getEncryptionContext("mrl_ts");
        
        const vec1 = Array.from({ length: 128 }, () => Math.random());
        const vec2 = [...vec1];
        // Mutate only tail dimensions
        for (let i = 32; i < 128; i++) {
            vec2[i] = Math.random();
        }
        
        const proj1 = (client as any)._projectCollectionVector("mrl_ts", vec1, (client as any).encryptionContexts["mrl_ts"], "cosine");
        const proj2 = (client as any)._projectCollectionVector("mrl_ts", vec2, (client as any).encryptionContexts["mrl_ts"], "cosine");
        
        // Verify head is identical
        for (let i = 0; i < 32; i++) {
            if (Math.abs(proj1[i] - proj2[i]) > 1e-12) {
                throw new Error(`MRL partitioning failed at head index ${i}: ${proj1[i]} vs ${proj2[i]}`);
            }
        }
        
        // Verify tail is different
        let tailDiff = false;
        for (let i = 32; i < 128; i++) {
            if (Math.abs(proj1[i] - proj2[i]) > 1e-5) {
                tailDiff = true;
                break;
            }
        }
        if (!tailDiff) {
            throw new Error("MRL tail should have differed but was identical!");
        }
        
        console.log("MRL block-diagonal projection ok");
    } catch (e) {
        console.error("MRL test failed:", e);
        failed++;
    }

    if (failed > 0) {
        process.exit(1);
    } else {
        console.log("ALL TS SDK CRYPTO TESTS PASSED SUCCESSFULLY!");
    }
}

runCryptoTests();
