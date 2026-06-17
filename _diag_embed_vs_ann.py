#!/usr/bin/env python3
"""
_diag_embed_vs_ann.py - rozdziela czas EMBEDDINGU od czasu ANN SEARCH.

Subagent (2026-06-17): "10s dla 11-token transformera na CPU to ANOMALIA -
moze waskie gardlo recall to ANN search w HSDB nie embedder". Ten skrypt
mierzy OSOBNO: (a) czysty vectorize (embed), (b) czysty search po gotowym
wektorze (ANN), (c) end-to-end search_text (embed+ANN). Twardy dowod gdzie
siedzi czas.
"""
import sys, os, time, statistics
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "sdks/python"))

def load_key():
    from dotenv import dotenv_values
    return dotenv_values(os.path.join(os.path.dirname(__file__), ".env")).get("HYPERSPACE_API_KEY", "")

COL = "gniewka_omniscient"

def med(xs): return statistics.median(xs) if xs else -1

def main():
    from hyperspace import HyperspaceClient
    c = HyperspaceClient(host="localhost:50051", api_key=load_key())
    texts = ["pamiec gniewki", "ramsey graph coloring", "co wiem o paulinie",
             "embedder gpu coreml", "hyperbolic lorentz distance"]

    print("=== (a) CZYSTY EMBED (vectorize) ===")
    emb = []
    vecs = []
    for t in texts:
        t0 = time.time(); v = c.vectorize(t); dt = time.time()-t0
        emb.append(dt); vecs.append(v)
        print(f"  {dt:6.3f}s  '{t[:30]}'")
    print(f"  MEDIANA embed: {med(emb):.3f}s")

    print("\n=== (b) CZYSTY ANN SEARCH (po gotowym wektorze, bez embed) ===")
    ann = []
    for v in vecs:
        try:
            t0 = time.time(); c.search(vector=v if not hasattr(v,'tolist') else v.tolist(), top_k=5, collection=COL); dt = time.time()-t0
            ann.append(dt); print(f"  {dt:6.3f}s")
        except Exception as e:
            print(f"  ANN err: {str(e)[:60]}")
    print(f"  MEDIANA ANN: {med(ann):.3f}s")

    print("\n=== (c) END-TO-END search_text (embed+ANN) ===")
    e2e = []
    for t in texts:
        try:
            t0 = time.time(); c.search_text(t, top_k=5, collection=COL); dt = time.time()-t0
            e2e.append(dt); print(f"  {dt:6.3f}s  '{t[:30]}'")
        except Exception as ex:
            print(f"  e2e err: {str(ex)[:60]}")
    print(f"  MEDIANA e2e: {med(e2e):.3f}s")

    print("\n=== WERDYKT ===")
    print(f"  embed={med(emb):.3f}s  ANN={med(ann):.3f}s  e2e={med(e2e):.3f}s")
    if med(emb) > med(ann)*3:
        print("  -> EMBEDDER to waskie gardlo (GPU/quant moglby pomoc)")
    elif med(ann) > med(emb)*3:
        print("  -> ANN SEARCH to waskie gardlo (czyszczenie 40k chunkow / index pomoze, NIE GPU)")
    else:
        print("  -> oba porownywalne")

if __name__ == "__main__":
    main()
