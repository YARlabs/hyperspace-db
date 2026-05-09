// This is a generated file - do not edit.
//
// Generated from hyperspace.proto.

// @dart = 3.3

// ignore_for_file: annotate_overrides, camel_case_types, comment_references
// ignore_for_file: constant_identifier_names
// ignore_for_file: curly_braces_in_flow_control_structures
// ignore_for_file: deprecated_member_use_from_same_package, library_prefixes
// ignore_for_file: non_constant_identifier_names, prefer_relative_imports
// ignore_for_file: unused_import

import 'dart:convert' as $convert;
import 'dart:core' as $core;
import 'dart:typed_data' as $typed_data;

@$core.Deprecated('Use quantizationModeDescriptor instead')
const QuantizationMode$json = {
  '1': 'QuantizationMode',
  '2': [
    {'1': 'NONE', '2': 0},
    {'1': 'SCALAR_I8', '2': 1},
  ],
};

/// Descriptor for `QuantizationMode`. Decode as a `google.protobuf.EnumDescriptorProto`.
final $typed_data.Uint8List quantizationModeDescriptor = $convert.base64Decode(
    'ChBRdWFudGl6YXRpb25Nb2RlEggKBE5PTkUQABINCglTQ0FMQVJfSTgQAQ==');

@$core.Deprecated('Use durabilityLevelDescriptor instead')
const DurabilityLevel$json = {
  '1': 'DurabilityLevel',
  '2': [
    {'1': 'DEFAULT_LEVEL', '2': 0},
    {'1': 'ASYNC', '2': 1},
    {'1': 'BATCH', '2': 2},
    {'1': 'STRICT', '2': 3},
  ],
};

/// Descriptor for `DurabilityLevel`. Decode as a `google.protobuf.EnumDescriptorProto`.
final $typed_data.Uint8List durabilityLevelDescriptor = $convert.base64Decode(
    'Cg9EdXJhYmlsaXR5TGV2ZWwSEQoNREVGQVVMVF9MRVZFTBAAEgkKBUFTWU5DEAESCQoFQkFUQ0'
    'gQAhIKCgZTVFJJQ1QQAw==');

@$core.Deprecated('Use eventTypeDescriptor instead')
const EventType$json = {
  '1': 'EventType',
  '2': [
    {'1': 'EVENT_UNKNOWN', '2': 0},
    {'1': 'VECTOR_INSERTED', '2': 1},
    {'1': 'VECTOR_DELETED', '2': 2},
    {'1': 'TRAJECTORY_STEP', '2': 3},
  ],
};

/// Descriptor for `EventType`. Decode as a `google.protobuf.EnumDescriptorProto`.
final $typed_data.Uint8List eventTypeDescriptor = $convert.base64Decode(
    'CglFdmVudFR5cGUSEQoNRVZFTlRfVU5LTk9XThAAEhMKD1ZFQ1RPUl9JTlNFUlRFRBABEhIKDl'
    'ZFQ1RPUl9ERUxFVEVEEAISEwoPVFJBSkVDVE9SWV9TVEVQEAM=');

@$core.Deprecated('Use replicationRequestDescriptor instead')
const ReplicationRequest$json = {
  '1': 'ReplicationRequest',
  '2': [
    {
      '1': 'last_logical_clock',
      '3': 1,
      '4': 1,
      '5': 4,
      '10': 'lastLogicalClock'
    },
  ],
};

/// Descriptor for `ReplicationRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List replicationRequestDescriptor = $convert.base64Decode(
    'ChJSZXBsaWNhdGlvblJlcXVlc3QSLAoSbGFzdF9sb2dpY2FsX2Nsb2NrGAEgASgEUhBsYXN0TG'
    '9naWNhbENsb2Nr');

@$core.Deprecated('Use replicationLogDescriptor instead')
const ReplicationLog$json = {
  '1': 'ReplicationLog',
  '2': [
    {'1': 'logical_clock', '3': 1, '4': 1, '5': 4, '10': 'logicalClock'},
    {'1': 'origin_node_id', '3': 2, '4': 1, '5': 9, '10': 'originNodeId'},
    {'1': 'collection', '3': 3, '4': 1, '5': 9, '10': 'collection'},
    {
      '1': 'insert',
      '3': 4,
      '4': 1,
      '5': 11,
      '6': '.hyperspace.InsertOp',
      '9': 0,
      '10': 'insert'
    },
    {
      '1': 'create_collection',
      '3': 5,
      '4': 1,
      '5': 11,
      '6': '.hyperspace.CreateCollectionOp',
      '9': 0,
      '10': 'createCollection'
    },
    {
      '1': 'delete_collection',
      '3': 6,
      '4': 1,
      '5': 11,
      '6': '.hyperspace.DeleteCollectionOp',
      '9': 0,
      '10': 'deleteCollection'
    },
    {
      '1': 'delete',
      '3': 7,
      '4': 1,
      '5': 11,
      '6': '.hyperspace.DeleteOp',
      '9': 0,
      '10': 'delete'
    },
  ],
  '8': [
    {'1': 'operation'},
  ],
};

/// Descriptor for `ReplicationLog`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List replicationLogDescriptor = $convert.base64Decode(
    'Cg5SZXBsaWNhdGlvbkxvZxIjCg1sb2dpY2FsX2Nsb2NrGAEgASgEUgxsb2dpY2FsQ2xvY2sSJA'
    'oOb3JpZ2luX25vZGVfaWQYAiABKAlSDG9yaWdpbk5vZGVJZBIeCgpjb2xsZWN0aW9uGAMgASgJ'
    'Ugpjb2xsZWN0aW9uEi4KBmluc2VydBgEIAEoCzIULmh5cGVyc3BhY2UuSW5zZXJ0T3BIAFIGaW'
    '5zZXJ0Ek0KEWNyZWF0ZV9jb2xsZWN0aW9uGAUgASgLMh4uaHlwZXJzcGFjZS5DcmVhdGVDb2xs'
    'ZWN0aW9uT3BIAFIQY3JlYXRlQ29sbGVjdGlvbhJNChFkZWxldGVfY29sbGVjdGlvbhgGIAEoCz'
    'IeLmh5cGVyc3BhY2UuRGVsZXRlQ29sbGVjdGlvbk9wSABSEGRlbGV0ZUNvbGxlY3Rpb24SLgoG'
    'ZGVsZXRlGAcgASgLMhQuaHlwZXJzcGFjZS5EZWxldGVPcEgAUgZkZWxldGVCCwoJb3BlcmF0aW'
    '9u');

@$core.Deprecated('Use insertOpDescriptor instead')
const InsertOp$json = {
  '1': 'InsertOp',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 13, '10': 'id'},
    {'1': 'vector', '3': 2, '4': 3, '5': 1, '10': 'vector'},
    {
      '1': 'metadata',
      '3': 3,
      '4': 3,
      '5': 11,
      '6': '.hyperspace.InsertOp.MetadataEntry',
      '10': 'metadata'
    },
    {
      '1': 'typed_metadata',
      '3': 4,
      '4': 3,
      '5': 11,
      '6': '.hyperspace.InsertOp.TypedMetadataEntry',
      '10': 'typedMetadata'
    },
  ],
  '3': [InsertOp_MetadataEntry$json, InsertOp_TypedMetadataEntry$json],
};

@$core.Deprecated('Use insertOpDescriptor instead')
const InsertOp_MetadataEntry$json = {
  '1': 'MetadataEntry',
  '2': [
    {'1': 'key', '3': 1, '4': 1, '5': 9, '10': 'key'},
    {'1': 'value', '3': 2, '4': 1, '5': 9, '10': 'value'},
  ],
  '7': {'7': true},
};

@$core.Deprecated('Use insertOpDescriptor instead')
const InsertOp_TypedMetadataEntry$json = {
  '1': 'TypedMetadataEntry',
  '2': [
    {'1': 'key', '3': 1, '4': 1, '5': 9, '10': 'key'},
    {
      '1': 'value',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.hyperspace.MetadataValue',
      '10': 'value'
    },
  ],
  '7': {'7': true},
};

/// Descriptor for `InsertOp`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List insertOpDescriptor = $convert.base64Decode(
    'CghJbnNlcnRPcBIOCgJpZBgBIAEoDVICaWQSFgoGdmVjdG9yGAIgAygBUgZ2ZWN0b3ISPgoIbW'
    'V0YWRhdGEYAyADKAsyIi5oeXBlcnNwYWNlLkluc2VydE9wLk1ldGFkYXRhRW50cnlSCG1ldGFk'
    'YXRhEk4KDnR5cGVkX21ldGFkYXRhGAQgAygLMicuaHlwZXJzcGFjZS5JbnNlcnRPcC5UeXBlZE'
    '1ldGFkYXRhRW50cnlSDXR5cGVkTWV0YWRhdGEaOwoNTWV0YWRhdGFFbnRyeRIQCgNrZXkYASAB'
    'KAlSA2tleRIUCgV2YWx1ZRgCIAEoCVIFdmFsdWU6AjgBGlsKElR5cGVkTWV0YWRhdGFFbnRyeR'
    'IQCgNrZXkYASABKAlSA2tleRIvCgV2YWx1ZRgCIAEoCzIZLmh5cGVyc3BhY2UuTWV0YWRhdGFW'
    'YWx1ZVIFdmFsdWU6AjgB');

@$core.Deprecated('Use createCollectionOpDescriptor instead')
const CreateCollectionOp$json = {
  '1': 'CreateCollectionOp',
  '2': [
    {'1': 'dimension', '3': 1, '4': 1, '5': 13, '10': 'dimension'},
    {'1': 'metric', '3': 2, '4': 1, '5': 9, '10': 'metric'},
  ],
};

/// Descriptor for `CreateCollectionOp`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List createCollectionOpDescriptor = $convert.base64Decode(
    'ChJDcmVhdGVDb2xsZWN0aW9uT3ASHAoJZGltZW5zaW9uGAEgASgNUglkaW1lbnNpb24SFgoGbW'
    'V0cmljGAIgASgJUgZtZXRyaWM=');

@$core.Deprecated('Use deleteCollectionOpDescriptor instead')
const DeleteCollectionOp$json = {
  '1': 'DeleteCollectionOp',
};

/// Descriptor for `DeleteCollectionOp`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List deleteCollectionOpDescriptor =
    $convert.base64Decode('ChJEZWxldGVDb2xsZWN0aW9uT3A=');

@$core.Deprecated('Use deleteOpDescriptor instead')
const DeleteOp$json = {
  '1': 'DeleteOp',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 13, '10': 'id'},
  ],
};

/// Descriptor for `DeleteOp`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List deleteOpDescriptor =
    $convert.base64Decode('CghEZWxldGVPcBIOCgJpZBgBIAEoDVICaWQ=');

@$core.Deprecated('Use quantizationConfigDescriptor instead')
const QuantizationConfig$json = {
  '1': 'QuantizationConfig',
  '2': [
    {
      '1': 'mode',
      '3': 1,
      '4': 1,
      '5': 14,
      '6': '.hyperspace.QuantizationMode',
      '10': 'mode'
    },
  ],
};

/// Descriptor for `QuantizationConfig`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List quantizationConfigDescriptor = $convert.base64Decode(
    'ChJRdWFudGl6YXRpb25Db25maWcSMAoEbW9kZRgBIAEoDjIcLmh5cGVyc3BhY2UuUXVhbnRpem'
    'F0aW9uTW9kZVIEbW9kZQ==');

@$core.Deprecated('Use createCollectionRequestDescriptor instead')
const CreateCollectionRequest$json = {
  '1': 'CreateCollectionRequest',
  '2': [
    {'1': 'name', '3': 1, '4': 1, '5': 9, '10': 'name'},
    {'1': 'dimension', '3': 2, '4': 1, '5': 13, '10': 'dimension'},
    {'1': 'metric', '3': 3, '4': 1, '5': 9, '10': 'metric'},
    {
      '1': 'components',
      '3': 4,
      '4': 3,
      '5': 11,
      '6': '.hyperspace.CollectionComponent',
      '10': 'components'
    },
  ],
};

/// Descriptor for `CreateCollectionRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List createCollectionRequestDescriptor = $convert.base64Decode(
    'ChdDcmVhdGVDb2xsZWN0aW9uUmVxdWVzdBISCgRuYW1lGAEgASgJUgRuYW1lEhwKCWRpbWVuc2'
    'lvbhgCIAEoDVIJZGltZW5zaW9uEhYKBm1ldHJpYxgDIAEoCVIGbWV0cmljEj8KCmNvbXBvbmVu'
    'dHMYBCADKAsyHy5oeXBlcnNwYWNlLkNvbGxlY3Rpb25Db21wb25lbnRSCmNvbXBvbmVudHM=');

@$core.Deprecated('Use collectionComponentDescriptor instead')
const CollectionComponent$json = {
  '1': 'CollectionComponent',
  '2': [
    {'1': 'space', '3': 1, '4': 1, '5': 9, '10': 'space'},
    {'1': 'dimension', '3': 2, '4': 1, '5': 13, '10': 'dimension'},
    {'1': 'metric', '3': 3, '4': 1, '5': 9, '10': 'metric'},
    {'1': 'weight', '3': 4, '4': 1, '5': 2, '9': 0, '10': 'weight', '17': true},
  ],
  '8': [
    {'1': '_weight'},
  ],
};

