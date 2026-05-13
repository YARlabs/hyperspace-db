///
//  Generated code. Do not modify.
//  source: hyperspace.proto
//
// @dart = 2.12
// ignore_for_file: annotate_overrides,camel_case_types,constant_identifier_names,deprecated_member_use_from_same_package,directives_ordering,library_prefixes,non_constant_identifier_names,prefer_final_fields,return_of_invalid_type,unnecessary_const,unnecessary_import,unnecessary_this,unused_import,unused_shown_name

import 'dart:core' as $core;
import 'dart:convert' as $convert;
import 'dart:typed_data' as $typed_data;
@$core.Deprecated('Use quantizationModeDescriptor instead')
const QuantizationMode$json = const {
  '1': 'QuantizationMode',
  '2': const [
    const {'1': 'NONE', '2': 0},
    const {'1': 'SCALAR_I8', '2': 1},
  ],
};

/// Descriptor for `QuantizationMode`. Decode as a `google.protobuf.EnumDescriptorProto`.
final $typed_data.Uint8List quantizationModeDescriptor = $convert.base64Decode('ChBRdWFudGl6YXRpb25Nb2RlEggKBE5PTkUQABINCglTQ0FMQVJfSTgQAQ==');
@$core.Deprecated('Use durabilityLevelDescriptor instead')
const DurabilityLevel$json = const {
  '1': 'DurabilityLevel',
  '2': const [
    const {'1': 'DEFAULT_LEVEL', '2': 0},
    const {'1': 'ASYNC', '2': 1},
    const {'1': 'BATCH', '2': 2},
    const {'1': 'STRICT', '2': 3},
  ],
};

/// Descriptor for `DurabilityLevel`. Decode as a `google.protobuf.EnumDescriptorProto`.
final $typed_data.Uint8List durabilityLevelDescriptor = $convert.base64Decode('Cg9EdXJhYmlsaXR5TGV2ZWwSEQoNREVGQVVMVF9MRVZFTBAAEgkKBUFTWU5DEAESCQoFQkFUQ0gQAhIKCgZTVFJJQ1QQAw==');
@$core.Deprecated('Use eventTypeDescriptor instead')
const EventType$json = const {
  '1': 'EventType',
  '2': const [
    const {'1': 'EVENT_UNKNOWN', '2': 0},
    const {'1': 'VECTOR_INSERTED', '2': 1},
    const {'1': 'VECTOR_DELETED', '2': 2},
    const {'1': 'TRAJECTORY_STEP', '2': 3},
  ],
};

/// Descriptor for `EventType`. Decode as a `google.protobuf.EnumDescriptorProto`.
final $typed_data.Uint8List eventTypeDescriptor = $convert.base64Decode('CglFdmVudFR5cGUSEQoNRVZFTlRfVU5LTk9XThAAEhMKD1ZFQ1RPUl9JTlNFUlRFRBABEhIKDlZFQ1RPUl9ERUxFVEVEEAISEwoPVFJBSkVDVE9SWV9TVEVQEAM=');
@$core.Deprecated('Use replicationRequestDescriptor instead')
const ReplicationRequest$json = const {
  '1': 'ReplicationRequest',
  '2': const [
    const {'1': 'last_logical_clock', '3': 1, '4': 1, '5': 4, '10': 'lastLogicalClock'},
  ],
};

/// Descriptor for `ReplicationRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List replicationRequestDescriptor = $convert.base64Decode('ChJSZXBsaWNhdGlvblJlcXVlc3QSLAoSbGFzdF9sb2dpY2FsX2Nsb2NrGAEgASgEUhBsYXN0TG9naWNhbENsb2Nr');
@$core.Deprecated('Use replicationLogDescriptor instead')
const ReplicationLog$json = const {
  '1': 'ReplicationLog',
  '2': const [
    const {'1': 'logical_clock', '3': 1, '4': 1, '5': 4, '10': 'logicalClock'},
    const {'1': 'origin_node_id', '3': 2, '4': 1, '5': 9, '10': 'originNodeId'},
    const {'1': 'collection', '3': 3, '4': 1, '5': 9, '10': 'collection'},
    const {'1': 'insert', '3': 4, '4': 1, '5': 11, '6': '.hyperspace.InsertOp', '9': 0, '10': 'insert'},
    const {'1': 'create_collection', '3': 5, '4': 1, '5': 11, '6': '.hyperspace.CreateCollectionOp', '9': 0, '10': 'createCollection'},
    const {'1': 'delete_collection', '3': 6, '4': 1, '5': 11, '6': '.hyperspace.DeleteCollectionOp', '9': 0, '10': 'deleteCollection'},
    const {'1': 'delete', '3': 7, '4': 1, '5': 11, '6': '.hyperspace.DeleteOp', '9': 0, '10': 'delete'},
  ],
  '8': const [
    const {'1': 'operation'},
  ],
};

/// Descriptor for `ReplicationLog`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List replicationLogDescriptor = $convert.base64Decode('Cg5SZXBsaWNhdGlvbkxvZxIjCg1sb2dpY2FsX2Nsb2NrGAEgASgEUgxsb2dpY2FsQ2xvY2sSJAoOb3JpZ2luX25vZGVfaWQYAiABKAlSDG9yaWdpbk5vZGVJZBIeCgpjb2xsZWN0aW9uGAMgASgJUgpjb2xsZWN0aW9uEi4KBmluc2VydBgEIAEoCzIULmh5cGVyc3BhY2UuSW5zZXJ0T3BIAFIGaW5zZXJ0Ek0KEWNyZWF0ZV9jb2xsZWN0aW9uGAUgASgLMh4uaHlwZXJzcGFjZS5DcmVhdGVDb2xsZWN0aW9uT3BIAFIQY3JlYXRlQ29sbGVjdGlvbhJNChFkZWxldGVfY29sbGVjdGlvbhgGIAEoCzIeLmh5cGVyc3BhY2UuRGVsZXRlQ29sbGVjdGlvbk9wSABSEGRlbGV0ZUNvbGxlY3Rpb24SLgoGZGVsZXRlGAcgASgLMhQuaHlwZXJzcGFjZS5EZWxldGVPcEgAUgZkZWxldGVCCwoJb3BlcmF0aW9u');
@$core.Deprecated('Use insertOpDescriptor instead')
const InsertOp$json = const {
  '1': 'InsertOp',
  '2': const [
    const {'1': 'id', '3': 1, '4': 1, '5': 13, '10': 'id'},
    const {'1': 'vector', '3': 2, '4': 3, '5': 1, '10': 'vector'},
    const {'1': 'metadata', '3': 3, '4': 3, '5': 11, '6': '.hyperspace.InsertOp.MetadataEntry', '10': 'metadata'},
    const {'1': 'typed_metadata', '3': 4, '4': 3, '5': 11, '6': '.hyperspace.InsertOp.TypedMetadataEntry', '10': 'typedMetadata'},
  ],
  '3': const [InsertOp_MetadataEntry$json, InsertOp_TypedMetadataEntry$json],
};

@$core.Deprecated('Use insertOpDescriptor instead')
const InsertOp_MetadataEntry$json = const {
  '1': 'MetadataEntry',
  '2': const [
    const {'1': 'key', '3': 1, '4': 1, '5': 9, '10': 'key'},
    const {'1': 'value', '3': 2, '4': 1, '5': 9, '10': 'value'},
  ],
  '7': const {'7': true},
};

@$core.Deprecated('Use insertOpDescriptor instead')
const InsertOp_TypedMetadataEntry$json = const {
  '1': 'TypedMetadataEntry',
  '2': const [
    const {'1': 'key', '3': 1, '4': 1, '5': 9, '10': 'key'},
    const {'1': 'value', '3': 2, '4': 1, '5': 11, '6': '.hyperspace.MetadataValue', '10': 'value'},
  ],
  '7': const {'7': true},
};

/// Descriptor for `InsertOp`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List insertOpDescriptor = $convert.base64Decode('CghJbnNlcnRPcBIOCgJpZBgBIAEoDVICaWQSFgoGdmVjdG9yGAIgAygBUgZ2ZWN0b3ISPgoIbWV0YWRhdGEYAyADKAsyIi5oeXBlcnNwYWNlLkluc2VydE9wLk1ldGFkYXRhRW50cnlSCG1ldGFkYXRhEk4KDnR5cGVkX21ldGFkYXRhGAQgAygLMicuaHlwZXJzcGFjZS5JbnNlcnRPcC5UeXBlZE1ldGFkYXRhRW50cnlSDXR5cGVkTWV0YWRhdGEaOwoNTWV0YWRhdGFFbnRyeRIQCgNrZXkYASABKAlSA2tleRIUCgV2YWx1ZRgCIAEoCVIFdmFsdWU6AjgBGlsKElR5cGVkTWV0YWRhdGFFbnRyeRIQCgNrZXkYASABKAlSA2tleRIvCgV2YWx1ZRgCIAEoCzIZLmh5cGVyc3BhY2UuTWV0YWRhdGFWYWx1ZVIFdmFsdWU6AjgB');
@$core.Deprecated('Use createCollectionOpDescriptor instead')
const CreateCollectionOp$json = const {
  '1': 'CreateCollectionOp',
  '2': const [
    const {'1': 'schema', '3': 3, '4': 1, '5': 11, '6': '.hyperspace.CollectionSchema', '9': 0, '10': 'schema', '17': true},
  ],
  '8': const [
    const {'1': '_schema'},
  ],
  '9': const [
    const {'1': 1, '2': 2},
    const {'1': 2, '2': 3},
  ],
};

/// Descriptor for `CreateCollectionOp`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List createCollectionOpDescriptor = $convert.base64Decode('ChJDcmVhdGVDb2xsZWN0aW9uT3ASOQoGc2NoZW1hGAMgASgLMhwuaHlwZXJzcGFjZS5Db2xsZWN0aW9uU2NoZW1hSABSBnNjaGVtYYgBAUIJCgdfc2NoZW1hSgQIARACSgQIAhAD');
@$core.Deprecated('Use deleteCollectionOpDescriptor instead')
const DeleteCollectionOp$json = const {
  '1': 'DeleteCollectionOp',
};

/// Descriptor for `DeleteCollectionOp`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List deleteCollectionOpDescriptor = $convert.base64Decode('ChJEZWxldGVDb2xsZWN0aW9uT3A=');
@$core.Deprecated('Use deleteOpDescriptor instead')
const DeleteOp$json = const {
  '1': 'DeleteOp',
  '2': const [
    const {'1': 'id', '3': 1, '4': 1, '5': 13, '10': 'id'},
  ],
};

/// Descriptor for `DeleteOp`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List deleteOpDescriptor = $convert.base64Decode('CghEZWxldGVPcBIOCgJpZBgBIAEoDVICaWQ=');
@$core.Deprecated('Use quantizationConfigDescriptor instead')
const QuantizationConfig$json = const {
  '1': 'QuantizationConfig',
  '2': const [
    const {'1': 'mode', '3': 1, '4': 1, '5': 14, '6': '.hyperspace.QuantizationMode', '10': 'mode'},
  ],
};

/// Descriptor for `QuantizationConfig`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List quantizationConfigDescriptor = $convert.base64Decode('ChJRdWFudGl6YXRpb25Db25maWcSMAoEbW9kZRgBIAEoDjIcLmh5cGVyc3BhY2UuUXVhbnRpemF0aW9uTW9kZVIEbW9kZQ==');
@$core.Deprecated('Use createCollectionRequestDescriptor instead')
const CreateCollectionRequest$json = const {
  '1': 'CreateCollectionRequest',
  '2': const [
    const {'1': 'name', '3': 1, '4': 1, '5': 9, '10': 'name'},
    const {'1': 'schema', '3': 5, '4': 1, '5': 11, '6': '.hyperspace.CollectionSchema', '9': 0, '10': 'schema', '17': true},
  ],
  '8': const [
    const {'1': '_schema'},
  ],
  '9': const [
    const {'1': 2, '2': 3},
    const {'1': 3, '2': 4},
    const {'1': 4, '2': 5},
  ],
};

/// Descriptor for `CreateCollectionRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List createCollectionRequestDescriptor = $convert.base64Decode('ChdDcmVhdGVDb2xsZWN0aW9uUmVxdWVzdBISCgRuYW1lGAEgASgJUgRuYW1lEjkKBnNjaGVtYRgFIAEoCzIcLmh5cGVyc3BhY2UuQ29sbGVjdGlvblNjaGVtYUgAUgZzY2hlbWGIAQFCCQoHX3NjaGVtYUoECAIQA0oECAMQBEoECAQQBQ==');
@$core.Deprecated('Use vectorComponentDescriptor instead')
const VectorComponent$json = const {
  '1': 'VectorComponent',
  '2': const [
    const {'1': 'name', '3': 1, '4': 1, '5': 9, '10': 'name'},
    const {'1': 'metric', '3': 2, '4': 1, '5': 9, '10': 'metric'},
    const {'1': 'full_dimension', '3': 3, '4': 1, '5': 13, '10': 'fullDimension'},
    const {'1': 'weight', '3': 4, '4': 1, '5': 2, '10': 'weight'},
  ],
};

/// Descriptor for `VectorComponent`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List vectorComponentDescriptor = $convert.base64Decode('Cg9WZWN0b3JDb21wb25lbnQSEgoEbmFtZRgBIAEoCVIEbmFtZRIWCgZtZXRyaWMYAiABKAlSBm1ldHJpYxIlCg5mdWxsX2RpbWVuc2lvbhgDIAEoDVINZnVsbERpbWVuc2lvbhIWCgZ3ZWlnaHQYBCABKAJSBndlaWdodA==');
@$core.Deprecated('Use mrlLayerDescriptor instead')
const MrlLayer$json = const {
  '1': 'MrlLayer',
  '2': const [
    const {'1': 'component_name', '3': 1, '4': 1, '5': 9, '10': 'componentName'},
    const {'1': 'cutoff_dimension', '3': 2, '4': 1, '5': 13, '10': 'cutoffDimension'},
    const {'1': 'store_in_ram', '3': 3, '4': 1, '5': 8, '10': 'storeInRam'},
    const {'1': 'rerank_top_k', '3': 4, '4': 1, '5': 13, '10': 'rerankTopK'},
  ],
};

