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

class EdgeType(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    UNKNOWN: _ClassVar[EdgeType]
    SIMILARITY: _ClassVar[EdgeType]
    HIERARCHY: _ClassVar[EdgeType]

class TraversalMode(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    GREEDY: _ClassVar[TraversalMode]
    DIFFUSIVE: _ClassVar[TraversalMode]
    MOMENTUM: _ClassVar[TraversalMode]

class EventType(int, metaclass=_enum_type_wrapper.EnumTypeWrapper):
    __slots__ = ()
    EVENT_UNKNOWN: _ClassVar[EventType]
    VECTOR_INSERTED: _ClassVar[EventType]
    VECTOR_DELETED: _ClassVar[EventType]
    TRAJECTORY_STEP: _ClassVar[EventType]
NONE: QuantizationMode
SCALAR_I8: QuantizationMode
DEFAULT_LEVEL: DurabilityLevel
ASYNC: DurabilityLevel
BATCH: DurabilityLevel
STRICT: DurabilityLevel
UNKNOWN: EdgeType
SIMILARITY: EdgeType
HIERARCHY: EdgeType
GREEDY: TraversalMode
DIFFUSIVE: TraversalMode
MOMENTUM: TraversalMode
EVENT_UNKNOWN: EventType
VECTOR_INSERTED: EventType
VECTOR_DELETED: EventType
TRAJECTORY_STEP: EventType

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
    __slots__ = ("schema",)
    SCHEMA_FIELD_NUMBER: _ClassVar[int]
    schema: CollectionSchema
    def __init__(self, schema: _Optional[_Union[CollectionSchema, _Mapping]] = ...) -> None: ...

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
    __slots__ = ("name", "schema")
    NAME_FIELD_NUMBER: _ClassVar[int]
    SCHEMA_FIELD_NUMBER: _ClassVar[int]
    name: str
    schema: CollectionSchema
    def __init__(self, name: _Optional[str] = ..., schema: _Optional[_Union[CollectionSchema, _Mapping]] = ...) -> None: ...

class VectorComponent(_message.Message):
    __slots__ = ("name", "metric", "full_dimension", "weight")
    NAME_FIELD_NUMBER: _ClassVar[int]
    METRIC_FIELD_NUMBER: _ClassVar[int]
    FULL_DIMENSION_FIELD_NUMBER: _ClassVar[int]
    WEIGHT_FIELD_NUMBER: _ClassVar[int]
    name: str
    metric: str
    full_dimension: int
    weight: float
    def __init__(self, name: _Optional[str] = ..., metric: _Optional[str] = ..., full_dimension: _Optional[int] = ..., weight: _Optional[float] = ...) -> None: ...

class MrlLayer(_message.Message):
    __slots__ = ("component_name", "cutoff_dimension", "store_in_ram", "rerank_top_k")
    COMPONENT_NAME_FIELD_NUMBER: _ClassVar[int]
    CUTOFF_DIMENSION_FIELD_NUMBER: _ClassVar[int]
    STORE_IN_RAM_FIELD_NUMBER: _ClassVar[int]
    RERANK_TOP_K_FIELD_NUMBER: _ClassVar[int]
    component_name: str
    cutoff_dimension: int
    store_in_ram: bool
    rerank_top_k: int
    def __init__(self, component_name: _Optional[str] = ..., cutoff_dimension: _Optional[int] = ..., store_in_ram: bool = ..., rerank_top_k: _Optional[int] = ...) -> None: ...

class CollectionSchema(_message.Message):
    __slots__ = ("components", "cascade_pipeline")
    COMPONENTS_FIELD_NUMBER: _ClassVar[int]
    CASCADE_PIPELINE_FIELD_NUMBER: _ClassVar[int]
    components: _containers.RepeatedCompositeFieldContainer[VectorComponent]
    cascade_pipeline: _containers.RepeatedCompositeFieldContainer[MrlLayer]
    def __init__(self, components: _Optional[_Iterable[_Union[VectorComponent, _Mapping]]] = ..., cascade_pipeline: _Optional[_Iterable[_Union[MrlLayer, _Mapping]]] = ...) -> None: ...

class CollectionComponent(_message.Message):
    __slots__ = ("space", "dimension", "metric", "weight")
    SPACE_FIELD_NUMBER: _ClassVar[int]
    DIMENSION_FIELD_NUMBER: _ClassVar[int]
    METRIC_FIELD_NUMBER: _ClassVar[int]
    WEIGHT_FIELD_NUMBER: _ClassVar[int]
    space: str
    dimension: int
    metric: str
    weight: float
    def __init__(self, space: _Optional[str] = ..., dimension: _Optional[int] = ..., metric: _Optional[str] = ..., weight: _Optional[float] = ...) -> None: ...

class DeleteCollectionRequest(_message.Message):
    __slots__ = ("name",)
    NAME_FIELD_NUMBER: _ClassVar[int]
    name: str
    def __init__(self, name: _Optional[str] = ...) -> None: ...

class CollectionSummary(_message.Message):
    __slots__ = ("name", "count", "schema")
    NAME_FIELD_NUMBER: _ClassVar[int]
    COUNT_FIELD_NUMBER: _ClassVar[int]
    SCHEMA_FIELD_NUMBER: _ClassVar[int]
    name: str
    count: int
    schema: CollectionSchema
    def __init__(self, name: _Optional[str] = ..., count: _Optional[int] = ..., schema: _Optional[_Union[CollectionSchema, _Mapping]] = ...) -> None: ...

class ListCollectionsResponse(_message.Message):
    __slots__ = ("collections",)
    COLLECTIONS_FIELD_NUMBER: _ClassVar[int]
    collections: _containers.RepeatedCompositeFieldContainer[CollectionSummary]
    def __init__(self, collections: _Optional[_Iterable[_Union[CollectionSummary, _Mapping]]] = ...) -> None: ...

class CollectionStatsRequest(_message.Message):
    __slots__ = ("name",)
    NAME_FIELD_NUMBER: _ClassVar[int]
    name: str
    def __init__(self, name: _Optional[str] = ...) -> None: ...

class CollectionStatsResponse(_message.Message):
    __slots__ = ("count", "indexing_queue", "disk_usage_bytes", "ram_usage_bytes", "active_tasks", "schema")
    COUNT_FIELD_NUMBER: _ClassVar[int]
    INDEXING_QUEUE_FIELD_NUMBER: _ClassVar[int]
    DISK_USAGE_BYTES_FIELD_NUMBER: _ClassVar[int]
    RAM_USAGE_BYTES_FIELD_NUMBER: _ClassVar[int]
    ACTIVE_TASKS_FIELD_NUMBER: _ClassVar[int]
    SCHEMA_FIELD_NUMBER: _ClassVar[int]
    count: int
    indexing_queue: int
    disk_usage_bytes: int
    ram_usage_bytes: int
    active_tasks: int
    schema: CollectionSchema
    def __init__(self, count: _Optional[int] = ..., indexing_queue: _Optional[int] = ..., disk_usage_bytes: _Optional[int] = ..., ram_usage_bytes: _Optional[int] = ..., active_tasks: _Optional[int] = ..., schema: _Optional[_Union[CollectionSchema, _Mapping]] = ...) -> None: ...

class RebuildIndexRequest(_message.Message):
    __slots__ = ("name", "filter_query")
    NAME_FIELD_NUMBER: _ClassVar[int]
    FILTER_QUERY_FIELD_NUMBER: _ClassVar[int]
    name: str
    filter_query: VacuumFilterQuery
    def __init__(self, name: _Optional[str] = ..., filter_query: _Optional[_Union[VacuumFilterQuery, _Mapping]] = ...) -> None: ...

class ConfigUpdate(_message.Message):
    __slots__ = ("collection", "ef_search", "ef_construction", "m")
    COLLECTION_FIELD_NUMBER: _ClassVar[int]
    EF_SEARCH_FIELD_NUMBER: _ClassVar[int]
    EF_CONSTRUCTION_FIELD_NUMBER: _ClassVar[int]
    M_FIELD_NUMBER: _ClassVar[int]
    collection: str
    ef_search: int
    ef_construction: int
    m: int
    def __init__(self, collection: _Optional[str] = ..., ef_search: _Optional[int] = ..., ef_construction: _Optional[int] = ..., m: _Optional[int] = ...) -> None: ...

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
    __slots__ = ("collection", "vector", "id", "metadata", "origin_node_id", "logical_clock", "durability", "typed_metadata", "payload")
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
    PAYLOAD_FIELD_NUMBER: _ClassVar[int]
    collection: str
    vector: _containers.RepeatedScalarFieldContainer[float]
    id: int
    metadata: _containers.ScalarMap[str, str]
    origin_node_id: str
    logical_clock: int
    durability: DurabilityLevel
    typed_metadata: _containers.MessageMap[str, MetadataValue]
    payload: bytes
    def __init__(self, collection: _Optional[str] = ..., vector: _Optional[_Iterable[float]] = ..., id: _Optional[int] = ..., metadata: _Optional[_Mapping[str, str]] = ..., origin_node_id: _Optional[str] = ..., logical_clock: _Optional[int] = ..., durability: _Optional[_Union[DurabilityLevel, str]] = ..., typed_metadata: _Optional[_Mapping[str, MetadataValue]] = ..., payload: _Optional[bytes] = ...) -> None: ...

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
    __slots__ = ("collection", "text", "top_k", "filter", "filters", "bm25_options", "hybrid_alpha", "include_payload", "component_weights")
    class FilterEntry(_message.Message):
        __slots__ = ("key", "value")
        KEY_FIELD_NUMBER: _ClassVar[int]
        VALUE_FIELD_NUMBER: _ClassVar[int]
        key: str
        value: str
        def __init__(self, key: _Optional[str] = ..., value: _Optional[str] = ...) -> None: ...
    class ComponentWeightsEntry(_message.Message):
        __slots__ = ("key", "value")
        KEY_FIELD_NUMBER: _ClassVar[int]
        VALUE_FIELD_NUMBER: _ClassVar[int]
        key: str
        value: float
        def __init__(self, key: _Optional[str] = ..., value: _Optional[float] = ...) -> None: ...
    COLLECTION_FIELD_NUMBER: _ClassVar[int]
    TEXT_FIELD_NUMBER: _ClassVar[int]
    TOP_K_FIELD_NUMBER: _ClassVar[int]
    FILTER_FIELD_NUMBER: _ClassVar[int]
    FILTERS_FIELD_NUMBER: _ClassVar[int]
    BM25_OPTIONS_FIELD_NUMBER: _ClassVar[int]
    HYBRID_ALPHA_FIELD_NUMBER: _ClassVar[int]
    INCLUDE_PAYLOAD_FIELD_NUMBER: _ClassVar[int]
    COMPONENT_WEIGHTS_FIELD_NUMBER: _ClassVar[int]
    collection: str
    text: str
    top_k: int
    filter: _containers.ScalarMap[str, str]
    filters: _containers.RepeatedCompositeFieldContainer[Filter]
    bm25_options: Bm25Options
    hybrid_alpha: float
    include_payload: bool
    component_weights: _containers.ScalarMap[str, float]
    def __init__(self, collection: _Optional[str] = ..., text: _Optional[str] = ..., top_k: _Optional[int] = ..., filter: _Optional[_Mapping[str, str]] = ..., filters: _Optional[_Iterable[_Union[Filter, _Mapping]]] = ..., bm25_options: _Optional[_Union[Bm25Options, _Mapping]] = ..., hybrid_alpha: _Optional[float] = ..., include_payload: bool = ..., component_weights: _Optional[_Mapping[str, float]] = ...) -> None: ...

class Bm25Options(_message.Message):
    __slots__ = ("method", "k1", "b", "delta", "language", "ngrams", "fusion_method")
    METHOD_FIELD_NUMBER: _ClassVar[int]
    K1_FIELD_NUMBER: _ClassVar[int]
    B_FIELD_NUMBER: _ClassVar[int]
    DELTA_FIELD_NUMBER: _ClassVar[int]
    LANGUAGE_FIELD_NUMBER: _ClassVar[int]
    NGRAMS_FIELD_NUMBER: _ClassVar[int]
    FUSION_METHOD_FIELD_NUMBER: _ClassVar[int]
    method: str
    k1: float
    b: float
    delta: float
    language: str
    ngrams: int
    fusion_method: str
    def __init__(self, method: _Optional[str] = ..., k1: _Optional[float] = ..., b: _Optional[float] = ..., delta: _Optional[float] = ..., language: _Optional[str] = ..., ngrams: _Optional[int] = ..., fusion_method: _Optional[str] = ...) -> None: ...

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

class GetPointsRequest(_message.Message):
    __slots__ = ("collection", "ids")
    COLLECTION_FIELD_NUMBER: _ClassVar[int]
    IDS_FIELD_NUMBER: _ClassVar[int]
    collection: str
    ids: _containers.RepeatedScalarFieldContainer[int]
    def __init__(self, collection: _Optional[str] = ..., ids: _Optional[_Iterable[int]] = ...) -> None: ...

class GetPointsResponse(_message.Message):
    __slots__ = ("points",)
    POINTS_FIELD_NUMBER: _ClassVar[int]
    points: _containers.RepeatedCompositeFieldContainer[VectorData]
    def __init__(self, points: _Optional[_Iterable[_Union[VectorData, _Mapping]]] = ...) -> None: ...

class UpdatePayloadRequest(_message.Message):
    __slots__ = ("collection", "id", "metadata", "typed_metadata")
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
    ID_FIELD_NUMBER: _ClassVar[int]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    TYPED_METADATA_FIELD_NUMBER: _ClassVar[int]
    collection: str
    id: int
    metadata: _containers.ScalarMap[str, str]
    typed_metadata: _containers.MessageMap[str, MetadataValue]
    def __init__(self, collection: _Optional[str] = ..., id: _Optional[int] = ..., metadata: _Optional[_Mapping[str, str]] = ..., typed_metadata: _Optional[_Mapping[str, MetadataValue]] = ...) -> None: ...

class ScrollRequest(_message.Message):
    __slots__ = ("collection", "limit", "offset", "filters")
    COLLECTION_FIELD_NUMBER: _ClassVar[int]
    LIMIT_FIELD_NUMBER: _ClassVar[int]
    OFFSET_FIELD_NUMBER: _ClassVar[int]
    FILTERS_FIELD_NUMBER: _ClassVar[int]
    collection: str
    limit: int
    offset: int
    filters: _containers.RepeatedCompositeFieldContainer[Filter]
    def __init__(self, collection: _Optional[str] = ..., limit: _Optional[int] = ..., offset: _Optional[int] = ..., filters: _Optional[_Iterable[_Union[Filter, _Mapping]]] = ...) -> None: ...

class ScrollResponse(_message.Message):
    __slots__ = ("points",)
    POINTS_FIELD_NUMBER: _ClassVar[int]
    points: _containers.RepeatedCompositeFieldContainer[VectorData]
    def __init__(self, points: _Optional[_Iterable[_Union[VectorData, _Mapping]]] = ...) -> None: ...

class CountRequest(_message.Message):
    __slots__ = ("collection", "filters")
    COLLECTION_FIELD_NUMBER: _ClassVar[int]
    FILTERS_FIELD_NUMBER: _ClassVar[int]
    collection: str
    filters: _containers.RepeatedCompositeFieldContainer[Filter]
    def __init__(self, collection: _Optional[str] = ..., filters: _Optional[_Iterable[_Union[Filter, _Mapping]]] = ...) -> None: ...

class CountResponse(_message.Message):
    __slots__ = ("count",)
    COUNT_FIELD_NUMBER: _ClassVar[int]
    count: int
    def __init__(self, count: _Optional[int] = ...) -> None: ...

class HealthCheckResponse(_message.Message):
    __slots__ = ("status",)
    STATUS_FIELD_NUMBER: _ClassVar[int]
    status: str
    def __init__(self, status: _Optional[str] = ...) -> None: ...

class SearchRequest(_message.Message):
    __slots__ = ("collection", "vector", "top_k", "filter", "filters", "hybrid_query", "hybrid_alpha", "use_wasserstein", "bm25_options", "mrl_dimension", "include_payload", "component_weights", "use_wave")
    class FilterEntry(_message.Message):
        __slots__ = ("key", "value")
        KEY_FIELD_NUMBER: _ClassVar[int]
        VALUE_FIELD_NUMBER: _ClassVar[int]
        key: str
        value: str
        def __init__(self, key: _Optional[str] = ..., value: _Optional[str] = ...) -> None: ...
    class ComponentWeightsEntry(_message.Message):
        __slots__ = ("key", "value")
        KEY_FIELD_NUMBER: _ClassVar[int]
        VALUE_FIELD_NUMBER: _ClassVar[int]
        key: str
        value: float
        def __init__(self, key: _Optional[str] = ..., value: _Optional[float] = ...) -> None: ...
    COLLECTION_FIELD_NUMBER: _ClassVar[int]
    VECTOR_FIELD_NUMBER: _ClassVar[int]
    TOP_K_FIELD_NUMBER: _ClassVar[int]
    FILTER_FIELD_NUMBER: _ClassVar[int]
    FILTERS_FIELD_NUMBER: _ClassVar[int]
    HYBRID_QUERY_FIELD_NUMBER: _ClassVar[int]
    HYBRID_ALPHA_FIELD_NUMBER: _ClassVar[int]
    USE_WASSERSTEIN_FIELD_NUMBER: _ClassVar[int]
    BM25_OPTIONS_FIELD_NUMBER: _ClassVar[int]
    MRL_DIMENSION_FIELD_NUMBER: _ClassVar[int]
    INCLUDE_PAYLOAD_FIELD_NUMBER: _ClassVar[int]
    COMPONENT_WEIGHTS_FIELD_NUMBER: _ClassVar[int]
    USE_WAVE_FIELD_NUMBER: _ClassVar[int]
    collection: str
    vector: _containers.RepeatedScalarFieldContainer[float]
    top_k: int
    filter: _containers.ScalarMap[str, str]
    filters: _containers.RepeatedCompositeFieldContainer[Filter]
    hybrid_query: str
    hybrid_alpha: float
    use_wasserstein: bool
    bm25_options: Bm25Options
    mrl_dimension: int
    include_payload: bool
    component_weights: _containers.ScalarMap[str, float]
    use_wave: bool
    def __init__(self, collection: _Optional[str] = ..., vector: _Optional[_Iterable[float]] = ..., top_k: _Optional[int] = ..., filter: _Optional[_Mapping[str, str]] = ..., filters: _Optional[_Iterable[_Union[Filter, _Mapping]]] = ..., hybrid_query: _Optional[str] = ..., hybrid_alpha: _Optional[float] = ..., use_wasserstein: bool = ..., bm25_options: _Optional[_Union[Bm25Options, _Mapping]] = ..., mrl_dimension: _Optional[int] = ..., include_payload: bool = ..., component_weights: _Optional[_Mapping[str, float]] = ..., use_wave: bool = ...) -> None: ...

class Filter(_message.Message):
    __slots__ = ("match", "range", "in_cone", "in_box", "in_ball", "and_op", "or_op", "not_op", "prefix")
    MATCH_FIELD_NUMBER: _ClassVar[int]
    RANGE_FIELD_NUMBER: _ClassVar[int]
    IN_CONE_FIELD_NUMBER: _ClassVar[int]
    IN_BOX_FIELD_NUMBER: _ClassVar[int]
    IN_BALL_FIELD_NUMBER: _ClassVar[int]
    AND_OP_FIELD_NUMBER: _ClassVar[int]
    OR_OP_FIELD_NUMBER: _ClassVar[int]
    NOT_OP_FIELD_NUMBER: _ClassVar[int]
    PREFIX_FIELD_NUMBER: _ClassVar[int]
    match: Match
    range: Range
    in_cone: InCone
    in_box: InBox
    in_ball: InBall
    and_op: FilterAnd
    or_op: FilterOr
    not_op: FilterNot
    prefix: Prefix
    def __init__(self, match: _Optional[_Union[Match, _Mapping]] = ..., range: _Optional[_Union[Range, _Mapping]] = ..., in_cone: _Optional[_Union[InCone, _Mapping]] = ..., in_box: _Optional[_Union[InBox, _Mapping]] = ..., in_ball: _Optional[_Union[InBall, _Mapping]] = ..., and_op: _Optional[_Union[FilterAnd, _Mapping]] = ..., or_op: _Optional[_Union[FilterOr, _Mapping]] = ..., not_op: _Optional[_Union[FilterNot, _Mapping]] = ..., prefix: _Optional[_Union[Prefix, _Mapping]] = ...) -> None: ...

class FilterAnd(_message.Message):
    __slots__ = ("conditions",)
    CONDITIONS_FIELD_NUMBER: _ClassVar[int]
    conditions: _containers.RepeatedCompositeFieldContainer[Filter]
    def __init__(self, conditions: _Optional[_Iterable[_Union[Filter, _Mapping]]] = ...) -> None: ...

class FilterOr(_message.Message):
    __slots__ = ("conditions",)
    CONDITIONS_FIELD_NUMBER: _ClassVar[int]
    conditions: _containers.RepeatedCompositeFieldContainer[Filter]
    def __init__(self, conditions: _Optional[_Iterable[_Union[Filter, _Mapping]]] = ...) -> None: ...

class FilterNot(_message.Message):
    __slots__ = ("condition",)
    CONDITION_FIELD_NUMBER: _ClassVar[int]
    condition: Filter
    def __init__(self, condition: _Optional[_Union[Filter, _Mapping]] = ...) -> None: ...

class Match(_message.Message):
    __slots__ = ("key", "value")
    KEY_FIELD_NUMBER: _ClassVar[int]
    VALUE_FIELD_NUMBER: _ClassVar[int]
    key: str
    value: str
    def __init__(self, key: _Optional[str] = ..., value: _Optional[str] = ...) -> None: ...

class Prefix(_message.Message):
    __slots__ = ("key", "prefix")
    KEY_FIELD_NUMBER: _ClassVar[int]
    PREFIX_FIELD_NUMBER: _ClassVar[int]
    key: str
    prefix: str
    def __init__(self, key: _Optional[str] = ..., prefix: _Optional[str] = ...) -> None: ...

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

class InCone(_message.Message):
    __slots__ = ("axes", "apertures", "cen")
    AXES_FIELD_NUMBER: _ClassVar[int]
    APERTURES_FIELD_NUMBER: _ClassVar[int]
    CEN_FIELD_NUMBER: _ClassVar[int]
    axes: _containers.RepeatedScalarFieldContainer[float]
    apertures: _containers.RepeatedScalarFieldContainer[float]
    cen: float
    def __init__(self, axes: _Optional[_Iterable[float]] = ..., apertures: _Optional[_Iterable[float]] = ..., cen: _Optional[float] = ...) -> None: ...

class InBox(_message.Message):
    __slots__ = ("min_bounds", "max_bounds")
    MIN_BOUNDS_FIELD_NUMBER: _ClassVar[int]
    MAX_BOUNDS_FIELD_NUMBER: _ClassVar[int]
    min_bounds: _containers.RepeatedScalarFieldContainer[float]
    max_bounds: _containers.RepeatedScalarFieldContainer[float]
    def __init__(self, min_bounds: _Optional[_Iterable[float]] = ..., max_bounds: _Optional[_Iterable[float]] = ...) -> None: ...

class InBall(_message.Message):
    __slots__ = ("center", "radius")
    CENTER_FIELD_NUMBER: _ClassVar[int]
    RADIUS_FIELD_NUMBER: _ClassVar[int]
    center: _containers.RepeatedScalarFieldContainer[float]
    radius: float
    def __init__(self, center: _Optional[_Iterable[float]] = ..., radius: _Optional[float] = ...) -> None: ...

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
    __slots__ = ("id", "distance", "metadata", "typed_metadata", "payload")
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
    PAYLOAD_FIELD_NUMBER: _ClassVar[int]
    id: int
    distance: float
    metadata: _containers.ScalarMap[str, str]
    typed_metadata: _containers.MessageMap[str, MetadataValue]
    payload: bytes
    def __init__(self, id: _Optional[int] = ..., distance: _Optional[float] = ..., metadata: _Optional[_Mapping[str, str]] = ..., typed_metadata: _Optional[_Mapping[str, MetadataValue]] = ..., payload: _Optional[bytes] = ...) -> None: ...

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
    __slots__ = ("id", "layer", "neighbors", "metadata", "typed_metadata", "edge_types")
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
    EDGE_TYPES_FIELD_NUMBER: _ClassVar[int]
    id: int
    layer: int
    neighbors: _containers.RepeatedScalarFieldContainer[int]
    metadata: _containers.ScalarMap[str, str]
    typed_metadata: _containers.MessageMap[str, MetadataValue]
    edge_types: _containers.RepeatedScalarFieldContainer[EdgeType]
    def __init__(self, id: _Optional[int] = ..., layer: _Optional[int] = ..., neighbors: _Optional[_Iterable[int]] = ..., metadata: _Optional[_Mapping[str, str]] = ..., typed_metadata: _Optional[_Mapping[str, MetadataValue]] = ..., edge_types: _Optional[_Iterable[_Union[EdgeType, str]]] = ...) -> None: ...

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
    __slots__ = ("collection", "start_id", "max_depth", "max_nodes", "layer", "filter", "filters", "traversal_mode", "breadth_limit")
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
    TRAVERSAL_MODE_FIELD_NUMBER: _ClassVar[int]
    BREADTH_LIMIT_FIELD_NUMBER: _ClassVar[int]
    collection: str
    start_id: int
    max_depth: int
    max_nodes: int
    layer: int
    filter: _containers.ScalarMap[str, str]
    filters: _containers.RepeatedCompositeFieldContainer[Filter]
    traversal_mode: TraversalMode
    breadth_limit: int
    def __init__(self, collection: _Optional[str] = ..., start_id: _Optional[int] = ..., max_depth: _Optional[int] = ..., max_nodes: _Optional[int] = ..., layer: _Optional[int] = ..., filter: _Optional[_Mapping[str, str]] = ..., filters: _Optional[_Iterable[_Union[Filter, _Mapping]]] = ..., traversal_mode: _Optional[_Union[TraversalMode, str]] = ..., breadth_limit: _Optional[int] = ...) -> None: ...

class TraverseResponse(_message.Message):
    __slots__ = ("nodes",)
    NODES_FIELD_NUMBER: _ClassVar[int]
    nodes: _containers.RepeatedCompositeFieldContainer[GraphNode]
    def __init__(self, nodes: _Optional[_Iterable[_Union[GraphNode, _Mapping]]] = ...) -> None: ...

class GetSubsumptionTreeRequest(_message.Message):
    __slots__ = ("collection", "root_id", "max_depth")
    COLLECTION_FIELD_NUMBER: _ClassVar[int]
    ROOT_ID_FIELD_NUMBER: _ClassVar[int]
    MAX_DEPTH_FIELD_NUMBER: _ClassVar[int]
    collection: str
    root_id: int
    max_depth: int
    def __init__(self, collection: _Optional[str] = ..., root_id: _Optional[int] = ..., max_depth: _Optional[int] = ...) -> None: ...

class GetSubsumptionTreeResponse(_message.Message):
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

class EdgeItem(_message.Message):
    __slots__ = ("source_id", "target_id", "relation", "weight")
    SOURCE_ID_FIELD_NUMBER: _ClassVar[int]
    TARGET_ID_FIELD_NUMBER: _ClassVar[int]
    RELATION_FIELD_NUMBER: _ClassVar[int]
    WEIGHT_FIELD_NUMBER: _ClassVar[int]
    source_id: int
    target_id: int
    relation: str
    weight: float
    def __init__(self, source_id: _Optional[int] = ..., target_id: _Optional[int] = ..., relation: _Optional[str] = ..., weight: _Optional[float] = ...) -> None: ...

class LinkRequest(_message.Message):
    __slots__ = ("collection", "edge")
    COLLECTION_FIELD_NUMBER: _ClassVar[int]
    EDGE_FIELD_NUMBER: _ClassVar[int]
    collection: str
    edge: EdgeItem
    def __init__(self, collection: _Optional[str] = ..., edge: _Optional[_Union[EdgeItem, _Mapping]] = ...) -> None: ...

class LinkBatchRequest(_message.Message):
    __slots__ = ("collection", "edges")
    COLLECTION_FIELD_NUMBER: _ClassVar[int]
    EDGES_FIELD_NUMBER: _ClassVar[int]
    collection: str
    edges: _containers.RepeatedCompositeFieldContainer[EdgeItem]
    def __init__(self, collection: _Optional[str] = ..., edges: _Optional[_Iterable[_Union[EdgeItem, _Mapping]]] = ...) -> None: ...

class LinkBatchResponse(_message.Message):
    __slots__ = ("success", "edges_linked")
    SUCCESS_FIELD_NUMBER: _ClassVar[int]
    EDGES_LINKED_FIELD_NUMBER: _ClassVar[int]
    success: bool
    edges_linked: int
    def __init__(self, success: bool = ..., edges_linked: _Optional[int] = ...) -> None: ...

class UnlinkRequest(_message.Message):
    __slots__ = ("collection", "source_id", "target_id", "relation")
    COLLECTION_FIELD_NUMBER: _ClassVar[int]
    SOURCE_ID_FIELD_NUMBER: _ClassVar[int]
    TARGET_ID_FIELD_NUMBER: _ClassVar[int]
    RELATION_FIELD_NUMBER: _ClassVar[int]
    collection: str
    source_id: int
    target_id: int
    relation: str
    def __init__(self, collection: _Optional[str] = ..., source_id: _Optional[int] = ..., target_id: _Optional[int] = ..., relation: _Optional[str] = ...) -> None: ...

class MetadataInductionRuleProto(_message.Message):
    __slots__ = ("field", "rule_type", "target_field", "relation", "weight")
    FIELD_FIELD_NUMBER: _ClassVar[int]
    RULE_TYPE_FIELD_NUMBER: _ClassVar[int]
    TARGET_FIELD_FIELD_NUMBER: _ClassVar[int]
    RELATION_FIELD_NUMBER: _ClassVar[int]
    WEIGHT_FIELD_NUMBER: _ClassVar[int]
    field: str
    rule_type: str
    target_field: str
    relation: str
    weight: float
    def __init__(self, field: _Optional[str] = ..., rule_type: _Optional[str] = ..., target_field: _Optional[str] = ..., relation: _Optional[str] = ..., weight: _Optional[float] = ...) -> None: ...

class InduceEdgesRequest(_message.Message):
    __slots__ = ("collection", "rules")
    COLLECTION_FIELD_NUMBER: _ClassVar[int]
    RULES_FIELD_NUMBER: _ClassVar[int]
    collection: str
    rules: _containers.RepeatedCompositeFieldContainer[MetadataInductionRuleProto]
    def __init__(self, collection: _Optional[str] = ..., rules: _Optional[_Iterable[_Union[MetadataInductionRuleProto, _Mapping]]] = ...) -> None: ...

class InduceEdgesResponse(_message.Message):
    __slots__ = ("success", "edges_created")
    SUCCESS_FIELD_NUMBER: _ClassVar[int]
    EDGES_CREATED_FIELD_NUMBER: _ClassVar[int]
    success: bool
    edges_created: int
    def __init__(self, success: bool = ..., edges_created: _Optional[int] = ...) -> None: ...

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

class TrajectoryStepEvent(_message.Message):
    __slots__ = ("id", "collection", "x", "y", "metadata")
    class MetadataEntry(_message.Message):
        __slots__ = ("key", "value")
        KEY_FIELD_NUMBER: _ClassVar[int]
        VALUE_FIELD_NUMBER: _ClassVar[int]
        key: str
        value: str
        def __init__(self, key: _Optional[str] = ..., value: _Optional[str] = ...) -> None: ...
    ID_FIELD_NUMBER: _ClassVar[int]
    COLLECTION_FIELD_NUMBER: _ClassVar[int]
    X_FIELD_NUMBER: _ClassVar[int]
    Y_FIELD_NUMBER: _ClassVar[int]
    METADATA_FIELD_NUMBER: _ClassVar[int]
    id: int
    collection: str
    x: float
    y: float
    metadata: _containers.ScalarMap[str, str]
    def __init__(self, id: _Optional[int] = ..., collection: _Optional[str] = ..., x: _Optional[float] = ..., y: _Optional[float] = ..., metadata: _Optional[_Mapping[str, str]] = ...) -> None: ...

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
    __slots__ = ("type", "vector_inserted", "vector_deleted", "trajectory_step")
    TYPE_FIELD_NUMBER: _ClassVar[int]
    VECTOR_INSERTED_FIELD_NUMBER: _ClassVar[int]
    VECTOR_DELETED_FIELD_NUMBER: _ClassVar[int]
    TRAJECTORY_STEP_FIELD_NUMBER: _ClassVar[int]
    type: EventType
    vector_inserted: VectorInsertedEvent
    vector_deleted: VectorDeletedEvent
    trajectory_step: TrajectoryStepEvent
    def __init__(self, type: _Optional[_Union[EventType, str]] = ..., vector_inserted: _Optional[_Union[VectorInsertedEvent, _Mapping]] = ..., vector_deleted: _Optional[_Union[VectorDeletedEvent, _Mapping]] = ..., trajectory_step: _Optional[_Union[TrajectoryStepEvent, _Mapping]] = ...) -> None: ...

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

class FreezeCollectionRequest(_message.Message):
    __slots__ = ("name",)
    NAME_FIELD_NUMBER: _ClassVar[int]
    name: str
    def __init__(self, name: _Optional[str] = ...) -> None: ...

class UnfreezeCollectionRequest(_message.Message):
    __slots__ = ("name",)
    NAME_FIELD_NUMBER: _ClassVar[int]
    name: str
    def __init__(self, name: _Optional[str] = ...) -> None: ...

class VerifyGenerationRequest(_message.Message):
    __slots__ = ("collection", "llm_answer", "context_ids", "max_distance_threshold", "check_geodesic")
    COLLECTION_FIELD_NUMBER: _ClassVar[int]
    LLM_ANSWER_FIELD_NUMBER: _ClassVar[int]
    CONTEXT_IDS_FIELD_NUMBER: _ClassVar[int]
    MAX_DISTANCE_THRESHOLD_FIELD_NUMBER: _ClassVar[int]
    CHECK_GEODESIC_FIELD_NUMBER: _ClassVar[int]
    collection: str
    llm_answer: str
    context_ids: _containers.RepeatedScalarFieldContainer[str]
    max_distance_threshold: float
    check_geodesic: bool
    def __init__(self, collection: _Optional[str] = ..., llm_answer: _Optional[str] = ..., context_ids: _Optional[_Iterable[str]] = ..., max_distance_threshold: _Optional[float] = ..., check_geodesic: bool = ...) -> None: ...

class VerifyGenerationResponse(_message.Message):
    __slots__ = ("trust_score", "is_valid", "min_distance", "avg_distance", "geodesic_distance", "geodesic_hops", "cone_satisfied", "details", "resolved_context_ids")
    TRUST_SCORE_FIELD_NUMBER: _ClassVar[int]
    IS_VALID_FIELD_NUMBER: _ClassVar[int]
    MIN_DISTANCE_FIELD_NUMBER: _ClassVar[int]
    AVG_DISTANCE_FIELD_NUMBER: _ClassVar[int]
    GEODESIC_DISTANCE_FIELD_NUMBER: _ClassVar[int]
    GEODESIC_HOPS_FIELD_NUMBER: _ClassVar[int]
    CONE_SATISFIED_FIELD_NUMBER: _ClassVar[int]
    DETAILS_FIELD_NUMBER: _ClassVar[int]
    RESOLVED_CONTEXT_IDS_FIELD_NUMBER: _ClassVar[int]
    trust_score: float
    is_valid: bool
    min_distance: float
    avg_distance: float
    geodesic_distance: float
    geodesic_hops: int
    cone_satisfied: bool
    details: str
    resolved_context_ids: _containers.RepeatedScalarFieldContainer[int]
    def __init__(self, trust_score: _Optional[float] = ..., is_valid: bool = ..., min_distance: _Optional[float] = ..., avg_distance: _Optional[float] = ..., geodesic_distance: _Optional[float] = ..., geodesic_hops: _Optional[int] = ..., cone_satisfied: bool = ..., details: _Optional[str] = ..., resolved_context_ids: _Optional[_Iterable[int]] = ...) -> None: ...