/// Descriptor for `CollectionComponent`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List collectionComponentDescriptor = $convert.base64Decode(
    'ChNDb2xsZWN0aW9uQ29tcG9uZW50EhQKBXNwYWNlGAEgASgJUgVzcGFjZRIcCglkaW1lbnNpb2'
    '4YAiABKA1SCWRpbWVuc2lvbhIWCgZtZXRyaWMYAyABKAlSBm1ldHJpYxIbCgZ3ZWlnaHQYBCAB'
    'KAJIAFIGd2VpZ2h0iAEBQgkKB193ZWlnaHQ=');

@$core.Deprecated('Use deleteCollectionRequestDescriptor instead')
const DeleteCollectionRequest$json = {
  '1': 'DeleteCollectionRequest',
  '2': [
    {'1': 'name', '3': 1, '4': 1, '5': 9, '10': 'name'},
  ],
};

/// Descriptor for `DeleteCollectionRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List deleteCollectionRequestDescriptor =
    $convert.base64Decode(
        'ChdEZWxldGVDb2xsZWN0aW9uUmVxdWVzdBISCgRuYW1lGAEgASgJUgRuYW1l');

@$core.Deprecated('Use collectionSummaryDescriptor instead')
const CollectionSummary$json = {
  '1': 'CollectionSummary',
  '2': [
    {'1': 'name', '3': 1, '4': 1, '5': 9, '10': 'name'},
    {'1': 'count', '3': 2, '4': 1, '5': 4, '10': 'count'},
    {'1': 'dimension', '3': 3, '4': 1, '5': 13, '10': 'dimension'},
    {'1': 'metric', '3': 4, '4': 1, '5': 9, '10': 'metric'},
  ],
};

/// Descriptor for `CollectionSummary`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List collectionSummaryDescriptor = $convert.base64Decode(
    'ChFDb2xsZWN0aW9uU3VtbWFyeRISCgRuYW1lGAEgASgJUgRuYW1lEhQKBWNvdW50GAIgASgEUg'
    'Vjb3VudBIcCglkaW1lbnNpb24YAyABKA1SCWRpbWVuc2lvbhIWCgZtZXRyaWMYBCABKAlSBm1l'
    'dHJpYw==');

@$core.Deprecated('Use listCollectionsResponseDescriptor instead')
const ListCollectionsResponse$json = {
  '1': 'ListCollectionsResponse',
  '2': [
    {
      '1': 'collections',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.hyperspace.CollectionSummary',
      '10': 'collections'
    },
  ],
};

/// Descriptor for `ListCollectionsResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listCollectionsResponseDescriptor =
    $convert.base64Decode(
        'ChdMaXN0Q29sbGVjdGlvbnNSZXNwb25zZRI/Cgtjb2xsZWN0aW9ucxgBIAMoCzIdLmh5cGVyc3'
        'BhY2UuQ29sbGVjdGlvblN1bW1hcnlSC2NvbGxlY3Rpb25z');

@$core.Deprecated('Use collectionStatsRequestDescriptor instead')
const CollectionStatsRequest$json = {
  '1': 'CollectionStatsRequest',
  '2': [
    {'1': 'name', '3': 1, '4': 1, '5': 9, '10': 'name'},
  ],
};

/// Descriptor for `CollectionStatsRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List collectionStatsRequestDescriptor =
    $convert.base64Decode(
        'ChZDb2xsZWN0aW9uU3RhdHNSZXF1ZXN0EhIKBG5hbWUYASABKAlSBG5hbWU=');

@$core.Deprecated('Use collectionStatsResponseDescriptor instead')
const CollectionStatsResponse$json = {
  '1': 'CollectionStatsResponse',
  '2': [
    {'1': 'count', '3': 1, '4': 1, '5': 4, '10': 'count'},
    {'1': 'dimension', '3': 2, '4': 1, '5': 13, '10': 'dimension'},
    {'1': 'metric', '3': 3, '4': 1, '5': 9, '10': 'metric'},
    {'1': 'indexing_queue', '3': 4, '4': 1, '5': 4, '10': 'indexingQueue'},
    {'1': 'disk_usage_bytes', '3': 5, '4': 1, '5': 4, '10': 'diskUsageBytes'},
    {'1': 'ram_usage_bytes', '3': 6, '4': 1, '5': 4, '10': 'ramUsageBytes'},
    {'1': 'active_tasks', '3': 7, '4': 1, '5': 4, '10': 'activeTasks'},
  ],
};

/// Descriptor for `CollectionStatsResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List collectionStatsResponseDescriptor = $convert.base64Decode(
    'ChdDb2xsZWN0aW9uU3RhdHNSZXNwb25zZRIUCgVjb3VudBgBIAEoBFIFY291bnQSHAoJZGltZW'
    '5zaW9uGAIgASgNUglkaW1lbnNpb24SFgoGbWV0cmljGAMgASgJUgZtZXRyaWMSJQoOaW5kZXhp'
    'bmdfcXVldWUYBCABKARSDWluZGV4aW5nUXVldWUSKAoQZGlza191c2FnZV9ieXRlcxgFIAEoBF'
    'IOZGlza1VzYWdlQnl0ZXMSJgoPcmFtX3VzYWdlX2J5dGVzGAYgASgEUg1yYW1Vc2FnZUJ5dGVz'
    'EiEKDGFjdGl2ZV90YXNrcxgHIAEoBFILYWN0aXZlVGFza3M=');

@$core.Deprecated('Use rebuildIndexRequestDescriptor instead')
const RebuildIndexRequest$json = {
  '1': 'RebuildIndexRequest',
  '2': [
    {'1': 'name', '3': 1, '4': 1, '5': 9, '10': 'name'},
    {
      '1': 'filter_query',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.hyperspace.VacuumFilterQuery',
      '9': 0,
      '10': 'filterQuery',
      '17': true
    },
  ],
  '8': [
    {'1': '_filter_query'},
  ],
};

/// Descriptor for `RebuildIndexRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List rebuildIndexRequestDescriptor = $convert.base64Decode(
    'ChNSZWJ1aWxkSW5kZXhSZXF1ZXN0EhIKBG5hbWUYASABKAlSBG5hbWUSRQoMZmlsdGVyX3F1ZX'
    'J5GAIgASgLMh0uaHlwZXJzcGFjZS5WYWN1dW1GaWx0ZXJRdWVyeUgAUgtmaWx0ZXJRdWVyeYgB'
    'AUIPCg1fZmlsdGVyX3F1ZXJ5');

@$core.Deprecated('Use configUpdateDescriptor instead')
const ConfigUpdate$json = {
  '1': 'ConfigUpdate',
  '2': [
    {'1': 'collection', '3': 1, '4': 1, '5': 9, '10': 'collection'},
    {
      '1': 'ef_search',
      '3': 2,
      '4': 1,
      '5': 13,
      '9': 0,
      '10': 'efSearch',
      '17': true
    },
    {
      '1': 'ef_construction',
      '3': 3,
      '4': 1,
      '5': 13,
      '9': 1,
      '10': 'efConstruction',
      '17': true
    },
    {'1': 'm', '3': 4, '4': 1, '5': 13, '9': 2, '10': 'm', '17': true},
  ],
  '8': [
    {'1': '_ef_search'},
    {'1': '_ef_construction'},
    {'1': '_m'},
  ],
};

/// Descriptor for `ConfigUpdate`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List configUpdateDescriptor = $convert.base64Decode(
    'CgxDb25maWdVcGRhdGUSHgoKY29sbGVjdGlvbhgBIAEoCVIKY29sbGVjdGlvbhIgCgllZl9zZW'
    'FyY2gYAiABKA1IAFIIZWZTZWFyY2iIAQESLAoPZWZfY29uc3RydWN0aW9uGAMgASgNSAFSDmVm'
    'Q29uc3RydWN0aW9uiAEBEhEKAW0YBCABKA1IAlIBbYgBAUIMCgpfZWZfc2VhcmNoQhIKEF9lZl'
    '9jb25zdHJ1Y3Rpb25CBAoCX20=');

@$core.Deprecated('Use vacuumFilterQueryDescriptor instead')
const VacuumFilterQuery$json = {
  '1': 'VacuumFilterQuery',
  '2': [
    {'1': 'key', '3': 1, '4': 1, '5': 9, '10': 'key'},
    {'1': 'op', '3': 2, '4': 1, '5': 9, '10': 'op'},
    {'1': 'value', '3': 3, '4': 1, '5': 1, '10': 'value'},
  ],
};

/// Descriptor for `VacuumFilterQuery`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List vacuumFilterQueryDescriptor = $convert.base64Decode(
    'ChFWYWN1dW1GaWx0ZXJRdWVyeRIQCgNrZXkYASABKAlSA2tleRIOCgJvcBgCIAEoCVICb3ASFA'
    'oFdmFsdWUYAyABKAFSBXZhbHVl');

@$core.Deprecated('Use reconsolidationRequestDescriptor instead')
const ReconsolidationRequest$json = {
  '1': 'ReconsolidationRequest',
  '2': [
    {'1': 'collection', '3': 1, '4': 1, '5': 9, '10': 'collection'},
    {'1': 'target_vector', '3': 2, '4': 3, '5': 1, '10': 'targetVector'},
    {'1': 'learning_rate', '3': 3, '4': 1, '5': 1, '10': 'learningRate'},
  ],
};

/// Descriptor for `ReconsolidationRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List reconsolidationRequestDescriptor = $convert.base64Decode(
    'ChZSZWNvbnNvbGlkYXRpb25SZXF1ZXN0Eh4KCmNvbGxlY3Rpb24YASABKAlSCmNvbGxlY3Rpb2'
    '4SIwoNdGFyZ2V0X3ZlY3RvchgCIAMoAVIMdGFyZ2V0VmVjdG9yEiMKDWxlYXJuaW5nX3JhdGUY'
    'AyABKAFSDGxlYXJuaW5nUmF0ZQ==');

@$core.Deprecated('Use insertRequestDescriptor instead')
const InsertRequest$json = {
  '1': 'InsertRequest',
  '2': [
    {'1': 'collection', '3': 1, '4': 1, '5': 9, '10': 'collection'},
    {'1': 'vector', '3': 2, '4': 3, '5': 1, '10': 'vector'},
    {'1': 'id', '3': 3, '4': 1, '5': 13, '10': 'id'},
    {
      '1': 'metadata',
      '3': 4,
      '4': 3,
      '5': 11,
      '6': '.hyperspace.InsertRequest.MetadataEntry',
      '10': 'metadata'
    },
    {'1': 'origin_node_id', '3': 5, '4': 1, '5': 9, '10': 'originNodeId'},
    {'1': 'logical_clock', '3': 6, '4': 1, '5': 4, '10': 'logicalClock'},
    {
      '1': 'durability',
      '3': 7,
      '4': 1,
      '5': 14,
      '6': '.hyperspace.DurabilityLevel',
      '10': 'durability'
    },
    {
      '1': 'typed_metadata',
      '3': 8,
      '4': 3,
      '5': 11,
      '6': '.hyperspace.InsertRequest.TypedMetadataEntry',
      '10': 'typedMetadata'
    },
  ],
  '3': [
    InsertRequest_MetadataEntry$json,
    InsertRequest_TypedMetadataEntry$json
  ],
};

@$core.Deprecated('Use insertRequestDescriptor instead')
const InsertRequest_MetadataEntry$json = {
  '1': 'MetadataEntry',
  '2': [
    {'1': 'key', '3': 1, '4': 1, '5': 9, '10': 'key'},
    {'1': 'value', '3': 2, '4': 1, '5': 9, '10': 'value'},
  ],
  '7': {'7': true},
};

@$core.Deprecated('Use insertRequestDescriptor instead')
const InsertRequest_TypedMetadataEntry$json = {
  '1': 'TypedMetadataEntry',
  '2': [
    {'1': 'key', '3': 1, '4': 1, '5': 9, '10': 'key'},
    {
      '1': 'value',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.hyperspace.MetadataValue',
      '10': 'value'
    },
  ],
  '7': {'7': true},
};

/// Descriptor for `InsertRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List insertRequestDescriptor = $convert.base64Decode(
    'Cg1JbnNlcnRSZXF1ZXN0Eh4KCmNvbGxlY3Rpb24YASABKAlSCmNvbGxlY3Rpb24SFgoGdmVjdG'
    '9yGAIgAygBUgZ2ZWN0b3ISDgoCaWQYAyABKA1SAmlkEkMKCG1ldGFkYXRhGAQgAygLMicuaHlw'
    'ZXJzcGFjZS5JbnNlcnRSZXF1ZXN0Lk1ldGFkYXRhRW50cnlSCG1ldGFkYXRhEiQKDm9yaWdpbl'
    '9ub2RlX2lkGAUgASgJUgxvcmlnaW5Ob2RlSWQSIwoNbG9naWNhbF9jbG9jaxgGIAEoBFIMbG9n'
    'aWNhbENsb2NrEjsKCmR1cmFiaWxpdHkYByABKA4yGy5oeXBlcnNwYWNlLkR1cmFiaWxpdHlMZX'
    'ZlbFIKZHVyYWJpbGl0eRJTCg50eXBlZF9tZXRhZGF0YRgIIAMoCzIsLmh5cGVyc3BhY2UuSW5z'
    'ZXJ0UmVxdWVzdC5UeXBlZE1ldGFkYXRhRW50cnlSDXR5cGVkTWV0YWRhdGEaOwoNTWV0YWRhdG'
    'FFbnRyeRIQCgNrZXkYASABKAlSA2tleRIUCgV2YWx1ZRgCIAEoCVIFdmFsdWU6AjgBGlsKElR5'
    'cGVkTWV0YWRhdGFFbnRyeRIQCgNrZXkYASABKAlSA2tleRIvCgV2YWx1ZRgCIAEoCzIZLmh5cG'
    'Vyc3BhY2UuTWV0YWRhdGFWYWx1ZVIFdmFsdWU6AjgB');