/// Descriptor for `MrlLayer`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List mrlLayerDescriptor = $convert.base64Decode('CghNcmxMYXllchIlCg5jb21wb25lbnRfbmFtZRgBIAEoCVINY29tcG9uZW50TmFtZRIpChBjdXRvZmZfZGltZW5zaW9uGAIgASgNUg9jdXRvZmZEaW1lbnNpb24SIAoMc3RvcmVfaW5fcmFtGAMgASgIUgpzdG9yZUluUmFtEiAKDHJlcmFua190b3BfaxgEIAEoDVIKcmVyYW5rVG9wSw==');
@$core.Deprecated('Use collectionSchemaDescriptor instead')
const CollectionSchema$json = const {
  '1': 'CollectionSchema',
  '2': const [
    const {'1': 'components', '3': 1, '4': 3, '5': 11, '6': '.hyperspace.VectorComponent', '10': 'components'},
    const {'1': 'cascade_pipeline', '3': 2, '4': 3, '5': 11, '6': '.hyperspace.MrlLayer', '10': 'cascadePipeline'},
  ],
};

/// Descriptor for `CollectionSchema`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List collectionSchemaDescriptor = $convert.base64Decode('ChBDb2xsZWN0aW9uU2NoZW1hEjsKCmNvbXBvbmVudHMYASADKAsyGy5oeXBlcnNwYWNlLlZlY3RvckNvbXBvbmVudFIKY29tcG9uZW50cxI/ChBjYXNjYWRlX3BpcGVsaW5lGAIgAygLMhQuaHlwZXJzcGFjZS5NcmxMYXllclIPY2FzY2FkZVBpcGVsaW5l');
@$core.Deprecated('Use collectionComponentDescriptor instead')
const CollectionComponent$json = const {
  '1': 'CollectionComponent',
  '2': const [
    const {'1': 'space', '3': 1, '4': 1, '5': 9, '10': 'space'},
    const {'1': 'dimension', '3': 2, '4': 1, '5': 13, '10': 'dimension'},
    const {'1': 'metric', '3': 3, '4': 1, '5': 9, '10': 'metric'},
    const {'1': 'weight', '3': 4, '4': 1, '5': 2, '9': 0, '10': 'weight', '17': true},
  ],
  '8': const [
    const {'1': '_weight'},
  ],
};

/// Descriptor for `CollectionComponent`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List collectionComponentDescriptor = $convert.base64Decode('ChNDb2xsZWN0aW9uQ29tcG9uZW50EhQKBXNwYWNlGAEgASgJUgVzcGFjZRIcCglkaW1lbnNpb24YAiABKA1SCWRpbWVuc2lvbhIWCgZtZXRyaWMYAyABKAlSBm1ldHJpYxIbCgZ3ZWlnaHQYBCABKAJIAFIGd2VpZ2h0iAEBQgkKB193ZWlnaHQ=');
@$core.Deprecated('Use deleteCollectionRequestDescriptor instead')
const DeleteCollectionRequest$json = const {
  '1': 'DeleteCollectionRequest',
  '2': const [
    const {'1': 'name', '3': 1, '4': 1, '5': 9, '10': 'name'},
  ],
};

/// Descriptor for `DeleteCollectionRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List deleteCollectionRequestDescriptor = $convert.base64Decode('ChdEZWxldGVDb2xsZWN0aW9uUmVxdWVzdBISCgRuYW1lGAEgASgJUgRuYW1l');
@$core.Deprecated('Use collectionSummaryDescriptor instead')
const CollectionSummary$json = const {
  '1': 'CollectionSummary',
  '2': const [
    const {'1': 'name', '3': 1, '4': 1, '5': 9, '10': 'name'},
    const {'1': 'count', '3': 2, '4': 1, '5': 4, '10': 'count'},
    const {'1': 'schema', '3': 5, '4': 1, '5': 11, '6': '.hyperspace.CollectionSchema', '9': 0, '10': 'schema', '17': true},
  ],
  '8': const [
    const {'1': '_schema'},
  ],
  '9': const [
    const {'1': 3, '2': 4},
    const {'1': 4, '2': 5},
  ],
};

/// Descriptor for `CollectionSummary`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List collectionSummaryDescriptor = $convert.base64Decode('ChFDb2xsZWN0aW9uU3VtbWFyeRISCgRuYW1lGAEgASgJUgRuYW1lEhQKBWNvdW50GAIgASgEUgVjb3VudBI5CgZzY2hlbWEYBSABKAsyHC5oeXBlcnNwYWNlLkNvbGxlY3Rpb25TY2hlbWFIAFIGc2NoZW1hiAEBQgkKB19zY2hlbWFKBAgDEARKBAgEEAU=');
@$core.Deprecated('Use listCollectionsResponseDescriptor instead')
const ListCollectionsResponse$json = const {
  '1': 'ListCollectionsResponse',
  '2': const [
    const {'1': 'collections', '3': 1, '4': 3, '5': 11, '6': '.hyperspace.CollectionSummary', '10': 'collections'},
  ],
};

/// Descriptor for `ListCollectionsResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listCollectionsResponseDescriptor = $convert.base64Decode('ChdMaXN0Q29sbGVjdGlvbnNSZXNwb25zZRI/Cgtjb2xsZWN0aW9ucxgBIAMoCzIdLmh5cGVyc3BhY2UuQ29sbGVjdGlvblN1bW1hcnlSC2NvbGxlY3Rpb25z');
@$core.Deprecated('Use collectionStatsRequestDescriptor instead')
const CollectionStatsRequest$json = const {
  '1': 'CollectionStatsRequest',
  '2': const [
    const {'1': 'name', '3': 1, '4': 1, '5': 9, '10': 'name'},
  ],
};

/// Descriptor for `CollectionStatsRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List collectionStatsRequestDescriptor = $convert.base64Decode('ChZDb2xsZWN0aW9uU3RhdHNSZXF1ZXN0EhIKBG5hbWUYASABKAlSBG5hbWU=');
@$core.Deprecated('Use collectionStatsResponseDescriptor instead')
const CollectionStatsResponse$json = const {
  '1': 'CollectionStatsResponse',
  '2': const [
    const {'1': 'count', '3': 1, '4': 1, '5': 4, '10': 'count'},
    const {'1': 'indexing_queue', '3': 4, '4': 1, '5': 4, '10': 'indexingQueue'},
    const {'1': 'disk_usage_bytes', '3': 5, '4': 1, '5': 4, '10': 'diskUsageBytes'},
    const {'1': 'ram_usage_bytes', '3': 6, '4': 1, '5': 4, '10': 'ramUsageBytes'},
    const {'1': 'active_tasks', '3': 7, '4': 1, '5': 4, '10': 'activeTasks'},
    const {'1': 'schema', '3': 8, '4': 1, '5': 11, '6': '.hyperspace.CollectionSchema', '9': 0, '10': 'schema', '17': true},
  ],
  '8': const [
    const {'1': '_schema'},
  ],
  '9': const [
    const {'1': 2, '2': 3},
    const {'1': 3, '2': 4},
  ],
};

/// Descriptor for `CollectionStatsResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List collectionStatsResponseDescriptor = $convert.base64Decode('ChdDb2xsZWN0aW9uU3RhdHNSZXNwb25zZRIUCgVjb3VudBgBIAEoBFIFY291bnQSJQoOaW5kZXhpbmdfcXVldWUYBCABKARSDWluZGV4aW5nUXVldWUSKAoQZGlza191c2FnZV9ieXRlcxgFIAEoBFIOZGlza1VzYWdlQnl0ZXMSJgoPcmFtX3VzYWdlX2J5dGVzGAYgASgEUg1yYW1Vc2FnZUJ5dGVzEiEKDGFjdGl2ZV90YXNrcxgHIAEoBFILYWN0aXZlVGFza3MSOQoGc2NoZW1hGAggASgLMhwuaHlwZXJzcGFjZS5Db2xsZWN0aW9uU2NoZW1hSABSBnNjaGVtYYgBAUIJCgdfc2NoZW1hSgQIAhADSgQIAxAE');
@$core.Deprecated('Use rebuildIndexRequestDescriptor instead')
const RebuildIndexRequest$json = const {
  '1': 'RebuildIndexRequest',
  '2': const [
    const {'1': 'name', '3': 1, '4': 1, '5': 9, '10': 'name'},
    const {'1': 'filter_query', '3': 2, '4': 1, '5': 11, '6': '.hyperspace.VacuumFilterQuery', '9': 0, '10': 'filterQuery', '17': true},
  ],
  '8': const [
    const {'1': '_filter_query'},
  ],
};

/// Descriptor for `RebuildIndexRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List rebuildIndexRequestDescriptor = $convert.base64Decode('ChNSZWJ1aWxkSW5kZXhSZXF1ZXN0EhIKBG5hbWUYASABKAlSBG5hbWUSRQoMZmlsdGVyX3F1ZXJ5GAIgASgLMh0uaHlwZXJzcGFjZS5WYWN1dW1GaWx0ZXJRdWVyeUgAUgtmaWx0ZXJRdWVyeYgBAUIPCg1fZmlsdGVyX3F1ZXJ5');
@$core.Deprecated('Use configUpdateDescriptor instead')
const ConfigUpdate$json = const {
  '1': 'ConfigUpdate',
  '2': const [
    const {'1': 'collection', '3': 1, '4': 1, '5': 9, '10': 'collection'},
    const {'1': 'ef_search', '3': 2, '4': 1, '5': 13, '9': 0, '10': 'efSearch', '17': true},
    const {'1': 'ef_construction', '3': 3, '4': 1, '5': 13, '9': 1, '10': 'efConstruction', '17': true},
    const {'1': 'm', '3': 4, '4': 1, '5': 13, '9': 2, '10': 'm', '17': true},
  ],
  '8': const [
    const {'1': '_ef_search'},
    const {'1': '_ef_construction'},
    const {'1': '_m'},
  ],
};

/// Descriptor for `ConfigUpdate`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List configUpdateDescriptor = $convert.base64Decode('CgxDb25maWdVcGRhdGUSHgoKY29sbGVjdGlvbhgBIAEoCVIKY29sbGVjdGlvbhIgCgllZl9zZWFyY2gYAiABKA1IAFIIZWZTZWFyY2iIAQESLAoPZWZfY29uc3RydWN0aW9uGAMgASgNSAFSDmVmQ29uc3RydWN0aW9uiAEBEhEKAW0YBCABKA1IAlIBbYgBAUIMCgpfZWZfc2VhcmNoQhIKEF9lZl9jb25zdHJ1Y3Rpb25CBAoCX20=');
@$core.Deprecated('Use vacuumFilterQueryDescriptor instead')
const VacuumFilterQuery$json = const {
  '1': 'VacuumFilterQuery',
  '2': const [
    const {'1': 'key', '3': 1, '4': 1, '5': 9, '10': 'key'},
    const {'1': 'op', '3': 2, '4': 1, '5': 9, '10': 'op'},
    const {'1': 'value', '3': 3, '4': 1, '5': 1, '10': 'value'},
  ],
};

/// Descriptor for `VacuumFilterQuery`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List vacuumFilterQueryDescriptor = $convert.base64Decode('ChFWYWN1dW1GaWx0ZXJRdWVyeRIQCgNrZXkYASABKAlSA2tleRIOCgJvcBgCIAEoCVICb3ASFAoFdmFsdWUYAyABKAFSBXZhbHVl');
@$core.Deprecated('Use reconsolidationRequestDescriptor instead')
const ReconsolidationRequest$json = const {
  '1': 'ReconsolidationRequest',
  '2': const [
    const {'1': 'collection', '3': 1, '4': 1, '5': 9, '10': 'collection'},
    const {'1': 'target_vector', '3': 2, '4': 3, '5': 1, '10': 'targetVector'},
    const {'1': 'learning_rate', '3': 3, '4': 1, '5': 1, '10': 'learningRate'},
  ],
};

/// Descriptor for `ReconsolidationRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List reconsolidationRequestDescriptor = $convert.base64Decode('ChZSZWNvbnNvbGlkYXRpb25SZXF1ZXN0Eh4KCmNvbGxlY3Rpb24YASABKAlSCmNvbGxlY3Rpb24SIwoNdGFyZ2V0X3ZlY3RvchgCIAMoAVIMdGFyZ2V0VmVjdG9yEiMKDWxlYXJuaW5nX3JhdGUYAyABKAFSDGxlYXJuaW5nUmF0ZQ==');
@$core.Deprecated('Use insertRequestDescriptor instead')
const InsertRequest$json = const {
  '1': 'InsertRequest',
  '2': const [
    const {'1': 'collection', '3': 1, '4': 1, '5': 9, '10': 'collection'},
    const {'1': 'vector', '3': 2, '4': 3, '5': 1, '10': 'vector'},
    const {'1': 'id', '3': 3, '4': 1, '5': 13, '10': 'id'},
    const {'1': 'metadata', '3': 4, '4': 3, '5': 11, '6': '.hyperspace.InsertRequest.MetadataEntry', '10': 'metadata'},
    const {'1': 'origin_node_id', '3': 5, '4': 1, '5': 9, '10': 'originNodeId'},
    const {'1': 'logical_clock', '3': 6, '4': 1, '5': 4, '10': 'logicalClock'},
    const {'1': 'durability', '3': 7, '4': 1, '5': 14, '6': '.hyperspace.DurabilityLevel', '10': 'durability'},
    const {'1': 'typed_metadata', '3': 8, '4': 3, '5': 11, '6': '.hyperspace.InsertRequest.TypedMetadataEntry', '10': 'typedMetadata'},
    const {'1': 'payload', '3': 9, '4': 1, '5': 12, '9': 0, '10': 'payload', '17': true},
  ],
  '3': const [InsertRequest_MetadataEntry$json, InsertRequest_TypedMetadataEntry$json],
  '8': const [
    const {'1': '_payload'},
  ],
};

@$core.Deprecated('Use insertRequestDescriptor instead')
const InsertRequest_MetadataEntry$json = const {
  '1': 'MetadataEntry',
  '2': const [
    const {'1': 'key', '3': 1, '4': 1, '5': 9, '10': 'key'},
    const {'1': 'value', '3': 2, '4': 1, '5': 9, '10': 'value'},
  ],
  '7': const {'7': true},
};

