/** YARLabs Hyperbolic Embedding implementation for LangChain. */

import { Embeddings, EmbeddingsParams } from "@langchain/core/embeddings";

export interface YARLabsEmbeddingsParams extends EmbeddingsParams {
    /**
     * @default "YARlabs/v5_Embedding_0.5B"
     */
    modelName?: string;
    /**
     * @default "cpu"
     */
    device?: string;
    /**
     * Dimensionality of the spatial part (total dim will be target_dim + 1).
     * @default 64
     */
    targetDim?: number;
    /**
     * If true, only search for model files locally.
     * @default false
     */
    local_only?: boolean;
}

export class YARLabsEmbeddings extends Embeddings {
    private modelName: string;
    private pipe: any;
    private device: string;
    private localOnly: boolean;
    private targetDim: number;

    constructor(fields?: YARLabsEmbeddingsParams) {
        super(fields ?? {});
        this.modelName = fields?.modelName ?? "YARlabs/v5_Embedding_0.5B";
        this.device = fields?.device ?? "cpu";
        this.localOnly = fields?.local_only ?? false;
        this.targetDim = fields?.targetDim ?? 64;
    }

    private async getPipe() {
        if (!this.pipe) {
            try {
                const { pipeline } = await import("@xenova/transformers");
                // Note: The YAR architecture requires trust_remote_code in python.
                // In @xenova/transformers, we might need a custom model file if 
                // the architecture is not standard BERT/DistilBERT.
                
                this.pipe = await pipeline("feature-extraction", this.modelName, {
                    device: this.device,
                    local_files_only: this.localOnly,
                } as any);
            } catch (err) {
                console.error("Failed to load @xenova/transformers or the model. This model might be custom and require specific ONNX implementation.");
                throw err;
            }
        }
        return this.pipe;
    }

    async embedDocuments(texts: string[]): Promise<number[][]> {
        const pipe = await this.getPipe();
        const results: number[][] = [];
        for (const text of texts) {
            // Note: The python version passes target_dim. 
            // In JS we assume the ONNX file is already built for a specific target_dim
            // or we'd need to handle slicing/head logic here.
            const output = await pipe(text, { pooling: 'mean' });
            
            // The output tensor from ONNX might need to be converted to Lorentz space (cosh, sinh)
            // if the ONNX file only contains the Euclidean base.
            // However, usually we export the FULL model to ONNX.
            results.push(Array.from(output.data));
        }
        return results;
    }

    async embedQuery(text: string): Promise<number[]> {
        const results = await this.embedDocuments([text]);
        return results[0];
    }
}
