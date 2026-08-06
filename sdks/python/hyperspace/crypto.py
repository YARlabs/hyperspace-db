import hashlib
import hmac
import os
from cryptography.hazmat.primitives.ciphers.aead import AESGCM

def derive_keys(password: str, collection_name: str) -> tuple:
    """
    Derives deterministic keys for a given collection from a user password.
    Returns: (aes_key: bytes, hmac_key: bytes)
    """
    # Salt is SHA-256 of the collection name to make it unique per collection
    salt = hashlib.sha256(collection_name.encode('utf-8')).digest()
    # Derive 64 bytes: 32 bytes for AES, 32 bytes for HMAC
    derived = hashlib.pbkdf2_hmac('sha256', password.encode('utf-8'), salt, 100_000, 64)
    return derived[:32], derived[32:]

def encrypt_payload(payload: bytes, aes_key: bytes) -> bytes:
    """Encrypts bytes using AES-256-GCM with a random IV."""
    iv = os.urandom(12)
    aesgcm = AESGCM(aes_key)
    ciphertext = aesgcm.encrypt(iv, payload, None)
    return iv + ciphertext

def decrypt_payload(ciphertext_with_iv: bytes, aes_key: bytes) -> bytes:
    """Decrypts AES-256-GCM ciphertext."""
    if len(ciphertext_with_iv) < 12:
        raise ValueError("Ciphertext too short")
    iv = ciphertext_with_iv[:12]
    ciphertext = ciphertext_with_iv[12:]
    aesgcm = AESGCM(aes_key)
    return aesgcm.decrypt(iv, ciphertext, None)

def hash_metadata_key(key: str, hmac_key: bytes) -> str:
    """Hashes a metadata key with HMAC-SHA256, prefixing with 'tag_'."""
    h = hmac.new(hmac_key, key.encode('utf-8'), hashlib.sha256)
    return "tag_" + h.hexdigest()[:16]

def hash_metadata_value(value, hmac_key: bytes) -> str:
    """Hashes a metadata value with HMAC-SHA256, prefixing with 'val_'."""
    val_str = str(value)
    h = hmac.new(hmac_key, val_str.encode('utf-8'), hashlib.sha256)
    return "val_" + h.hexdigest()