@$core.Deprecated('Use insertRequestDescriptor instead')
const InsertRequest_TypedMetadataEntry$json = const {
  '1': 'TypedMetadataEntry',
  '2': const [
    const {'1': 'key', '3': 1, '4': 1, '5': 9, '10': 'key'},
    const {'1': 'value', '3': 2, '4': 1, '5': 11, '6': '.hyperspace.MetadataValue', '10': 'value'},
  ],
  '7': const {'7': true},
};

/// Descriptor for `InsertRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List insertRequestDescriptor = $convert.base64Decode('Cg1JbnNlcnRSZXF1ZXN0Eh4KCmNvbGxlY3Rpb24YASABKAlSCmNvbGxlY3Rpb24SFgoGdmVjdG9yGAIgAygBUgZ2ZWN0b3ISDgoCaWQYAyABKA1SAmlkEkMKCG1ldGFkYXRhGAQgAygLMicuaHlwZXJzcGFjZS5JbnNlcnRSZXF1ZXN0Lk1ldGFkYXRhRW50cnlSCG1ldGFkYXRhEiQKDm9yaWdpbl9ub2RlX2lkGAUgASgJUgxvcmlnaW5Ob2RlSWQSIwoNbG9naWNhbF9jbG9jaxgGIAEoBFIMbG9naWNhbENsb2NrEjsKCmR1cmFiaWxpdHkYByABKA4yGy5oeXBlcnNwYWNlLkR1cmFiaWxpdHlMZXZlbFIKZHVyYWJpbGl0eRJTCg50eXBlZF9tZXRhZGF0YRgIIAMoCzIsLmh5cGVyc3BhY2UuSW5zZXJ0UmVxdWVzdC5UeXBlZE1ldGFkYXRhRW50cnlSDXR5cGVkTWV0YWRhdGESHQoHcGF5bG9hZBgJIAEoDEgAUgdwYXlsb2FkiAEBGjsKDU1ldGFkYXRhRW50cnkSEAoDa2V5GAEgASgJUgNrZXkSFAoFdmFsdWUYAiABKAlSBXZhbHVlOgI4ARpbChJUeXBlZE1ldGFkYXRhRW50cnkSEAoDa2V5GAEgASgJUgNrZXkSLwoFdmFsdWUYAiABKAsyGS5oeXBlcnNwYWNlLk1ldGFkYXRhVmFsdWVSBXZhbHVlOgI4AUIKCghfcGF5bG9hZA==');
@$core.Deprecated('Use vectorDataDescriptor instead')
const VectorData$json = const {
  '1': 'VectorData',
  '2': const [
    const {'1': 'vector', '3': 1, '4': 3, '5': 1, '10': 'vector'},
    const {'1': 'id', '3': 2, '4': 1, '5': 13, '10': 'id'},
    const {'1': 'metadata', '3': 3, '4': 3, '5': 11, '6': '.hyperspace.VectorData.MetadataEntry', '10': 'metadata'},
    const {'1': 'typed_metadata', '3': 4, '4': 3, '5': 11, '6': '.hyperspace.VectorData.TypedMetadataEntry', '10': 'typedMetadata'},
  ],
  '3': const [VectorData_MetadataEntry$json, VectorData_TypedMetadataEntry$json],
};

@$core.Deprecated('Use vectorDataDescriptor instead')
const VectorData_MetadataEntry$json = const {
  '1': 'MetadataEntry',
  '2': const [
    const {'1': 'key', '3': 1, '4': 1, '5': 9, '10': 'key'},
    const {'1': 'value', '3': 2, '4': 1, '5': 9, '10': 'value'},
  ],
  '7': const {'7': true},
};

@$core.Deprecated('Use vectorDataDescriptor instead')
const VectorData_TypedMetadataEntry$json = const {
  '1': 'TypedMetadataEntry',
  '2': const [
    const {'1': 'key', '3': 1, '4': 1, '5': 9, '10': 'key'},
    const {'1': 'value', '3': 2, '4': 1, '5': 11, '6': '.hyperspace.MetadataValue', '10': 'value'},
  ],
  '7': const {'7': true},
};

/// Descriptor for `VectorData`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List vectorDataDescriptor = $convert.base64Decode('CgpWZWN0b3JEYXRhEhYKBnZlY3RvchgBIAMoAVIGdmVjdG9yEg4KAmlkGAIgASgNUgJpZBJACghtZXRhZGF0YRgDIAMoCzIkLmh5cGVyc3BhY2UuVmVjdG9yRGF0YS5NZXRhZGF0YUVudHJ5UghtZXRhZGF0YRJQCg50eXBlZF9tZXRhZGF0YRgEIAMoCzIpLmh5cGVyc3BhY2UuVmVjdG9yRGF0YS5UeXBlZE1ldGFkYXRhRW50cnlSDXR5cGVkTWV0YWRhdGEaOwoNTWV0YWRhdGFFbnRyeRIQCgNrZXkYASABKAlSA2tleRIUCgV2YWx1ZRgCIAEoCVIFdmFsdWU6AjgBGlsKElR5cGVkTWV0YWRhdGFFbnRyeRIQCgNrZXkYASABKAlSA2tleRIvCgV2YWx1ZRgCIAEoCzIZLmh5cGVyc3BhY2UuTWV0YWRhdGFWYWx1ZVIFdmFsdWU6AjgB');
@$core.Deprecated('Use batchInsertRequestDescriptor instead')
const BatchInsertRequest$json = const {
  '1': 'BatchInsertRequest',
  '2': const [
    const {'1': 'collection', '3': 1, '4': 1, '5': 9, '10': 'collection'},
    const {'1': 'vectors', '3': 2, '4': 3, '5': 11, '6': '.hyperspace.VectorData', '10': 'vectors'},
    const {'1': 'origin_node_id', '3': 3, '4': 1, '5': 9, '10': 'originNodeId'},
    const {'1': 'logical_clock', '3': 4, '4': 1, '5': 4, '10': 'logicalClock'},
    const {'1': 'durability', '3': 5, '4': 1, '5': 14, '6': '.hyperspace.DurabilityLevel', '10': 'durability'},
  ],
};

/// Descriptor for `BatchInsertRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List batchInsertRequestDescriptor = $convert.base64Decode('ChJCYXRjaEluc2VydFJlcXVlc3QSHgoKY29sbGVjdGlvbhgBIAEoCVIKY29sbGVjdGlvbhIwCgd2ZWN0b3JzGAIgAygLMhYuaHlwZXJzcGFjZS5WZWN0b3JEYXRhUgd2ZWN0b3JzEiQKDm9yaWdpbl9ub2RlX2lkGAMgASgJUgxvcmlnaW5Ob2RlSWQSIwoNbG9naWNhbF9jbG9jaxgEIAEoBFIMbG9naWNhbENsb2NrEjsKCmR1cmFiaWxpdHkYBSABKA4yGy5oeXBlcnNwYWNlLkR1cmFiaWxpdHlMZXZlbFIKZHVyYWJpbGl0eQ==');
@$core.Deprecated('Use insertTextRequestDescriptor instead')
const InsertTextRequest$json = const {
  '1': 'InsertTextRequest',
  '2': const [
    const {'1': 'collection', '3': 1, '4': 1, '5': 9, '10': 'collection'},
    const {'1': 'id', '3': 2, '4': 1, '5': 13, '10': 'id'},
    const {'1': 'text', '3': 3, '4': 1, '5': 9, '10': 'text'},
    const {'1': 'metadata', '3': 4, '4': 3, '5': 11, '6': '.hyperspace.InsertTextRequest.MetadataEntry', '10': 'metadata'},
    const {'1': 'durability', '3': 5, '4': 1, '5': 14, '6': '.hyperspace.DurabilityLevel', '10': 'durability'},
  ],
  '3': const [InsertTextRequest_MetadataEntry$json],
};

@$core.Deprecated('Use insertTextRequestDescriptor instead')
const InsertTextRequest_MetadataEntry$json = const {
  '1': 'MetadataEntry',
  '2': const [
    const {'1': 'key', '3': 1, '4': 1, '5': 9, '10': 'key'},
    const {'1': 'value', '3': 2, '4': 1, '5': 9, '10': 'value'},
  ],
  '7': const {'7': true},
};

/// Descriptor for `InsertTextRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List insertTextRequestDescriptor = $convert.base64Decode('ChFJbnNlcnRUZXh0UmVxdWVzdBIeCgpjb2xsZWN0aW9uGAEgASgJUgpjb2xsZWN0aW9uEg4KAmlkGAIgASgNUgJpZBISCgR0ZXh0GAMgASgJUgR0ZXh0EkcKCG1ldGFkYXRhGAQgAygLMisuaHlwZXJzcGFjZS5JbnNlcnRUZXh0UmVxdWVzdC5NZXRhZGF0YUVudHJ5UghtZXRhZGF0YRI7CgpkdXJhYmlsaXR5GAUgASgOMhsuaHlwZXJzcGFjZS5EdXJhYmlsaXR5TGV2ZWxSCmR1cmFiaWxpdHkaOwoNTWV0YWRhdGFFbnRyeRIQCgNrZXkYASABKAlSA2tleRIUCgV2YWx1ZRgCIAEoCVIFdmFsdWU6AjgB');
@$core.Deprecated('Use vectorizeRequestDescriptor instead')
const VectorizeRequest$json = const {
  '1': 'VectorizeRequest',
  '2': const [
    const {'1': 'text', '3': 1, '4': 1, '5': 9, '10': 'text'},
    const {'1': 'metric', '3': 2, '4': 1, '5': 9, '10': 'metric'},
  ],
};

/// Descriptor for `VectorizeRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List vectorizeRequestDescriptor = $convert.base64Decode('ChBWZWN0b3JpemVSZXF1ZXN0EhIKBHRleHQYASABKAlSBHRleHQSFgoGbWV0cmljGAIgASgJUgZtZXRyaWM=');
@$core.Deprecated('Use vectorizeResponseDescriptor instead')
const VectorizeResponse$json = const {
  '1': 'VectorizeResponse',
  '2': const [
    const {'1': 'vector', '3': 1, '4': 3, '5': 1, '10': 'vector'},
  ],
};

/// Descriptor for `VectorizeResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List vectorizeResponseDescriptor = $convert.base64Decode('ChFWZWN0b3JpemVSZXNwb25zZRIWCgZ2ZWN0b3IYASADKAFSBnZlY3Rvcg==');
@$core.Deprecated('Use searchTextRequestDescriptor instead')
const SearchTextRequest$json = const {
  '1': 'SearchTextRequest',
  '2': const [
    const {'1': 'collection', '3': 1, '4': 1, '5': 9, '10': 'collection'},
    const {'1': 'text', '3': 2, '4': 1, '5': 9, '10': 'text'},
    const {'1': 'top_k', '3': 3, '4': 1, '5': 13, '10': 'topK'},
    const {'1': 'filter', '3': 4, '4': 3, '5': 11, '6': '.hyperspace.SearchTextRequest.FilterEntry', '10': 'filter'},
    const {'1': 'filters', '3': 5, '4': 3, '5': 11, '6': '.hyperspace.Filter', '10': 'filters'},
    const {'1': 'bm25_options', '3': 6, '4': 1, '5': 11, '6': '.hyperspace.Bm25Options', '9': 0, '10': 'bm25Options', '17': true},
    const {'1': 'hybrid_alpha', '3': 7, '4': 1, '5': 2, '9': 1, '10': 'hybridAlpha', '17': true},
    const {'1': 'include_payload', '3': 8, '4': 1, '5': 8, '10': 'includePayload'},
    const {'1': 'component_weights', '3': 9, '4': 3, '5': 11, '6': '.hyperspace.SearchTextRequest.ComponentWeightsEntry', '10': 'componentWeights'},
  ],
  '3': const [SearchTextRequest_FilterEntry$json, SearchTextRequest_ComponentWeightsEntry$json],
  '8': const [
    const {'1': '_bm25_options'},
    const {'1': '_hybrid_alpha'},
  ],
};

@$core.Deprecated('Use searchTextRequestDescriptor instead')
const SearchTextRequest_FilterEntry$json = const {
  '1': 'FilterEntry',
  '2': const [
    const {'1': 'key', '3': 1, '4': 1, '5': 9, '10': 'key'},
    const {'1': 'value', '3': 2, '4': 1, '5': 9, '10': 'value'},
  ],
  '7': const {'7': true},
};

@$core.Deprecated('Use searchTextRequestDescriptor instead')
const SearchTextRequest_ComponentWeightsEntry$json = const {
  '1': 'ComponentWeightsEntry',
  '2': const [
    const {'1': 'key', '3': 1, '4': 1, '5': 9, '10': 'key'},
    const {'1': 'value', '3': 2, '4': 1, '5': 2, '10': 'value'},
  ],
  '7': const {'7': true},
};

