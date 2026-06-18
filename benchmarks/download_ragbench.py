#!/usr/bin/env python3
"""
download_datasets.py — Real-World Dataset Downloader & Local Cacher
=====================================================================
Downloads and caches ALL benchmark datasets used in run_rag_standards.py.

Supported dataset groups:
  Group A — Galileo RAGBench (rungalileo/ragbench):
    covidqa, finqa, cuad, msmarco, tatqa, pubmedqa, techqa, hotpotqa

  Group B — Structural / Graph / Legal (custom adapters):
    maud       — MAUD M&A merger agreement legal QA (TheAtticusProject/maud)
    legalbench — LegalBench legal reasoning tasks (nguha/legalbench, contract_qa subset)
    trec_ct    — TREC Clinical Trials 2021 patient-to-trial matching (irds/clinicaltrials_2021_trec-ct-2021)
    bioasq     — BioASQ biomedical QA retrieval (enelpol/rag-mini-bioasq)
    scidocs    — SciDocs citation prediction (BeIR/scidocs via BEIR format)

Usage:
    python3 download_datasets.py                   # Download everything
    python3 download_datasets.py --group a         # Only RAGBench 8 domains
    python3 download_datasets.py --group b         # Only the 5 new structural datasets
    python3 download_datasets.py --domain maud bioasq  # Specific domains only
"""

import os
import sys
import json
import argparse
from tqdm import tqdm

try:
    from datasets import load_dataset
    DATASETS_AVAILABLE = True
except ImportError:
    DATASETS_AVAILABLE = False

# ── Cache directories ──────────────────────────────────────────────────────────
BASE_DIR = os.path.abspath(os.path.join(os.path.dirname(__file__), "data"))
RAGBENCH_DIR = os.path.join(BASE_DIR, "ragbench")   # Group A (existing format)
EXTRA_DIR    = os.path.join(BASE_DIR, "extra")      # Group B (new datasets)

# ── Group A — Galileo RAGBench domains ────────────────────────────────────────
RAGBENCH_DOMAINS = [
    "covidqa", "finqa", "cuad", "msmarco",
    "tatqa", "pubmedqa", "techqa", "hotpotqa",
]

# ── Group B — Extra structural / domain-specific datasets ─────────────────────
# Each entry: (domain_id, description, download_function)
EXTRA_DOMAINS = ["maud", "legalbench", "trec_ct", "bioasq", "scidocs", "frames", "rgb"]

ALL_DOMAINS = RAGBENCH_DOMAINS + EXTRA_DOMAINS


# ══════════════════════════════════════════════════════════════════════════════
# Group A — Galileo RAGBench
# ══════════════════════════════════════════════════════════════════════════════

def download_ragbench(domain: str) -> bool:
    """Download one RAGBench domain (same format as original download_ragbench.py)."""
    print(f"\n📥 Downloading Galileo RAGBench: {domain.upper()}...")
    output_path = os.path.join(RAGBENCH_DIR, f"{domain}_ragbench.jsonl")
    os.makedirs(RAGBENCH_DIR, exist_ok=True)

    try:
        dataset = load_dataset("rungalileo/ragbench", domain, split="test", streaming=False)
        count = 0
        with open(output_path, "w", encoding="utf-8") as f:
            for row in tqdm(dataset, desc=f"  {domain}"):
                clean_row = {
                    "question": row["question"],
                    "documents": row["documents"],
                    "all_relevant_sentence_keys": row.get("all_relevant_sentence_keys", [])
                }
                f.write(json.dumps(clean_row, ensure_ascii=False) + "\n")
                count += 1
        print(f"   ✅ Cached {count} rows → {output_path}")
        return True
    except Exception as e:
        print(f"   ❌ Failed: {e}")
        return False


# ══════════════════════════════════════════════════════════════════════════════
# Group B — Structural / Domain-Specific Datasets
#
# All saved in the unified JSONL format:
#   {"question": str, "documents": [str, ...], "all_relevant_sentence_keys": [str, ...]}
#
# This is IDENTICAL to the RAGBench format so load_domain_dataset() can load them
# without any code changes — just a different cache directory path.
# ══════════════════════════════════════════════════════════════════════════════