@$core.Deprecated('Use vectorDataDescriptor instead')
const VectorData$json = {
  '1': 'VectorData',
  '2': [
    {'1': 'vector', '3': 1, '4': 3, '5': 1, '10': 'vector'},
    {'1': 'id', '3': 2, '4': 1, '5': 13, '10': 'id'},
    {
      '1': 'metadata',
      '3': 3,
      '4': 3,
      '5': 11,
      '6': '.hyperspace.VectorData.MetadataEntry',
      '10': 'metadata'
    },
    {
      '1': 'typed_metadata',
      '3': 4,
      '4': 3,
      '5': 11,
      '6': '.hyperspace.VectorData.TypedMetadataEntry',
      '10': 'typedMetadata'
    },
  ],
  '3': [VectorData_MetadataEntry$json, VectorData_TypedMetadataEntry$json],
};

@$core.Deprecated('Use vectorDataDescriptor instead')
const VectorData_MetadataEntry$json = {
  '1': 'MetadataEntry',
  '2': [
    {'1': 'key', '3': 1, '4': 1, '5': 9, '10': 'key'},
    {'1': 'value', '3': 2, '4': 1, '5': 9, '10': 'value'},
  ],
  '7': {'7': true},
};

@$core.Deprecated('Use vectorDataDescriptor instead')
const VectorData_TypedMetadataEntry$json = {
  '1': 'TypedMetadataEntry',
  '2': [
    {'1': 'key', '3': 1, '4': 1, '5': 9, '10': 'key'},
    {
      '1': 'value',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.hyperspace.MetadataValue',
      '10': 'value'
    },
  ],
  '7': {'7': true},
};

/// Descriptor for `VectorData`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List vectorDataDescriptor = $convert.base64Decode(
    'CgpWZWN0b3JEYXRhEhYKBnZlY3RvchgBIAMoAVIGdmVjdG9yEg4KAmlkGAIgASgNUgJpZBJACg'
    'htZXRhZGF0YRgDIAMoCzIkLmh5cGVyc3BhY2UuVmVjdG9yRGF0YS5NZXRhZGF0YUVudHJ5Ught'
    'ZXRhZGF0YRJQCg50eXBlZF9tZXRhZGF0YRgEIAMoCzIpLmh5cGVyc3BhY2UuVmVjdG9yRGF0YS'
    '5UeXBlZE1ldGFkYXRhRW50cnlSDXR5cGVkTWV0YWRhdGEaOwoNTWV0YWRhdGFFbnRyeRIQCgNr'
    'ZXkYASABKAlSA2tleRIUCgV2YWx1ZRgCIAEoCVIFdmFsdWU6AjgBGlsKElR5cGVkTWV0YWRhdG'
    'FFbnRyeRIQCgNrZXkYASABKAlSA2tleRIvCgV2YWx1ZRgCIAEoCzIZLmh5cGVyc3BhY2UuTWV0'
    'YWRhdGFWYWx1ZVIFdmFsdWU6AjgB');

@$core.Deprecated('Use batchInsertRequestDescriptor instead')
const BatchInsertRequest$json = {
  '1': 'BatchInsertRequest',
  '2': [
    {'1': 'collection', '3': 1, '4': 1, '5': 9, '10': 'collection'},
    {
      '1': 'vectors',
      '3': 2,
      '4': 3,
      '5': 11,
      '6': '.hyperspace.VectorData',
      '10': 'vectors'
    },
    {'1': 'origin_node_id', '3': 3, '4': 1, '5': 9, '10': 'originNodeId'},
    {'1': 'logical_clock', '3': 4, '4': 1, '5': 4, '10': 'logicalClock'},
    {
      '1': 'durability',
      '3': 5,
      '4': 1,
      '5': 14,
      '6': '.hyperspace.DurabilityLevel',
      '10': 'durability'
    },
  ],
};

/// Descriptor for `BatchInsertRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List batchInsertRequestDescriptor = $convert.base64Decode(
    'ChJCYXRjaEluc2VydFJlcXVlc3QSHgoKY29sbGVjdGlvbhgBIAEoCVIKY29sbGVjdGlvbhIwCg'
    'd2ZWN0b3JzGAIgAygLMhYuaHlwZXJzcGFjZS5WZWN0b3JEYXRhUgd2ZWN0b3JzEiQKDm9yaWdp'
    'bl9ub2RlX2lkGAMgASgJUgxvcmlnaW5Ob2RlSWQSIwoNbG9naWNhbF9jbG9jaxgEIAEoBFIMbG'
    '9naWNhbENsb2NrEjsKCmR1cmFiaWxpdHkYBSABKA4yGy5oeXBlcnNwYWNlLkR1cmFiaWxpdHlM'
    'ZXZlbFIKZHVyYWJpbGl0eQ==');

@$core.Deprecated('Use insertTextRequestDescriptor instead')
const InsertTextRequest$json = {
  '1': 'InsertTextRequest',
  '2': [
    {'1': 'collection', '3': 1, '4': 1, '5': 9, '10': 'collection'},
    {'1': 'id', '3': 2, '4': 1, '5': 13, '10': 'id'},
    {'1': 'text', '3': 3, '4': 1, '5': 9, '10': 'text'},
    {
      '1': 'metadata',
      '3': 4,
      '4': 3,
      '5': 11,
      '6': '.hyperspace.InsertTextRequest.MetadataEntry',
      '10': 'metadata'
    },
    {
      '1': 'durability',
      '3': 5,
      '4': 1,
      '5': 14,
      '6': '.hyperspace.DurabilityLevel',
      '10': 'durability'
    },
  ],
  '3': [InsertTextRequest_MetadataEntry$json],
};

@$core.Deprecated('Use insertTextRequestDescriptor instead')
const InsertTextRequest_MetadataEntry$json = {
  '1': 'MetadataEntry',
  '2': [
    {'1': 'key', '3': 1, '4': 1, '5': 9, '10': 'key'},
    {'1': 'value', '3': 2, '4': 1, '5': 9, '10': 'value'},
  ],
  '7': {'7': true},
};

/// Descriptor for `InsertTextRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List insertTextRequestDescriptor = $convert.base64Decode(
    'ChFJbnNlcnRUZXh0UmVxdWVzdBIeCgpjb2xsZWN0aW9uGAEgASgJUgpjb2xsZWN0aW9uEg4KAm'
    'lkGAIgASgNUgJpZBISCgR0ZXh0GAMgASgJUgR0ZXh0EkcKCG1ldGFkYXRhGAQgAygLMisuaHlw'
    'ZXJzcGFjZS5JbnNlcnRUZXh0UmVxdWVzdC5NZXRhZGF0YUVudHJ5UghtZXRhZGF0YRI7CgpkdX'
    'JhYmlsaXR5GAUgASgOMhsuaHlwZXJzcGFjZS5EdXJhYmlsaXR5TGV2ZWxSCmR1cmFiaWxpdHka'
    'OwoNTWV0YWRhdGFFbnRyeRIQCgNrZXkYASABKAlSA2tleRIUCgV2YWx1ZRgCIAEoCVIFdmFsdW'
    'U6AjgB');

@$core.Deprecated('Use vectorizeRequestDescriptor instead')
const VectorizeRequest$json = {
  '1': 'VectorizeRequest',
  '2': [
    {'1': 'text', '3': 1, '4': 1, '5': 9, '10': 'text'},
    {'1': 'metric', '3': 2, '4': 1, '5': 9, '10': 'metric'},
  ],
};

/// Descriptor for `VectorizeRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List vectorizeRequestDescriptor = $convert.base64Decode(
    'ChBWZWN0b3JpemVSZXF1ZXN0EhIKBHRleHQYASABKAlSBHRleHQSFgoGbWV0cmljGAIgASgJUg'
    'ZtZXRyaWM=');

@$core.Deprecated('Use vectorizeResponseDescriptor instead')
const VectorizeResponse$json = {
  '1': 'VectorizeResponse',
  '2': [
    {'1': 'vector', '3': 1, '4': 3, '5': 1, '10': 'vector'},
  ],
};

/// Descriptor for `VectorizeResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List vectorizeResponseDescriptor = $convert.base64Decode(
    'ChFWZWN0b3JpemVSZXNwb25zZRIWCgZ2ZWN0b3IYASADKAFSBnZlY3Rvcg==');

@$core.Deprecated('Use searchTextRequestDescriptor instead')
const SearchTextRequest$json = {
  '1': 'SearchTextRequest',
  '2': [
    {'1': 'collection', '3': 1, '4': 1, '5': 9, '10': 'collection'},
    {'1': 'text', '3': 2, '4': 1, '5': 9, '10': 'text'},
    {'1': 'top_k', '3': 3, '4': 1, '5': 13, '10': 'topK'},
    {
      '1': 'filter',
      '3': 4,
      '4': 3,
      '5': 11,
      '6': '.hyperspace.SearchTextRequest.FilterEntry',
      '10': 'filter'
    },
    {
      '1': 'filters',
      '3': 5,
      '4': 3,
      '5': 11,
      '6': '.hyperspace.Filter',
      '10': 'filters'
    },
    {
      '1': 'bm25_options',
      '3': 6,
      '4': 1,
      '5': 11,
      '6': '.hyperspace.Bm25Options',
      '9': 0,
      '10': 'bm25Options',
      '17': true
    },
    {
      '1': 'hybrid_alpha',
      '3': 7,
      '4': 1,
      '5': 2,
      '9': 1,
      '10': 'hybridAlpha',
      '17': true
    },
  ],
  '3': [SearchTextRequest_FilterEntry$json],
  '8': [
    {'1': '_bm25_options'},
    {'1': '_hybrid_alpha'},
  ],
};

@$core.Deprecated('Use searchTextRequestDescriptor instead')
const SearchTextRequest_FilterEntry$json = {
  '1': 'FilterEntry',
  '2': [
    {'1': 'key', '3': 1, '4': 1, '5': 9, '10': 'key'},
    {'1': 'value', '3': 2, '4': 1, '5': 9, '10': 'value'},
  ],
  '7': {'7': true},
};

/// Descriptor for `SearchTextRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List searchTextRequestDescriptor = $convert.base64Decode(
    'ChFTZWFyY2hUZXh0UmVxdWVzdBIeCgpjb2xsZWN0aW9uGAEgASgJUgpjb2xsZWN0aW9uEhIKBH'
    'RleHQYAiABKAlSBHRleHQSEwoFdG9wX2sYAyABKA1SBHRvcEsSQQoGZmlsdGVyGAQgAygLMiku'
    'aHlwZXJzcGFjZS5TZWFyY2hUZXh0UmVxdWVzdC5GaWx0ZXJFbnRyeVIGZmlsdGVyEiwKB2ZpbH'
    'RlcnMYBSADKAsyEi5oeXBlcnNwYWNlLkZpbHRlclIHZmlsdGVycxI/CgxibTI1X29wdGlvbnMY'
    'BiABKAsyFy5oeXBlcnNwYWNlLkJtMjVPcHRpb25zSABSC2JtMjVPcHRpb25ziAEBEiYKDGh5Yn'
    'JpZF9hbHBoYRgHIAEoAkgBUgtoeWJyaWRBbHBoYYgBARo5CgtGaWx0ZXJFbnRyeRIQCgNrZXkY'
    'ASABKAlSA2tleRIUCgV2YWx1ZRgCIAEoCVIFdmFsdWU6AjgBQg8KDV9ibTI1X29wdGlvbnNCDw'
    'oNX2h5YnJpZF9hbHBoYQ==');

@$core.Deprecated('Use bm25OptionsDescriptor instead')
const Bm25Options$json = {
  '1': 'Bm25Options',
  '2': [
    {'1': 'method', '3': 1, '4': 1, '5': 9, '9': 0, '10': 'method', '17': true},
    {'1': 'k1', '3': 2, '4': 1, '5': 2, '9': 1, '10': 'k1', '17': true},
    {'1': 'b', '3': 3, '4': 1, '5': 2, '9': 2, '10': 'b', '17': true},
    {'1': 'delta', '3': 4, '4': 1, '5': 2, '9': 3, '10': 'delta', '17': true},
    {
      '1': 'language',
      '3': 5,
      '4': 1,
      '5': 9,
      '9': 4,
      '10': 'language',
      '17': true
    },
    {
      '1': 'ngrams',
      '3': 6,
      '4': 1,
      '5': 13,
      '9': 5,
      '10': 'ngrams',
      '17': true
    },
    {
      '1': 'fusion_method',
      '3': 7,
      '4': 1,
      '5': 9,
      '9': 6,
      '10': 'fusionMethod',
      '17': true
    },
  ],
  '8': [
    {'1': '_method'},
    {'1': '_k1'},
    {'1': '_b'},
    {'1': '_delta'},
    {'1': '_language'},
    {'1': '_ngrams'},
    {'1': '_fusion_method'},
  ],
};