/// Descriptor for `SearchTextRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List searchTextRequestDescriptor = $convert.base64Decode('ChFTZWFyY2hUZXh0UmVxdWVzdBIeCgpjb2xsZWN0aW9uGAEgASgJUgpjb2xsZWN0aW9uEhIKBHRleHQYAiABKAlSBHRleHQSEwoFdG9wX2sYAyABKA1SBHRvcEsSQQoGZmlsdGVyGAQgAygLMikuaHlwZXJzcGFjZS5TZWFyY2hUZXh0UmVxdWVzdC5GaWx0ZXJFbnRyeVIGZmlsdGVyEiwKB2ZpbHRlcnMYBSADKAsyEi5oeXBlcnNwYWNlLkZpbHRlclIHZmlsdGVycxI/CgxibTI1X29wdGlvbnMYBiABKAsyFy5oeXBlcnNwYWNlLkJtMjVPcHRpb25zSABSC2JtMjVPcHRpb25ziAEBEiYKDGh5YnJpZF9hbHBoYRgHIAEoAkgBUgtoeWJyaWRBbHBoYYgBARInCg9pbmNsdWRlX3BheWxvYWQYCCABKAhSDmluY2x1ZGVQYXlsb2FkEmAKEWNvbXBvbmVudF93ZWlnaHRzGAkgAygLMjMuaHlwZXJzcGFjZS5TZWFyY2hUZXh0UmVxdWVzdC5Db21wb25lbnRXZWlnaHRzRW50cnlSEGNvbXBvbmVudFdlaWdodHMaOQoLRmlsdGVyRW50cnkSEAoDa2V5GAEgASgJUgNrZXkSFAoFdmFsdWUYAiABKAlSBXZhbHVlOgI4ARpDChVDb21wb25lbnRXZWlnaHRzRW50cnkSEAoDa2V5GAEgASgJUgNrZXkSFAoFdmFsdWUYAiABKAJSBXZhbHVlOgI4AUIPCg1fYm0yNV9vcHRpb25zQg8KDV9oeWJyaWRfYWxwaGE=');
@$core.Deprecated('Use bm25OptionsDescriptor instead')
const Bm25Options$json = const {
  '1': 'Bm25Options',
  '2': const [
    const {'1': 'method', '3': 1, '4': 1, '5': 9, '9': 0, '10': 'method', '17': true},
    const {'1': 'k1', '3': 2, '4': 1, '5': 2, '9': 1, '10': 'k1', '17': true},
    const {'1': 'b', '3': 3, '4': 1, '5': 2, '9': 2, '10': 'b', '17': true},
    const {'1': 'delta', '3': 4, '4': 1, '5': 2, '9': 3, '10': 'delta', '17': true},
    const {'1': 'language', '3': 5, '4': 1, '5': 9, '9': 4, '10': 'language', '17': true},
    const {'1': 'ngrams', '3': 6, '4': 1, '5': 13, '9': 5, '10': 'ngrams', '17': true},
    const {'1': 'fusion_method', '3': 7, '4': 1, '5': 9, '9': 6, '10': 'fusionMethod', '17': true},
  ],
  '8': const [
    const {'1': '_method'},
    const {'1': '_k1'},
    const {'1': '_b'},
    const {'1': '_delta'},
    const {'1': '_language'},
    const {'1': '_ngrams'},
    const {'1': '_fusion_method'},
  ],
};

/// Descriptor for `Bm25Options`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List bm25OptionsDescriptor = $convert.base64Decode('CgtCbTI1T3B0aW9ucxIbCgZtZXRob2QYASABKAlIAFIGbWV0aG9kiAEBEhMKAmsxGAIgASgCSAFSAmsxiAEBEhEKAWIYAyABKAJIAlIBYogBARIZCgVkZWx0YRgEIAEoAkgDUgVkZWx0YYgBARIfCghsYW5ndWFnZRgFIAEoCUgEUghsYW5ndWFnZYgBARIbCgZuZ3JhbXMYBiABKA1IBVIGbmdyYW1ziAEBEigKDWZ1c2lvbl9tZXRob2QYByABKAlIBlIMZnVzaW9uTWV0aG9kiAEBQgkKB19tZXRob2RCBQoDX2sxQgQKAl9iQggKBl9kZWx0YUILCglfbGFuZ3VhZ2VCCQoHX25ncmFtc0IQCg5fZnVzaW9uX21ldGhvZA==');
@$core.Deprecated('Use insertResponseDescriptor instead')
const InsertResponse$json = const {
  '1': 'InsertResponse',
  '2': const [
    const {'1': 'success', '3': 1, '4': 1, '5': 8, '10': 'success'},
  ],
};

/// Descriptor for `InsertResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List insertResponseDescriptor = $convert.base64Decode('Cg5JbnNlcnRSZXNwb25zZRIYCgdzdWNjZXNzGAEgASgIUgdzdWNjZXNz');
@$core.Deprecated('Use deleteRequestDescriptor instead')
const DeleteRequest$json = const {
  '1': 'DeleteRequest',
  '2': const [
    const {'1': 'collection', '3': 1, '4': 1, '5': 9, '10': 'collection'},
    const {'1': 'id', '3': 2, '4': 1, '5': 13, '10': 'id'},
  ],
};

/// Descriptor for `DeleteRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List deleteRequestDescriptor = $convert.base64Decode('Cg1EZWxldGVSZXF1ZXN0Eh4KCmNvbGxlY3Rpb24YASABKAlSCmNvbGxlY3Rpb24SDgoCaWQYAiABKA1SAmlk');
@$core.Deprecated('Use deleteResponseDescriptor instead')
const DeleteResponse$json = const {
  '1': 'DeleteResponse',
  '2': const [
    const {'1': 'success', '3': 1, '4': 1, '5': 8, '10': 'success'},
  ],
};

/// Descriptor for `DeleteResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List deleteResponseDescriptor = $convert.base64Decode('Cg5EZWxldGVSZXNwb25zZRIYCgdzdWNjZXNzGAEgASgIUgdzdWNjZXNz');
@$core.Deprecated('Use getPointsRequestDescriptor instead')
const GetPointsRequest$json = const {
  '1': 'GetPointsRequest',
  '2': const [
    const {'1': 'collection', '3': 1, '4': 1, '5': 9, '10': 'collection'},
    const {'1': 'ids', '3': 2, '4': 3, '5': 13, '10': 'ids'},
  ],
};

/// Descriptor for `GetPointsRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getPointsRequestDescriptor = $convert.base64Decode('ChBHZXRQb2ludHNSZXF1ZXN0Eh4KCmNvbGxlY3Rpb24YASABKAlSCmNvbGxlY3Rpb24SEAoDaWRzGAIgAygNUgNpZHM=');
@$core.Deprecated('Use getPointsResponseDescriptor instead')
const GetPointsResponse$json = const {
  '1': 'GetPointsResponse',
  '2': const [
    const {'1': 'points', '3': 1, '4': 3, '5': 11, '6': '.hyperspace.VectorData', '10': 'points'},
  ],
};

/// Descriptor for `GetPointsResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getPointsResponseDescriptor = $convert.base64Decode('ChFHZXRQb2ludHNSZXNwb25zZRIuCgZwb2ludHMYASADKAsyFi5oeXBlcnNwYWNlLlZlY3RvckRhdGFSBnBvaW50cw==');
@$core.Deprecated('Use updatePayloadRequestDescriptor instead')
const UpdatePayloadRequest$json = const {
  '1': 'UpdatePayloadRequest',
  '2': const [
    const {'1': 'collection', '3': 1, '4': 1, '5': 9, '10': 'collection'},
    const {'1': 'id', '3': 2, '4': 1, '5': 13, '10': 'id'},
    const {'1': 'metadata', '3': 3, '4': 3, '5': 11, '6': '.hyperspace.UpdatePayloadRequest.MetadataEntry', '10': 'metadata'},
    const {'1': 'typed_metadata', '3': 4, '4': 3, '5': 11, '6': '.hyperspace.UpdatePayloadRequest.TypedMetadataEntry', '10': 'typedMetadata'},
  ],
  '3': const [UpdatePayloadRequest_MetadataEntry$json, UpdatePayloadRequest_TypedMetadataEntry$json],
};

@$core.Deprecated('Use updatePayloadRequestDescriptor instead')
const UpdatePayloadRequest_MetadataEntry$json = const {
  '1': 'MetadataEntry',
  '2': const [
    const {'1': 'key', '3': 1, '4': 1, '5': 9, '10': 'key'},
    const {'1': 'value', '3': 2, '4': 1, '5': 9, '10': 'value'},
  ],
  '7': const {'7': true},
};

@$core.Deprecated('Use updatePayloadRequestDescriptor instead')
const UpdatePayloadRequest_TypedMetadataEntry$json = const {
  '1': 'TypedMetadataEntry',
  '2': const [
    const {'1': 'key', '3': 1, '4': 1, '5': 9, '10': 'key'},
    const {'1': 'value', '3': 2, '4': 1, '5': 11, '6': '.hyperspace.MetadataValue', '10': 'value'},
  ],
  '7': const {'7': true},
};

/// Descriptor for `UpdatePayloadRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List updatePayloadRequestDescriptor = $convert.base64Decode('ChRVcGRhdGVQYXlsb2FkUmVxdWVzdBIeCgpjb2xsZWN0aW9uGAEgASgJUgpjb2xsZWN0aW9uEg4KAmlkGAIgASgNUgJpZBJKCghtZXRhZGF0YRgDIAMoCzIuLmh5cGVyc3BhY2UuVXBkYXRlUGF5bG9hZFJlcXVlc3QuTWV0YWRhdGFFbnRyeVIIbWV0YWRhdGESWgoOdHlwZWRfbWV0YWRhdGEYBCADKAsyMy5oeXBlcnNwYWNlLlVwZGF0ZVBheWxvYWRSZXF1ZXN0LlR5cGVkTWV0YWRhdGFFbnRyeVINdHlwZWRNZXRhZGF0YRo7Cg1NZXRhZGF0YUVudHJ5EhAKA2tleRgBIAEoCVIDa2V5EhQKBXZhbHVlGAIgASgJUgV2YWx1ZToCOAEaWwoSVHlwZWRNZXRhZGF0YUVudHJ5EhAKA2tleRgBIAEoCVIDa2V5Ei8KBXZhbHVlGAIgASgLMhkuaHlwZXJzcGFjZS5NZXRhZGF0YVZhbHVlUgV2YWx1ZToCOAE=');
@$core.Deprecated('Use scrollRequestDescriptor instead')
const ScrollRequest$json = const {
  '1': 'ScrollRequest',
  '2': const [
    const {'1': 'collection', '3': 1, '4': 1, '5': 9, '10': 'collection'},
    const {'1': 'limit', '3': 2, '4': 1, '5': 13, '10': 'limit'},
    const {'1': 'offset', '3': 3, '4': 1, '5': 13, '10': 'offset'},
    const {'1': 'filters', '3': 4, '4': 3, '5': 11, '6': '.hyperspace.Filter', '10': 'filters'},
  ],
};

/// Descriptor for `ScrollRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List scrollRequestDescriptor = $convert.base64Decode('Cg1TY3JvbGxSZXF1ZXN0Eh4KCmNvbGxlY3Rpb24YASABKAlSCmNvbGxlY3Rpb24SFAoFbGltaXQYAiABKA1SBWxpbWl0EhYKBm9mZnNldBgDIAEoDVIGb2Zmc2V0EiwKB2ZpbHRlcnMYBCADKAsyEi5oeXBlcnNwYWNlLkZpbHRlclIHZmlsdGVycw==');
@$core.Deprecated('Use scrollResponseDescriptor instead')
const ScrollResponse$json = const {
  '1': 'ScrollResponse',
  '2': const [
    const {'1': 'points', '3': 1, '4': 3, '5': 11, '6': '.hyperspace.VectorData', '10': 'points'},
  ],
};

/// Descriptor for `ScrollResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List scrollResponseDescriptor = $convert.base64Decode('Cg5TY3JvbGxSZXNwb25zZRIuCgZwb2ludHMYASADKAsyFi5oeXBlcnNwYWNlLlZlY3RvckRhdGFSBnBvaW50cw==');
@$core.Deprecated('Use countRequestDescriptor instead')
const CountRequest$json = const {
  '1': 'CountRequest',
  '2': const [
    const {'1': 'collection', '3': 1, '4': 1, '5': 9, '10': 'collection'},
    const {'1': 'filters', '3': 2, '4': 3, '5': 11, '6': '.hyperspace.Filter', '10': 'filters'},
  ],
};

/// Descriptor for `CountRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List countRequestDescriptor = $convert.base64Decode('CgxDb3VudFJlcXVlc3QSHgoKY29sbGVjdGlvbhgBIAEoCVIKY29sbGVjdGlvbhIsCgdmaWx0ZXJzGAIgAygLMhIuaHlwZXJzcGFjZS5GaWx0ZXJSB2ZpbHRlcnM=');
@$core.Deprecated('Use countResponseDescriptor instead')
const CountResponse$json = const {
  '1': 'CountResponse',
  '2': const [
    const {'1': 'count', '3': 1, '4': 1, '5': 4, '10': 'count'},
  ],
};

/// Descriptor for `CountResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List countResponseDescriptor = $convert.base64Decode('Cg1Db3VudFJlc3BvbnNlEhQKBWNvdW50GAEgASgEUgVjb3VudA==');
@$core.Deprecated('Use healthCheckResponseDescriptor instead')
const HealthCheckResponse$json = const {
  '1': 'HealthCheckResponse',
  '2': const [
    const {'1': 'status', '3': 1, '4': 1, '5': 9, '10': 'status'},
  ],
};

/// Descriptor for `HealthCheckResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List healthCheckResponseDescriptor = $convert.base64Decode('ChNIZWFsdGhDaGVja1Jlc3BvbnNlEhYKBnN0YXR1cxgBIAEoCVIGc3RhdHVz');
@$core.Deprecated('Use searchRequestDescriptor instead')
const SearchRequest$json = const {
  '1': 'SearchRequest',
  '2': const [
    const {'1': 'collection', '3': 1, '4': 1, '5': 9, '10': 'collection'},
    const {'1': 'vector', '3': 2, '4': 3, '5': 1, '10': 'vector'},
    const {'1': 'top_k', '3': 3, '4': 1, '5': 13, '10': 'topK'},
    const {'1': 'filter', '3': 4, '4': 3, '5': 11, '6': '.hyperspace.SearchRequest.FilterEntry', '10': 'filter'},
    const {'1': 'filters', '3': 5, '4': 3, '5': 11, '6': '.hyperspace.Filter', '10': 'filters'},
    const {'1': 'hybrid_query', '3': 6, '4': 1, '5': 9, '9': 0, '10': 'hybridQuery', '17': true},
    const {'1': 'hybrid_alpha', '3': 7, '4': 1, '5': 2, '9': 1, '10': 'hybridAlpha', '17': true},
    const {'1': 'use_wasserstein', '3': 8, '4': 1, '5': 8, '10': 'useWasserstein'},
    const {'1': 'bm25_options', '3': 9, '4': 1, '5': 11, '6': '.hyperspace.Bm25Options', '9': 2, '10': 'bm25Options', '17': true},
    const {'1': 'mrl_dimension', '3': 10, '4': 1, '5': 13, '9': 3, '10': 'mrlDimension', '17': true},
    const {'1': 'include_payload', '3': 11, '4': 1, '5': 8, '10': 'includePayload'},
    const {'1': 'component_weights', '3': 12, '4': 3, '5': 11, '6': '.hyperspace.SearchRequest.ComponentWeightsEntry', '10': 'componentWeights'},
  ],
  '3': const [SearchRequest_FilterEntry$json, SearchRequest_ComponentWeightsEntry$json],
  '8': const [
    const {'1': '_hybrid_query'},
    const {'1': '_hybrid_alpha'},
    const {'1': '_bm25_options'},
    const {'1': '_mrl_dimension'},
  ],
};

