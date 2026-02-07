## 📊 Reproducible Benchmarks

**Don't trust marketing claims. Run the benchmarks yourself.**

```bash
cd benchmarks && ./run_all.sh
```

This will:
- Start HyperspaceDB, Qdrant, Weaviate, and Milvus in Docker
- Run identical workload on each database
- Generate comparison report with real numbers

**Duration**: ~10 minutes  
**See**: [`BENCHMARK_QUICKSTART.md`](BENCHMARK_QUICKSTART.md)

---

### Why Reproducible?

✅ **Fixed random seed** - Same vectors every time  
✅ **Identical workload** - Fair comparison  
✅ **Version pinned** - Consistent results  
✅ **Open source** - Audit the code  

**Anyone can verify our performance claims.**