/// Descriptor for `Bm25Options`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List bm25OptionsDescriptor = $convert.base64Decode(
    'CgtCbTI1T3B0aW9ucxIbCgZtZXRob2QYASABKAlIAFIGbWV0aG9kiAEBEhMKAmsxGAIgASgCSA'
    'FSAmsxiAEBEhEKAWIYAyABKAJIAlIBYogBARIZCgVkZWx0YRgEIAEoAkgDUgVkZWx0YYgBARIf'
    'CghsYW5ndWFnZRgFIAEoCUgEUghsYW5ndWFnZYgBARIbCgZuZ3JhbXMYBiABKA1IBVIGbmdyYW'
    '1ziAEBEigKDWZ1c2lvbl9tZXRob2QYByABKAlIBlIMZnVzaW9uTWV0aG9kiAEBQgkKB19tZXRo'
    'b2RCBQoDX2sxQgQKAl9iQggKBl9kZWx0YUILCglfbGFuZ3VhZ2VCCQoHX25ncmFtc0IQCg5fZn'
    'VzaW9uX21ldGhvZA==');

@$core.Deprecated('Use insertResponseDescriptor instead')
const InsertResponse$json = {
  '1': 'InsertResponse',
  '2': [
    {'1': 'success', '3': 1, '4': 1, '5': 8, '10': 'success'},
  ],
};

/// Descriptor for `InsertResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List insertResponseDescriptor = $convert
    .base64Decode('Cg5JbnNlcnRSZXNwb25zZRIYCgdzdWNjZXNzGAEgASgIUgdzdWNjZXNz');

@$core.Deprecated('Use deleteRequestDescriptor instead')
const DeleteRequest$json = {
  '1': 'DeleteRequest',
  '2': [
    {'1': 'collection', '3': 1, '4': 1, '5': 9, '10': 'collection'},
    {'1': 'id', '3': 2, '4': 1, '5': 13, '10': 'id'},
  ],
};

/// Descriptor for `DeleteRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List deleteRequestDescriptor = $convert.base64Decode(
    'Cg1EZWxldGVSZXF1ZXN0Eh4KCmNvbGxlY3Rpb24YASABKAlSCmNvbGxlY3Rpb24SDgoCaWQYAi'
    'ABKA1SAmlk');

@$core.Deprecated('Use deleteResponseDescriptor instead')
const DeleteResponse$json = {
  '1': 'DeleteResponse',
  '2': [
    {'1': 'success', '3': 1, '4': 1, '5': 8, '10': 'success'},
  ],
};

/// Descriptor for `DeleteResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List deleteResponseDescriptor = $convert
    .base64Decode('Cg5EZWxldGVSZXNwb25zZRIYCgdzdWNjZXNzGAEgASgIUgdzdWNjZXNz');

@$core.Deprecated('Use getPointsRequestDescriptor instead')
const GetPointsRequest$json = {
  '1': 'GetPointsRequest',
  '2': [
    {'1': 'collection', '3': 1, '4': 1, '5': 9, '10': 'collection'},
    {'1': 'ids', '3': 2, '4': 3, '5': 13, '10': 'ids'},
  ],
};

/// Descriptor for `GetPointsRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getPointsRequestDescriptor = $convert.base64Decode(
    'ChBHZXRQb2ludHNSZXF1ZXN0Eh4KCmNvbGxlY3Rpb24YASABKAlSCmNvbGxlY3Rpb24SEAoDaW'
    'RzGAIgAygNUgNpZHM=');

@$core.Deprecated('Use getPointsResponseDescriptor instead')
const GetPointsResponse$json = {
  '1': 'GetPointsResponse',
  '2': [
    {
      '1': 'points',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.hyperspace.VectorData',
      '10': 'points'
    },
  ],
};

/// Descriptor for `GetPointsResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getPointsResponseDescriptor = $convert.base64Decode(
    'ChFHZXRQb2ludHNSZXNwb25zZRIuCgZwb2ludHMYASADKAsyFi5oeXBlcnNwYWNlLlZlY3Rvck'
    'RhdGFSBnBvaW50cw==');

@$core.Deprecated('Use updatePayloadRequestDescriptor instead')
const UpdatePayloadRequest$json = {
  '1': 'UpdatePayloadRequest',
  '2': [
    {'1': 'collection', '3': 1, '4': 1, '5': 9, '10': 'collection'},
    {'1': 'id', '3': 2, '4': 1, '5': 13, '10': 'id'},
    {
      '1': 'metadata',
      '3': 3,
      '4': 3,
      '5': 11,
      '6': '.hyperspace.UpdatePayloadRequest.MetadataEntry',
      '10': 'metadata'
    },
    {
      '1': 'typed_metadata',
      '3': 4,
      '4': 3,
      '5': 11,
      '6': '.hyperspace.UpdatePayloadRequest.TypedMetadataEntry',
      '10': 'typedMetadata'
    },
  ],
  '3': [
    UpdatePayloadRequest_MetadataEntry$json,
    UpdatePayloadRequest_TypedMetadataEntry$json
  ],
};

@$core.Deprecated('Use updatePayloadRequestDescriptor instead')
const UpdatePayloadRequest_MetadataEntry$json = {
  '1': 'MetadataEntry',
  '2': [
    {'1': 'key', '3': 1, '4': 1, '5': 9, '10': 'key'},
    {'1': 'value', '3': 2, '4': 1, '5': 9, '10': 'value'},
  ],
  '7': {'7': true},
};

@$core.Deprecated('Use updatePayloadRequestDescriptor instead')
const UpdatePayloadRequest_TypedMetadataEntry$json = {
  '1': 'TypedMetadataEntry',
  '2': [
    {'1': 'key', '3': 1, '4': 1, '5': 9, '10': 'key'},
    {
      '1': 'value',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.hyperspace.MetadataValue',
      '10': 'value'
    },
  ],
  '7': {'7': true},
};

/// Descriptor for `UpdatePayloadRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List updatePayloadRequestDescriptor = $convert.base64Decode(
    'ChRVcGRhdGVQYXlsb2FkUmVxdWVzdBIeCgpjb2xsZWN0aW9uGAEgASgJUgpjb2xsZWN0aW9uEg'
    '4KAmlkGAIgASgNUgJpZBJKCghtZXRhZGF0YRgDIAMoCzIuLmh5cGVyc3BhY2UuVXBkYXRlUGF5'
    'bG9hZFJlcXVlc3QuTWV0YWRhdGFFbnRyeVIIbWV0YWRhdGESWgoOdHlwZWRfbWV0YWRhdGEYBC'
    'ADKAsyMy5oeXBlcnNwYWNlLlVwZGF0ZVBheWxvYWRSZXF1ZXN0LlR5cGVkTWV0YWRhdGFFbnRy'
    'eVINdHlwZWRNZXRhZGF0YRo7Cg1NZXRhZGF0YUVudHJ5EhAKA2tleRgBIAEoCVIDa2V5EhQKBX'
    'ZhbHVlGAIgASgJUgV2YWx1ZToCOAEaWwoSVHlwZWRNZXRhZGF0YUVudHJ5EhAKA2tleRgBIAEo'
    'CVIDa2V5Ei8KBXZhbHVlGAIgASgLMhkuaHlwZXJzcGFjZS5NZXRhZGF0YVZhbHVlUgV2YWx1ZT'
    'oCOAE=');

@$core.Deprecated('Use scrollRequestDescriptor instead')
const ScrollRequest$json = {
  '1': 'ScrollRequest',
  '2': [
    {'1': 'collection', '3': 1, '4': 1, '5': 9, '10': 'collection'},
    {'1': 'limit', '3': 2, '4': 1, '5': 13, '10': 'limit'},
    {'1': 'offset', '3': 3, '4': 1, '5': 13, '10': 'offset'},
    {
      '1': 'filters',
      '3': 4,
      '4': 3,
      '5': 11,
      '6': '.hyperspace.Filter',
      '10': 'filters'
    },
  ],
};

/// Descriptor for `ScrollRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List scrollRequestDescriptor = $convert.base64Decode(
    'Cg1TY3JvbGxSZXF1ZXN0Eh4KCmNvbGxlY3Rpb24YASABKAlSCmNvbGxlY3Rpb24SFAoFbGltaX'
    'QYAiABKA1SBWxpbWl0EhYKBm9mZnNldBgDIAEoDVIGb2Zmc2V0EiwKB2ZpbHRlcnMYBCADKAsy'
    'Ei5oeXBlcnNwYWNlLkZpbHRlclIHZmlsdGVycw==');

@$core.Deprecated('Use scrollResponseDescriptor instead')
const ScrollResponse$json = {
  '1': 'ScrollResponse',
  '2': [
    {
      '1': 'points',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.hyperspace.VectorData',
      '10': 'points'
    },
  ],
};

/// Descriptor for `ScrollResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List scrollResponseDescriptor = $convert.base64Decode(
    'Cg5TY3JvbGxSZXNwb25zZRIuCgZwb2ludHMYASADKAsyFi5oeXBlcnNwYWNlLlZlY3RvckRhdG'
    'FSBnBvaW50cw==');

@$core.Deprecated('Use countRequestDescriptor instead')
const CountRequest$json = {
  '1': 'CountRequest',
  '2': [
    {'1': 'collection', '3': 1, '4': 1, '5': 9, '10': 'collection'},
    {
      '1': 'filters',
      '3': 2,
      '4': 3,
      '5': 11,
      '6': '.hyperspace.Filter',
      '10': 'filters'
    },
  ],
};

/// Descriptor for `CountRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List countRequestDescriptor = $convert.base64Decode(
    'CgxDb3VudFJlcXVlc3QSHgoKY29sbGVjdGlvbhgBIAEoCVIKY29sbGVjdGlvbhIsCgdmaWx0ZX'
    'JzGAIgAygLMhIuaHlwZXJzcGFjZS5GaWx0ZXJSB2ZpbHRlcnM=');

@$core.Deprecated('Use countResponseDescriptor instead')
const CountResponse$json = {
  '1': 'CountResponse',
  '2': [
    {'1': 'count', '3': 1, '4': 1, '5': 4, '10': 'count'},
  ],
};

/// Descriptor for `CountResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List countResponseDescriptor = $convert
    .base64Decode('Cg1Db3VudFJlc3BvbnNlEhQKBWNvdW50GAEgASgEUgVjb3VudA==');

@$core.Deprecated('Use healthCheckResponseDescriptor instead')
const HealthCheckResponse$json = {
  '1': 'HealthCheckResponse',
  '2': [
    {'1': 'status', '3': 1, '4': 1, '5': 9, '10': 'status'},
  ],
};

/// Descriptor for `HealthCheckResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List healthCheckResponseDescriptor =
    $convert.base64Decode(
        'ChNIZWFsdGhDaGVja1Jlc3BvbnNlEhYKBnN0YXR1cxgBIAEoCVIGc3RhdHVz');

@$core.Deprecated('Use searchRequestDescriptor instead')
const SearchRequest$json = {
  '1': 'SearchRequest',
  '2': [
    {'1': 'collection', '3': 1, '4': 1, '5': 9, '10': 'collection'},
    {'1': 'vector', '3': 2, '4': 3, '5': 1, '10': 'vector'},
    {'1': 'top_k', '3': 3, '4': 1, '5': 13, '10': 'topK'},
    {
      '1': 'filter',
      '3': 4,
      '4': 3,
      '5': 11,
      '6': '.hyperspace.SearchRequest.FilterEntry',
      '10': 'filter'
    },
    {
      '1': 'filters',
      '3': 5,
      '4': 3,
      '5': 11,
      '6': '.hyperspace.Filter',
      '10': 'filters'
    },
    {
      '1': 'hybrid_query',
      '3': 6,
      '4': 1,
      '5': 9,
      '9': 0,
      '10': 'hybridQuery',
      '17': true
    },
    {
      '1': 'hybrid_alpha',
      '3': 7,
      '4': 1,
      '5': 2,
      '9': 1,
      '10': 'hybridAlpha',
      '17': true
    },
    {'1': 'use_wasserstein', '3': 8, '4': 1, '5': 8, '10': 'useWasserstein'},
    {
      '1': 'bm25_options',
      '3': 9,
      '4': 1,
      '5': 11,
      '6': '.hyperspace.Bm25Options',
      '9': 2,
      '10': 'bm25Options',
      '17': true
    },
    {
      '1': 'mrl_dimension',
      '3': 10,
      '4': 1,
      '5': 13,
      '9': 3,
      '10': 'mrlDimension',
      '17': true
    },
  ],
  '3': [SearchRequest_FilterEntry$json],
  '8': [
    {'1': '_hybrid_query'},
    {'1': '_hybrid_alpha'},
    {'1': '_bm25_options'},
    {'1': '_mrl_dimension'},
  ],
};

@$core.Deprecated('Use searchRequestDescriptor instead')
const SearchRequest_FilterEntry$json = {
  '1': 'FilterEntry',
  '2': [
    {'1': 'key', '3': 1, '4': 1, '5': 9, '10': 'key'},
    {'1': 'value', '3': 2, '4': 1, '5': 9, '10': 'value'},
  ],
  '7': {'7': true},
};