@$core.Deprecated('Use searchRequestDescriptor instead')
const SearchRequest_FilterEntry$json = const {
  '1': 'FilterEntry',
  '2': const [
    const {'1': 'key', '3': 1, '4': 1, '5': 9, '10': 'key'},
    const {'1': 'value', '3': 2, '4': 1, '5': 9, '10': 'value'},
  ],
  '7': const {'7': true},
};

@$core.Deprecated('Use searchRequestDescriptor instead')
const SearchRequest_ComponentWeightsEntry$json = const {
  '1': 'ComponentWeightsEntry',
  '2': const [
    const {'1': 'key', '3': 1, '4': 1, '5': 9, '10': 'key'},
    const {'1': 'value', '3': 2, '4': 1, '5': 2, '10': 'value'},
  ],
  '7': const {'7': true},
};

/// Descriptor for `SearchRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List searchRequestDescriptor = $convert.base64Decode('Cg1TZWFyY2hSZXF1ZXN0Eh4KCmNvbGxlY3Rpb24YASABKAlSCmNvbGxlY3Rpb24SFgoGdmVjdG9yGAIgAygBUgZ2ZWN0b3ISEwoFdG9wX2sYAyABKA1SBHRvcEsSPQoGZmlsdGVyGAQgAygLMiUuaHlwZXJzcGFjZS5TZWFyY2hSZXF1ZXN0LkZpbHRlckVudHJ5UgZmaWx0ZXISLAoHZmlsdGVycxgFIAMoCzISLmh5cGVyc3BhY2UuRmlsdGVyUgdmaWx0ZXJzEiYKDGh5YnJpZF9xdWVyeRgGIAEoCUgAUgtoeWJyaWRRdWVyeYgBARImCgxoeWJyaWRfYWxwaGEYByABKAJIAVILaHlicmlkQWxwaGGIAQESJwoPdXNlX3dhc3NlcnN0ZWluGAggASgIUg51c2VXYXNzZXJzdGVpbhI/CgxibTI1X29wdGlvbnMYCSABKAsyFy5oeXBlcnNwYWNlLkJtMjVPcHRpb25zSAJSC2JtMjVPcHRpb25ziAEBEigKDW1ybF9kaW1lbnNpb24YCiABKA1IA1IMbXJsRGltZW5zaW9uiAEBEicKD2luY2x1ZGVfcGF5bG9hZBgLIAEoCFIOaW5jbHVkZVBheWxvYWQSXAoRY29tcG9uZW50X3dlaWdodHMYDCADKAsyLy5oeXBlcnNwYWNlLlNlYXJjaFJlcXVlc3QuQ29tcG9uZW50V2VpZ2h0c0VudHJ5UhBjb21wb25lbnRXZWlnaHRzGjkKC0ZpbHRlckVudHJ5EhAKA2tleRgBIAEoCVIDa2V5EhQKBXZhbHVlGAIgASgJUgV2YWx1ZToCOAEaQwoVQ29tcG9uZW50V2VpZ2h0c0VudHJ5EhAKA2tleRgBIAEoCVIDa2V5EhQKBXZhbHVlGAIgASgCUgV2YWx1ZToCOAFCDwoNX2h5YnJpZF9xdWVyeUIPCg1faHlicmlkX2FscGhhQg8KDV9ibTI1X29wdGlvbnNCEAoOX21ybF9kaW1lbnNpb24=');
@$core.Deprecated('Use filterDescriptor instead')
const Filter$json = const {
  '1': 'Filter',
  '2': const [
    const {'1': 'match', '3': 1, '4': 1, '5': 11, '6': '.hyperspace.Match', '9': 0, '10': 'match'},
    const {'1': 'range', '3': 2, '4': 1, '5': 11, '6': '.hyperspace.Range', '9': 0, '10': 'range'},
    const {'1': 'in_cone', '3': 3, '4': 1, '5': 11, '6': '.hyperspace.InCone', '9': 0, '10': 'inCone'},
    const {'1': 'in_box', '3': 4, '4': 1, '5': 11, '6': '.hyperspace.InBox', '9': 0, '10': 'inBox'},
    const {'1': 'in_ball', '3': 5, '4': 1, '5': 11, '6': '.hyperspace.InBall', '9': 0, '10': 'inBall'},
    const {'1': 'and_op', '3': 6, '4': 1, '5': 11, '6': '.hyperspace.FilterAnd', '9': 0, '10': 'andOp'},
    const {'1': 'or_op', '3': 7, '4': 1, '5': 11, '6': '.hyperspace.FilterOr', '9': 0, '10': 'orOp'},
    const {'1': 'not_op', '3': 8, '4': 1, '5': 11, '6': '.hyperspace.FilterNot', '9': 0, '10': 'notOp'},
  ],
  '8': const [
    const {'1': 'condition'},
  ],
};

/// Descriptor for `Filter`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List filterDescriptor = $convert.base64Decode('CgZGaWx0ZXISKQoFbWF0Y2gYASABKAsyES5oeXBlcnNwYWNlLk1hdGNoSABSBW1hdGNoEikKBXJhbmdlGAIgASgLMhEuaHlwZXJzcGFjZS5SYW5nZUgAUgVyYW5nZRItCgdpbl9jb25lGAMgASgLMhIuaHlwZXJzcGFjZS5JbkNvbmVIAFIGaW5Db25lEioKBmluX2JveBgEIAEoCzIRLmh5cGVyc3BhY2UuSW5Cb3hIAFIFaW5Cb3gSLQoHaW5fYmFsbBgFIAEoCzISLmh5cGVyc3BhY2UuSW5CYWxsSABSBmluQmFsbBIuCgZhbmRfb3AYBiABKAsyFS5oeXBlcnNwYWNlLkZpbHRlckFuZEgAUgVhbmRPcBIrCgVvcl9vcBgHIAEoCzIULmh5cGVyc3BhY2UuRmlsdGVyT3JIAFIEb3JPcBIuCgZub3Rfb3AYCCABKAsyFS5oeXBlcnNwYWNlLkZpbHRlck5vdEgAUgVub3RPcEILCgljb25kaXRpb24=');
@$core.Deprecated('Use filterAndDescriptor instead')
const FilterAnd$json = const {
  '1': 'FilterAnd',
  '2': const [
    const {'1': 'conditions', '3': 1, '4': 3, '5': 11, '6': '.hyperspace.Filter', '10': 'conditions'},
  ],
};

/// Descriptor for `FilterAnd`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List filterAndDescriptor = $convert.base64Decode('CglGaWx0ZXJBbmQSMgoKY29uZGl0aW9ucxgBIAMoCzISLmh5cGVyc3BhY2UuRmlsdGVyUgpjb25kaXRpb25z');
@$core.Deprecated('Use filterOrDescriptor instead')
const FilterOr$json = const {
  '1': 'FilterOr',
  '2': const [
    const {'1': 'conditions', '3': 1, '4': 3, '5': 11, '6': '.hyperspace.Filter', '10': 'conditions'},
  ],
};

/// Descriptor for `FilterOr`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List filterOrDescriptor = $convert.base64Decode('CghGaWx0ZXJPchIyCgpjb25kaXRpb25zGAEgAygLMhIuaHlwZXJzcGFjZS5GaWx0ZXJSCmNvbmRpdGlvbnM=');
@$core.Deprecated('Use filterNotDescriptor instead')
const FilterNot$json = const {
  '1': 'FilterNot',
  '2': const [
    const {'1': 'condition', '3': 1, '4': 1, '5': 11, '6': '.hyperspace.Filter', '10': 'condition'},
  ],
};

/// Descriptor for `FilterNot`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List filterNotDescriptor = $convert.base64Decode('CglGaWx0ZXJOb3QSMAoJY29uZGl0aW9uGAEgASgLMhIuaHlwZXJzcGFjZS5GaWx0ZXJSCWNvbmRpdGlvbg==');
@$core.Deprecated('Use matchDescriptor instead')
const Match$json = const {
  '1': 'Match',
  '2': const [
    const {'1': 'key', '3': 1, '4': 1, '5': 9, '10': 'key'},
    const {'1': 'value', '3': 2, '4': 1, '5': 9, '10': 'value'},
  ],
};

/// Descriptor for `Match`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List matchDescriptor = $convert.base64Decode('CgVNYXRjaBIQCgNrZXkYASABKAlSA2tleRIUCgV2YWx1ZRgCIAEoCVIFdmFsdWU=');
@$core.Deprecated('Use rangeDescriptor instead')
const Range$json = const {
  '1': 'Range',
  '2': const [
    const {'1': 'key', '3': 1, '4': 1, '5': 9, '10': 'key'},
    const {'1': 'gte', '3': 2, '4': 1, '5': 3, '9': 0, '10': 'gte', '17': true},
    const {'1': 'lte', '3': 3, '4': 1, '5': 3, '9': 1, '10': 'lte', '17': true},
    const {'1': 'gte_f64', '3': 4, '4': 1, '5': 1, '9': 2, '10': 'gteF64', '17': true},
    const {'1': 'lte_f64', '3': 5, '4': 1, '5': 1, '9': 3, '10': 'lteF64', '17': true},
  ],
  '8': const [
    const {'1': '_gte'},
    const {'1': '_lte'},
    const {'1': '_gte_f64'},
    const {'1': '_lte_f64'},
  ],
};

/// Descriptor for `Range`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List rangeDescriptor = $convert.base64Decode('CgVSYW5nZRIQCgNrZXkYASABKAlSA2tleRIVCgNndGUYAiABKANIAFIDZ3RliAEBEhUKA2x0ZRgDIAEoA0gBUgNsdGWIAQESHAoHZ3RlX2Y2NBgEIAEoAUgCUgZndGVGNjSIAQESHAoHbHRlX2Y2NBgFIAEoAUgDUgZsdGVGNjSIAQFCBgoEX2d0ZUIGCgRfbHRlQgoKCF9ndGVfZjY0QgoKCF9sdGVfZjY0');
@$core.Deprecated('Use inConeDescriptor instead')
const InCone$json = const {
  '1': 'InCone',
  '2': const [
    const {'1': 'axes', '3': 1, '4': 3, '5': 1, '10': 'axes'},
    const {'1': 'apertures', '3': 2, '4': 3, '5': 1, '10': 'apertures'},
    const {'1': 'cen', '3': 3, '4': 1, '5': 1, '10': 'cen'},
  ],
};

/// Descriptor for `InCone`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List inConeDescriptor = $convert.base64Decode('CgZJbkNvbmUSEgoEYXhlcxgBIAMoAVIEYXhlcxIcCglhcGVydHVyZXMYAiADKAFSCWFwZXJ0dXJlcxIQCgNjZW4YAyABKAFSA2Nlbg==');
@$core.Deprecated('Use inBoxDescriptor instead')
const InBox$json = const {
  '1': 'InBox',
  '2': const [
    const {'1': 'min_bounds', '3': 1, '4': 3, '5': 1, '10': 'minBounds'},
    const {'1': 'max_bounds', '3': 2, '4': 3, '5': 1, '10': 'maxBounds'},
  ],
};

/// Descriptor for `InBox`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List inBoxDescriptor = $convert.base64Decode('CgVJbkJveBIdCgptaW5fYm91bmRzGAEgAygBUgltaW5Cb3VuZHMSHQoKbWF4X2JvdW5kcxgCIAMoAVIJbWF4Qm91bmRz');
@$core.Deprecated('Use inBallDescriptor instead')
const InBall$json = const {
  '1': 'InBall',
  '2': const [
    const {'1': 'center', '3': 1, '4': 3, '5': 1, '10': 'center'},
    const {'1': 'radius', '3': 2, '4': 1, '5': 1, '10': 'radius'},
  ],
};

/// Descriptor for `InBall`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List inBallDescriptor = $convert.base64Decode('CgZJbkJhbGwSFgoGY2VudGVyGAEgAygBUgZjZW50ZXISFgoGcmFkaXVzGAIgASgBUgZyYWRpdXM=');
@$core.Deprecated('Use searchResponseDescriptor instead')
const SearchResponse$json = const {
  '1': 'SearchResponse',
  '2': const [
    const {'1': 'results', '3': 1, '4': 3, '5': 11, '6': '.hyperspace.SearchResult', '10': 'results'},
  ],
};

/// Descriptor for `SearchResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List searchResponseDescriptor = $convert.base64Decode('Cg5TZWFyY2hSZXNwb25zZRIyCgdyZXN1bHRzGAEgAygLMhguaHlwZXJzcGFjZS5TZWFyY2hSZXN1bHRSB3Jlc3VsdHM=');
@$core.Deprecated('Use batchSearchRequestDescriptor instead')
const BatchSearchRequest$json = const {
  '1': 'BatchSearchRequest',
  '2': const [
    const {'1': 'searches', '3': 1, '4': 3, '5': 11, '6': '.hyperspace.SearchRequest', '10': 'searches'},
  ],
};

/// Descriptor for `BatchSearchRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List batchSearchRequestDescriptor = $convert.base64Decode('ChJCYXRjaFNlYXJjaFJlcXVlc3QSNQoIc2VhcmNoZXMYASADKAsyGS5oeXBlcnNwYWNlLlNlYXJjaFJlcXVlc3RSCHNlYXJjaGVz');
@$core.Deprecated('Use batchSearchResponseDescriptor instead')
const BatchSearchResponse$json = const {
  '1': 'BatchSearchResponse',
  '2': const [
    const {'1': 'responses', '3': 1, '4': 3, '5': 11, '6': '.hyperspace.SearchResponse', '10': 'responses'},
  ],
};

