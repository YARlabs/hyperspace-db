#!/usr/bin/env python3
"""
_clean_code_chunks.py - C: usuwa code_chunk z produkcyjnej gniewka_omniscient.

ROOT CAUSE (2026-06-17): code_rag.py zalal pamiec produkcyjna ~40k chunkami kodu
(metadata.type='code_chunk') -> dlawil recall + server-side ONNX vectorizer.
Source naprawiony osobno (code_rag COLLECTION -> gniewka_code).

BEZPIECZENSTWO:
  - DRY-RUN domyslnie (tylko liczy + pokazuje probke). Czyszczenie WYMAGA --execute.
  - delete() = tombstone (znika z search natychmiast, count maleje po kompakcji).
  - Backup cold ZROBIONY wczesniej (hsdb_backup_20260617_*, checksum MATCH).
  - Zachowuje WSZYSTKO co NIE jest code_chunk (prawdziwa pamiec z 12 dni nietknieta).

USAGE:
  python3 _clean_code_chunks.py            # DRY-RUN: policz + probka
  python3 _clean_code_chunks.py --execute  # FAKTYCZNE czyszczenie (po zgodzie Pauliny)
"""
import sys, os, time, argparse
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "sdks/python"))

def load_key():
    from dotenv import dotenv_values
    d = dotenv_values(os.path.join(os.path.dirname(__file__), ".env"))
    return d.get("HYPERSPACE_API_KEY", "")

COLLECTION = "gniewka_omniscient"
CHUNK_TYPE = "code_chunk"

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--execute", action="store_true", help="faktyczne czyszczenie (domyslnie dry-run)")
    ap.add_argument("--batch", type=int, default=500, help="ile ID zbierac na raz")
    args = ap.parse_args()

    from hyperspace import HyperspaceClient
    c = HyperspaceClient(host="localhost:50051", api_key=load_key())

    # sanity: collection zyje
    cols = c.list_collections()
    info = next((x for x in cols if x.get("name") == COLLECTION), None)
    if not info:
        print(f"STOP: brak kolekcji {COLLECTION}")
        return 1
    print(f"collection {COLLECTION}: count={info['count']} dim={info['dimension']} metric={info['metric']}")

    # zbierz ID code_chunk przez filter search (iteracyjnie, duzy top_k)
    print(f"\nzbieram ID gdzie type={CHUNK_TYPE} (filter search)...")
    ids = set()
    # search_text z filtrem - trzeba roznych zapytan zeby pokryc, ALBO duzy top_k na pustym
    # strategia: kilka generycznych query + filter type=code_chunk, agreguj ID
    seeds = ["def", "class", "import", "return", "function", "if", "self", "for", "value", "error",
             "test", "config", "path", "data", "model", "async", "await", "None", "True", "list"]
    for q in seeds:
        try:
            hits = c.search_text(q, top_k=2000, filter={"type": CHUNK_TYPE}, collection=COLLECTION) or []
            for h in hits:
                hid = h.get("id")
                if hid is not None:
                    ids.add(hid)
        except Exception as e:
            print(f"  query '{q}' err: {str(e)[:50]}")
        time.sleep(0.05)
    print(f"zebrano UNIKALNYCH code_chunk ID: {len(ids)}")

    if not ids:
        print("brak code_chunk do usuniecia (albo filter nie dziala serverside - sprawdz)")
        return 0

    # probka co usuwamy
    print("\nPROBKA (pierwsze 3):")
    sample = list(ids)[:3]
    for sid in sample:
        try:
            hh = c.search_text("code", top_k=1, filter={"type": CHUNK_TYPE}, collection=COLLECTION)
        except Exception:
            pass
    print(f"  ID: {sample}")

    if not args.execute:
        print(f"\n=== DRY-RUN === znaleziono ~{len(ids)} code_chunk. NIE usunieto nic.")
        print("Aby wyczyscic: python3 _clean_code_chunks.py --execute")
        return 0

    # FAKTYCZNE usuwanie
    print(f"\n=== EXECUTE === usuwam {len(ids)} code_chunk...")
    deleted = 0
    t0 = time.time()
    for hid in ids:
        try:
            if c.delete(hid, collection=COLLECTION):
                deleted += 1
        except Exception as e:
            if deleted % 1000 == 0:
                print(f"  err na {hid}: {str(e)[:40]}")
    dt = time.time() - t0
    print(f"usunieto {deleted}/{len(ids)} w {dt:.1f}s (tombstone - count zmaleje po kompakcji)")
    print("UWAGA: to byla 1 runda. Code_chunk moze byc wiecej niz filter zlapal -")
    print("uruchom ponownie az 'zebrano' bedzie ~0.")
    return 0

if __name__ == "__main__":
    sys.exit(main())