/// Descriptor for `SearchRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List searchRequestDescriptor = $convert.base64Decode(
    'Cg1TZWFyY2hSZXF1ZXN0Eh4KCmNvbGxlY3Rpb24YASABKAlSCmNvbGxlY3Rpb24SFgoGdmVjdG'
    '9yGAIgAygBUgZ2ZWN0b3ISEwoFdG9wX2sYAyABKA1SBHRvcEsSPQoGZmlsdGVyGAQgAygLMiUu'
    'aHlwZXJzcGFjZS5TZWFyY2hSZXF1ZXN0LkZpbHRlckVudHJ5UgZmaWx0ZXISLAoHZmlsdGVycx'
    'gFIAMoCzISLmh5cGVyc3BhY2UuRmlsdGVyUgdmaWx0ZXJzEiYKDGh5YnJpZF9xdWVyeRgGIAEo'
    'CUgAUgtoeWJyaWRRdWVyeYgBARImCgxoeWJyaWRfYWxwaGEYByABKAJIAVILaHlicmlkQWxwaG'
    'GIAQESJwoPdXNlX3dhc3NlcnN0ZWluGAggASgIUg51c2VXYXNzZXJzdGVpbhI/CgxibTI1X29w'
    'dGlvbnMYCSABKAsyFy5oeXBlcnNwYWNlLkJtMjVPcHRpb25zSAJSC2JtMjVPcHRpb25ziAEBEi'
    'gKDW1ybF9kaW1lbnNpb24YCiABKA1IA1IMbXJsRGltZW5zaW9uiAEBGjkKC0ZpbHRlckVudHJ5'
    'EhAKA2tleRgBIAEoCVIDa2V5EhQKBXZhbHVlGAIgASgJUgV2YWx1ZToCOAFCDwoNX2h5YnJpZF'
    '9xdWVyeUIPCg1faHlicmlkX2FscGhhQg8KDV9ibTI1X29wdGlvbnNCEAoOX21ybF9kaW1lbnNp'
    'b24=');

@$core.Deprecated('Use filterDescriptor instead')
const Filter$json = {
  '1': 'Filter',
  '2': [
    {
      '1': 'match',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.hyperspace.Match',
      '9': 0,
      '10': 'match'
    },
    {
      '1': 'range',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.hyperspace.Range',
      '9': 0,
      '10': 'range'
    },
    {
      '1': 'in_cone',
      '3': 3,
      '4': 1,
      '5': 11,
      '6': '.hyperspace.InCone',
      '9': 0,
      '10': 'inCone'
    },
    {
      '1': 'in_box',
      '3': 4,
      '4': 1,
      '5': 11,
      '6': '.hyperspace.InBox',
      '9': 0,
      '10': 'inBox'
    },
    {
      '1': 'in_ball',
      '3': 5,
      '4': 1,
      '5': 11,
      '6': '.hyperspace.InBall',
      '9': 0,
      '10': 'inBall'
    },
    {
      '1': 'and_op',
      '3': 6,
      '4': 1,
      '5': 11,
      '6': '.hyperspace.FilterAnd',
      '9': 0,
      '10': 'andOp'
    },
    {
      '1': 'or_op',
      '3': 7,
      '4': 1,
      '5': 11,
      '6': '.hyperspace.FilterOr',
      '9': 0,
      '10': 'orOp'
    },
    {
      '1': 'not_op',
      '3': 8,
      '4': 1,
      '5': 11,
      '6': '.hyperspace.FilterNot',
      '9': 0,
      '10': 'notOp'
    },
  ],
  '8': [
    {'1': 'condition'},
  ],
};

/// Descriptor for `Filter`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List filterDescriptor = $convert.base64Decode(
    'CgZGaWx0ZXISKQoFbWF0Y2gYASABKAsyES5oeXBlcnNwYWNlLk1hdGNoSABSBW1hdGNoEikKBX'
    'JhbmdlGAIgASgLMhEuaHlwZXJzcGFjZS5SYW5nZUgAUgVyYW5nZRItCgdpbl9jb25lGAMgASgL'
    'MhIuaHlwZXJzcGFjZS5JbkNvbmVIAFIGaW5Db25lEioKBmluX2JveBgEIAEoCzIRLmh5cGVyc3'
    'BhY2UuSW5Cb3hIAFIFaW5Cb3gSLQoHaW5fYmFsbBgFIAEoCzISLmh5cGVyc3BhY2UuSW5CYWxs'
    'SABSBmluQmFsbBIuCgZhbmRfb3AYBiABKAsyFS5oeXBlcnNwYWNlLkZpbHRlckFuZEgAUgVhbm'
    'RPcBIrCgVvcl9vcBgHIAEoCzIULmh5cGVyc3BhY2UuRmlsdGVyT3JIAFIEb3JPcBIuCgZub3Rf'
    'b3AYCCABKAsyFS5oeXBlcnNwYWNlLkZpbHRlck5vdEgAUgVub3RPcEILCgljb25kaXRpb24=');

@$core.Deprecated('Use filterAndDescriptor instead')
const FilterAnd$json = {
  '1': 'FilterAnd',
  '2': [
    {
      '1': 'conditions',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.hyperspace.Filter',
      '10': 'conditions'
    },
  ],
};

/// Descriptor for `FilterAnd`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List filterAndDescriptor = $convert.base64Decode(
    'CglGaWx0ZXJBbmQSMgoKY29uZGl0aW9ucxgBIAMoCzISLmh5cGVyc3BhY2UuRmlsdGVyUgpjb2'
    '5kaXRpb25z');

@$core.Deprecated('Use filterOrDescriptor instead')
const FilterOr$json = {
  '1': 'FilterOr',
  '2': [
    {
      '1': 'conditions',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.hyperspace.Filter',
      '10': 'conditions'
    },
  ],
};

/// Descriptor for `FilterOr`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List filterOrDescriptor = $convert.base64Decode(
    'CghGaWx0ZXJPchIyCgpjb25kaXRpb25zGAEgAygLMhIuaHlwZXJzcGFjZS5GaWx0ZXJSCmNvbm'
    'RpdGlvbnM=');

@$core.Deprecated('Use filterNotDescriptor instead')
const FilterNot$json = {
  '1': 'FilterNot',
  '2': [
    {
      '1': 'condition',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.hyperspace.Filter',
      '10': 'condition'
    },
  ],
};

/// Descriptor for `FilterNot`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List filterNotDescriptor = $convert.base64Decode(
    'CglGaWx0ZXJOb3QSMAoJY29uZGl0aW9uGAEgASgLMhIuaHlwZXJzcGFjZS5GaWx0ZXJSCWNvbm'
    'RpdGlvbg==');

@$core.Deprecated('Use matchDescriptor instead')
const Match$json = {
  '1': 'Match',
  '2': [
    {'1': 'key', '3': 1, '4': 1, '5': 9, '10': 'key'},
    {'1': 'value', '3': 2, '4': 1, '5': 9, '10': 'value'},
  ],
};

/// Descriptor for `Match`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List matchDescriptor = $convert.base64Decode(
    'CgVNYXRjaBIQCgNrZXkYASABKAlSA2tleRIUCgV2YWx1ZRgCIAEoCVIFdmFsdWU=');

@$core.Deprecated('Use rangeDescriptor instead')
const Range$json = {
  '1': 'Range',
  '2': [
    {'1': 'key', '3': 1, '4': 1, '5': 9, '10': 'key'},
    {'1': 'gte', '3': 2, '4': 1, '5': 3, '9': 0, '10': 'gte', '17': true},
    {'1': 'lte', '3': 3, '4': 1, '5': 3, '9': 1, '10': 'lte', '17': true},
    {
      '1': 'gte_f64',
      '3': 4,
      '4': 1,
      '5': 1,
      '9': 2,
      '10': 'gteF64',
      '17': true
    },
    {
      '1': 'lte_f64',
      '3': 5,
      '4': 1,
      '5': 1,
      '9': 3,
      '10': 'lteF64',
      '17': true
    },
  ],
  '8': [
    {'1': '_gte'},
    {'1': '_lte'},
    {'1': '_gte_f64'},
    {'1': '_lte_f64'},
  ],
};

/// Descriptor for `Range`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List rangeDescriptor = $convert.base64Decode(
    'CgVSYW5nZRIQCgNrZXkYASABKAlSA2tleRIVCgNndGUYAiABKANIAFIDZ3RliAEBEhUKA2x0ZR'
    'gDIAEoA0gBUgNsdGWIAQESHAoHZ3RlX2Y2NBgEIAEoAUgCUgZndGVGNjSIAQESHAoHbHRlX2Y2'
    'NBgFIAEoAUgDUgZsdGVGNjSIAQFCBgoEX2d0ZUIGCgRfbHRlQgoKCF9ndGVfZjY0QgoKCF9sdG'
    'VfZjY0');

@$core.Deprecated('Use inConeDescriptor instead')
const InCone$json = {
  '1': 'InCone',
  '2': [
    {'1': 'axes', '3': 1, '4': 3, '5': 1, '10': 'axes'},
    {'1': 'apertures', '3': 2, '4': 3, '5': 1, '10': 'apertures'},
    {'1': 'cen', '3': 3, '4': 1, '5': 1, '10': 'cen'},
  ],
};

/// Descriptor for `InCone`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List inConeDescriptor = $convert.base64Decode(
    'CgZJbkNvbmUSEgoEYXhlcxgBIAMoAVIEYXhlcxIcCglhcGVydHVyZXMYAiADKAFSCWFwZXJ0dX'
    'JlcxIQCgNjZW4YAyABKAFSA2Nlbg==');

@$core.Deprecated('Use inBoxDescriptor instead')
const InBox$json = {
  '1': 'InBox',
  '2': [
    {'1': 'min_bounds', '3': 1, '4': 3, '5': 1, '10': 'minBounds'},
    {'1': 'max_bounds', '3': 2, '4': 3, '5': 1, '10': 'maxBounds'},
  ],
};

/// Descriptor for `InBox`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List inBoxDescriptor = $convert.base64Decode(
    'CgVJbkJveBIdCgptaW5fYm91bmRzGAEgAygBUgltaW5Cb3VuZHMSHQoKbWF4X2JvdW5kcxgCIA'
    'MoAVIJbWF4Qm91bmRz');

@$core.Deprecated('Use inBallDescriptor instead')
const InBall$json = {
  '1': 'InBall',
  '2': [
    {'1': 'center', '3': 1, '4': 3, '5': 1, '10': 'center'},
    {'1': 'radius', '3': 2, '4': 1, '5': 1, '10': 'radius'},
  ],
};

/// Descriptor for `InBall`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List inBallDescriptor = $convert.base64Decode(
    'CgZJbkJhbGwSFgoGY2VudGVyGAEgAygBUgZjZW50ZXISFgoGcmFkaXVzGAIgASgBUgZyYWRpdX'
    'M=');

@$core.Deprecated('Use searchResponseDescriptor instead')
const SearchResponse$json = {
  '1': 'SearchResponse',
  '2': [
    {
      '1': 'results',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.hyperspace.SearchResult',
      '10': 'results'
    },
  ],
};

/// Descriptor for `SearchResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List searchResponseDescriptor = $convert.base64Decode(
    'Cg5TZWFyY2hSZXNwb25zZRIyCgdyZXN1bHRzGAEgAygLMhguaHlwZXJzcGFjZS5TZWFyY2hSZX'
    'N1bHRSB3Jlc3VsdHM=');

@$core.Deprecated('Use batchSearchRequestDescriptor instead')
const BatchSearchRequest$json = {
  '1': 'BatchSearchRequest',
  '2': [
    {
      '1': 'searches',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.hyperspace.SearchRequest',
      '10': 'searches'
    },
  ],
};

/// Descriptor for `BatchSearchRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List batchSearchRequestDescriptor = $convert.base64Decode(
    'ChJCYXRjaFNlYXJjaFJlcXVlc3QSNQoIc2VhcmNoZXMYASADKAsyGS5oeXBlcnNwYWNlLlNlYX'
    'JjaFJlcXVlc3RSCHNlYXJjaGVz');

@$core.Deprecated('Use batchSearchResponseDescriptor instead')
const BatchSearchResponse$json = {
  '1': 'BatchSearchResponse',
  '2': [
    {
      '1': 'responses',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.hyperspace.SearchResponse',
      '10': 'responses'
    },
  ],
};

/// Descriptor for `BatchSearchResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List batchSearchResponseDescriptor = $convert.base64Decode(
    'ChNCYXRjaFNlYXJjaFJlc3BvbnNlEjgKCXJlc3BvbnNlcxgBIAMoCzIaLmh5cGVyc3BhY2UuU2'
    'VhcmNoUmVzcG9uc2VSCXJlc3BvbnNlcw==');

@$core.Deprecated('Use searchMultiCollectionRequestDescriptor instead')
const SearchMultiCollectionRequest$json = {
  '1': 'SearchMultiCollectionRequest',
  '2': [
    {'1': 'collections', '3': 1, '4': 3, '5': 9, '10': 'collections'},
    {'1': 'vector', '3': 2, '4': 3, '5': 1, '10': 'vector'},
    {'1': 'top_k', '3': 3, '4': 1, '5': 13, '10': 'topK'},
  ],
};

/// Descriptor for `SearchMultiCollectionRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List searchMultiCollectionRequestDescriptor =
    $convert.base64Decode(
        'ChxTZWFyY2hNdWx0aUNvbGxlY3Rpb25SZXF1ZXN0EiAKC2NvbGxlY3Rpb25zGAEgAygJUgtjb2'
        'xsZWN0aW9ucxIWCgZ2ZWN0b3IYAiADKAFSBnZlY3RvchITCgV0b3BfaxgDIAEoDVIEdG9wSw==');

@$core.Deprecated('Use searchMultiCollectionResponseDescriptor instead')
const SearchMultiCollectionResponse$json = {
  '1': 'SearchMultiCollectionResponse',
  '2': [
    {
      '1': 'responses',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.hyperspace.SearchMultiCollectionResponse.ResponsesEntry',
      '10': 'responses'
    },
  ],
  '3': [SearchMultiCollectionResponse_ResponsesEntry$json],
};