/// Descriptor for `BatchSearchResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List batchSearchResponseDescriptor = $convert.base64Decode('ChNCYXRjaFNlYXJjaFJlc3BvbnNlEjgKCXJlc3BvbnNlcxgBIAMoCzIaLmh5cGVyc3BhY2UuU2VhcmNoUmVzcG9uc2VSCXJlc3BvbnNlcw==');
@$core.Deprecated('Use searchMultiCollectionRequestDescriptor instead')
const SearchMultiCollectionRequest$json = const {
  '1': 'SearchMultiCollectionRequest',
  '2': const [
    const {'1': 'collections', '3': 1, '4': 3, '5': 9, '10': 'collections'},
    const {'1': 'vector', '3': 2, '4': 3, '5': 1, '10': 'vector'},
    const {'1': 'top_k', '3': 3, '4': 1, '5': 13, '10': 'topK'},
  ],
};

/// Descriptor for `SearchMultiCollectionRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List searchMultiCollectionRequestDescriptor = $convert.base64Decode('ChxTZWFyY2hNdWx0aUNvbGxlY3Rpb25SZXF1ZXN0EiAKC2NvbGxlY3Rpb25zGAEgAygJUgtjb2xsZWN0aW9ucxIWCgZ2ZWN0b3IYAiADKAFSBnZlY3RvchITCgV0b3BfaxgDIAEoDVIEdG9wSw==');
@$core.Deprecated('Use searchMultiCollectionResponseDescriptor instead')
const SearchMultiCollectionResponse$json = const {
  '1': 'SearchMultiCollectionResponse',
  '2': const [
    const {'1': 'responses', '3': 1, '4': 3, '5': 11, '6': '.hyperspace.SearchMultiCollectionResponse.ResponsesEntry', '10': 'responses'},
  ],
  '3': const [SearchMultiCollectionResponse_ResponsesEntry$json],
};

@$core.Deprecated('Use searchMultiCollectionResponseDescriptor instead')
const SearchMultiCollectionResponse_ResponsesEntry$json = const {
  '1': 'ResponsesEntry',
  '2': const [
    const {'1': 'key', '3': 1, '4': 1, '5': 9, '10': 'key'},
    const {'1': 'value', '3': 2, '4': 1, '5': 11, '6': '.hyperspace.SearchResponse', '10': 'value'},
  ],
  '7': const {'7': true},
};

/// Descriptor for `SearchMultiCollectionResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List searchMultiCollectionResponseDescriptor = $convert.base64Decode('Ch1TZWFyY2hNdWx0aUNvbGxlY3Rpb25SZXNwb25zZRJWCglyZXNwb25zZXMYASADKAsyOC5oeXBlcnNwYWNlLlNlYXJjaE11bHRpQ29sbGVjdGlvblJlc3BvbnNlLlJlc3BvbnNlc0VudHJ5UglyZXNwb25zZXMaWAoOUmVzcG9uc2VzRW50cnkSEAoDa2V5GAEgASgJUgNrZXkSMAoFdmFsdWUYAiABKAsyGi5oeXBlcnNwYWNlLlNlYXJjaFJlc3BvbnNlUgV2YWx1ZToCOAE=');
@$core.Deprecated('Use searchResultDescriptor instead')
const SearchResult$json = const {
  '1': 'SearchResult',
  '2': const [
    const {'1': 'id', '3': 1, '4': 1, '5': 13, '10': 'id'},
    const {'1': 'distance', '3': 2, '4': 1, '5': 1, '10': 'distance'},
    const {'1': 'metadata', '3': 3, '4': 3, '5': 11, '6': '.hyperspace.SearchResult.MetadataEntry', '10': 'metadata'},
    const {'1': 'typed_metadata', '3': 4, '4': 3, '5': 11, '6': '.hyperspace.SearchResult.TypedMetadataEntry', '10': 'typedMetadata'},
    const {'1': 'payload', '3': 5, '4': 1, '5': 12, '9': 0, '10': 'payload', '17': true},
  ],
  '3': const [SearchResult_MetadataEntry$json, SearchResult_TypedMetadataEntry$json],
  '8': const [
    const {'1': '_payload'},
  ],
};

@$core.Deprecated('Use searchResultDescriptor instead')
const SearchResult_MetadataEntry$json = const {
  '1': 'MetadataEntry',
  '2': const [
    const {'1': 'key', '3': 1, '4': 1, '5': 9, '10': 'key'},
    const {'1': 'value', '3': 2, '4': 1, '5': 9, '10': 'value'},
  ],
  '7': const {'7': true},
};

@$core.Deprecated('Use searchResultDescriptor instead')
const SearchResult_TypedMetadataEntry$json = const {
  '1': 'TypedMetadataEntry',
  '2': const [
    const {'1': 'key', '3': 1, '4': 1, '5': 9, '10': 'key'},
    const {'1': 'value', '3': 2, '4': 1, '5': 11, '6': '.hyperspace.MetadataValue', '10': 'value'},
  ],
  '7': const {'7': true},
};

/// Descriptor for `SearchResult`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List searchResultDescriptor = $convert.base64Decode('CgxTZWFyY2hSZXN1bHQSDgoCaWQYASABKA1SAmlkEhoKCGRpc3RhbmNlGAIgASgBUghkaXN0YW5jZRJCCghtZXRhZGF0YRgDIAMoCzImLmh5cGVyc3BhY2UuU2VhcmNoUmVzdWx0Lk1ldGFkYXRhRW50cnlSCG1ldGFkYXRhElIKDnR5cGVkX21ldGFkYXRhGAQgAygLMisuaHlwZXJzcGFjZS5TZWFyY2hSZXN1bHQuVHlwZWRNZXRhZGF0YUVudHJ5Ug10eXBlZE1ldGFkYXRhEh0KB3BheWxvYWQYBSABKAxIAFIHcGF5bG9hZIgBARo7Cg1NZXRhZGF0YUVudHJ5EhAKA2tleRgBIAEoCVIDa2V5EhQKBXZhbHVlGAIgASgJUgV2YWx1ZToCOAEaWwoSVHlwZWRNZXRhZGF0YUVudHJ5EhAKA2tleRgBIAEoCVIDa2V5Ei8KBXZhbHVlGAIgASgLMhkuaHlwZXJzcGFjZS5NZXRhZGF0YVZhbHVlUgV2YWx1ZToCOAFCCgoIX3BheWxvYWQ=');
@$core.Deprecated('Use getNodeRequestDescriptor instead')
const GetNodeRequest$json = const {
  '1': 'GetNodeRequest',
  '2': const [
    const {'1': 'collection', '3': 1, '4': 1, '5': 9, '10': 'collection'},
    const {'1': 'id', '3': 2, '4': 1, '5': 13, '10': 'id'},
    const {'1': 'layer', '3': 3, '4': 1, '5': 13, '10': 'layer'},
  ],
};

/// Descriptor for `GetNodeRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getNodeRequestDescriptor = $convert.base64Decode('Cg5HZXROb2RlUmVxdWVzdBIeCgpjb2xsZWN0aW9uGAEgASgJUgpjb2xsZWN0aW9uEg4KAmlkGAIgASgNUgJpZBIUCgVsYXllchgDIAEoDVIFbGF5ZXI=');
@$core.Deprecated('Use graphNodeDescriptor instead')
const GraphNode$json = const {
  '1': 'GraphNode',
  '2': const [
    const {'1': 'id', '3': 1, '4': 1, '5': 13, '10': 'id'},
    const {'1': 'layer', '3': 2, '4': 1, '5': 13, '10': 'layer'},
    const {'1': 'neighbors', '3': 3, '4': 3, '5': 13, '10': 'neighbors'},
    const {'1': 'metadata', '3': 4, '4': 3, '5': 11, '6': '.hyperspace.GraphNode.MetadataEntry', '10': 'metadata'},
    const {'1': 'typed_metadata', '3': 5, '4': 3, '5': 11, '6': '.hyperspace.GraphNode.TypedMetadataEntry', '10': 'typedMetadata'},
  ],
  '3': const [GraphNode_MetadataEntry$json, GraphNode_TypedMetadataEntry$json],
};

@$core.Deprecated('Use graphNodeDescriptor instead')
const GraphNode_MetadataEntry$json = const {
  '1': 'MetadataEntry',
  '2': const [
    const {'1': 'key', '3': 1, '4': 1, '5': 9, '10': 'key'},
    const {'1': 'value', '3': 2, '4': 1, '5': 9, '10': 'value'},
  ],
  '7': const {'7': true},
};

@$core.Deprecated('Use graphNodeDescriptor instead')
const GraphNode_TypedMetadataEntry$json = const {
  '1': 'TypedMetadataEntry',
  '2': const [
    const {'1': 'key', '3': 1, '4': 1, '5': 9, '10': 'key'},
    const {'1': 'value', '3': 2, '4': 1, '5': 11, '6': '.hyperspace.MetadataValue', '10': 'value'},
  ],
  '7': const {'7': true},
};

/// Descriptor for `GraphNode`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List graphNodeDescriptor = $convert.base64Decode('CglHcmFwaE5vZGUSDgoCaWQYASABKA1SAmlkEhQKBWxheWVyGAIgASgNUgVsYXllchIcCgluZWlnaGJvcnMYAyADKA1SCW5laWdoYm9ycxI/CghtZXRhZGF0YRgEIAMoCzIjLmh5cGVyc3BhY2UuR3JhcGhOb2RlLk1ldGFkYXRhRW50cnlSCG1ldGFkYXRhEk8KDnR5cGVkX21ldGFkYXRhGAUgAygLMiguaHlwZXJzcGFjZS5HcmFwaE5vZGUuVHlwZWRNZXRhZGF0YUVudHJ5Ug10eXBlZE1ldGFkYXRhGjsKDU1ldGFkYXRhRW50cnkSEAoDa2V5GAEgASgJUgNrZXkSFAoFdmFsdWUYAiABKAlSBXZhbHVlOgI4ARpbChJUeXBlZE1ldGFkYXRhRW50cnkSEAoDa2V5GAEgASgJUgNrZXkSLwoFdmFsdWUYAiABKAsyGS5oeXBlcnNwYWNlLk1ldGFkYXRhVmFsdWVSBXZhbHVlOgI4AQ==');
@$core.Deprecated('Use getNeighborsRequestDescriptor instead')
const GetNeighborsRequest$json = const {
  '1': 'GetNeighborsRequest',
  '2': const [
    const {'1': 'collection', '3': 1, '4': 1, '5': 9, '10': 'collection'},
    const {'1': 'id', '3': 2, '4': 1, '5': 13, '10': 'id'},
    const {'1': 'layer', '3': 3, '4': 1, '5': 13, '10': 'layer'},
    const {'1': 'limit', '3': 4, '4': 1, '5': 13, '10': 'limit'},
    const {'1': 'offset', '3': 5, '4': 1, '5': 13, '10': 'offset'},
  ],
};

/// Descriptor for `GetNeighborsRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getNeighborsRequestDescriptor = $convert.base64Decode('ChNHZXROZWlnaGJvcnNSZXF1ZXN0Eh4KCmNvbGxlY3Rpb24YASABKAlSCmNvbGxlY3Rpb24SDgoCaWQYAiABKA1SAmlkEhQKBWxheWVyGAMgASgNUgVsYXllchIUCgVsaW1pdBgEIAEoDVIFbGltaXQSFgoGb2Zmc2V0GAUgASgNUgZvZmZzZXQ=');
@$core.Deprecated('Use getNeighborsResponseDescriptor instead')
const GetNeighborsResponse$json = const {
  '1': 'GetNeighborsResponse',
  '2': const [
    const {'1': 'neighbors', '3': 1, '4': 3, '5': 11, '6': '.hyperspace.GraphNode', '10': 'neighbors'},
    const {'1': 'edge_weights', '3': 2, '4': 3, '5': 1, '10': 'edgeWeights'},
  ],
};

/// Descriptor for `GetNeighborsResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getNeighborsResponseDescriptor = $convert.base64Decode('ChRHZXROZWlnaGJvcnNSZXNwb25zZRIzCgluZWlnaGJvcnMYASADKAsyFS5oeXBlcnNwYWNlLkdyYXBoTm9kZVIJbmVpZ2hib3JzEiEKDGVkZ2Vfd2VpZ2h0cxgCIAMoAVILZWRnZVdlaWdodHM=');
@$core.Deprecated('Use traverseRequestDescriptor instead')
const TraverseRequest$json = const {
  '1': 'TraverseRequest',
  '2': const [
    const {'1': 'collection', '3': 1, '4': 1, '5': 9, '10': 'collection'},
    const {'1': 'start_id', '3': 2, '4': 1, '5': 13, '10': 'startId'},
    const {'1': 'max_depth', '3': 3, '4': 1, '5': 13, '10': 'maxDepth'},
    const {'1': 'max_nodes', '3': 4, '4': 1, '5': 13, '10': 'maxNodes'},
    const {'1': 'layer', '3': 5, '4': 1, '5': 13, '10': 'layer'},
    const {'1': 'filter', '3': 6, '4': 3, '5': 11, '6': '.hyperspace.TraverseRequest.FilterEntry', '10': 'filter'},
    const {'1': 'filters', '3': 7, '4': 3, '5': 11, '6': '.hyperspace.Filter', '10': 'filters'},
  ],
  '3': const [TraverseRequest_FilterEntry$json],
};

@$core.Deprecated('Use traverseRequestDescriptor instead')
const TraverseRequest_FilterEntry$json = const {
  '1': 'FilterEntry',
  '2': const [
    const {'1': 'key', '3': 1, '4': 1, '5': 9, '10': 'key'},
    const {'1': 'value', '3': 2, '4': 1, '5': 9, '10': 'value'},
  ],
  '7': const {'7': true},
};

