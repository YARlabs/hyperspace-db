"""YarHyperbolicEmbeddings implementation for LangChain.

This model natively operates in Hyperbolic (Lorentz) space.
Output vectors are Lorentz vectors (t + spatial dimensions).
"""

from typing import List, Optional, Any
import logging
from langchain_core.embeddings import Embeddings

logger = logging.getLogger(__name__)

class YARLabsEmbeddings(Embeddings):
    """LangChain wrapper for YAR.INK V5 Embedding Model.
    
    Model: YARlabs/v5_Embedding_0.5B
    Geometry: Lorentz (Hyperbolic)
    """

    def __init__(
        self,
        model_id: str = "YARlabs/v5_Embedding_0.5B",
        target_dim: int = 64,
        device: Optional[str] = None,
        trust_remote_code: bool = True,
        **kwargs: Any
    ):
        """Initialize YAR Labs embeddings.
        
        Args:
            model_id: HuggingFace model path or local directory.
            target_dim: Dimensionality of the spatial part (total dim will be target_dim + 1).
            device: 'cpu', 'cuda', 'mps'.
            trust_remote_code: Required for custom YAR architecture.
        """
        try:
            import torch
            from transformers import AutoTokenizer, AutoModel
        except ImportError:
            raise ImportError(
                "Could not import torch or transformers. "
                "Please install them with `pip install torch transformers`."
            )
            
        if device is None:
            self.device = "cuda" if torch.cuda.is_available() else ("mps" if torch.backends.mps.is_available() else "cpu")
        else:
            self.device = device
            
        logger.info(f"Loading YARLabs model '{model_id}' onto {self.device}...")
        self.tokenizer = AutoTokenizer.from_pretrained(model_id, trust_remote_code=trust_remote_code)
        self.model = AutoModel.from_pretrained(model_id, trust_remote_code=trust_remote_code).to(self.device)
        self.model.eval()
        self.target_dim = target_dim

    def embed_documents(self, texts: List[str]) -> List[List[float]]:
        """Embed a list of documents into Lorentz space."""
        import torch
        
        # Simple batching (could be improved for very large lists)
        inputs = self.tokenizer(
            texts, 
            padding=True, 
            truncation=True, 
            max_length=512, 
            return_tensors="pt"
        ).to(self.device)
        
        with torch.no_grad():
            # The model forward call accepts target_dim
            vecs = self.model(**inputs, target_dim=self.target_dim)
            
        return vecs.cpu().tolist()

    def embed_query(self, text: str) -> List[float]:
        """Embed a single query into Lorentz space."""
        return self.embed_documents([text])[0]