def download_maud() -> bool:
    """
    MAUD — Merger Agreement Understanding Dataset
    Source: TheAtticusProject/maud on Hugging Face
    ~47k annotations across 152 M&A contracts (ABA 2021 Deal Points Study)
    Schema: text (contract passage), question, answer, contract_name
    Strategy: group passages by contract_name so each contract becomes a
    multi-document entry. The question for each entry is the deal-point question.
    """
    print(f"\n📥 Downloading MAUD (M&A Legal Contracts)...")
    output_path = os.path.join(EXTRA_DIR, "maud_ragbench.jsonl")
    os.makedirs(EXTRA_DIR, exist_ok=True)

    try:
        dataset = load_dataset("TheAtticusProject/maud", split="test", streaming=False)

        # Group rows by (contract_name, question) so each deal-point question
        # across one contract becomes a RAG entry with multiple text passages.
        from collections import defaultdict
        groups = defaultdict(list)  # key=(contract_name, question) -> list of texts

        for row in tqdm(dataset, desc="  indexing maud"):
            contract  = row.get("contract_name", "") or ""
            question  = row.get("question", "")    or ""
            text      = row.get("text", "")        or ""
            if not contract or not question or not text:
                continue
            key = (contract, question)
            groups[key].append(text)

        count = 0
        with open(output_path, "w", encoding="utf-8") as f:
            for (contract, question), passages in tqdm(groups.items(), desc="  writing maud"):
                # All passages for this deal-point are the candidate documents;
                # each is marked relevant (they all answer the same deal-point question)
                relevant_keys = [f"{i}_0" for i in range(len(passages))]
                clean_row = {
                    "question": question,
                    "documents": passages,
                    "all_relevant_sentence_keys": relevant_keys
                }
                f.write(json.dumps(clean_row, ensure_ascii=False) + "\n")
                count += 1

        print(f"   ✅ Cached {count} contract×question entries → {output_path}")
        return True
    except Exception as e:
        print(f"   ❌ Failed to download MAUD: {e}")
        return False


def download_legalbench() -> bool:
    """
    LegalBench — Legal Reasoning Tasks (contract_qa subset)
    Source: nguha/legalbench on Hugging Face
    We use the 'contract_qa' task which is closest to document retrieval:
    given a contract clause, answer yes/no questions — requires finding the right clause.
    Other good subsets for retrieval: 'privacy_policy_qa', 'sara_prolog'
    """
    print(f"\n📥 Downloading LegalBench (contract_qa subset)...")
    output_path = os.path.join(EXTRA_DIR, "legalbench_ragbench.jsonl")
    os.makedirs(EXTRA_DIR, exist_ok=True)

    # We'll try multiple subsets and merge them for a richer retrieval corpus
    subsets_to_try = ["contract_qa", "privacy_policy_qa", "definition_classification"]
    count = 0

    try:
        with open(output_path, "w", encoding="utf-8") as f:
            for subset in subsets_to_try:
                try:
                    dataset = load_dataset("nguha/legalbench", subset, split="test", streaming=False)
                    print(f"   Loading subset: {subset} ({len(dataset)} rows)")
                    for row in tqdm(dataset, desc=f"  legalbench/{subset}"):
                        text = row.get("text", "") or row.get("context", "") or ""
                        question = row.get("question", "") or row.get("instruction", "") or ""
                        answer = str(row.get("answer", "") or row.get("label", "") or "")

                        if not text or not question:
                            continue

                        # Format: question asks about the text; text is the relevant document
                        clean_row = {
                            "question": question,
                            "documents": [text],
                            "all_relevant_sentence_keys": ["0_0"]
                        }
                        f.write(json.dumps(clean_row, ensure_ascii=False) + "\n")
                        count += 1
                except Exception as sub_e:
                    print(f"   ⚠️  Subset {subset} failed: {sub_e}")
                    continue

        if count > 0:
            print(f"   ✅ Cached {count} rows total → {output_path}")
            return True
        else:
            print(f"   ❌ No rows loaded from any LegalBench subset")
            return False
    except Exception as e:
        print(f"   ❌ Failed to download LegalBench: {e}")
        return False


