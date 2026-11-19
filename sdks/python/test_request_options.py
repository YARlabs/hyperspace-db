"""Offline tests for request building (no server needed): compact vectors and ef_search.

Run: python3 test_request_options.py   (or: python3 -m pytest test_request_options.py)
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from hyperspace import HyperspaceClient  # noqa: E402


class _CaptureStub:
    """Records the request passed to each RPC instead of sending it."""

    def __init__(self):
        self.calls = {}

    def __getattr__(self, rpc):
        def call(req, metadata=None):
            self.calls[rpc] = req
            raise _Stop()

        return call


class _Stop(Exception):
    pass


def _client(**kw):
    c = HyperspaceClient("localhost:1", **kw)
    stub = _CaptureStub()
    c.stubs = [stub] * len(c.stubs)
    return c, stub


def _call(fn, *a, **kw):
    try:
        fn(*a, **kw)
    except _Stop:
        pass
    except Exception as e:  # SDK may wrap errors; the request is captured either way
        if not isinstance(getattr(e, "__cause__", None), _Stop):
            pass


def test_ef_kwargs():
    assert HyperspaceClient._ef_kwargs(None) == {}
    assert HyperspaceClient._ef_kwargs(0) == {"ef_search": 0}
    assert HyperspaceClient._ef_kwargs(256) == {"ef_search": 256}
    try:
        HyperspaceClient._ef_kwargs(-1)
        raise AssertionError("negative ef_search must be rejected")
    except ValueError:
        pass


def test_search_sends_ef_search_only_when_set():
    c, stub = _client()
    _call(c.search, vector=[0.1, 0.2], top_k=5, collection="c", ef_search=300)
    req = stub.calls["Search"]
    assert req.HasField("ef_search") and req.ef_search == 300
    _call(c.search, vector=[0.1, 0.2], top_k=5, collection="c")
    assert not stub.calls["Search"].HasField("ef_search")


def test_batch_and_multi_collection_forward_ef_search():
    c, stub = _client()
    _call(c.search_batch, [[0.1, 0.2], [0.3, 0.4]], top_k=3, collection="c", ef_search=128)
    assert all(s.ef_search == 128 for s in stub.calls["SearchBatch"].searches)
    _call(c.search_multi_collection, [0.1, 0.2], ["a", "b"], top_k=3, ef_search=64)
    assert stub.calls["SearchMultiCollection"].ef_search == 64


def test_compact_vectors_uses_vector_f32():
    c, stub = _client(compact_vectors=True)
    _call(c.search, vector=[0.5, 0.25], top_k=1, collection="c")
    req = stub.calls["Search"]
    assert list(req.vector) == [] and len(req.vector_f32) == 2


if __name__ == "__main__":
    for name, fn in sorted(globals().items()):
        if name.startswith("test_") and callable(fn):
            fn()
            print(f"ok  {name}")
