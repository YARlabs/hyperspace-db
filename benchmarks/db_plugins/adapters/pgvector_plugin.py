
import time
import numpy as np
from tqdm import tqdm
from db_plugins.base import DatabasePlugin
from plugin_runtime import BenchmarkContext, Result
import run_benchmark_legacy as legacy

class PgVectorPlugin(DatabasePlugin):
    name = "pgvector"

    def is_available(self) -> bool:
        try:
            import psycopg2
            from pgvector.psycopg2 import register_vector
            return True
        except ImportError:
            return False

    def run(self, ctx: BenchmarkContext) -> Result:
        import psycopg2
        from pgvector.psycopg2 import register_vector
        
        try:
            conn = psycopg2.connect(
                dbname="vectordb",
                user="postgres",
                password="password",
                host="localhost",
                port=5432
            )
            # Enable auto-commit for DDL statements
            conn.autocommit = True
            
            with conn.cursor() as cur:
                cur.execute("CREATE EXTENSION IF NOT EXISTS vector")
                register_vector(conn)
                
                table_name = "bench_semantic"
                cur.execute(f"DROP TABLE IF EXISTS {table_name}")
                cur.execute(f"CREATE TABLE {table_name} (id bigserial PRIMARY KEY, doc_id text, embedding vector({ctx.cfg.dim_base}))")
            
            # Start Insertion
            print("   Inserting into Pgvector...")
            t0 = time.time()
            
            # Use COPY for fast bulk insert?
            # Or standard INSERT. For benchmarking, standard batch insert is safer/common.
            batch_size = 1000
            
            with conn.cursor() as cur:
                # Prepare data
                args_str = []
                for i in tqdm(range(0, len(ctx.doc_vecs_euc), batch_size), desc="Prepare Batch"):
                    batch_vecs = ctx.doc_vecs_euc[i : i + batch_size]
                    batch_ids = ctx.doc_ids[i : i + batch_size]
                    
                    # Construct INSERT VALUES efficiently
                    vals = [
                        (doc_id, vec.tolist())
                        for doc_id, vec in zip(batch_ids, batch_vecs)
                    ]
                    
                    # Using execute_values if possible, but manual constructing for simplicity if extras is not installed
                    # pgvector python client supports list directly
                    from psycopg2.extras import execute_values
                    execute_values(cur, 
                        f"INSERT INTO {table_name} (doc_id, embedding) VALUES %s", 
                        vals, 
                        template="(%s, %s::vector)"
                    )
            
            # Create Index (HNSW)
            print("   Building HNSW Index in Pgvector...")
            with conn.cursor() as cur:
                # cosine distance (<->)
                cur.execute(f"CREATE INDEX ON {table_name} USING hnsw (embedding vector_cosine_ops) WITH (m = 16, ef_construction = 64)")
                
            v_dur = time.time() - t0
            
            # Search
            print("   Searching Pgvector...")
            all_res_ids = []
            lats = []
            
            search_t0 = time.time()
            with conn.cursor() as cur:
                cur.execute(f"SET hnsw.ef_search = 64") # increased search accuracy
                
                for i, q_vec in enumerate(tqdm(ctx.q_vecs_euc, desc="Pgvector Search")):
                    ts = time.time()
                    cur.execute(f"SELECT doc_id FROM {table_name} ORDER BY embedding <=> %s::vector LIMIT 10", (q_vec.tolist(),))
                    res = cur.fetchall()
                    lats.append((time.time() - ts) * 1000)
                    all_res_ids.append([str(r[0]) for r in res])
            
            search_dur = time.time() - search_t0
            
            # Metrics
            all_gt_ids = [ctx.valid_qrels.get(qid, []) for qid in ctx.test_query_ids]
            recall, mrr, ndcg = legacy.calculate_accuracy(all_res_ids, all_gt_ids, 10)
            recall_sys = legacy.calculate_system_recall(all_res_ids, ctx.math_gt_euc, 10)
            
            # Concurrency
            q_list = ctx.q_vecs_euc[0].tolist()
            def pg_query():
                with conn.cursor() as c:
                    c.execute(f"SELECT doc_id FROM {table_name} ORDER BY embedding <=> %s::vector LIMIT 10", (q_list,))
                    c.fetchall()
            
            conc = legacy.run_concurrency_profile(pg_query)
            disk = legacy.format_size(legacy.get_docker_disk("postgres"))
            
            with conn.cursor() as cur:
                cur.execute(f"DROP TABLE {table_name}")
            conn.close()
            
            return Result(
                database="Pgvector",
                dimension=ctx.cfg.dim_base,
                geometry="Euclidean",
                metric="Cosine",
                insert_qps=len(ctx.docs) / v_dur,
                search_qps=len(ctx.test_queries) / search_dur,
                p50=np.percentile(lats, 50),
                p95=np.percentile(lats, 95),
                p99=np.percentile(lats, 99),
                recall=recall,
                recall_sys=recall_sys,
                mrr=mrr,
                ndcg=ndcg,
                c1_qps=conc.get(1, 0.0),
                c10_qps=conc.get(10, 0.0),
                c30_qps=conc.get(30, 0.0),
                disk_usage=disk,
                status="Success"
            )

        except Exception as e:
            return Result("Pgvector", ctx.cfg.dim_base, "Euclidean", "Cosine", 0,0,0,0,0,0,0,0,0,0,0,0,"0", f"Error: {e}")

PLUGIN = PgVectorPlugin()