def download_trec_ct() -> bool:
    """
    NFCorpus — Biomedical Information Retrieval (BEIR format)
    Source: BeIR/nfcorpus on Hugging Face
    Replaces TREC Clinical Trials (broken trust_remote_code API).
    Task: given a health/nutrition query, find relevant medical literature.
    3633 medical documents, 323 queries, 12,334 qrels.
    Structural domain: queries map to clusters of thematically related papers
    (e.g. 'statin breast cancer' → multiple related studies).
    """
    print(f"\n📥 Downloading NFCorpus (Biomedical / Clinical retrieval, replaces TREC CT)...")
    output_path = os.path.join(EXTRA_DIR, "trec_ct_ragbench.jsonl")
    os.makedirs(EXTRA_DIR, exist_ok=True)

    try:
        print("   Loading queries...")
        queries_ds = load_dataset("BeIR/nfcorpus", "queries", split="queries", streaming=False)

        print("   Loading corpus (medical papers)...")
        corpus_ds  = load_dataset("BeIR/nfcorpus", "corpus",  split="corpus",  streaming=False)

        print("   Loading qrels...")
        qrels_ds   = load_dataset("BeIR/nfcorpus-qrels", split="test", streaming=False)

        # Build document map
        doc_map = {}
        for doc in tqdm(corpus_ds, desc="  indexing papers"):
            doc_id = str(doc.get("_id", "") or doc.get("id", ""))
            title  = doc.get("title", "") or ""
            text   = doc.get("text", "")  or ""
            full   = f"{title}. {text}".strip(". ")
            if doc_id and full:
                doc_map[doc_id] = full

        # Build qrels map: query_id -> [relevant_doc_ids]
        qrels_map = {}
        for qrel in qrels_ds:
            q_id  = str(qrel.get("query-id", ""))
            d_id  = str(qrel.get("corpus-id", ""))
            score = int(qrel.get("score", 0))
            if score >= 1:
                qrels_map.setdefault(q_id, []).append(d_id)

        count = 0
        with open(output_path, "w", encoding="utf-8") as f:
            for query in tqdm(queries_ds, desc="  nfcorpus"):
                q_id   = str(query.get("_id", "") or query.get("id", ""))
                q_text = query.get("text", "") or ""

                relevant_ids = qrels_map.get(q_id, [])
                if not relevant_ids or not q_text:
                    continue

                relevant_texts = [doc_map[d] for d in relevant_ids if d in doc_map]
                if not relevant_texts:
                    continue

                relevant_keys = [f"{i}_0" for i in range(len(relevant_texts))]
                clean_row = {
                    "question": q_text,
                    "documents": relevant_texts,
                    "all_relevant_sentence_keys": relevant_keys
                }
                f.write(json.dumps(clean_row, ensure_ascii=False) + "\n")
                count += 1

        print(f"   ✅ Cached {count} query-document pairs → {output_path}")
        return True
    except Exception as e:
        print(f"   ❌ Failed to download NFCorpus: {e}")
        return False


def download_bioasq() -> bool:
    """
    BioASQ — Biomedical Question Answering (RAG-mini subset)
    Source: enelpol/rag-mini-bioasq on Hugging Face
    Corpus split: 'test' (not 'passages' — that split doesn't exist)
    relevant_passage_ids field is a JSON string, not a Python list.
    """
    print(f"\n📥 Downloading BioASQ (rag-mini subset)...")
    output_path = os.path.join(EXTRA_DIR, "bioasq_ragbench.jsonl")
    os.makedirs(EXTRA_DIR, exist_ok=True)

    try:
        print("   Loading QA pairs...")
        qa_ds = load_dataset(
            "enelpol/rag-mini-bioasq", "question-answer-passages",
            split="test", streaming=False
        )

        print("   Loading passage corpus...")
        corpus_ds = load_dataset(
            "enelpol/rag-mini-bioasq", "text-corpus",
            split="test",  # Only split available is 'test'
            streaming=False
        )

        print("   Building passage index...")
        passage_map = {}
        for item in tqdm(corpus_ds, desc="  indexing passages"):
            pid  = str(item.get("id", ""))
            text = item.get("passage", "") or item.get("text", "") or ""
            if pid and text:
                passage_map[pid] = text

        count = 0
        with open(output_path, "w", encoding="utf-8") as f:
            for row in tqdm(qa_ds, desc="  bioasq"):
                question = row.get("question", "") or ""

                # relevant_passage_ids is stored as a JSON string like '["12345", "67890"]'
                raw_ids = row.get("relevant_passage_ids", "[]")
                if isinstance(raw_ids, str):
                    try:
                        relevant_ids = json.loads(raw_ids)
                    except (json.JSONDecodeError, ValueError):
                        relevant_ids = []
                elif isinstance(raw_ids, list):
                    relevant_ids = raw_ids
                else:
                    relevant_ids = []

                if not question or not relevant_ids:
                    continue

                relevant_texts = [passage_map[str(pid)] for pid in relevant_ids
                                  if str(pid) in passage_map]
                if not relevant_texts:
                    continue

                relevant_keys = [f"{i}_0" for i in range(len(relevant_texts))]
                clean_row = {
                    "question": question,
                    "documents": relevant_texts,
                    "all_relevant_sentence_keys": relevant_keys
                }
                f.write(json.dumps(clean_row, ensure_ascii=False) + "\n")
                count += 1

        print(f"   ✅ Cached {count} rows → {output_path}")
        return True
    except Exception as e:
        print(f"   ❌ Failed to download BioASQ: {e}")
        return False