/// Descriptor for `TraverseRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List traverseRequestDescriptor = $convert.base64Decode('Cg9UcmF2ZXJzZVJlcXVlc3QSHgoKY29sbGVjdGlvbhgBIAEoCVIKY29sbGVjdGlvbhIZCghzdGFydF9pZBgCIAEoDVIHc3RhcnRJZBIbCgltYXhfZGVwdGgYAyABKA1SCG1heERlcHRoEhsKCW1heF9ub2RlcxgEIAEoDVIIbWF4Tm9kZXMSFAoFbGF5ZXIYBSABKA1SBWxheWVyEj8KBmZpbHRlchgGIAMoCzInLmh5cGVyc3BhY2UuVHJhdmVyc2VSZXF1ZXN0LkZpbHRlckVudHJ5UgZmaWx0ZXISLAoHZmlsdGVycxgHIAMoCzISLmh5cGVyc3BhY2UuRmlsdGVyUgdmaWx0ZXJzGjkKC0ZpbHRlckVudHJ5EhAKA2tleRgBIAEoCVIDa2V5EhQKBXZhbHVlGAIgASgJUgV2YWx1ZToCOAE=');
@$core.Deprecated('Use traverseResponseDescriptor instead')
const TraverseResponse$json = const {
  '1': 'TraverseResponse',
  '2': const [
    const {'1': 'nodes', '3': 1, '4': 3, '5': 11, '6': '.hyperspace.GraphNode', '10': 'nodes'},
  ],
};

/// Descriptor for `TraverseResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List traverseResponseDescriptor = $convert.base64Decode('ChBUcmF2ZXJzZVJlc3BvbnNlEisKBW5vZGVzGAEgAygLMhUuaHlwZXJzcGFjZS5HcmFwaE5vZGVSBW5vZGVz');
@$core.Deprecated('Use findSemanticClustersRequestDescriptor instead')
const FindSemanticClustersRequest$json = const {
  '1': 'FindSemanticClustersRequest',
  '2': const [
    const {'1': 'collection', '3': 1, '4': 1, '5': 9, '10': 'collection'},
    const {'1': 'layer', '3': 2, '4': 1, '5': 13, '10': 'layer'},
    const {'1': 'min_cluster_size', '3': 3, '4': 1, '5': 13, '10': 'minClusterSize'},
    const {'1': 'max_clusters', '3': 4, '4': 1, '5': 13, '10': 'maxClusters'},
    const {'1': 'max_nodes', '3': 5, '4': 1, '5': 13, '10': 'maxNodes'},
  ],
};

/// Descriptor for `FindSemanticClustersRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List findSemanticClustersRequestDescriptor = $convert.base64Decode('ChtGaW5kU2VtYW50aWNDbHVzdGVyc1JlcXVlc3QSHgoKY29sbGVjdGlvbhgBIAEoCVIKY29sbGVjdGlvbhIUCgVsYXllchgCIAEoDVIFbGF5ZXISKAoQbWluX2NsdXN0ZXJfc2l6ZRgDIAEoDVIObWluQ2x1c3RlclNpemUSIQoMbWF4X2NsdXN0ZXJzGAQgASgNUgttYXhDbHVzdGVycxIbCgltYXhfbm9kZXMYBSABKA1SCG1heE5vZGVz');
@$core.Deprecated('Use getConceptParentsRequestDescriptor instead')
const GetConceptParentsRequest$json = const {
  '1': 'GetConceptParentsRequest',
  '2': const [
    const {'1': 'collection', '3': 1, '4': 1, '5': 9, '10': 'collection'},
    const {'1': 'id', '3': 2, '4': 1, '5': 13, '10': 'id'},
    const {'1': 'layer', '3': 3, '4': 1, '5': 13, '10': 'layer'},
    const {'1': 'limit', '3': 4, '4': 1, '5': 13, '10': 'limit'},
  ],
};

/// Descriptor for `GetConceptParentsRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getConceptParentsRequestDescriptor = $convert.base64Decode('ChhHZXRDb25jZXB0UGFyZW50c1JlcXVlc3QSHgoKY29sbGVjdGlvbhgBIAEoCVIKY29sbGVjdGlvbhIOCgJpZBgCIAEoDVICaWQSFAoFbGF5ZXIYAyABKA1SBWxheWVyEhQKBWxpbWl0GAQgASgNUgVsaW1pdA==');
@$core.Deprecated('Use getConceptParentsResponseDescriptor instead')
const GetConceptParentsResponse$json = const {
  '1': 'GetConceptParentsResponse',
  '2': const [
    const {'1': 'parents', '3': 1, '4': 3, '5': 11, '6': '.hyperspace.GraphNode', '10': 'parents'},
  ],
};

/// Descriptor for `GetConceptParentsResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getConceptParentsResponseDescriptor = $convert.base64Decode('ChlHZXRDb25jZXB0UGFyZW50c1Jlc3BvbnNlEi8KB3BhcmVudHMYASADKAsyFS5oeXBlcnNwYWNlLkdyYXBoTm9kZVIHcGFyZW50cw==');
@$core.Deprecated('Use graphClusterDescriptor instead')
const GraphCluster$json = const {
  '1': 'GraphCluster',
  '2': const [
    const {'1': 'node_ids', '3': 1, '4': 3, '5': 13, '10': 'nodeIds'},
  ],
};

/// Descriptor for `GraphCluster`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List graphClusterDescriptor = $convert.base64Decode('CgxHcmFwaENsdXN0ZXISGQoIbm9kZV9pZHMYASADKA1SB25vZGVJZHM=');
@$core.Deprecated('Use findSemanticClustersResponseDescriptor instead')
const FindSemanticClustersResponse$json = const {
  '1': 'FindSemanticClustersResponse',
  '2': const [
    const {'1': 'clusters', '3': 1, '4': 3, '5': 11, '6': '.hyperspace.GraphCluster', '10': 'clusters'},
  ],
};

/// Descriptor for `FindSemanticClustersResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List findSemanticClustersResponseDescriptor = $convert.base64Decode('ChxGaW5kU2VtYW50aWNDbHVzdGVyc1Jlc3BvbnNlEjQKCGNsdXN0ZXJzGAEgAygLMhguaHlwZXJzcGFjZS5HcmFwaENsdXN0ZXJSCGNsdXN0ZXJz');
@$core.Deprecated('Use metadataValueDescriptor instead')
const MetadataValue$json = const {
  '1': 'MetadataValue',
  '2': const [
    const {'1': 'string_value', '3': 1, '4': 1, '5': 9, '9': 0, '10': 'stringValue'},
    const {'1': 'int_value', '3': 2, '4': 1, '5': 3, '9': 0, '10': 'intValue'},
    const {'1': 'double_value', '3': 3, '4': 1, '5': 1, '9': 0, '10': 'doubleValue'},
    const {'1': 'bool_value', '3': 4, '4': 1, '5': 8, '9': 0, '10': 'boolValue'},
  ],
  '8': const [
    const {'1': 'kind'},
  ],
};

/// Descriptor for `MetadataValue`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List metadataValueDescriptor = $convert.base64Decode('Cg1NZXRhZGF0YVZhbHVlEiMKDHN0cmluZ192YWx1ZRgBIAEoCUgAUgtzdHJpbmdWYWx1ZRIdCglpbnRfdmFsdWUYAiABKANIAFIIaW50VmFsdWUSIwoMZG91YmxlX3ZhbHVlGAMgASgBSABSC2RvdWJsZVZhbHVlEh8KCmJvb2xfdmFsdWUYBCABKAhIAFIJYm9vbFZhbHVlQgYKBGtpbmQ=');
@$core.Deprecated('Use eventSubscriptionRequestDescriptor instead')
const EventSubscriptionRequest$json = const {
  '1': 'EventSubscriptionRequest',
  '2': const [
    const {'1': 'types', '3': 1, '4': 3, '5': 14, '6': '.hyperspace.EventType', '10': 'types'},
    const {'1': 'collection', '3': 2, '4': 1, '5': 9, '9': 0, '10': 'collection', '17': true},
  ],
  '8': const [
    const {'1': '_collection'},
  ],
};

/// Descriptor for `EventSubscriptionRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List eventSubscriptionRequestDescriptor = $convert.base64Decode('ChhFdmVudFN1YnNjcmlwdGlvblJlcXVlc3QSKwoFdHlwZXMYASADKA4yFS5oeXBlcnNwYWNlLkV2ZW50VHlwZVIFdHlwZXMSIwoKY29sbGVjdGlvbhgCIAEoCUgAUgpjb2xsZWN0aW9uiAEBQg0KC19jb2xsZWN0aW9u');
@$core.Deprecated('Use vectorInsertedEventDescriptor instead')
const VectorInsertedEvent$json = const {
  '1': 'VectorInsertedEvent',
  '2': const [
    const {'1': 'id', '3': 1, '4': 1, '5': 13, '10': 'id'},
    const {'1': 'collection', '3': 2, '4': 1, '5': 9, '10': 'collection'},
    const {'1': 'logical_clock', '3': 3, '4': 1, '5': 4, '10': 'logicalClock'},
    const {'1': 'origin_node_id', '3': 4, '4': 1, '5': 9, '10': 'originNodeId'},
    const {'1': 'metadata', '3': 5, '4': 3, '5': 11, '6': '.hyperspace.VectorInsertedEvent.MetadataEntry', '10': 'metadata'},
    const {'1': 'typed_metadata', '3': 6, '4': 3, '5': 11, '6': '.hyperspace.VectorInsertedEvent.TypedMetadataEntry', '10': 'typedMetadata'},
  ],
  '3': const [VectorInsertedEvent_MetadataEntry$json, VectorInsertedEvent_TypedMetadataEntry$json],
};

@$core.Deprecated('Use vectorInsertedEventDescriptor instead')
const VectorInsertedEvent_MetadataEntry$json = const {
  '1': 'MetadataEntry',
  '2': const [
    const {'1': 'key', '3': 1, '4': 1, '5': 9, '10': 'key'},
    const {'1': 'value', '3': 2, '4': 1, '5': 9, '10': 'value'},
  ],
  '7': const {'7': true},
};

@$core.Deprecated('Use vectorInsertedEventDescriptor instead')
const VectorInsertedEvent_TypedMetadataEntry$json = const {
  '1': 'TypedMetadataEntry',
  '2': const [
    const {'1': 'key', '3': 1, '4': 1, '5': 9, '10': 'key'},
    const {'1': 'value', '3': 2, '4': 1, '5': 11, '6': '.hyperspace.MetadataValue', '10': 'value'},
  ],
  '7': const {'7': true},
};

/// Descriptor for `VectorInsertedEvent`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List vectorInsertedEventDescriptor = $convert.base64Decode('ChNWZWN0b3JJbnNlcnRlZEV2ZW50Eg4KAmlkGAEgASgNUgJpZBIeCgpjb2xsZWN0aW9uGAIgASgJUgpjb2xsZWN0aW9uEiMKDWxvZ2ljYWxfY2xvY2sYAyABKARSDGxvZ2ljYWxDbG9jaxIkCg5vcmlnaW5fbm9kZV9pZBgEIAEoCVIMb3JpZ2luTm9kZUlkEkkKCG1ldGFkYXRhGAUgAygLMi0uaHlwZXJzcGFjZS5WZWN0b3JJbnNlcnRlZEV2ZW50Lk1ldGFkYXRhRW50cnlSCG1ldGFkYXRhElkKDnR5cGVkX21ldGFkYXRhGAYgAygLMjIuaHlwZXJzcGFjZS5WZWN0b3JJbnNlcnRlZEV2ZW50LlR5cGVkTWV0YWRhdGFFbnRyeVINdHlwZWRNZXRhZGF0YRo7Cg1NZXRhZGF0YUVudHJ5EhAKA2tleRgBIAEoCVIDa2V5EhQKBXZhbHVlGAIgASgJUgV2YWx1ZToCOAEaWwoSVHlwZWRNZXRhZGF0YUVudHJ5EhAKA2tleRgBIAEoCVIDa2V5Ei8KBXZhbHVlGAIgASgLMhkuaHlwZXJzcGFjZS5NZXRhZGF0YVZhbHVlUgV2YWx1ZToCOAE=');
@$core.Deprecated('Use trajectoryStepEventDescriptor instead')
const TrajectoryStepEvent$json = const {
  '1': 'TrajectoryStepEvent',
  '2': const [
    const {'1': 'id', '3': 1, '4': 1, '5': 13, '10': 'id'},
    const {'1': 'collection', '3': 2, '4': 1, '5': 9, '10': 'collection'},
    const {'1': 'x', '3': 3, '4': 1, '5': 2, '10': 'x'},
    const {'1': 'y', '3': 4, '4': 1, '5': 2, '10': 'y'},
    const {'1': 'metadata', '3': 5, '4': 3, '5': 11, '6': '.hyperspace.TrajectoryStepEvent.MetadataEntry', '10': 'metadata'},
  ],
  '3': const [TrajectoryStepEvent_MetadataEntry$json],
};

@$core.Deprecated('Use trajectoryStepEventDescriptor instead')
const TrajectoryStepEvent_MetadataEntry$json = const {
  '1': 'MetadataEntry',
  '2': const [
    const {'1': 'key', '3': 1, '4': 1, '5': 9, '10': 'key'},
    const {'1': 'value', '3': 2, '4': 1, '5': 9, '10': 'value'},
  ],
  '7': const {'7': true},
};

/// Descriptor for `TrajectoryStepEvent`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List trajectoryStepEventDescriptor = $convert.base64Decode('ChNUcmFqZWN0b3J5U3RlcEV2ZW50Eg4KAmlkGAEgASgNUgJpZBIeCgpjb2xsZWN0aW9uGAIgASgJUgpjb2xsZWN0aW9uEgwKAXgYAyABKAJSAXgSDAoBeRgEIAEoAlIBeRJJCghtZXRhZGF0YRgFIAMoCzItLmh5cGVyc3BhY2UuVHJhamVjdG9yeVN0ZXBFdmVudC5NZXRhZGF0YUVudHJ5UghtZXRhZGF0YRo7Cg1NZXRhZGF0YUVudHJ5EhAKA2tleRgBIAEoCVIDa2V5EhQKBXZhbHVlGAIgASgJUgV2YWx1ZToCOAE=');
@$core.Deprecated('Use vectorDeletedEventDescriptor instead')
const VectorDeletedEvent$json = const {
  '1': 'VectorDeletedEvent',
  '2': const [
    const {'1': 'id', '3': 1, '4': 1, '5': 13, '10': 'id'},
    const {'1': 'collection', '3': 2, '4': 1, '5': 9, '10': 'collection'},
    const {'1': 'logical_clock', '3': 3, '4': 1, '5': 4, '10': 'logicalClock'},
    const {'1': 'origin_node_id', '3': 4, '4': 1, '5': 9, '10': 'originNodeId'},
  ],
};

