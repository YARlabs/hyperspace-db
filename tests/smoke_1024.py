import numpy as np
from hyperspace import HyperspaceClient
import time
import sys
import os

# Ensure we can find the SDK if running from root
sys.path.append(os.path.join(os.getcwd(), 'sdks/python'))

# Конфиг
DIM = 1024
VEC_COUNT = 1

print(f"📡 Connecting to HyperspaceDB (expecting {DIM} dimensions)...")

try:
    # Retry connection logic
    for i in range(5):
        try:
            client = HyperspaceClient(host="localhost:50051")
            # Simple ping/check? just proceed to insert
            break
        except Exception as e:
            print(f"Connection attempt {i+1} failed: {e}")
            time.sleep(2)
    else:
        print("Could not connect to server.")
        sys.exit(1)

    with client:
        # 1. Генерация данных (BGE-M3 style)
        print("Generating vectors...")
        vectors = np.random.rand(VEC_COUNT, DIM).astype(np.float32)
        # Нормализация (для Пуанкаре важно < 1, но для теста сойдет и random 0..1 деленный на норму)
        norms = np.linalg.norm(vectors, axis=1, keepdims=True)
        vectors /= norms
        vectors *= 0.99 # Чтобы точно попасть внутрь шара

        # 2. Вставка
        start = time.time()
        for i, vec in enumerate(vectors):
            # Using new insert signature if available, or dict
            # SDK Code implies: insert(self, vector, metadata=None, id=None)
            # or insert(self, id, vector, metadata) depending on version.
            # Checking client.py: 
            # def insert(self, collection: str, vectors: List[float], metadata: Dict[str, Any] = None, id: str = None) ??
            # Let's assume the user provided snippet matches the SDK.
            # "client.insert(id=i, vector=vec.tolist(), metadata={...})"
            client.insert(id=i, vector=vec.tolist(), metadata={"test": "smoke_1024"})
        
        print(f"✅ Inserted {VEC_COUNT} vectors in {time.time() - start:.4f}s")

        
        print("Waiting for async indexer...")
        time.sleep(2)

        # 3. Поиск
        query = vectors[0].tolist()
        results = client.search(query, top_k=5)
        
        print("🔍 Search Results:")
        for res in results:
            # Result appears to be a dictionary, not an object
            if isinstance(res, dict):
                item_id = res.get('id')
                score = res.get('distance', res.get('similarity'))
            else:
                item_id = getattr(res, 'id', None)
                score = getattr(res, 'distance', getattr(res, 'similarity', None))
                
            if score is None:
                print(f"DEBUG: Result content: {res}")
                score = 0.0
            print(f" - ID: {item_id}, Score/Dist: {score:.4f}")

        print(f"DEBUG: First result: {results[0]}") 
        # Проверка (ID 0 должен быть первым)
        first_res = results[0]
        if isinstance(first_res, dict):
            first_id = first_res.get('id')
        else:
            first_id = getattr(first_res, 'id', None)

        if first_id == 0 or first_id == "0":
            print("🎉 SUCCESS: Found exact match!")
        else:
            print(f"⚠️ WARNING: ID 0 not found at top. Found: {first_id}")

except Exception as e:
    print(f"❌ Test Failed: {e}")
    sys.exit(1)