def download_scidocs() -> bool:
    """
    SciDocs — Scientific Citation Prediction (BEIR format)
    Source: BeIR/scidocs on Hugging Face (BEIR benchmark format)
    Task: given a query paper's abstract, find cited papers in the corpus.
    This is a GRAPH/STRUCTURAL domain: citation graph traversal.
    Wave diffusion should excel here — papers cluster by research area in the graph.
    """
    print(f"\n📥 Downloading SciDocs (BEIR/citation prediction)...")
    output_path = os.path.join(EXTRA_DIR, "scidocs_ragbench.jsonl")
    os.makedirs(EXTRA_DIR, exist_ok=True)

    try:
        # BEIR format: separate corpus, queries, qrels splits
        print("   Loading queries (paper abstracts)...")
        queries_ds = load_dataset("BeIR/scidocs", "queries", split="queries", streaming=False)

        print("   Loading corpus (paper titles + abstracts)...")
        corpus_ds  = load_dataset("BeIR/scidocs", "corpus", split="corpus", streaming=False)

        print("   Loading qrels (citation relevance labels)...")
        qrels_ds   = load_dataset("BeIR/scidocs-qrels", split="test", streaming=False)

        # Build document map: doc_id -> text
        print("   Building document index...")
        doc_map = {}
        for doc in tqdm(corpus_ds, desc="  indexing papers"):
            doc_id = str(doc.get("_id", "") or doc.get("id", ""))
            title  = doc.get("title", "") or ""
            text   = doc.get("text", "") or ""
            full   = f"{title}. {text}".strip(". ")
            if doc_id and full:
                doc_map[doc_id] = full

        # Build qrels map: query_id -> list of cited doc_ids (score >= 1)
        qrels_map = {}
        for qrel in qrels_ds:
            q_id  = str(qrel.get("query-id", ""))
            d_id  = str(qrel.get("corpus-id", ""))
            score = int(qrel.get("score", 0))
            if score >= 1:
                qrels_map.setdefault(q_id, []).append(d_id)

        # Write output
        count = 0
        with open(output_path, "w", encoding="utf-8") as f:
            for query in tqdm(queries_ds, desc="  scidocs"):
                q_id   = str(query.get("_id", "") or query.get("id", ""))
                q_text = query.get("text", "") or ""

                relevant_ids = qrels_map.get(q_id, [])
                if not relevant_ids or not q_text:
                    continue

                relevant_texts = [doc_map[d] for d in relevant_ids if d in doc_map]
                if not relevant_texts:
                    continue

                relevant_keys = [f"{i}_0" for i in range(len(relevant_texts))]

                clean_row = {
                    "question": q_text,
                    "documents": relevant_texts,
                    "all_relevant_sentence_keys": relevant_keys
                }
                f.write(json.dumps(clean_row, ensure_ascii=False) + "\n")
                count += 1

        print(f"   ✅ Cached {count} rows → {output_path}")
        return True
    except Exception as e:
        print(f"   ❌ Failed to download SciDocs: {e}")
        print("   ℹ️  SciDocs requires both BeIR/scidocs and BeIR/scidocs-qrels.")
        return False


def extract_wiki_title(url):
    if not url or url == 'None' or not isinstance(url, str):
        return None
    url = url.split('#')[0]
    if '/wiki/' in url:
        title = url.split('/wiki/')[-1]
        import urllib.parse
        return urllib.parse.unquote(title).strip()
    return None