/// Descriptor for `VectorDeletedEvent`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List vectorDeletedEventDescriptor = $convert.base64Decode('ChJWZWN0b3JEZWxldGVkRXZlbnQSDgoCaWQYASABKA1SAmlkEh4KCmNvbGxlY3Rpb24YAiABKAlSCmNvbGxlY3Rpb24SIwoNbG9naWNhbF9jbG9jaxgDIAEoBFIMbG9naWNhbENsb2NrEiQKDm9yaWdpbl9ub2RlX2lkGAQgASgJUgxvcmlnaW5Ob2RlSWQ=');
@$core.Deprecated('Use eventMessageDescriptor instead')
const EventMessage$json = const {
  '1': 'EventMessage',
  '2': const [
    const {'1': 'type', '3': 1, '4': 1, '5': 14, '6': '.hyperspace.EventType', '10': 'type'},
    const {'1': 'vector_inserted', '3': 2, '4': 1, '5': 11, '6': '.hyperspace.VectorInsertedEvent', '9': 0, '10': 'vectorInserted'},
    const {'1': 'vector_deleted', '3': 3, '4': 1, '5': 11, '6': '.hyperspace.VectorDeletedEvent', '9': 0, '10': 'vectorDeleted'},
    const {'1': 'trajectory_step', '3': 4, '4': 1, '5': 11, '6': '.hyperspace.TrajectoryStepEvent', '9': 0, '10': 'trajectoryStep'},
  ],
  '8': const [
    const {'1': 'payload'},
  ],
};

/// Descriptor for `EventMessage`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List eventMessageDescriptor = $convert.base64Decode('CgxFdmVudE1lc3NhZ2USKQoEdHlwZRgBIAEoDjIVLmh5cGVyc3BhY2UuRXZlbnRUeXBlUgR0eXBlEkoKD3ZlY3Rvcl9pbnNlcnRlZBgCIAEoCzIfLmh5cGVyc3BhY2UuVmVjdG9ySW5zZXJ0ZWRFdmVudEgAUg52ZWN0b3JJbnNlcnRlZBJHCg52ZWN0b3JfZGVsZXRlZBgDIAEoCzIeLmh5cGVyc3BhY2UuVmVjdG9yRGVsZXRlZEV2ZW50SABSDXZlY3RvckRlbGV0ZWQSSgoPdHJhamVjdG9yeV9zdGVwGAQgASgLMh8uaHlwZXJzcGFjZS5UcmFqZWN0b3J5U3RlcEV2ZW50SABSDnRyYWplY3RvcnlTdGVwQgkKB3BheWxvYWQ=');
@$core.Deprecated('Use emptyDescriptor instead')
const Empty$json = const {
  '1': 'Empty',
};

/// Descriptor for `Empty`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List emptyDescriptor = $convert.base64Decode('CgVFbXB0eQ==');
@$core.Deprecated('Use statusResponseDescriptor instead')
const StatusResponse$json = const {
  '1': 'StatusResponse',
  '2': const [
    const {'1': 'status', '3': 1, '4': 1, '5': 9, '10': 'status'},
  ],
};

/// Descriptor for `StatusResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List statusResponseDescriptor = $convert.base64Decode('Cg5TdGF0dXNSZXNwb25zZRIWCgZzdGF0dXMYASABKAlSBnN0YXR1cw==');
@$core.Deprecated('Use monitorRequestDescriptor instead')
const MonitorRequest$json = const {
  '1': 'MonitorRequest',
};

/// Descriptor for `MonitorRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List monitorRequestDescriptor = $convert.base64Decode('Cg5Nb25pdG9yUmVxdWVzdA==');
@$core.Deprecated('Use systemStatsDescriptor instead')
const SystemStats$json = const {
  '1': 'SystemStats',
  '2': const [
    const {'1': 'total_collections', '3': 1, '4': 1, '5': 4, '10': 'totalCollections'},
    const {'1': 'total_vectors', '3': 2, '4': 1, '5': 4, '10': 'totalVectors'},
    const {'1': 'total_memory_mb', '3': 3, '4': 1, '5': 1, '10': 'totalMemoryMb'},
    const {'1': 'qps', '3': 4, '4': 1, '5': 1, '10': 'qps'},
  ],
};

/// Descriptor for `SystemStats`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List systemStatsDescriptor = $convert.base64Decode('CgtTeXN0ZW1TdGF0cxIrChF0b3RhbF9jb2xsZWN0aW9ucxgBIAEoBFIQdG90YWxDb2xsZWN0aW9ucxIjCg10b3RhbF92ZWN0b3JzGAIgASgEUgx0b3RhbFZlY3RvcnMSJgoPdG90YWxfbWVtb3J5X21iGAMgASgBUg10b3RhbE1lbW9yeU1iEhAKA3FwcxgEIAEoAVIDcXBz');
@$core.Deprecated('Use digestRequestDescriptor instead')
const DigestRequest$json = const {
  '1': 'DigestRequest',
  '2': const [
    const {'1': 'collection', '3': 1, '4': 1, '5': 9, '10': 'collection'},
  ],
};

/// Descriptor for `DigestRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List digestRequestDescriptor = $convert.base64Decode('Cg1EaWdlc3RSZXF1ZXN0Eh4KCmNvbGxlY3Rpb24YASABKAlSCmNvbGxlY3Rpb24=');
@$core.Deprecated('Use digestResponseDescriptor instead')
const DigestResponse$json = const {
  '1': 'DigestResponse',
  '2': const [
    const {'1': 'logical_clock', '3': 1, '4': 1, '5': 4, '10': 'logicalClock'},
    const {'1': 'state_hash', '3': 2, '4': 1, '5': 4, '10': 'stateHash'},
    const {'1': 'buckets', '3': 3, '4': 3, '5': 4, '10': 'buckets'},
    const {'1': 'count', '3': 4, '4': 1, '5': 4, '10': 'count'},
  ],
};

/// Descriptor for `DigestResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List digestResponseDescriptor = $convert.base64Decode('Cg5EaWdlc3RSZXNwb25zZRIjCg1sb2dpY2FsX2Nsb2NrGAEgASgEUgxsb2dpY2FsQ2xvY2sSHQoKc3RhdGVfaGFzaBgCIAEoBFIJc3RhdGVIYXNoEhgKB2J1Y2tldHMYAyADKARSB2J1Y2tldHMSFAoFY291bnQYBCABKARSBWNvdW50');
@$core.Deprecated('Use syncHandshakeRequestDescriptor instead')
const SyncHandshakeRequest$json = const {
  '1': 'SyncHandshakeRequest',
  '2': const [
    const {'1': 'collection', '3': 1, '4': 1, '5': 9, '10': 'collection'},
    const {'1': 'client_buckets', '3': 2, '4': 3, '5': 4, '10': 'clientBuckets'},
    const {'1': 'client_logical_clock', '3': 3, '4': 1, '5': 4, '10': 'clientLogicalClock'},
    const {'1': 'client_count', '3': 4, '4': 1, '5': 4, '10': 'clientCount'},
  ],
};

/// Descriptor for `SyncHandshakeRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List syncHandshakeRequestDescriptor = $convert.base64Decode('ChRTeW5jSGFuZHNoYWtlUmVxdWVzdBIeCgpjb2xsZWN0aW9uGAEgASgJUgpjb2xsZWN0aW9uEiUKDmNsaWVudF9idWNrZXRzGAIgAygEUg1jbGllbnRCdWNrZXRzEjAKFGNsaWVudF9sb2dpY2FsX2Nsb2NrGAMgASgEUhJjbGllbnRMb2dpY2FsQ2xvY2sSIQoMY2xpZW50X2NvdW50GAQgASgEUgtjbGllbnRDb3VudA==');
@$core.Deprecated('Use diffBucketDescriptor instead')
const DiffBucket$json = const {
  '1': 'DiffBucket',
  '2': const [
    const {'1': 'bucket_index', '3': 1, '4': 1, '5': 13, '10': 'bucketIndex'},
    const {'1': 'server_hash', '3': 2, '4': 1, '5': 4, '10': 'serverHash'},
    const {'1': 'client_hash', '3': 3, '4': 1, '5': 4, '10': 'clientHash'},
  ],
};

/// Descriptor for `DiffBucket`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List diffBucketDescriptor = $convert.base64Decode('CgpEaWZmQnVja2V0EiEKDGJ1Y2tldF9pbmRleBgBIAEoDVILYnVja2V0SW5kZXgSHwoLc2VydmVyX2hhc2gYAiABKARSCnNlcnZlckhhc2gSHwoLY2xpZW50X2hhc2gYAyABKARSCmNsaWVudEhhc2g=');
@$core.Deprecated('Use syncHandshakeResponseDescriptor instead')
const SyncHandshakeResponse$json = const {
  '1': 'SyncHandshakeResponse',
  '2': const [
    const {'1': 'diff_buckets', '3': 1, '4': 3, '5': 11, '6': '.hyperspace.DiffBucket', '10': 'diffBuckets'},
    const {'1': 'server_logical_clock', '3': 2, '4': 1, '5': 4, '10': 'serverLogicalClock'},
    const {'1': 'server_count', '3': 3, '4': 1, '5': 4, '10': 'serverCount'},
    const {'1': 'in_sync', '3': 4, '4': 1, '5': 8, '10': 'inSync'},
  ],
};

/// Descriptor for `SyncHandshakeResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List syncHandshakeResponseDescriptor = $convert.base64Decode('ChVTeW5jSGFuZHNoYWtlUmVzcG9uc2USOQoMZGlmZl9idWNrZXRzGAEgAygLMhYuaHlwZXJzcGFjZS5EaWZmQnVja2V0UgtkaWZmQnVja2V0cxIwChRzZXJ2ZXJfbG9naWNhbF9jbG9jaxgCIAEoBFISc2VydmVyTG9naWNhbENsb2NrEiEKDHNlcnZlcl9jb3VudBgDIAEoBFILc2VydmVyQ291bnQSFwoHaW5fc3luYxgEIAEoCFIGaW5TeW5j');
@$core.Deprecated('Use syncPullRequestDescriptor instead')
const SyncPullRequest$json = const {
  '1': 'SyncPullRequest',
  '2': const [
    const {'1': 'collection', '3': 1, '4': 1, '5': 9, '10': 'collection'},
    const {'1': 'bucket_indices', '3': 2, '4': 3, '5': 13, '10': 'bucketIndices'},
  ],
};

/// Descriptor for `SyncPullRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List syncPullRequestDescriptor = $convert.base64Decode('Cg9TeW5jUHVsbFJlcXVlc3QSHgoKY29sbGVjdGlvbhgBIAEoCVIKY29sbGVjdGlvbhIlCg5idWNrZXRfaW5kaWNlcxgCIAMoDVINYnVja2V0SW5kaWNlcw==');
@$core.Deprecated('Use syncVectorDataDescriptor instead')
const SyncVectorData$json = const {
  '1': 'SyncVectorData',
  '2': const [
    const {'1': 'collection', '3': 1, '4': 1, '5': 9, '10': 'collection'},
    const {'1': 'id', '3': 2, '4': 1, '5': 13, '10': 'id'},
    const {'1': 'vector', '3': 3, '4': 3, '5': 1, '10': 'vector'},
    const {'1': 'metadata', '3': 4, '4': 3, '5': 11, '6': '.hyperspace.SyncVectorData.MetadataEntry', '10': 'metadata'},
    const {'1': 'bucket_index', '3': 5, '4': 1, '5': 13, '10': 'bucketIndex'},
  ],
  '3': const [SyncVectorData_MetadataEntry$json],
};

@$core.Deprecated('Use syncVectorDataDescriptor instead')
const SyncVectorData_MetadataEntry$json = const {
  '1': 'MetadataEntry',
  '2': const [
    const {'1': 'key', '3': 1, '4': 1, '5': 9, '10': 'key'},
    const {'1': 'value', '3': 2, '4': 1, '5': 9, '10': 'value'},
  ],
  '7': const {'7': true},
};

/// Descriptor for `SyncVectorData`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List syncVectorDataDescriptor = $convert.base64Decode('Cg5TeW5jVmVjdG9yRGF0YRIeCgpjb2xsZWN0aW9uGAEgASgJUgpjb2xsZWN0aW9uEg4KAmlkGAIgASgNUgJpZBIWCgZ2ZWN0b3IYAyADKAFSBnZlY3RvchJECghtZXRhZGF0YRgEIAMoCzIoLmh5cGVyc3BhY2UuU3luY1ZlY3RvckRhdGEuTWV0YWRhdGFFbnRyeVIIbWV0YWRhdGESIQoMYnVja2V0X2luZGV4GAUgASgNUgtidWNrZXRJbmRleBo7Cg1NZXRhZGF0YUVudHJ5EhAKA2tleRgBIAEoCVIDa2V5EhQKBXZhbHVlGAIgASgJUgV2YWx1ZToCOAE=');
@$core.Deprecated('Use syncPushResponseDescriptor instead')
const SyncPushResponse$json = const {
  '1': 'SyncPushResponse',
  '2': const [
    const {'1': 'accepted', '3': 1, '4': 1, '5': 13, '10': 'accepted'},
    const {'1': 'rejected', '3': 2, '4': 1, '5': 13, '10': 'rejected'},
    const {'1': 'duplicates', '3': 3, '4': 1, '5': 13, '10': 'duplicates'},
  ],
};

/// Descriptor for `SyncPushResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List syncPushResponseDescriptor = $convert.base64Decode('ChBTeW5jUHVzaFJlc3BvbnNlEhoKCGFjY2VwdGVkGAEgASgNUghhY2NlcHRlZBIaCghyZWplY3RlZBgCIAEoDVIIcmVqZWN0ZWQSHgoKZHVwbGljYXRlcxgDIAEoDVIKZHVwbGljYXRlcw==');