@$core.Deprecated('Use searchMultiCollectionResponseDescriptor instead')
const SearchMultiCollectionResponse_ResponsesEntry$json = {
  '1': 'ResponsesEntry',
  '2': [
    {'1': 'key', '3': 1, '4': 1, '5': 9, '10': 'key'},
    {
      '1': 'value',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.hyperspace.SearchResponse',
      '10': 'value'
    },
  ],
  '7': {'7': true},
};

/// Descriptor for `SearchMultiCollectionResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List searchMultiCollectionResponseDescriptor = $convert.base64Decode(
    'Ch1TZWFyY2hNdWx0aUNvbGxlY3Rpb25SZXNwb25zZRJWCglyZXNwb25zZXMYASADKAsyOC5oeX'
    'BlcnNwYWNlLlNlYXJjaE11bHRpQ29sbGVjdGlvblJlc3BvbnNlLlJlc3BvbnNlc0VudHJ5Ugly'
    'ZXNwb25zZXMaWAoOUmVzcG9uc2VzRW50cnkSEAoDa2V5GAEgASgJUgNrZXkSMAoFdmFsdWUYAi'
    'ABKAsyGi5oeXBlcnNwYWNlLlNlYXJjaFJlc3BvbnNlUgV2YWx1ZToCOAE=');

@$core.Deprecated('Use searchResultDescriptor instead')
const SearchResult$json = {
  '1': 'SearchResult',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 13, '10': 'id'},
    {'1': 'distance', '3': 2, '4': 1, '5': 1, '10': 'distance'},
    {
      '1': 'metadata',
      '3': 3,
      '4': 3,
      '5': 11,
      '6': '.hyperspace.SearchResult.MetadataEntry',
      '10': 'metadata'
    },
    {
      '1': 'typed_metadata',
      '3': 4,
      '4': 3,
      '5': 11,
      '6': '.hyperspace.SearchResult.TypedMetadataEntry',
      '10': 'typedMetadata'
    },
  ],
  '3': [SearchResult_MetadataEntry$json, SearchResult_TypedMetadataEntry$json],
};

@$core.Deprecated('Use searchResultDescriptor instead')
const SearchResult_MetadataEntry$json = {
  '1': 'MetadataEntry',
  '2': [
    {'1': 'key', '3': 1, '4': 1, '5': 9, '10': 'key'},
    {'1': 'value', '3': 2, '4': 1, '5': 9, '10': 'value'},
  ],
  '7': {'7': true},
};

@$core.Deprecated('Use searchResultDescriptor instead')
const SearchResult_TypedMetadataEntry$json = {
  '1': 'TypedMetadataEntry',
  '2': [
    {'1': 'key', '3': 1, '4': 1, '5': 9, '10': 'key'},
    {
      '1': 'value',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.hyperspace.MetadataValue',
      '10': 'value'
    },
  ],
  '7': {'7': true},
};

/// Descriptor for `SearchResult`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List searchResultDescriptor = $convert.base64Decode(
    'CgxTZWFyY2hSZXN1bHQSDgoCaWQYASABKA1SAmlkEhoKCGRpc3RhbmNlGAIgASgBUghkaXN0YW'
    '5jZRJCCghtZXRhZGF0YRgDIAMoCzImLmh5cGVyc3BhY2UuU2VhcmNoUmVzdWx0Lk1ldGFkYXRh'
    'RW50cnlSCG1ldGFkYXRhElIKDnR5cGVkX21ldGFkYXRhGAQgAygLMisuaHlwZXJzcGFjZS5TZW'
    'FyY2hSZXN1bHQuVHlwZWRNZXRhZGF0YUVudHJ5Ug10eXBlZE1ldGFkYXRhGjsKDU1ldGFkYXRh'
    'RW50cnkSEAoDa2V5GAEgASgJUgNrZXkSFAoFdmFsdWUYAiABKAlSBXZhbHVlOgI4ARpbChJUeX'
    'BlZE1ldGFkYXRhRW50cnkSEAoDa2V5GAEgASgJUgNrZXkSLwoFdmFsdWUYAiABKAsyGS5oeXBl'
    'cnNwYWNlLk1ldGFkYXRhVmFsdWVSBXZhbHVlOgI4AQ==');

@$core.Deprecated('Use getNodeRequestDescriptor instead')
const GetNodeRequest$json = {
  '1': 'GetNodeRequest',
  '2': [
    {'1': 'collection', '3': 1, '4': 1, '5': 9, '10': 'collection'},
    {'1': 'id', '3': 2, '4': 1, '5': 13, '10': 'id'},
    {'1': 'layer', '3': 3, '4': 1, '5': 13, '10': 'layer'},
  ],
};

/// Descriptor for `GetNodeRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getNodeRequestDescriptor = $convert.base64Decode(
    'Cg5HZXROb2RlUmVxdWVzdBIeCgpjb2xsZWN0aW9uGAEgASgJUgpjb2xsZWN0aW9uEg4KAmlkGA'
    'IgASgNUgJpZBIUCgVsYXllchgDIAEoDVIFbGF5ZXI=');

@$core.Deprecated('Use graphNodeDescriptor instead')
const GraphNode$json = {
  '1': 'GraphNode',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 13, '10': 'id'},
    {'1': 'layer', '3': 2, '4': 1, '5': 13, '10': 'layer'},
    {'1': 'neighbors', '3': 3, '4': 3, '5': 13, '10': 'neighbors'},
    {
      '1': 'metadata',
      '3': 4,
      '4': 3,
      '5': 11,
      '6': '.hyperspace.GraphNode.MetadataEntry',
      '10': 'metadata'
    },
    {
      '1': 'typed_metadata',
      '3': 5,
      '4': 3,
      '5': 11,
      '6': '.hyperspace.GraphNode.TypedMetadataEntry',
      '10': 'typedMetadata'
    },
  ],
  '3': [GraphNode_MetadataEntry$json, GraphNode_TypedMetadataEntry$json],
};

@$core.Deprecated('Use graphNodeDescriptor instead')
const GraphNode_MetadataEntry$json = {
  '1': 'MetadataEntry',
  '2': [
    {'1': 'key', '3': 1, '4': 1, '5': 9, '10': 'key'},
    {'1': 'value', '3': 2, '4': 1, '5': 9, '10': 'value'},
  ],
  '7': {'7': true},
};

@$core.Deprecated('Use graphNodeDescriptor instead')
const GraphNode_TypedMetadataEntry$json = {
  '1': 'TypedMetadataEntry',
  '2': [
    {'1': 'key', '3': 1, '4': 1, '5': 9, '10': 'key'},
    {
      '1': 'value',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.hyperspace.MetadataValue',
      '10': 'value'
    },
  ],
  '7': {'7': true},
};

/// Descriptor for `GraphNode`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List graphNodeDescriptor = $convert.base64Decode(
    'CglHcmFwaE5vZGUSDgoCaWQYASABKA1SAmlkEhQKBWxheWVyGAIgASgNUgVsYXllchIcCgluZW'
    'lnaGJvcnMYAyADKA1SCW5laWdoYm9ycxI/CghtZXRhZGF0YRgEIAMoCzIjLmh5cGVyc3BhY2Uu'
    'R3JhcGhOb2RlLk1ldGFkYXRhRW50cnlSCG1ldGFkYXRhEk8KDnR5cGVkX21ldGFkYXRhGAUgAy'
    'gLMiguaHlwZXJzcGFjZS5HcmFwaE5vZGUuVHlwZWRNZXRhZGF0YUVudHJ5Ug10eXBlZE1ldGFk'
    'YXRhGjsKDU1ldGFkYXRhRW50cnkSEAoDa2V5GAEgASgJUgNrZXkSFAoFdmFsdWUYAiABKAlSBX'
    'ZhbHVlOgI4ARpbChJUeXBlZE1ldGFkYXRhRW50cnkSEAoDa2V5GAEgASgJUgNrZXkSLwoFdmFs'
    'dWUYAiABKAsyGS5oeXBlcnNwYWNlLk1ldGFkYXRhVmFsdWVSBXZhbHVlOgI4AQ==');

@$core.Deprecated('Use getNeighborsRequestDescriptor instead')
const GetNeighborsRequest$json = {
  '1': 'GetNeighborsRequest',
  '2': [
    {'1': 'collection', '3': 1, '4': 1, '5': 9, '10': 'collection'},
    {'1': 'id', '3': 2, '4': 1, '5': 13, '10': 'id'},
    {'1': 'layer', '3': 3, '4': 1, '5': 13, '10': 'layer'},
    {'1': 'limit', '3': 4, '4': 1, '5': 13, '10': 'limit'},
    {'1': 'offset', '3': 5, '4': 1, '5': 13, '10': 'offset'},
  ],
};

/// Descriptor for `GetNeighborsRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getNeighborsRequestDescriptor = $convert.base64Decode(
    'ChNHZXROZWlnaGJvcnNSZXF1ZXN0Eh4KCmNvbGxlY3Rpb24YASABKAlSCmNvbGxlY3Rpb24SDg'
    'oCaWQYAiABKA1SAmlkEhQKBWxheWVyGAMgASgNUgVsYXllchIUCgVsaW1pdBgEIAEoDVIFbGlt'
    'aXQSFgoGb2Zmc2V0GAUgASgNUgZvZmZzZXQ=');

@$core.Deprecated('Use getNeighborsResponseDescriptor instead')
const GetNeighborsResponse$json = {
  '1': 'GetNeighborsResponse',
  '2': [
    {
      '1': 'neighbors',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.hyperspace.GraphNode',
      '10': 'neighbors'
    },
    {'1': 'edge_weights', '3': 2, '4': 3, '5': 1, '10': 'edgeWeights'},
  ],
};

/// Descriptor for `GetNeighborsResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getNeighborsResponseDescriptor = $convert.base64Decode(
    'ChRHZXROZWlnaGJvcnNSZXNwb25zZRIzCgluZWlnaGJvcnMYASADKAsyFS5oeXBlcnNwYWNlLk'
    'dyYXBoTm9kZVIJbmVpZ2hib3JzEiEKDGVkZ2Vfd2VpZ2h0cxgCIAMoAVILZWRnZVdlaWdodHM=');

@$core.Deprecated('Use traverseRequestDescriptor instead')
const TraverseRequest$json = {
  '1': 'TraverseRequest',
  '2': [
    {'1': 'collection', '3': 1, '4': 1, '5': 9, '10': 'collection'},
    {'1': 'start_id', '3': 2, '4': 1, '5': 13, '10': 'startId'},
    {'1': 'max_depth', '3': 3, '4': 1, '5': 13, '10': 'maxDepth'},
    {'1': 'max_nodes', '3': 4, '4': 1, '5': 13, '10': 'maxNodes'},
    {'1': 'layer', '3': 5, '4': 1, '5': 13, '10': 'layer'},
    {
      '1': 'filter',
      '3': 6,
      '4': 3,
      '5': 11,
      '6': '.hyperspace.TraverseRequest.FilterEntry',
      '10': 'filter'
    },
    {
      '1': 'filters',
      '3': 7,
      '4': 3,
      '5': 11,
      '6': '.hyperspace.Filter',
      '10': 'filters'
    },
  ],
  '3': [TraverseRequest_FilterEntry$json],
};

@$core.Deprecated('Use traverseRequestDescriptor instead')
const TraverseRequest_FilterEntry$json = {
  '1': 'FilterEntry',
  '2': [
    {'1': 'key', '3': 1, '4': 1, '5': 9, '10': 'key'},
    {'1': 'value', '3': 2, '4': 1, '5': 9, '10': 'value'},
  ],
  '7': {'7': true},
};

/// Descriptor for `TraverseRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List traverseRequestDescriptor = $convert.base64Decode(
    'Cg9UcmF2ZXJzZVJlcXVlc3QSHgoKY29sbGVjdGlvbhgBIAEoCVIKY29sbGVjdGlvbhIZCghzdG'
    'FydF9pZBgCIAEoDVIHc3RhcnRJZBIbCgltYXhfZGVwdGgYAyABKA1SCG1heERlcHRoEhsKCW1h'
    'eF9ub2RlcxgEIAEoDVIIbWF4Tm9kZXMSFAoFbGF5ZXIYBSABKA1SBWxheWVyEj8KBmZpbHRlch'
    'gGIAMoCzInLmh5cGVyc3BhY2UuVHJhdmVyc2VSZXF1ZXN0LkZpbHRlckVudHJ5UgZmaWx0ZXIS'
    'LAoHZmlsdGVycxgHIAMoCzISLmh5cGVyc3BhY2UuRmlsdGVyUgdmaWx0ZXJzGjkKC0ZpbHRlck'
    'VudHJ5EhAKA2tleRgBIAEoCVIDa2V5EhQKBXZhbHVlGAIgASgJUgV2YWx1ZToCOAE=');

@$core.Deprecated('Use traverseResponseDescriptor instead')
const TraverseResponse$json = {
  '1': 'TraverseResponse',
  '2': [
    {
      '1': 'nodes',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.hyperspace.GraphNode',
      '10': 'nodes'
    },
  ],
};

/// Descriptor for `TraverseResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List traverseResponseDescriptor = $convert.base64Decode(
    'ChBUcmF2ZXJzZVJlc3BvbnNlEisKBW5vZGVzGAEgAygLMhUuaHlwZXJzcGFjZS5HcmFwaE5vZG'
    'VSBW5vZGVz');