def download_frames() -> bool:
    """
    Google FRAMES — Factuality, Retrieval, And Reasoning Measurement Set
    Source: google/frames-benchmark on Hugging Face
    Wikipedia REST API fetches page extracts for the links, cached locally.
    """
    print(f"\n📥 Downloading Google FRAMES (Multi-hop Wikipedia)...")
    output_path = os.path.join(EXTRA_DIR, "frames_ragbench.jsonl")
    cache_path = os.path.join(EXTRA_DIR, "frames_wiki_cache.json")
    os.makedirs(EXTRA_DIR, exist_ok=True)

    try:
        import urllib.request
        import urllib.parse
        from concurrent.futures import ThreadPoolExecutor

        print("   Loading google/frames-benchmark dataset...")
        dataset = load_dataset("google/frames-benchmark", split="test")

        # Load existing cache if available
        wiki_cache = {}
        if os.path.exists(cache_path):
            try:
                with open(cache_path, "r", encoding="utf-8") as f:
                    wiki_cache = json.load(f)
                print(f"   💾 Loaded {len(wiki_cache)} cached Wikipedia summaries from {cache_path}")
            except Exception as ce:
                print(f"   ⚠️ Could not load cache: {ce}")

        # Extract all unique titles needed
        urls_to_fetch = set()
        for row in dataset:
            for i in range(1, 11):
                val = row.get(f"wikipedia_link_{i}")
                if val and val != "None":
                    urls_to_fetch.add(val)
            val = row.get("wikipedia_link_11+")
            if val and val != "None":
                urls_to_fetch.add(val)

        title_to_url = {}
        for url in urls_to_fetch:
            title = extract_wiki_title(url)
            if title:
                title_to_url[title] = url

        titles_needed = [t for t in title_to_url.keys() if t not in wiki_cache]
        print(f"   Total unique Wikipedia articles: {len(title_to_url)}")
        print(f"   Articles to fetch (uncached): {len(titles_needed)}")

        if titles_needed:
            print(f"   Fetching abstracts via Wikipedia REST API using 15 threads...")
            
            def fetch_summary(title):
                url = f"https://en.wikipedia.org/api/rest_v1/page/summary/{urllib.parse.quote(title)}"
                req = urllib.request.Request(
                    url,
                    headers={"User-Agent": "HyperspaceDB-Benchmark/1.0 (contact: info@hyperspacedb.com)"}
                )
                try:
                    with urllib.request.urlopen(req, timeout=10) as response:
                        res_data = json.loads(response.read().decode('utf-8'))
                        extract = res_data.get("extract", "")
                        return title, extract
                except Exception as e:
                    # Return empty string to cache failure and avoid re-fetching
                    return title, ""

            with ThreadPoolExecutor(max_workers=15) as executor:
                results = list(tqdm(executor.map(fetch_summary, titles_needed), total=len(titles_needed), desc="  fetching wiki"))

            # Update cache
            for title, extract in results:
                wiki_cache[title] = extract

            # Save cache
            with open(cache_path, "w", encoding="utf-8") as f:
                json.dump(wiki_cache, f, ensure_ascii=False, indent=2)
            print(f"   💾 Saved updated cache with {len(wiki_cache)} entries to {cache_path}")

        # Now, assemble the unified dataset
        count = 0
        with open(output_path, "w", encoding="utf-8") as f:
            for row in tqdm(dataset, desc="  assembling frames"):
                prompt = row.get("Prompt", "")
                if not prompt:
                    continue

                # Gather all associated documents (abstracts) for this row
                docs = []
                for i in range(1, 11):
                    val = row.get(f"wikipedia_link_{i}")
                    title = extract_wiki_title(val)
                    if title and wiki_cache.get(title):
                        docs.append(wiki_cache[title])
                val = row.get("wikipedia_link_11+")
                title = extract_wiki_title(val)
                if title and wiki_cache.get(title):
                    docs.append(wiki_cache[title])

                # Deduplicate docs while preserving order
                seen_docs = set()
                unique_docs = []
                for doc in docs:
                    if doc not in seen_docs:
                        seen_docs.add(doc)
                        unique_docs.append(doc)

                if not unique_docs:
                    continue

                relevant_keys = [f"{i}_0" for i in range(len(unique_docs))]
                clean_row = {
                    "question": prompt,
                    "documents": unique_docs,
                    "all_relevant_sentence_keys": relevant_keys
                }
                f.write(json.dumps(clean_row, ensure_ascii=False) + "\n")
                count += 1

        print(f"   ✅ Cached {count} FRAMES rows → {output_path}")
        return True
    except Exception as e:
        print(f"   ❌ Failed to download FRAMES: {e}")
        import traceback
        traceback.print_exc()
        return False


