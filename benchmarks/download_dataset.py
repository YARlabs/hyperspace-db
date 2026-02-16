#!/usr/bin/env python3
import sys
import os

# Ensure we can import from vectordb_bench
try:
    from vectordb_bench.backend.cases import (
        Performance1536D50K,
        Performance1536D500K,
        Performance1536D500K1P,
        Performance1536D500K99P,
        Performance1536D5M,
        Performance1536D5M1P,
        Performance1536D5M99P,
        Performance768D1M,
        Performance768D1M1P,
        Performance768D1M99P,
        Performance768D10M,
        Performance768D10M1P,
        Performance768D10M99P,
        Performance768D100M,
        Performance1024D1M,
        Performance1024D10M,
        CapacityDim128,
        CapacityDim960,
    )
    from vectordb_bench.backend.data_source import DatasetSource
except ImportError:
    print("❌ Error: vectordb_bench not found. Please install it first:")
    print("pip install vectordb-bench")
    sys.exit(1)

CASE_MAP = {
    "1": ("Performance1536D50K", Performance1536D50K),
    "2": ("Performance1536D500K", Performance1536D500K),
    "3": ("Performance1536D5M", Performance1536D5M),
    "4": ("Performance768D1M", Performance768D1M),
    "5": ("Performance768D10M", Performance768D10M),
    "6": ("Performance1024D1M", Performance1024D1M),
    "7": ("Performance1024D10M", Performance1024D10M),
    "8": ("CapacityDim128", CapacityDim128),
    "9": ("CapacityDim960", CapacityDim960),
    "10": ("Performance768D100M", Performance768D100M),
}

def main():
    print("\n📦 Dataset Downloader for Hyperspace Benchmarks")
    print("============================================")
    for key, (name, _) in CASE_MAP.items():
        print(f"{key:>2}. {name}")
    
    choice = input("\nSelect dataset number to download (or 'q' to quit): ").strip()
    
    if choice.lower() == 'q':
        return

    if choice in CASE_MAP:
        name, case_cls = CASE_MAP[choice]
        print(f"\n🚀 Preparing to download: {name}")
        try:
            case_inst = case_cls()
            ds = case_inst.dataset
            print(f"   Dataset Name: {ds.data.name}")
            print(f"   Target Directory: {ds.data_dir}")
            print(f"   Size: {ds.data.size:,} | Dimension: {ds.data.dim}")
            
            print("\n📥 Downloading from S3... (This might take a while)")
            ds.prepare(source=DatasetSource.S3)
            print(f"\n✅ Successfully downloaded and verified {name}!")
            print(f"   Files are located in: {ds.data_dir}")
        except Exception as e:
            print(f"\n❌ Error during download: {e}")
    else:
        print("Invalid selection.")

if __name__ == "__main__":
    main()
