from google.protobuf.internal import containers as _containers
from google.protobuf.internal import enum_type_wrapper as _enum_type_wrapper
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class QuantizationMode(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    NONE: _ClassVar[QuantizationMode]
    SCALAR_I8: _ClassVar[QuantizationMode]

class DurabilityLevel(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    DEFAULT_LEVEL: _ClassVar[DurabilityLevel]
    ASYNC: _ClassVar[DurabilityLevel]
    BATCH: _ClassVar[DurabilityLevel]
    STRICT: _ClassVar[DurabilityLevel]

class EventType(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    EVENT_UNKNOWN: _ClassVar[EventType]
    VECTOR_INSERTED: _ClassVar[EventType]
    VECTOR_DELETED: _ClassVar[EventType]
NONE: QuantizationMode
SCALAR_I8: QuantizationMode
DEFAULT_LEVEL: DurabilityLevel
ASYNC: DurabilityLevel
BATCH: DurabilityLevel
STRICT: DurabilityLevel
EVENT_UNKNOWN: EventType
VECTOR_INSERTED: EventType
VECTOR_DELETED: EventType

class ReplicationRequest(_message.Message):
    __slots__ = ("last_logical_clock",)
    LAST_LOGICAL_CLOCK_FIELD_NUMBER: _ClassVar[int]
    last_logical_clock: int
    def __init__(self, last_logical_clock: _Optional[int] = ...) -> None: ...

class ReplicationLog(_message.Message):
    __slots__ = ("logical_clock", "origin_node_id", "collection", "insert", "create_collection", "delete_collection", "delete")
    LOGICAL_CLOCK_FIELD_NUMBER: _ClassVar[int]
    ORIGIN_NODE_ID_FIELD_NUMBER: _ClassVar[int]
    COLLECTION_FIELD_NUMBER: _ClassVar[int]
    INSERT_FIELD_NUMBER: _ClassVar[int]
    CREATE_COLLECTION_FIELD_NUMBER: _ClassVar[int]
    DELETE_COLLECTION_FIELD_NUMBER: _ClassVar[int]
    DELETE_FIELD_NUMBER: _ClassVar[int]
    logical_clock: int
    origin_node_id: str
    collection: str
    insert: InsertOp
    create_collection: CreateCollectionOp
    delete_collection: DeleteCollectionOp
    delete: DeleteOp
    def __init__(self, logical_clock: _Optional[int] = ..., origin_node_id: _Optional[str] = ..., collection: _Optional[str] = ..., insert: _Optional[_Union[InsertOp, _Mapping]] = ..., create_collection: _Optional[_Union[CreateCollectionOp, _Mapping]] = ..., delete_collection: _Optional[_Union[DeleteCollectionOp, _Mapping]] = ..., delete: _Optional[_Union[DeleteOp, _Mapping]] = ...) -> None: ...

class InsertOp(_message.Message):
    __slots__ = ("id", "vector", "metadata", "typed_metadata")
    class MetadataEntry(_message.Message):
        __slots__ = ("key", "value")
        KEY_FIELD_NUMBER: _ClassVar[int]
        VALUE_FIELD_NUMBER: _ClassVar[int]
        key: str
        value: str
        def __init__(self, key: _Optional[str] = ..., value: _Optional[str] = ...) -> None: ...
    class TypedMetadataEntry(_message.Message):
        __slots__ = ("key", "value")
        KEY_FIELD_NUMBER: _ClassVar[int]
        VALUE_FIELD_NUMBER: _ClassVar[int]
        key: str
        value: MetadataValue
        def __init__(self, key: _Optional[str] = ..., value: _Optional[_Union[MetadataValue, _Mapping]] = ...) -> None: ...
    ID_FIELD_NUMBER: _ClassVar[int]
    VECTOR_FIELD_NUMBER: _ClassVar[int]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    TYPED_METADATA_FIELD_NUMBER: _ClassVar[int]
    id: int
    vector: _containers.RepeatedScalarFieldContainer[float]
    metadata: _containers.ScalarMap[str, str]
    typed_metadata: _containers.MessageMap[str, MetadataValue]
    def __init__(self, id: _Optional[int] = ..., vector: _Optional[_Iterable[float]] = ..., metadata: _Optional[_Mapping[str, str]] = ..., typed_metadata: _Optional[_Mapping[str, MetadataValue]] = ...) -> None: ...

class CreateCollectionOp(_message.Message):
    __slots__ = ("dimension", "metric")
    DIMENSION_FIELD_NUMBER: _ClassVar[int]
    METRIC_FIELD_NUMBER: _ClassVar[int]
    dimension: int
    metric: str
    def __init__(self, dimension: _Optional[int] = ..., metric: _Optional[str] = ...) -> None: ...

class DeleteCollectionOp(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class DeleteOp(_message.Message):
    __slots__ = ("id",)
    ID_FIELD_NUMBER: _ClassVar[int]
    id: int
    def __init__(self, id: _Optional[int] = ...) -> None: ...

class QuantizationConfig(_message.Message):
    __slots__ = ("mode",)
    MODE_FIELD_NUMBER: _ClassVar[int]
    mode: QuantizationMode
    def __init__(self, mode: _Optional[_Union[QuantizationMode, str]] = ...) -> None: ...

class CreateCollectionRequest(_message.Message):
    __slots__ = ("name", "dimension", "metric")
    NAME_FIELD_NUMBER: _ClassVar[int]
    DIMENSION_FIELD_NUMBER: _ClassVar[int]
    METRIC_FIELD_NUMBER: _ClassVar[int]
    name: str
    dimension: int
    metric: str
    def __init__(self, name: _Optional[str] = ..., dimension: _Optional[int] = ..., metric: _Optional[str] = ...) -> None: ...

class DeleteCollectionRequest(_message.Message):
    __slots__ = ("name",)
    NAME_FIELD_NUMBER: _ClassVar[int]
    name: str
    def __init__(self, name: _Optional[str] = ...) -> None: ...

class ListCollectionsResponse(_message.Message):
    __slots__ = ("collections",)
    COLLECTIONS_FIELD_NUMBER: _ClassVar[int]
    collections: _containers.RepeatedScalarFieldContainer[str]
    def __init__(self, collections: _Optional[_Iterable[str]] = ...) -> None: ...

class CollectionStatsRequest(_message.Message):
    __slots__ = ("name",)
    NAME_FIELD_NUMBER: _ClassVar[int]
    name: str
    def __init__(self, name: _Optional[str] = ...) -> None: ...

class CollectionStatsResponse(_message.Message):
    __slots__ = ("count", "dimension", "metric", "indexing_queue")
    COUNT_FIELD_NUMBER: _ClassVar[int]
    DIMENSION_FIELD_NUMBER: _ClassVar[int]
    METRIC_FIELD_NUMBER: _ClassVar[int]
    INDEXING_QUEUE_FIELD_NUMBER: _ClassVar[int]
    count: int
    dimension: int
    metric: str
    indexing_queue: int
    def __init__(self, count: _Optional[int] = ..., dimension: _Optional[int] = ..., metric: _Optional[str] = ..., indexing_queue: _Optional[int] = ...) -> None: ...

class RebuildIndexRequest(_message.Message):
    __slots__ = ("name", "filter_query")
    NAME_FIELD_NUMBER: _ClassVar[int]
    FILTER_QUERY_FIELD_NUMBER: _ClassVar[int]
    name: str
    filter_query: VacuumFilterQuery
    def __init__(self, name: _Optional[str] = ..., filter_query: _Optional[_Union[VacuumFilterQuery, _Mapping]] = ...) -> None: ...

class ConfigUpdate(_message.Message):
    __slots__ = ("collection", "ef_search", "ef_construction")
    COLLECTION_FIELD_NUMBER: _ClassVar[int]
    EF_SEARCH_FIELD_NUMBER: _ClassVar[int]
    EF_CONSTRUCTION_FIELD_NUMBER: _ClassVar[int]
    collection: str
    ef_search: int
    ef_construction: int
    def __init__(self, collection: _Optional[str] = ..., ef_search: _Optional[int] = ..., ef_construction: _Optional[int] = ...) -> None: ...

class VacuumFilterQuery(_message.Message):
    __slots__ = ("key", "op", "value")
    KEY_FIELD_NUMBER: _ClassVar[int]
    OP_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    key: str
    op: str
    value: float
    def __init__(self, key: _Optional[str] = ..., op: _Optional[str] = ..., value: _Optional[float] = ...) -> None: ...

class ReconsolidationRequest(_message.Message):
    __slots__ = ("collection", "target_vector", "learning_rate")
    COLLECTION_FIELD_NUMBER: _ClassVar[int]
    TARGET_VECTOR_FIELD_NUMBER: _ClassVar[int]
    LEARNING_RATE_FIELD_NUMBER: _ClassVar[int]
    collection: str
    target_vector: _containers.RepeatedScalarFieldContainer[float]
    learning_rate: float
    def __init__(self, collection: _Optional[str] = ..., target_vector: _Optional[_Iterable[float]] = ..., learning_rate: _Optional[float] = ...) -> None: ...

class InsertRequest(_message.Message):
    __slots__ = ("collection", "vector", "id", "metadata", "origin_node_id", "logical_clock", "durability", "typed_metadata")
    class MetadataEntry(_message.Message):
        __slots__ = ("key", "value")
        KEY_FIELD_NUMBER: _ClassVar[int]
        VALUE_FIELD_NUMBER: _ClassVar[int]
        key: str
        value: str
        def __init__(self, key: _Optional[str] = ..., value: _Optional[str] = ...) -> None: ...
    class TypedMetadataEntry(_message.Message):
        __slots__ = ("key", "value")
        KEY_FIELD_NUMBER: _ClassVar[int]
        VALUE_FIELD_NUMBER: _ClassVar[int]
        key: str
        value: MetadataValue
        def __init__(self, key: _Optional[str] = ..., value: _Optional[_Union[MetadataValue, _Mapping]] = ...) -> None: ...
    COLLECTION_FIELD_NUMBER: _ClassVar[int]
    VECTOR_FIELD_NUMBER: _ClassVar[int]
    ID_FIELD_NUMBER: _ClassVar[int]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    ORIGIN_NODE_ID_FIELD_NUMBER: _ClassVar[int]
    LOGICAL_CLOCK_FIELD_NUMBER: _ClassVar[int]
    DURABILITY_FIELD_NUMBER: _ClassVar[int]
    TYPED_METADATA_FIELD_NUMBER: _ClassVar[int]
    collection: str
    vector: _containers.RepeatedScalarFieldContainer[float]
    id: int
    metadata: _containers.ScalarMap[str, str]
    origin_node_id: str
    logical_clock: int
    durability: DurabilityLevel
    typed_metadata: _containers.MessageMap[str, MetadataValue]
    def __init__(self, collection: _Optional[str] = ..., vector: _Optional[_Iterable[float]] = ..., id: _Optional[int] = ..., metadata: _Optional[_Mapping[str, str]] = ..., origin_node_id: _Optional[str] = ..., logical_clock: _Optional[int] = ..., durability: _Optional[_Union[DurabilityLevel, str]] = ..., typed_metadata: _Optional[_Mapping[str, MetadataValue]] = ...) -> None: ...

class VectorData(_message.Message):
    __slots__ = ("vector", "id", "metadata", "typed_metadata")
    class MetadataEntry(_message.Message):
        __slots__ = ("key", "value")
        KEY_FIELD_NUMBER: _ClassVar[int]
        VALUE_FIELD_NUMBER: _ClassVar[int]
        key: str
        value: str
        def __init__(self, key: _Optional[str] = ..., value: _Optional[str] = ...) -> None: ...
    class TypedMetadataEntry(_message.Message):
        __slots__ = ("key", "value")
        KEY_FIELD_NUMBER: _ClassVar[int]
        VALUE_FIELD_NUMBER: _ClassVar[int]
        key: str
        value: MetadataValue
        def __init__(self, key: _Optional[str] = ..., value: _Optional[_Union[MetadataValue, _Mapping]] = ...) -> None: ...
    VECTOR_FIELD_NUMBER: _ClassVar[int]
    ID_FIELD_NUMBER: _ClassVar[int]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    TYPED_METADATA_FIELD_NUMBER: _ClassVar[int]
    vector: _containers.RepeatedScalarFieldContainer[float]
    id: int
    metadata: _containers.ScalarMap[str, str]
    typed_metadata: _containers.MessageMap[str, MetadataValue]
    def __init__(self, vector: _Optional[_Iterable[float]] = ..., id: _Optional[int] = ..., metadata: _Optional[_Mapping[str, str]] = ..., typed_metadata: _Optional[_Mapping[str, MetadataValue]] = ...) -> None: ...

class BatchInsertRequest(_message.Message):
    __slots__ = ("collection", "vectors", "origin_node_id", "logical_clock", "durability")
    COLLECTION_FIELD_NUMBER: _ClassVar[int]
    VECTORS_FIELD_NUMBER: _ClassVar[int]
    ORIGIN_NODE_ID_FIELD_NUMBER: _ClassVar[int]
    LOGICAL_CLOCK_FIELD_NUMBER: _ClassVar[int]
    DURABILITY_FIELD_NUMBER: _ClassVar[int]
    collection: str
    vectors: _containers.RepeatedCompositeFieldContainer[VectorData]
    origin_node_id: str
    logical_clock: int
    durability: DurabilityLevel
    def __init__(self, collection: _Optional[str] = ..., vectors: _Optional[_Iterable[_Union[VectorData, _Mapping]]] = ..., origin_node_id: _Optional[str] = ..., logical_clock: _Optional[int] = ..., durability: _Optional[_Union[DurabilityLevel, str]] = ...) -> None: ...

class InsertTextRequest(_message.Message):
    __slots__ = ("collection", "id", "text", "metadata", "durability")
    class MetadataEntry(_message.Message):
        __slots__ = ("key", "value")
        KEY_FIELD_NUMBER: _ClassVar[int]
        VALUE_FIELD_NUMBER: _ClassVar[int]
        key: str
        value: str
        def __init__(self, key: _Optional[str] = ..., value: _Optional[str] = ...) -> None: ...
    COLLECTION_FIELD_NUMBER: _ClassVar[int]
    ID_FIELD_NUMBER: _ClassVar[int]
    TEXT_FIELD_NUMBER: _ClassVar[int]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    DURABILITY_FIELD_NUMBER: _ClassVar[int]
    collection: str
    id: int
    text: str
    metadata: _containers.ScalarMap[str, str]
    durability: DurabilityLevel
    def __init__(self, collection: _Optional[str] = ..., id: _Optional[int] = ..., text: _Optional[str] = ..., metadata: _Optional[_Mapping[str, str]] = ..., durability: _Optional[_Union[DurabilityLevel, str]] = ...) -> None: ...

class VectorizeRequest(_message.Message):
    __slots__ = ("text", "metric")
    TEXT_FIELD_NUMBER: _ClassVar[int]
    METRIC_FIELD_NUMBER: _ClassVar[int]
    text: str
    metric: str
    def __init__(self, text: _Optional[str] = ..., metric: _Optional[str] = ...) -> None: ...

class VectorizeResponse(_message.Message):
    __slots__ = ("vector",)
    VECTOR_FIELD_NUMBER: _ClassVar[int]
    vector: _containers.RepeatedScalarFieldContainer[float]
    def __init__(self, vector: _Optional[_Iterable[float]] = ...) -> None: ...

class SearchTextRequest(_message.Message):
    __slots__ = ("collection", "text", "top_k", "filter", "filters")
    class FilterEntry(_message.Message):
        __slots__ = ("key", "value")
        KEY_FIELD_NUMBER: _ClassVar[int]
        VALUE_FIELD_NUMBER: _ClassVar[int]
        key: str
        value: str
        def __init__(self, key: _Optional[str] = ..., value: _Optional[str] = ...) -> None: ...
    COLLECTION_FIELD_NUMBER: _ClassVar[int]
    TEXT_FIELD_NUMBER: _ClassVar[int]
    TOP_K_FIELD_NUMBER: _ClassVar[int]
    FILTER_FIELD_NUMBER: _ClassVar[int]
    FILTERS_FIELD_NUMBER: _ClassVar[int]
    collection: str
    text: str
    top_k: int
    filter: _containers.ScalarMap[str, str]
    filters: _containers.RepeatedCompositeFieldContainer[Filter]
    def __init__(self, collection: _Optional[str] = ..., text: _Optional[str] = ..., top_k: _Optional[int] = ..., filter: _Optional[_Mapping[str, str]] = ..., filters: _Optional[_Iterable[_Union[Filter, _Mapping]]] = ...) -> None: ...

class InsertResponse(_message.Message):
    __slots__ = ("success",)
    SUCCESS_FIELD_NUMBER: _ClassVar[int]
    success: bool
    def __init__(self, success: bool = ...) -> None: ...

class DeleteRequest(_message.Message):
    __slots__ = ("collection", "id")
    COLLECTION_FIELD_NUMBER: _ClassVar[int]
    ID_FIELD_NUMBER: _ClassVar[int]
    collection: str
    id: int
    def __init__(self, collection: _Optional[str] = ..., id: _Optional[int] = ...) -> None: ...

class DeleteResponse(_message.Message):
    __slots__ = ("success",)
    SUCCESS_FIELD_NUMBER: _ClassVar[int]
    success: bool
    def __init__(self, success: bool = ...) -> None: ...

class SearchRequest(_message.Message):
    __slots__ = ("collection", "vector", "top_k", "filter", "filters", "hybrid_query", "hybrid_alpha", "use_wasserstein")
    class FilterEntry(_message.Message):
        __slots__ = ("key", "value")
        KEY_FIELD_NUMBER: _ClassVar[int]
        VALUE_FIELD_NUMBER: _ClassVar[int]
        key: str
        value: str
        def __init__(self, key: _Optional[str] = ..., value: _Optional[str] = ...) -> None: ...
    COLLECTION_FIELD_NUMBER: _ClassVar[int]
    VECTOR_FIELD_NUMBER: _ClassVar[int]
    TOP_K_FIELD_NUMBER: _ClassVar[int]
    FILTER_FIELD_NUMBER: _ClassVar[int]
    FILTERS_FIELD_NUMBER: _ClassVar[int]
    HYBRID_QUERY_FIELD_NUMBER: _ClassVar[int]
    HYBRID_ALPHA_FIELD_NUMBER: _ClassVar[int]
    USE_WASSERSTEIN_FIELD_NUMBER: _ClassVar[int]
    collection: str
    vector: _containers.RepeatedScalarFieldContainer[float]
    top_k: int
    filter: _containers.ScalarMap[str, str]
    filters: _containers.RepeatedCompositeFieldContainer[Filter]
    hybrid_query: str
    hybrid_alpha: float
    use_wasserstein: bool
    def __init__(self, collection: _Optional[str] = ..., vector: _Optional[_Iterable[float]] = ..., top_k: _Optional[int] = ..., filter: _Optional[_Mapping[str, str]] = ..., filters: _Optional[_Iterable[_Union[Filter, _Mapping]]] = ..., hybrid_query: _Optional[str] = ..., hybrid_alpha: _Optional[float] = ..., use_wasserstein: bool = ...) -> None: ...

class Filter(_message.Message):
    __slots__ = ("match", "range")
    MATCH_FIELD_NUMBER: _ClassVar[int]
    RANGE_FIELD_NUMBER: _ClassVar[int]
    match: Match
    range: Range
    def __init__(self, match: _Optional[_Union[Match, _Mapping]] = ..., range: _Optional[_Union[Range, _Mapping]] = ...) -> None: ...

class Match(_message.Message):
    __slots__ = ("key", "value")
    KEY_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    key: str
    value: str
    def __init__(self, key: _Optional[str] = ..., value: _Optional[str] = ...) -> None: ...

class Range(_message.Message):
    __slots__ = ("key", "gte", "lte", "gte_f64", "lte_f64")
    KEY_FIELD_NUMBER: _ClassVar[int]
    GTE_FIELD_NUMBER: _ClassVar[int]
    LTE_FIELD_NUMBER: _ClassVar[int]
    GTE_F64_FIELD_NUMBER: _ClassVar[int]
    LTE_F64_FIELD_NUMBER: _ClassVar[int]
    key: str
    gte: int
    lte: int
    gte_f64: float
    lte_f64: float
    def __init__(self, key: _Optional[str] = ..., gte: _Optional[int] = ..., lte: _Optional[int] = ..., gte_f64: _Optional[float] = ..., lte_f64: _Optional[float] = ...) -> None: ...

class SearchResponse(_message.Message):
    __slots__ = ("results",)
    RESULTS_FIELD_NUMBER: _ClassVar[int]
    results: _containers.RepeatedCompositeFieldContainer[SearchResult]
    def __init__(self, results: _Optional[_Iterable[_Union[SearchResult, _Mapping]]] = ...) -> None: ...

class BatchSearchRequest(_message.Message):
    __slots__ = ("searches",)
    SEARCHES_FIELD_NUMBER: _ClassVar[int]
    searches: _containers.RepeatedCompositeFieldContainer[SearchRequest]
    def __init__(self, searches: _Optional[_Iterable[_Union[SearchRequest, _Mapping]]] = ...) -> None: ...

class BatchSearchResponse(_message.Message):
    __slots__ = ("responses",)
    RESPONSES_FIELD_NUMBER: _ClassVar[int]
    responses: _containers.RepeatedCompositeFieldContainer[SearchResponse]
    def __init__(self, responses: _Optional[_Iterable[_Union[SearchResponse, _Mapping]]] = ...) -> None: ...

class SearchMultiCollectionRequest(_message.Message):
    __slots__ = ("collections", "vector", "top_k")
    COLLECTIONS_FIELD_NUMBER: _ClassVar[int]
    VECTOR_FIELD_NUMBER: _ClassVar[int]
    TOP_K_FIELD_NUMBER: _ClassVar[int]
    collections: _containers.RepeatedScalarFieldContainer[str]
    vector: _containers.RepeatedScalarFieldContainer[float]
    top_k: int
    def __init__(self, collections: _Optional[_Iterable[str]] = ..., vector: _Optional[_Iterable[float]] = ..., top_k: _Optional[int] = ...) -> None: ...

class SearchMultiCollectionResponse(_message.Message):
    __slots__ = ("responses",)
    class ResponsesEntry(_message.Message):
        __slots__ = ("key", "value")
        KEY_FIELD_NUMBER: _ClassVar[int]
        VALUE_FIELD_NUMBER: _ClassVar[int]
        key: str
        value: SearchResponse
        def __init__(self, key: _Optional[str] = ..., value: _Optional[_Union[SearchResponse, _Mapping]] = ...) -> None: ...
    RESPONSES_FIELD_NUMBER: _ClassVar[int]
    responses: _containers.MessageMap[str, SearchResponse]
    def __init__(self, responses: _Optional[_Mapping[str, SearchResponse]] = ...) -> None: ...

class SearchResult(_message.Message):
    __slots__ = ("id", "distance", "metadata", "typed_metadata")
    class MetadataEntry(_message.Message):
        __slots__ = ("key", "value")
        KEY_FIELD_NUMBER: _ClassVar[int]
        VALUE_FIELD_NUMBER: _ClassVar[int]
        key: str
        value: str
        def __init__(self, key: _Optional[str] = ..., value: _Optional[str] = ...) -> None: ...
    class TypedMetadataEntry(_message.Message):
        __slots__ = ("key", "value")
        KEY_FIELD_NUMBER: _ClassVar[int]
        VALUE_FIELD_NUMBER: _ClassVar[int]
        key: str
        value: MetadataValue
        def __init__(self, key: _Optional[str] = ..., value: _Optional[_Union[MetadataValue, _Mapping]] = ...) -> None: ...
    ID_FIELD_NUMBER: _ClassVar[int]
    DISTANCE_FIELD_NUMBER: _ClassVar[int]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    TYPED_METADATA_FIELD_NUMBER: _ClassVar[int]
    id: int
    distance: float
    metadata: _containers.ScalarMap[str, str]
    typed_metadata: _containers.MessageMap[str, MetadataValue]
    def __init__(self, id: _Optional[int] = ..., distance: _Optional[float] = ..., metadata: _Optional[_Mapping[str, str]] = ..., typed_metadata: _Optional[_Mapping[str, MetadataValue]] = ...) -> None: ...

class GetNodeRequest(_message.Message):
    __slots__ = ("collection", "id", "layer")
    COLLECTION_FIELD_NUMBER: _ClassVar[int]
    ID_FIELD_NUMBER: _ClassVar[int]
    LAYER_FIELD_NUMBER: _ClassVar[int]
    collection: str
    id: int
    layer: int
    def __init__(self, collection: _Optional[str] = ..., id: _Optional[int] = ..., layer: _Optional[int] = ...) -> None: ...

class GraphNode(_message.Message):
    __slots__ = ("id", "layer", "neighbors", "metadata", "typed_metadata")
    class MetadataEntry(_message.Message):
        __slots__ = ("key", "value")
        KEY_FIELD_NUMBER: _ClassVar[int]
        VALUE_FIELD_NUMBER: _ClassVar[int]
        key: str
        value: str
        def __init__(self, key: _Optional[str] = ..., value: _Optional[str] = ...) -> None: ...
    class TypedMetadataEntry(_message.Message):
        __slots__ = ("key", "value")
        KEY_FIELD_NUMBER: _ClassVar[int]
        VALUE_FIELD_NUMBER: _ClassVar[int]
        key: str
        value: MetadataValue
        def __init__(self, key: _Optional[str] = ..., value: _Optional[_Union[MetadataValue, _Mapping]] = ...) -> None: ...
    ID_FIELD_NUMBER: _ClassVar[int]
    LAYER_FIELD_NUMBER: _ClassVar[int]
    NEIGHBORS_FIELD_NUMBER: _ClassVar[int]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    TYPED_METADATA_FIELD_NUMBER: _ClassVar[int]
    id: int
    layer: int
    neighbors: _containers.RepeatedScalarFieldContainer[int]
    metadata: _containers.ScalarMap[str, str]
    typed_metadata: _containers.MessageMap[str, MetadataValue]
    def __init__(self, id: _Optional[int] = ..., layer: _Optional[int] = ..., neighbors: _Optional[_Iterable[int]] = ..., metadata: _Optional[_Mapping[str, str]] = ..., typed_metadata: _Optional[_Mapping[str, MetadataValue]] = ...) -> None: ...

class GetNeighborsRequest(_message.Message):
    __slots__ = ("collection", "id", "layer", "limit", "offset")
    COLLECTION_FIELD_NUMBER: _ClassVar[int]
    ID_FIELD_NUMBER: _ClassVar[int]
    LAYER_FIELD_NUMBER: _ClassVar[int]
    LIMIT_FIELD_NUMBER: _ClassVar[int]
    OFFSET_FIELD_NUMBER: _ClassVar[int]
    collection: str
    id: int
    layer: int
    limit: int
    offset: int
    def __init__(self, collection: _Optional[str] = ..., id: _Optional[int] = ..., layer: _Optional[int] = ..., limit: _Optional[int] = ..., offset: _Optional[int] = ...) -> None: ...

class GetNeighborsResponse(_message.Message):
    __slots__ = ("neighbors", "edge_weights")
    NEIGHBORS_FIELD_NUMBER: _ClassVar[int]
    EDGE_WEIGHTS_FIELD_NUMBER: _ClassVar[int]
    neighbors: _containers.RepeatedCompositeFieldContainer[GraphNode]
    edge_weights: _containers.RepeatedScalarFieldContainer[float]
    def __init__(self, neighbors: _Optional[_Iterable[_Union[GraphNode, _Mapping]]] = ..., edge_weights: _Optional[_Iterable[float]] = ...) -> None: ...

class TraverseRequest(_message.Message):
    __slots__ = ("collection", "start_id", "max_depth", "max_nodes", "layer", "filter", "filters")
    class FilterEntry(_message.Message):
        __slots__ = ("key", "value")
        KEY_FIELD_NUMBER: _ClassVar[int]
        VALUE_FIELD_NUMBER: _ClassVar[int]
        key: str
        value: str
        def __init__(self, key: _Optional[str] = ..., value: _Optional[str] = ...) -> None: ...
    COLLECTION_FIELD_NUMBER: _ClassVar[int]
    START_ID_FIELD_NUMBER: _ClassVar[int]
    MAX_DEPTH_FIELD_NUMBER: _ClassVar[int]
    MAX_NODES_FIELD_NUMBER: _ClassVar[int]
    LAYER_FIELD_NUMBER: _ClassVar[int]
    FILTER_FIELD_NUMBER: _ClassVar[int]
    FILTERS_FIELD_NUMBER: _ClassVar[int]
    collection: str
    start_id: int
    max_depth: int
    max_nodes: int
    layer: int
    filter: _containers.ScalarMap[str, str]
    filters: _containers.RepeatedCompositeFieldContainer[Filter]
    def __init__(self, collection: _Optional[str] = ..., start_id: _Optional[int] = ..., max_depth: _Optional[int] = ..., max_nodes: _Optional[int] = ..., layer: _Optional[int] = ..., filter: _Optional[_Mapping[str, str]] = ..., filters: _Optional[_Iterable[_Union[Filter, _Mapping]]] = ...) -> None: ...

class TraverseResponse(_message.Message):
    __slots__ = ("nodes",)
    NODES_FIELD_NUMBER: _ClassVar[int]
    nodes: _containers.RepeatedCompositeFieldContainer[GraphNode]
    def __init__(self, nodes: _Optional[_Iterable[_Union[GraphNode, _Mapping]]] = ...) -> None: ...

class FindSemanticClustersRequest(_message.Message):
    __slots__ = ("collection", "layer", "min_cluster_size", "max_clusters", "max_nodes")
    COLLECTION_FIELD_NUMBER: _ClassVar[int]
    LAYER_FIELD_NUMBER: _ClassVar[int]
    MIN_CLUSTER_SIZE_FIELD_NUMBER: _ClassVar[int]
    MAX_CLUSTERS_FIELD_NUMBER: _ClassVar[int]
    MAX_NODES_FIELD_NUMBER: _ClassVar[int]
    collection: str
    layer: int
    min_cluster_size: int
    max_clusters: int
    max_nodes: int
    def __init__(self, collection: _Optional[str] = ..., layer: _Optional[int] = ..., min_cluster_size: _Optional[int] = ..., max_clusters: _Optional[int] = ..., max_nodes: _Optional[int] = ...) -> None: ...

class GetConceptParentsRequest(_message.Message):
    __slots__ = ("collection", "id", "layer", "limit")
    COLLECTION_FIELD_NUMBER: _ClassVar[int]
    ID_FIELD_NUMBER: _ClassVar[int]
    LAYER_FIELD_NUMBER: _ClassVar[int]
    LIMIT_FIELD_NUMBER: _ClassVar[int]
    collection: str
    id: int
    layer: int
    limit: int
    def __init__(self, collection: _Optional[str] = ..., id: _Optional[int] = ..., layer: _Optional[int] = ..., limit: _Optional[int] = ...) -> None: ...

class GetConceptParentsResponse(_message.Message):
    __slots__ = ("parents",)
    PARENTS_FIELD_NUMBER: _ClassVar[int]
    parents: _containers.RepeatedCompositeFieldContainer[GraphNode]
    def __init__(self, parents: _Optional[_Iterable[_Union[GraphNode, _Mapping]]] = ...) -> None: ...

class GraphCluster(_message.Message):
    __slots__ = ("node_ids",)
    NODE_IDS_FIELD_NUMBER: _ClassVar[int]
    node_ids: _containers.RepeatedScalarFieldContainer[int]
    def __init__(self, node_ids: _Optional[_Iterable[int]] = ...) -> None: ...

class FindSemanticClustersResponse(_message.Message):
    __slots__ = ("clusters",)
    CLUSTERS_FIELD_NUMBER: _ClassVar[int]
    clusters: _containers.RepeatedCompositeFieldContainer[GraphCluster]
    def __init__(self, clusters: _Optional[_Iterable[_Union[GraphCluster, _Mapping]]] = ...) -> None: ...

class MetadataValue(_message.Message):
    __slots__ = ("string_value", "int_value", "double_value", "bool_value")
    STRING_VALUE_FIELD_NUMBER: _ClassVar[int]
    INT_VALUE_FIELD_NUMBER: _ClassVar[int]
    DOUBLE_VALUE_FIELD_NUMBER: _ClassVar[int]
    BOOL_VALUE_FIELD_NUMBER: _ClassVar[int]
    string_value: str
    int_value: int
    double_value: float
    bool_value: bool
    def __init__(self, string_value: _Optional[str] = ..., int_value: _Optional[int] = ..., double_value: _Optional[float] = ..., bool_value: bool = ...) -> None: ...

class EventSubscriptionRequest(_message.Message):
    __slots__ = ("types", "collection")
    TYPES_FIELD_NUMBER: _ClassVar[int]
    COLLECTION_FIELD_NUMBER: _ClassVar[int]
    types: _containers.RepeatedScalarFieldContainer[EventType]
    collection: str
    def __init__(self, types: _Optional[_Iterable[_Union[EventType, str]]] = ..., collection: _Optional[str] = ...) -> None: ...

class VectorInsertedEvent(_message.Message):
    __slots__ = ("id", "collection", "logical_clock", "origin_node_id", "metadata", "typed_metadata")
    class MetadataEntry(_message.Message):
        __slots__ = ("key", "value")
        KEY_FIELD_NUMBER: _ClassVar[int]
        VALUE_FIELD_NUMBER: _ClassVar[int]
        key: str
        value: str
        def __init__(self, key: _Optional[str] = ..., value: _Optional[str] = ...) -> None: ...
    class TypedMetadataEntry(_message.Message):
        __slots__ = ("key", "value")
        KEY_FIELD_NUMBER: _ClassVar[int]
        VALUE_FIELD_NUMBER: _ClassVar[int]
        key: str
        value: MetadataValue
        def __init__(self, key: _Optional[str] = ..., value: _Optional[_Union[MetadataValue, _Mapping]] = ...) -> None: ...
    ID_FIELD_NUMBER: _ClassVar[int]
    COLLECTION_FIELD_NUMBER: _ClassVar[int]
    LOGICAL_CLOCK_FIELD_NUMBER: _ClassVar[int]
    ORIGIN_NODE_ID_FIELD_NUMBER: _ClassVar[int]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    TYPED_METADATA_FIELD_NUMBER: _ClassVar[int]
    id: int
    collection: str
    logical_clock: int
    origin_node_id: str
    metadata: _containers.ScalarMap[str, str]
    typed_metadata: _containers.MessageMap[str, MetadataValue]
    def __init__(self, id: _Optional[int] = ..., collection: _Optional[str] = ..., logical_clock: _Optional[int] = ..., origin_node_id: _Optional[str] = ..., metadata: _Optional[_Mapping[str, str]] = ..., typed_metadata: _Optional[_Mapping[str, MetadataValue]] = ...) -> None: ...

class VectorDeletedEvent(_message.Message):
    __slots__ = ("id", "collection", "logical_clock", "origin_node_id")
    ID_FIELD_NUMBER: _ClassVar[int]
    COLLECTION_FIELD_NUMBER: _ClassVar[int]
    LOGICAL_CLOCK_FIELD_NUMBER: _ClassVar[int]
    ORIGIN_NODE_ID_FIELD_NUMBER: _ClassVar[int]
    id: int
    collection: str
    logical_clock: int
    origin_node_id: str
    def __init__(self, id: _Optional[int] = ..., collection: _Optional[str] = ..., logical_clock: _Optional[int] = ..., origin_node_id: _Optional[str] = ...) -> None: ...

class EventMessage(_message.Message):
    __slots__ = ("type", "vector_inserted", "vector_deleted")
    TYPE_FIELD_NUMBER: _ClassVar[int]
    VECTOR_INSERTED_FIELD_NUMBER: _ClassVar[int]
    VECTOR_DELETED_FIELD_NUMBER: _ClassVar[int]
    type: EventType
    vector_inserted: VectorInsertedEvent
    vector_deleted: VectorDeletedEvent
    def __init__(self, type: _Optional[_Union[EventType, str]] = ..., vector_inserted: _Optional[_Union[VectorInsertedEvent, _Mapping]] = ..., vector_deleted: _Optional[_Union[VectorDeletedEvent, _Mapping]] = ...) -> None: ...

class Empty(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class StatusResponse(_message.Message):
    __slots__ = ("status",)
    STATUS_FIELD_NUMBER: _ClassVar[int]
    status: str
    def __init__(self, status: _Optional[str] = ...) -> None: ...

class MonitorRequest(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class SystemStats(_message.Message):
    __slots__ = ("total_collections", "total_vectors", "total_memory_mb", "qps")
    TOTAL_COLLECTIONS_FIELD_NUMBER: _ClassVar[int]
    TOTAL_VECTORS_FIELD_NUMBER: _ClassVar[int]
    TOTAL_MEMORY_MB_FIELD_NUMBER: _ClassVar[int]
    QPS_FIELD_NUMBER: _ClassVar[int]
    total_collections: int
    total_vectors: int
    total_memory_mb: float
    qps: float
    def __init__(self, total_collections: _Optional[int] = ..., total_vectors: _Optional[int] = ..., total_memory_mb: _Optional[float] = ..., qps: _Optional[float] = ...) -> None: ...

class DigestRequest(_message.Message):
    __slots__ = ("collection",)
    COLLECTION_FIELD_NUMBER: _ClassVar[int]
    collection: str
    def __init__(self, collection: _Optional[str] = ...) -> None: ...

class DigestResponse(_message.Message):
    __slots__ = ("logical_clock", "state_hash", "buckets", "count")
    LOGICAL_CLOCK_FIELD_NUMBER: _ClassVar[int]
    STATE_HASH_FIELD_NUMBER: _ClassVar[int]
    BUCKETS_FIELD_NUMBER: _ClassVar[int]
    COUNT_FIELD_NUMBER: _ClassVar[int]
    logical_clock: int
    state_hash: int
    buckets: _containers.RepeatedScalarFieldContainer[int]
    count: int
    def __init__(self, logical_clock: _Optional[int] = ..., state_hash: _Optional[int] = ..., buckets: _Optional[_Iterable[int]] = ..., count: _Optional[int] = ...) -> None: ...

class SyncHandshakeRequest(_message.Message):
    __slots__ = ("collection", "client_buckets", "client_logical_clock", "client_count")
    COLLECTION_FIELD_NUMBER: _ClassVar[int]
    CLIENT_BUCKETS_FIELD_NUMBER: _ClassVar[int]
    CLIENT_LOGICAL_CLOCK_FIELD_NUMBER: _ClassVar[int]
    CLIENT_COUNT_FIELD_NUMBER: _ClassVar[int]
    collection: str
    client_buckets: _containers.RepeatedScalarFieldContainer[int]
    client_logical_clock: int
    client_count: int
    def __init__(self, collection: _Optional[str] = ..., client_buckets: _Optional[_Iterable[int]] = ..., client_logical_clock: _Optional[int] = ..., client_count: _Optional[int] = ...) -> None: ...

class DiffBucket(_message.Message):
    __slots__ = ("bucket_index", "server_hash", "client_hash")
    BUCKET_INDEX_FIELD_NUMBER: _ClassVar[int]
    SERVER_HASH_FIELD_NUMBER: _ClassVar[int]
    CLIENT_HASH_FIELD_NUMBER: _ClassVar[int]
    bucket_index: int
    server_hash: int
    client_hash: int
    def __init__(self, bucket_index: _Optional[int] = ..., server_hash: _Optional[int] = ..., client_hash: _Optional[int] = ...) -> None: ...

class SyncHandshakeResponse(_message.Message):
    __slots__ = ("diff_buckets", "server_logical_clock", "server_count", "in_sync")
    DIFF_BUCKETS_FIELD_NUMBER: _ClassVar[int]
    SERVER_LOGICAL_CLOCK_FIELD_NUMBER: _ClassVar[int]
    SERVER_COUNT_FIELD_NUMBER: _ClassVar[int]
    IN_SYNC_FIELD_NUMBER: _ClassVar[int]
    diff_buckets: _containers.RepeatedCompositeFieldContainer[DiffBucket]
    server_logical_clock: int
    server_count: int
    in_sync: bool
    def __init__(self, diff_buckets: _Optional[_Iterable[_Union[DiffBucket, _Mapping]]] = ..., server_logical_clock: _Optional[int] = ..., server_count: _Optional[int] = ..., in_sync: bool = ...) -> None: ...

class SyncPullRequest(_message.Message):
    __slots__ = ("collection", "bucket_indices")
    COLLECTION_FIELD_NUMBER: _ClassVar[int]
    BUCKET_INDICES_FIELD_NUMBER: _ClassVar[int]
    collection: str
    bucket_indices: _containers.RepeatedScalarFieldContainer[int]
    def __init__(self, collection: _Optional[str] = ..., bucket_indices: _Optional[_Iterable[int]] = ...) -> None: ...

class SyncVectorData(_message.Message):
    __slots__ = ("collection", "id", "vector", "metadata", "bucket_index")
    class MetadataEntry(_message.Message):
        __slots__ = ("key", "value")
        KEY_FIELD_NUMBER: _ClassVar[int]
        VALUE_FIELD_NUMBER: _ClassVar[int]
        key: str
        value: str
        def __init__(self, key: _Optional[str] = ..., value: _Optional[str] = ...) -> None: ...
    COLLECTION_FIELD_NUMBER: _ClassVar[int]
    ID_FIELD_NUMBER: _ClassVar[int]
    VECTOR_FIELD_NUMBER: _ClassVar[int]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    BUCKET_INDEX_FIELD_NUMBER: _ClassVar[int]
    collection: str
    id: int
    vector: _containers.RepeatedScalarFieldContainer[float]
    metadata: _containers.ScalarMap[str, str]
    bucket_index: int
    def __init__(self, collection: _Optional[str] = ..., id: _Optional[int] = ..., vector: _Optional[_Iterable[float]] = ..., metadata: _Optional[_Mapping[str, str]] = ..., bucket_index: _Optional[int] = ...) -> None: ...

class SyncPushResponse(_message.Message):
    __slots__ = ("accepted", "rejected", "duplicates")
    ACCEPTED_FIELD_NUMBER: _ClassVar[int]
    REJECTED_FIELD_NUMBER: _ClassVar[int]
    DUPLICATES_FIELD_NUMBER: _ClassVar[int]
    accepted: int
    rejected: int
    duplicates: int
    def __init__(self, accepted: _Optional[int] = ..., rejected: _Optional[int] = ..., duplicates: _Optional[int] = ...) -> None: ...