def download_rgb() -> bool:
    """
    RGB — Retrieval-Augmented Generation Benchmark
    Source: chen700564/RGB GitHub repository (data/en.json)
    Tests noise robustness and negative rejection.
    """
    print(f"\n📥 Downloading RGB (Noise Robustness Benchmark)...")
    output_path = os.path.join(EXTRA_DIR, "rgb_ragbench.jsonl")
    os.makedirs(EXTRA_DIR, exist_ok=True)

    url = "https://raw.githubusercontent.com/chen700564/RGB/master/data/en.json"
    try:
        import urllib.request
        print(f"   Fetching RGB English dataset from: {url}")
        req = urllib.request.Request(
            url,
            headers={"User-Agent": "HyperspaceDB-Benchmark/1.0 (contact: info@hyperspacedb.com)"}
        )
        with urllib.request.urlopen(req, timeout=30) as response:
            content = response.read().decode('utf-8')

        lines = content.strip().split('\n')
        count = 0
        with open(output_path, "w", encoding="utf-8") as f:
            for line in tqdm(lines, desc="  processing rgb"):
                if not line.strip():
                    continue
                row = json.loads(line)
                query = row.get("query", "")
                positives = row.get("positive", [])
                negatives = row.get("negative", [])

                if not query or not positives:
                    continue

                # Combine positives and negatives. Positives go first, so their indices are 0 to len(positives)-1
                combined_docs = positives + negatives
                relevant_keys = [f"{i}_0" for i in range(len(positives))]

                clean_row = {
                    "question": query,
                    "documents": combined_docs,
                    "all_relevant_sentence_keys": relevant_keys
                }
                f.write(json.dumps(clean_row, ensure_ascii=False) + "\n")
                count += 1

        print(f"   ✅ Cached {count} RGB rows → {output_path}")
        return True
    except Exception as e:
        print(f"   ❌ Failed to download RGB: {e}")
        return False


# ══════════════════════════════════════════════════════════════════════════════
# Dispatch table
# ══════════════════════════════════════════════════════════════════════════════

EXTRA_DOWNLOADERS = {
    "maud":       download_maud,
    "legalbench": download_legalbench,
    "trec_ct":    download_trec_ct,
    "bioasq":     download_bioasq,
    "scidocs":    download_scidocs,
    "frames":     download_frames,
    "rgb":        download_rgb,
}


def main():
    parser = argparse.ArgumentParser(
        description="Hyperspace RAG Benchmark — Dataset Downloader",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog=__doc__
    )
    parser.add_argument(
        "--group", choices=["a", "b", "all"], default="all",
        help="Dataset group: a=RAGBench, b=Structural extras, all=everything (default: all)"
    )
    parser.add_argument(
        "--domain", nargs="*",
        help=f"Specific domains to download. Available: {', '.join(ALL_DOMAINS)}"
    )
    args = parser.parse_args()

    if not DATASETS_AVAILABLE:
        print("❌ 'datasets' package is missing. Install with:")
        print("   pip install datasets huggingface_hub")
        sys.exit(1)

    # Determine which domains to download
    if args.domain:
        domains = [d.lower() for d in args.domain]
        invalid = [d for d in domains if d not in ALL_DOMAINS]
        if invalid:
            print(f"❌ Unknown domains: {invalid}")
            print(f"   Available: {ALL_DOMAINS}")
            sys.exit(1)
    elif args.group == "a":
        domains = RAGBENCH_DOMAINS
    elif args.group == "b":
        domains = EXTRA_DOMAINS
    else:  # all
        domains = ALL_DOMAINS

    print("=" * 70)
    print("🚀 Hyperspace RAG Benchmark — Dataset Downloader")
    print("=" * 70)
    print(f"Downloading {len(domains)} dataset(s): {', '.join(domains)}\n")

    success = []
    failed  = []

    for domain in domains:
        if domain in RAGBENCH_DOMAINS:
            ok = download_ragbench(domain)
        else:
            downloader = EXTRA_DOWNLOADERS.get(domain)
            if downloader:
                ok = downloader()
            else:
                print(f"⚠️  No downloader for '{domain}' — skipping")
                ok = False

        (success if ok else failed).append(domain)

    print("\n" + "=" * 70)
    print(f"🏁 Done. Success: {len(success)}/{len(domains)}")
    if success:
        print(f"   ✅ {', '.join(success)}")
    if failed:
        print(f"   ❌ Failed: {', '.join(failed)}")
    print(f"\nCache locations:")
    print(f"  RAGBench (Group A):  {RAGBENCH_DIR}")
    print(f"  Structural (Group B): {EXTRA_DIR}")
    print("=" * 70)


if __name__ == "__main__":
    main()