@$core.Deprecated('Use findSemanticClustersRequestDescriptor instead')
const FindSemanticClustersRequest$json = {
  '1': 'FindSemanticClustersRequest',
  '2': [
    {'1': 'collection', '3': 1, '4': 1, '5': 9, '10': 'collection'},
    {'1': 'layer', '3': 2, '4': 1, '5': 13, '10': 'layer'},
    {'1': 'min_cluster_size', '3': 3, '4': 1, '5': 13, '10': 'minClusterSize'},
    {'1': 'max_clusters', '3': 4, '4': 1, '5': 13, '10': 'maxClusters'},
    {'1': 'max_nodes', '3': 5, '4': 1, '5': 13, '10': 'maxNodes'},
  ],
};

/// Descriptor for `FindSemanticClustersRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List findSemanticClustersRequestDescriptor = $convert.base64Decode(
    'ChtGaW5kU2VtYW50aWNDbHVzdGVyc1JlcXVlc3QSHgoKY29sbGVjdGlvbhgBIAEoCVIKY29sbG'
    'VjdGlvbhIUCgVsYXllchgCIAEoDVIFbGF5ZXISKAoQbWluX2NsdXN0ZXJfc2l6ZRgDIAEoDVIO'
    'bWluQ2x1c3RlclNpemUSIQoMbWF4X2NsdXN0ZXJzGAQgASgNUgttYXhDbHVzdGVycxIbCgltYX'
    'hfbm9kZXMYBSABKA1SCG1heE5vZGVz');

@$core.Deprecated('Use getConceptParentsRequestDescriptor instead')
const GetConceptParentsRequest$json = {
  '1': 'GetConceptParentsRequest',
  '2': [
    {'1': 'collection', '3': 1, '4': 1, '5': 9, '10': 'collection'},
    {'1': 'id', '3': 2, '4': 1, '5': 13, '10': 'id'},
    {'1': 'layer', '3': 3, '4': 1, '5': 13, '10': 'layer'},
    {'1': 'limit', '3': 4, '4': 1, '5': 13, '10': 'limit'},
  ],
};

/// Descriptor for `GetConceptParentsRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getConceptParentsRequestDescriptor = $convert.base64Decode(
    'ChhHZXRDb25jZXB0UGFyZW50c1JlcXVlc3QSHgoKY29sbGVjdGlvbhgBIAEoCVIKY29sbGVjdG'
    'lvbhIOCgJpZBgCIAEoDVICaWQSFAoFbGF5ZXIYAyABKA1SBWxheWVyEhQKBWxpbWl0GAQgASgN'
    'UgVsaW1pdA==');

@$core.Deprecated('Use getConceptParentsResponseDescriptor instead')
const GetConceptParentsResponse$json = {
  '1': 'GetConceptParentsResponse',
  '2': [
    {
      '1': 'parents',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.hyperspace.GraphNode',
      '10': 'parents'
    },
  ],
};

/// Descriptor for `GetConceptParentsResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getConceptParentsResponseDescriptor =
    $convert.base64Decode(
        'ChlHZXRDb25jZXB0UGFyZW50c1Jlc3BvbnNlEi8KB3BhcmVudHMYASADKAsyFS5oeXBlcnNwYW'
        'NlLkdyYXBoTm9kZVIHcGFyZW50cw==');

@$core.Deprecated('Use graphClusterDescriptor instead')
const GraphCluster$json = {
  '1': 'GraphCluster',
  '2': [
    {'1': 'node_ids', '3': 1, '4': 3, '5': 13, '10': 'nodeIds'},
  ],
};

/// Descriptor for `GraphCluster`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List graphClusterDescriptor = $convert
    .base64Decode('CgxHcmFwaENsdXN0ZXISGQoIbm9kZV9pZHMYASADKA1SB25vZGVJZHM=');

@$core.Deprecated('Use findSemanticClustersResponseDescriptor instead')
const FindSemanticClustersResponse$json = {
  '1': 'FindSemanticClustersResponse',
  '2': [
    {
      '1': 'clusters',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.hyperspace.GraphCluster',
      '10': 'clusters'
    },
  ],
};

/// Descriptor for `FindSemanticClustersResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List findSemanticClustersResponseDescriptor =
    $convert.base64Decode(
        'ChxGaW5kU2VtYW50aWNDbHVzdGVyc1Jlc3BvbnNlEjQKCGNsdXN0ZXJzGAEgAygLMhguaHlwZX'
        'JzcGFjZS5HcmFwaENsdXN0ZXJSCGNsdXN0ZXJz');

@$core.Deprecated('Use metadataValueDescriptor instead')
const MetadataValue$json = {
  '1': 'MetadataValue',
  '2': [
    {'1': 'string_value', '3': 1, '4': 1, '5': 9, '9': 0, '10': 'stringValue'},
    {'1': 'int_value', '3': 2, '4': 1, '5': 3, '9': 0, '10': 'intValue'},
    {'1': 'double_value', '3': 3, '4': 1, '5': 1, '9': 0, '10': 'doubleValue'},
    {'1': 'bool_value', '3': 4, '4': 1, '5': 8, '9': 0, '10': 'boolValue'},
  ],
  '8': [
    {'1': 'kind'},
  ],
};

/// Descriptor for `MetadataValue`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List metadataValueDescriptor = $convert.base64Decode(
    'Cg1NZXRhZGF0YVZhbHVlEiMKDHN0cmluZ192YWx1ZRgBIAEoCUgAUgtzdHJpbmdWYWx1ZRIdCg'
    'lpbnRfdmFsdWUYAiABKANIAFIIaW50VmFsdWUSIwoMZG91YmxlX3ZhbHVlGAMgASgBSABSC2Rv'
    'dWJsZVZhbHVlEh8KCmJvb2xfdmFsdWUYBCABKAhIAFIJYm9vbFZhbHVlQgYKBGtpbmQ=');

@$core.Deprecated('Use eventSubscriptionRequestDescriptor instead')
const EventSubscriptionRequest$json = {
  '1': 'EventSubscriptionRequest',
  '2': [
    {
      '1': 'types',
      '3': 1,
      '4': 3,
      '5': 14,
      '6': '.hyperspace.EventType',
      '10': 'types'
    },
    {
      '1': 'collection',
      '3': 2,
      '4': 1,
      '5': 9,
      '9': 0,
      '10': 'collection',
      '17': true
    },
  ],
  '8': [
    {'1': '_collection'},
  ],
};

/// Descriptor for `EventSubscriptionRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List eventSubscriptionRequestDescriptor = $convert.base64Decode(
    'ChhFdmVudFN1YnNjcmlwdGlvblJlcXVlc3QSKwoFdHlwZXMYASADKA4yFS5oeXBlcnNwYWNlLk'
    'V2ZW50VHlwZVIFdHlwZXMSIwoKY29sbGVjdGlvbhgCIAEoCUgAUgpjb2xsZWN0aW9uiAEBQg0K'
    'C19jb2xsZWN0aW9u');

@$core.Deprecated('Use vectorInsertedEventDescriptor instead')
const VectorInsertedEvent$json = {
  '1': 'VectorInsertedEvent',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 13, '10': 'id'},
    {'1': 'collection', '3': 2, '4': 1, '5': 9, '10': 'collection'},
    {'1': 'logical_clock', '3': 3, '4': 1, '5': 4, '10': 'logicalClock'},
    {'1': 'origin_node_id', '3': 4, '4': 1, '5': 9, '10': 'originNodeId'},
    {
      '1': 'metadata',
      '3': 5,
      '4': 3,
      '5': 11,
      '6': '.hyperspace.VectorInsertedEvent.MetadataEntry',
      '10': 'metadata'
    },
    {
      '1': 'typed_metadata',
      '3': 6,
      '4': 3,
      '5': 11,
      '6': '.hyperspace.VectorInsertedEvent.TypedMetadataEntry',
      '10': 'typedMetadata'
    },
  ],
  '3': [
    VectorInsertedEvent_MetadataEntry$json,
    VectorInsertedEvent_TypedMetadataEntry$json
  ],
};

@$core.Deprecated('Use vectorInsertedEventDescriptor instead')
const VectorInsertedEvent_MetadataEntry$json = {
  '1': 'MetadataEntry',
  '2': [
    {'1': 'key', '3': 1, '4': 1, '5': 9, '10': 'key'},
    {'1': 'value', '3': 2, '4': 1, '5': 9, '10': 'value'},
  ],
  '7': {'7': true},
};

@$core.Deprecated('Use vectorInsertedEventDescriptor instead')
const VectorInsertedEvent_TypedMetadataEntry$json = {
  '1': 'TypedMetadataEntry',
  '2': [
    {'1': 'key', '3': 1, '4': 1, '5': 9, '10': 'key'},
    {
      '1': 'value',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.hyperspace.MetadataValue',
      '10': 'value'
    },
  ],
  '7': {'7': true},
};

/// Descriptor for `VectorInsertedEvent`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List vectorInsertedEventDescriptor = $convert.base64Decode(
    'ChNWZWN0b3JJbnNlcnRlZEV2ZW50Eg4KAmlkGAEgASgNUgJpZBIeCgpjb2xsZWN0aW9uGAIgAS'
    'gJUgpjb2xsZWN0aW9uEiMKDWxvZ2ljYWxfY2xvY2sYAyABKARSDGxvZ2ljYWxDbG9jaxIkCg5v'
    'cmlnaW5fbm9kZV9pZBgEIAEoCVIMb3JpZ2luTm9kZUlkEkkKCG1ldGFkYXRhGAUgAygLMi0uaH'
    'lwZXJzcGFjZS5WZWN0b3JJbnNlcnRlZEV2ZW50Lk1ldGFkYXRhRW50cnlSCG1ldGFkYXRhElkK'
    'DnR5cGVkX21ldGFkYXRhGAYgAygLMjIuaHlwZXJzcGFjZS5WZWN0b3JJbnNlcnRlZEV2ZW50Ll'
    'R5cGVkTWV0YWRhdGFFbnRyeVINdHlwZWRNZXRhZGF0YRo7Cg1NZXRhZGF0YUVudHJ5EhAKA2tl'
    'eRgBIAEoCVIDa2V5EhQKBXZhbHVlGAIgASgJUgV2YWx1ZToCOAEaWwoSVHlwZWRNZXRhZGF0YU'
    'VudHJ5EhAKA2tleRgBIAEoCVIDa2V5Ei8KBXZhbHVlGAIgASgLMhkuaHlwZXJzcGFjZS5NZXRh'
    'ZGF0YVZhbHVlUgV2YWx1ZToCOAE=');

@$core.Deprecated('Use trajectoryStepEventDescriptor instead')
const TrajectoryStepEvent$json = {
  '1': 'TrajectoryStepEvent',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 13, '10': 'id'},
    {'1': 'collection', '3': 2, '4': 1, '5': 9, '10': 'collection'},
    {'1': 'x', '3': 3, '4': 1, '5': 2, '10': 'x'},
    {'1': 'y', '3': 4, '4': 1, '5': 2, '10': 'y'},
    {
      '1': 'metadata',
      '3': 5,
      '4': 3,
      '5': 11,
      '6': '.hyperspace.TrajectoryStepEvent.MetadataEntry',
      '10': 'metadata'
    },
  ],
  '3': [TrajectoryStepEvent_MetadataEntry$json],
};

@$core.Deprecated('Use trajectoryStepEventDescriptor instead')
const TrajectoryStepEvent_MetadataEntry$json = {
  '1': 'MetadataEntry',
  '2': [
    {'1': 'key', '3': 1, '4': 1, '5': 9, '10': 'key'},
    {'1': 'value', '3': 2, '4': 1, '5': 9, '10': 'value'},
  ],
  '7': {'7': true},
};

/// Descriptor for `TrajectoryStepEvent`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List trajectoryStepEventDescriptor = $convert.base64Decode(
    'ChNUcmFqZWN0b3J5U3RlcEV2ZW50Eg4KAmlkGAEgASgNUgJpZBIeCgpjb2xsZWN0aW9uGAIgAS'
    'gJUgpjb2xsZWN0aW9uEgwKAXgYAyABKAJSAXgSDAoBeRgEIAEoAlIBeRJJCghtZXRhZGF0YRgF'
    'IAMoCzItLmh5cGVyc3BhY2UuVHJhamVjdG9yeVN0ZXBFdmVudC5NZXRhZGF0YUVudHJ5UghtZX'
    'RhZGF0YRo7Cg1NZXRhZGF0YUVudHJ5EhAKA2tleRgBIAEoCVIDa2V5EhQKBXZhbHVlGAIgASgJ'
    'UgV2YWx1ZToCOAE=');

@$core.Deprecated('Use vectorDeletedEventDescriptor instead')
const VectorDeletedEvent$json = {
  '1': 'VectorDeletedEvent',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 13, '10': 'id'},
    {'1': 'collection', '3': 2, '4': 1, '5': 9, '10': 'collection'},
    {'1': 'logical_clock', '3': 3, '4': 1, '5': 4, '10': 'logicalClock'},
    {'1': 'origin_node_id', '3': 4, '4': 1, '5': 9, '10': 'originNodeId'},
  ],
};

/// Descriptor for `VectorDeletedEvent`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List vectorDeletedEventDescriptor = $convert.base64Decode(
    'ChJWZWN0b3JEZWxldGVkRXZlbnQSDgoCaWQYASABKA1SAmlkEh4KCmNvbGxlY3Rpb24YAiABKA'
    'lSCmNvbGxlY3Rpb24SIwoNbG9naWNhbF9jbG9jaxgDIAEoBFIMbG9naWNhbENsb2NrEiQKDm9y'
    'aWdpbl9ub2RlX2lkGAQgASgJUgxvcmlnaW5Ob2RlSWQ=');

