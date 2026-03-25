from abc import ABC, abstractmethod
from typing import List, Optional

class BaseEmbedder(ABC):
    """Abstract Base Class for all Embedders."""
    
    @abstractmethod
    def encode(self, text: str) -> List[float]:
        """Encodes a single string into a vector."""
        pass

class OpenAIEmbedder(BaseEmbedder):
    """Embedder for OpenAI models (e.g., text-embedding-3-small)."""
    
    def __init__(self, api_key: str, model: str = "text-embedding-3-small"):
        try:
            import openai
        except ImportError:
            raise ImportError("Please run 'pip install openai' to use OpenAIEmbedder")
        
        self.client = openai.OpenAI(api_key=api_key)
        self.model = model

    def encode(self, text: str) -> List[float]:
        text = text.replace("\n", " ")
        return self.client.embeddings.create(input=[text], model=self.model).data[0].embedding

class OpenRouterEmbedder(BaseEmbedder):
    """Embedder for OpenRouter models (OpenAI compatible)."""
    
    def __init__(self, api_key: str, model: str, base_url: str = "https://openrouter.ai/api/v1"):
        try:
            import openai
        except ImportError:
            raise ImportError("Please run 'pip install openai' to use OpenRouterEmbedder")
            
        self.client = openai.OpenAI(
            base_url=base_url,
            api_key=api_key,
        )
        self.model = model

    def encode(self, text: str) -> List[float]:
        text = text.replace("\n", " ")
        return self.client.embeddings.create(input=[text], model=self.model).data[0].embedding

class CohereEmbedder(BaseEmbedder):
    """Embedder for Cohere models (e.g., embed-english-v3.0)."""
    
    def __init__(self, api_key: str, model: str = "embed-english-v3.0", input_type: str = "search_document"):
        try:
            import cohere
        except ImportError:
            raise ImportError("Please run 'pip install cohere' to use CohereEmbedder")
            
        self.client = cohere.Client(api_key)
        self.model = model
        self.input_type = input_type

    def encode(self, text: str) -> List[float]:
        # input_type is required for v3 models
        response = self.client.embed(
            texts=[text],
            model=self.model,
            input_type=self.input_type,
            embedding_types=['float']
        )
        return response.embeddings.float[0]

class VoyageEmbedder(BaseEmbedder):
    """Embedder for Voyage AI models (e.g., voyage-large-3.5)."""
    
    def __init__(self, api_key: str, model: str = "voyage-large-3.5"):
        try:
            import voyageai
        except ImportError:
            raise ImportError("Please run 'pip install voyageai' to use VoyageEmbedder")
        
        self.client = voyageai.Client(api_key=api_key)
        self.model = model

    def encode(self, text: str) -> List[float]:
        return self.client.embed([text], model=self.model).embeddings[0]

class GoogleEmbedder(BaseEmbedder):
    """Embedder for Google Gemini models (e.g., models/embedding-001)."""
    
    def __init__(self, api_key: str, model: str = "models/embedding-001"):
        try:
            import google.generativeai as genai
        except ImportError:
            raise ImportError("Please run 'pip install google-generativeai' to use GoogleEmbedder")
            
        genai.configure(api_key=api_key)
        self.model = model

    def encode(self, text: str) -> List[float]:
        import google.generativeai as genai
        result = genai.embed_content(
            model=self.model,
            content=text,
            task_type="retrieval_document"
        )
        return result['embedding']

class SentenceTransformerEmbedder(BaseEmbedder):
    """
    Local Embedder using SentenceTransformers (HuggingFace).
    Good for BAAI/bge-m3, all-MiniLM-L6-v2, etc.
    """
    
    def __init__(self, model_name: str = "BAAI/bge-m3", device: str = None):
        try:
            from sentence_transformers import SentenceTransformer
        except ImportError:
            raise ImportError("Please run 'pip install sentence-transformers' to use SentenceTransformerEmbedder")
        
        self.model = SentenceTransformer(model_name, device=device)

    def encode(self, text: str) -> List[float]:
        # Returns ndarray, convert to list
        return self.model.encode(text).tolist()