@$core.Deprecated('Use eventMessageDescriptor instead')
const EventMessage$json = {
  '1': 'EventMessage',
  '2': [
    {
      '1': 'type',
      '3': 1,
      '4': 1,
      '5': 14,
      '6': '.hyperspace.EventType',
      '10': 'type'
    },
    {
      '1': 'vector_inserted',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.hyperspace.VectorInsertedEvent',
      '9': 0,
      '10': 'vectorInserted'
    },
    {
      '1': 'vector_deleted',
      '3': 3,
      '4': 1,
      '5': 11,
      '6': '.hyperspace.VectorDeletedEvent',
      '9': 0,
      '10': 'vectorDeleted'
    },
    {
      '1': 'trajectory_step',
      '3': 4,
      '4': 1,
      '5': 11,
      '6': '.hyperspace.TrajectoryStepEvent',
      '9': 0,
      '10': 'trajectoryStep'
    },
  ],
  '8': [
    {'1': 'payload'},
  ],
};

/// Descriptor for `EventMessage`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List eventMessageDescriptor = $convert.base64Decode(
    'CgxFdmVudE1lc3NhZ2USKQoEdHlwZRgBIAEoDjIVLmh5cGVyc3BhY2UuRXZlbnRUeXBlUgR0eX'
    'BlEkoKD3ZlY3Rvcl9pbnNlcnRlZBgCIAEoCzIfLmh5cGVyc3BhY2UuVmVjdG9ySW5zZXJ0ZWRF'
    'dmVudEgAUg52ZWN0b3JJbnNlcnRlZBJHCg52ZWN0b3JfZGVsZXRlZBgDIAEoCzIeLmh5cGVyc3'
    'BhY2UuVmVjdG9yRGVsZXRlZEV2ZW50SABSDXZlY3RvckRlbGV0ZWQSSgoPdHJhamVjdG9yeV9z'
    'dGVwGAQgASgLMh8uaHlwZXJzcGFjZS5UcmFqZWN0b3J5U3RlcEV2ZW50SABSDnRyYWplY3Rvcn'
    'lTdGVwQgkKB3BheWxvYWQ=');

@$core.Deprecated('Use emptyDescriptor instead')
const Empty$json = {
  '1': 'Empty',
};

/// Descriptor for `Empty`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List emptyDescriptor =
    $convert.base64Decode('CgVFbXB0eQ==');

@$core.Deprecated('Use statusResponseDescriptor instead')
const StatusResponse$json = {
  '1': 'StatusResponse',
  '2': [
    {'1': 'status', '3': 1, '4': 1, '5': 9, '10': 'status'},
  ],
};

/// Descriptor for `StatusResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List statusResponseDescriptor = $convert
    .base64Decode('Cg5TdGF0dXNSZXNwb25zZRIWCgZzdGF0dXMYASABKAlSBnN0YXR1cw==');

@$core.Deprecated('Use monitorRequestDescriptor instead')
const MonitorRequest$json = {
  '1': 'MonitorRequest',
};

/// Descriptor for `MonitorRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List monitorRequestDescriptor =
    $convert.base64Decode('Cg5Nb25pdG9yUmVxdWVzdA==');

@$core.Deprecated('Use systemStatsDescriptor instead')
const SystemStats$json = {
  '1': 'SystemStats',
  '2': [
    {
      '1': 'total_collections',
      '3': 1,
      '4': 1,
      '5': 4,
      '10': 'totalCollections'
    },
    {'1': 'total_vectors', '3': 2, '4': 1, '5': 4, '10': 'totalVectors'},
    {'1': 'total_memory_mb', '3': 3, '4': 1, '5': 1, '10': 'totalMemoryMb'},
    {'1': 'qps', '3': 4, '4': 1, '5': 1, '10': 'qps'},
  ],
};

/// Descriptor for `SystemStats`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List systemStatsDescriptor = $convert.base64Decode(
    'CgtTeXN0ZW1TdGF0cxIrChF0b3RhbF9jb2xsZWN0aW9ucxgBIAEoBFIQdG90YWxDb2xsZWN0aW'
    '9ucxIjCg10b3RhbF92ZWN0b3JzGAIgASgEUgx0b3RhbFZlY3RvcnMSJgoPdG90YWxfbWVtb3J5'
    'X21iGAMgASgBUg10b3RhbE1lbW9yeU1iEhAKA3FwcxgEIAEoAVIDcXBz');

@$core.Deprecated('Use digestRequestDescriptor instead')
const DigestRequest$json = {
  '1': 'DigestRequest',
  '2': [
    {'1': 'collection', '3': 1, '4': 1, '5': 9, '10': 'collection'},
  ],
};

/// Descriptor for `DigestRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List digestRequestDescriptor = $convert.base64Decode(
    'Cg1EaWdlc3RSZXF1ZXN0Eh4KCmNvbGxlY3Rpb24YASABKAlSCmNvbGxlY3Rpb24=');

@$core.Deprecated('Use digestResponseDescriptor instead')
const DigestResponse$json = {
  '1': 'DigestResponse',
  '2': [
    {'1': 'logical_clock', '3': 1, '4': 1, '5': 4, '10': 'logicalClock'},
    {'1': 'state_hash', '3': 2, '4': 1, '5': 4, '10': 'stateHash'},
    {'1': 'buckets', '3': 3, '4': 3, '5': 4, '10': 'buckets'},
    {'1': 'count', '3': 4, '4': 1, '5': 4, '10': 'count'},
  ],
};

/// Descriptor for `DigestResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List digestResponseDescriptor = $convert.base64Decode(
    'Cg5EaWdlc3RSZXNwb25zZRIjCg1sb2dpY2FsX2Nsb2NrGAEgASgEUgxsb2dpY2FsQ2xvY2sSHQ'
    'oKc3RhdGVfaGFzaBgCIAEoBFIJc3RhdGVIYXNoEhgKB2J1Y2tldHMYAyADKARSB2J1Y2tldHMS'
    'FAoFY291bnQYBCABKARSBWNvdW50');

@$core.Deprecated('Use syncHandshakeRequestDescriptor instead')
const SyncHandshakeRequest$json = {
  '1': 'SyncHandshakeRequest',
  '2': [
    {'1': 'collection', '3': 1, '4': 1, '5': 9, '10': 'collection'},
    {'1': 'client_buckets', '3': 2, '4': 3, '5': 4, '10': 'clientBuckets'},
    {
      '1': 'client_logical_clock',
      '3': 3,
      '4': 1,
      '5': 4,
      '10': 'clientLogicalClock'
    },
    {'1': 'client_count', '3': 4, '4': 1, '5': 4, '10': 'clientCount'},
  ],
};

/// Descriptor for `SyncHandshakeRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List syncHandshakeRequestDescriptor = $convert.base64Decode(
    'ChRTeW5jSGFuZHNoYWtlUmVxdWVzdBIeCgpjb2xsZWN0aW9uGAEgASgJUgpjb2xsZWN0aW9uEi'
    'UKDmNsaWVudF9idWNrZXRzGAIgAygEUg1jbGllbnRCdWNrZXRzEjAKFGNsaWVudF9sb2dpY2Fs'
    'X2Nsb2NrGAMgASgEUhJjbGllbnRMb2dpY2FsQ2xvY2sSIQoMY2xpZW50X2NvdW50GAQgASgEUg'
    'tjbGllbnRDb3VudA==');

@$core.Deprecated('Use diffBucketDescriptor instead')
const DiffBucket$json = {
  '1': 'DiffBucket',
  '2': [
    {'1': 'bucket_index', '3': 1, '4': 1, '5': 13, '10': 'bucketIndex'},
    {'1': 'server_hash', '3': 2, '4': 1, '5': 4, '10': 'serverHash'},
    {'1': 'client_hash', '3': 3, '4': 1, '5': 4, '10': 'clientHash'},
  ],
};

/// Descriptor for `DiffBucket`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List diffBucketDescriptor = $convert.base64Decode(
    'CgpEaWZmQnVja2V0EiEKDGJ1Y2tldF9pbmRleBgBIAEoDVILYnVja2V0SW5kZXgSHwoLc2Vydm'
    'VyX2hhc2gYAiABKARSCnNlcnZlckhhc2gSHwoLY2xpZW50X2hhc2gYAyABKARSCmNsaWVudEhh'
    'c2g=');

@$core.Deprecated('Use syncHandshakeResponseDescriptor instead')
const SyncHandshakeResponse$json = {
  '1': 'SyncHandshakeResponse',
  '2': [
    {
      '1': 'diff_buckets',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.hyperspace.DiffBucket',
      '10': 'diffBuckets'
    },
    {
      '1': 'server_logical_clock',
      '3': 2,
      '4': 1,
      '5': 4,
      '10': 'serverLogicalClock'
    },
    {'1': 'server_count', '3': 3, '4': 1, '5': 4, '10': 'serverCount'},
    {'1': 'in_sync', '3': 4, '4': 1, '5': 8, '10': 'inSync'},
  ],
};

/// Descriptor for `SyncHandshakeResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List syncHandshakeResponseDescriptor = $convert.base64Decode(
    'ChVTeW5jSGFuZHNoYWtlUmVzcG9uc2USOQoMZGlmZl9idWNrZXRzGAEgAygLMhYuaHlwZXJzcG'
    'FjZS5EaWZmQnVja2V0UgtkaWZmQnVja2V0cxIwChRzZXJ2ZXJfbG9naWNhbF9jbG9jaxgCIAEo'
    'BFISc2VydmVyTG9naWNhbENsb2NrEiEKDHNlcnZlcl9jb3VudBgDIAEoBFILc2VydmVyQ291bn'
    'QSFwoHaW5fc3luYxgEIAEoCFIGaW5TeW5j');

@$core.Deprecated('Use syncPullRequestDescriptor instead')
const SyncPullRequest$json = {
  '1': 'SyncPullRequest',
  '2': [
    {'1': 'collection', '3': 1, '4': 1, '5': 9, '10': 'collection'},
    {'1': 'bucket_indices', '3': 2, '4': 3, '5': 13, '10': 'bucketIndices'},
  ],
};

/// Descriptor for `SyncPullRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List syncPullRequestDescriptor = $convert.base64Decode(
    'Cg9TeW5jUHVsbFJlcXVlc3QSHgoKY29sbGVjdGlvbhgBIAEoCVIKY29sbGVjdGlvbhIlCg5idW'
    'NrZXRfaW5kaWNlcxgCIAMoDVINYnVja2V0SW5kaWNlcw==');

@$core.Deprecated('Use syncVectorDataDescriptor instead')
const SyncVectorData$json = {
  '1': 'SyncVectorData',
  '2': [
    {'1': 'collection', '3': 1, '4': 1, '5': 9, '10': 'collection'},
    {'1': 'id', '3': 2, '4': 1, '5': 13, '10': 'id'},
    {'1': 'vector', '3': 3, '4': 3, '5': 1, '10': 'vector'},
    {
      '1': 'metadata',
      '3': 4,
      '4': 3,
      '5': 11,
      '6': '.hyperspace.SyncVectorData.MetadataEntry',
      '10': 'metadata'
    },
    {'1': 'bucket_index', '3': 5, '4': 1, '5': 13, '10': 'bucketIndex'},
  ],
  '3': [SyncVectorData_MetadataEntry$json],
};

@$core.Deprecated('Use syncVectorDataDescriptor instead')
const SyncVectorData_MetadataEntry$json = {
  '1': 'MetadataEntry',
  '2': [
    {'1': 'key', '3': 1, '4': 1, '5': 9, '10': 'key'},
    {'1': 'value', '3': 2, '4': 1, '5': 9, '10': 'value'},
  ],
  '7': {'7': true},
};

/// Descriptor for `SyncVectorData`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List syncVectorDataDescriptor = $convert.base64Decode(
    'Cg5TeW5jVmVjdG9yRGF0YRIeCgpjb2xsZWN0aW9uGAEgASgJUgpjb2xsZWN0aW9uEg4KAmlkGA'
    'IgASgNUgJpZBIWCgZ2ZWN0b3IYAyADKAFSBnZlY3RvchJECghtZXRhZGF0YRgEIAMoCzIoLmh5'
    'cGVyc3BhY2UuU3luY1ZlY3RvckRhdGEuTWV0YWRhdGFFbnRyeVIIbWV0YWRhdGESIQoMYnVja2'
    'V0X2luZGV4GAUgASgNUgtidWNrZXRJbmRleBo7Cg1NZXRhZGF0YUVudHJ5EhAKA2tleRgBIAEo'
    'CVIDa2V5EhQKBXZhbHVlGAIgASgJUgV2YWx1ZToCOAE=');

@$core.Deprecated('Use syncPushResponseDescriptor instead')
const SyncPushResponse$json = {
  '1': 'SyncPushResponse',
  '2': [
    {'1': 'accepted', '3': 1, '4': 1, '5': 13, '10': 'accepted'},
    {'1': 'rejected', '3': 2, '4': 1, '5': 13, '10': 'rejected'},
    {'1': 'duplicates', '3': 3, '4': 1, '5': 13, '10': 'duplicates'},
  ],
};

/// Descriptor for `SyncPushResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List syncPushResponseDescriptor = $convert.base64Decode(
    'ChBTeW5jUHVzaFJlc3BvbnNlEhoKCGFjY2VwdGVkGAEgASgNUghhY2NlcHRlZBIaCghyZWplY3'
    'RlZBgCIAEoDVIIcmVqZWN0ZWQSHgoKZHVwbGljYXRlcxgDIAEoDVIKZHVwbGljYXRlcw==');
