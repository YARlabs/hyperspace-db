// This is a generated file - do not edit.
//
// Generated from hyperspace.proto.

// @dart = 3.3

// ignore_for_file: annotate_overrides, camel_case_types, comment_references
// ignore_for_file: constant_identifier_names
// ignore_for_file: curly_braces_in_flow_control_structures
// ignore_for_file: deprecated_member_use_from_same_package, library_prefixes
// ignore_for_file: non_constant_identifier_names, prefer_relative_imports

import 'dart:core' as $core;

import 'package:fixnum/fixnum.dart' as $fixnum;
import 'package:protobuf/protobuf.dart' as $pb;

import 'hyperspace.pbenum.dart';

export 'package:protobuf/protobuf.dart' show GeneratedMessageGenericExtensions;

export 'hyperspace.pbenum.dart';

class ReplicationRequest extends $pb.GeneratedMessage {
  factory ReplicationRequest({
    $fixnum.Int64? lastLogicalClock,
  }) {
    final result = create();
    if (lastLogicalClock != null) result.lastLogicalClock = lastLogicalClock;
    return result;
  }

  ReplicationRequest._();

  factory ReplicationRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ReplicationRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ReplicationRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'hyperspace'),
      createEmptyInstance: create)
    ..a<$fixnum.Int64>(
        1, _omitFieldNames ? '' : 'lastLogicalClock', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ReplicationRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ReplicationRequest copyWith(void Function(ReplicationRequest) updates) =>
      super.copyWith((message) => updates(message as ReplicationRequest))
          as ReplicationRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ReplicationRequest create() => ReplicationRequest._();
  @$core.override
  ReplicationRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ReplicationRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ReplicationRequest>(create);
  static ReplicationRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $fixnum.Int64 get lastLogicalClock => $_getI64(0);
  @$pb.TagNumber(1)
  set lastLogicalClock($fixnum.Int64 value) => $_setInt64(0, value);
  @$pb.TagNumber(1)
  $core.bool hasLastLogicalClock() => $_has(0);
  @$pb.TagNumber(1)
  void clearLastLogicalClock() => $_clearField(1);
}

enum ReplicationLog_Operation {
  insert,
  createCollection,
  deleteCollection,
  delete,
  notSet
}

class ReplicationLog extends $pb.GeneratedMessage {
  factory ReplicationLog({
    $fixnum.Int64? logicalClock,
    $core.String? originNodeId,
    $core.String? collection,
    InsertOp? insert,
    CreateCollectionOp? createCollection,
    DeleteCollectionOp? deleteCollection,
    DeleteOp? delete,
  }) {
    final result = create();
    if (logicalClock != null) result.logicalClock = logicalClock;
    if (originNodeId != null) result.originNodeId = originNodeId;
    if (collection != null) result.collection = collection;
    if (insert != null) result.insert = insert;
    if (createCollection != null) result.createCollection = createCollection;
    if (deleteCollection != null) result.deleteCollection = deleteCollection;
    if (delete != null) result.delete = delete;
    return result;
  }

  ReplicationLog._();

  factory ReplicationLog.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ReplicationLog.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static const $core.Map<$core.int, ReplicationLog_Operation>
      _ReplicationLog_OperationByTag = {
    4: ReplicationLog_Operation.insert,
    5: ReplicationLog_Operation.createCollection,
    6: ReplicationLog_Operation.deleteCollection,
    7: ReplicationLog_Operation.delete,
    0: ReplicationLog_Operation.notSet
  };
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ReplicationLog',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'hyperspace'),
      createEmptyInstance: create)
    ..oo(0, [4, 5, 6, 7])
    ..a<$fixnum.Int64>(
        1, _omitFieldNames ? '' : 'logicalClock', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..aOS(2, _omitFieldNames ? '' : 'originNodeId')
    ..aOS(3, _omitFieldNames ? '' : 'collection')
    ..aOM<InsertOp>(4, _omitFieldNames ? '' : 'insert',
        subBuilder: InsertOp.create)
    ..aOM<CreateCollectionOp>(5, _omitFieldNames ? '' : 'createCollection',
        subBuilder: CreateCollectionOp.create)
    ..aOM<DeleteCollectionOp>(6, _omitFieldNames ? '' : 'deleteCollection',
        subBuilder: DeleteCollectionOp.create)
    ..aOM<DeleteOp>(7, _omitFieldNames ? '' : 'delete',
        subBuilder: DeleteOp.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ReplicationLog clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ReplicationLog copyWith(void Function(ReplicationLog) updates) =>
      super.copyWith((message) => updates(message as ReplicationLog))
          as ReplicationLog;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ReplicationLog create() => ReplicationLog._();
  @$core.override
  ReplicationLog createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ReplicationLog getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ReplicationLog>(create);
  static ReplicationLog? _defaultInstance;

  @$pb.TagNumber(4)
  @$pb.TagNumber(5)
  @$pb.TagNumber(6)
  @$pb.TagNumber(7)
  ReplicationLog_Operation whichOperation() =>
      _ReplicationLog_OperationByTag[$_whichOneof(0)]!;
  @$pb.TagNumber(4)
  @$pb.TagNumber(5)
  @$pb.TagNumber(6)
  @$pb.TagNumber(7)
  void clearOperation() => $_clearField($_whichOneof(0));

  @$pb.TagNumber(1)
  $fixnum.Int64 get logicalClock => $_getI64(0);
  @$pb.TagNumber(1)
  set logicalClock($fixnum.Int64 value) => $_setInt64(0, value);
  @$pb.TagNumber(1)
  $core.bool hasLogicalClock() => $_has(0);
  @$pb.TagNumber(1)
  void clearLogicalClock() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get originNodeId => $_getSZ(1);
  @$pb.TagNumber(2)
  set originNodeId($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasOriginNodeId() => $_has(1);
  @$pb.TagNumber(2)
  void clearOriginNodeId() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get collection => $_getSZ(2);
  @$pb.TagNumber(3)
  set collection($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasCollection() => $_has(2);
  @$pb.TagNumber(3)
  void clearCollection() => $_clearField(3);

  @$pb.TagNumber(4)
  InsertOp get insert => $_getN(3);
  @$pb.TagNumber(4)
  set insert(InsertOp value) => $_setField(4, value);
  @$pb.TagNumber(4)
  $core.bool hasInsert() => $_has(3);
  @$pb.TagNumber(4)
  void clearInsert() => $_clearField(4);
  @$pb.TagNumber(4)
  InsertOp ensureInsert() => $_ensure(3);

  @$pb.TagNumber(5)
  CreateCollectionOp get createCollection => $_getN(4);
  @$pb.TagNumber(5)
  set createCollection(CreateCollectionOp value) => $_setField(5, value);
  @$pb.TagNumber(5)
  $core.bool hasCreateCollection() => $_has(4);
  @$pb.TagNumber(5)
  void clearCreateCollection() => $_clearField(5);
  @$pb.TagNumber(5)
  CreateCollectionOp ensureCreateCollection() => $_ensure(4);

  @$pb.TagNumber(6)
  DeleteCollectionOp get deleteCollection => $_getN(5);
  @$pb.TagNumber(6)
  set deleteCollection(DeleteCollectionOp value) => $_setField(6, value);
  @$pb.TagNumber(6)
  $core.bool hasDeleteCollection() => $_has(5);
  @$pb.TagNumber(6)
  void clearDeleteCollection() => $_clearField(6);
  @$pb.TagNumber(6)
  DeleteCollectionOp ensureDeleteCollection() => $_ensure(5);

  @$pb.TagNumber(7)
  DeleteOp get delete => $_getN(6);
  @$pb.TagNumber(7)
  set delete(DeleteOp value) => $_setField(7, value);
  @$pb.TagNumber(7)
  $core.bool hasDelete() => $_has(6);
  @$pb.TagNumber(7)
  void clearDelete() => $_clearField(7);
  @$pb.TagNumber(7)
  DeleteOp ensureDelete() => $_ensure(6);
}

class InsertOp extends $pb.GeneratedMessage {
  factory InsertOp({
    $core.int? id,
    $core.Iterable<$core.double>? vector,
    $core.Iterable<$core.MapEntry<$core.String, $core.String>>? metadata,
    $core.Iterable<$core.MapEntry<$core.String, MetadataValue>>? typedMetadata,
  }) {
    final result = create();
    if (id != null) result.id = id;
    if (vector != null) result.vector.addAll(vector);
    if (metadata != null) result.metadata.addEntries(metadata);
    if (typedMetadata != null) result.typedMetadata.addEntries(typedMetadata);
    return result;
  }

  InsertOp._();

  factory InsertOp.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory InsertOp.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'InsertOp',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'hyperspace'),
      createEmptyInstance: create)
    ..aI(1, _omitFieldNames ? '' : 'id', fieldType: $pb.PbFieldType.OU3)
    ..p<$core.double>(2, _omitFieldNames ? '' : 'vector', $pb.PbFieldType.KD)
    ..m<$core.String, $core.String>(3, _omitFieldNames ? '' : 'metadata',
        entryClassName: 'InsertOp.MetadataEntry',
        keyFieldType: $pb.PbFieldType.OS,
        valueFieldType: $pb.PbFieldType.OS,
        packageName: const $pb.PackageName('hyperspace'))
    ..m<$core.String, MetadataValue>(4, _omitFieldNames ? '' : 'typedMetadata',
        entryClassName: 'InsertOp.TypedMetadataEntry',
        keyFieldType: $pb.PbFieldType.OS,
        valueFieldType: $pb.PbFieldType.OM,
        valueCreator: MetadataValue.create,
        valueDefaultOrMaker: MetadataValue.getDefault,
        packageName: const $pb.PackageName('hyperspace'))
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  InsertOp clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  InsertOp copyWith(void Function(InsertOp) updates) =>
      super.copyWith((message) => updates(message as InsertOp)) as InsertOp;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static InsertOp create() => InsertOp._();
  @$core.override
  InsertOp createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static InsertOp getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<InsertOp>(create);
  static InsertOp? _defaultInstance;

  @$pb.TagNumber(1)
  $core.int get id => $_getIZ(0);
  @$pb.TagNumber(1)
  set id($core.int value) => $_setUnsignedInt32(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  @$pb.TagNumber(2)
  $pb.PbList<$core.double> get vector => $_getList(1);

  @$pb.TagNumber(3)
  $pb.PbMap<$core.String, $core.String> get metadata => $_getMap(2);

  @$pb.TagNumber(4)
  $pb.PbMap<$core.String, MetadataValue> get typedMetadata => $_getMap(3);
}

class CreateCollectionOp extends $pb.GeneratedMessage {
  factory CreateCollectionOp({
    CollectionSchema? schema,
  }) {
    final result = create();
    if (schema != null) result.schema = schema;
    return result;
  }

  CreateCollectionOp._();

  factory CreateCollectionOp.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory CreateCollectionOp.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'CreateCollectionOp',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'hyperspace'),
      createEmptyInstance: create)
    ..aOM<CollectionSchema>(3, _omitFieldNames ? '' : 'schema',
        subBuilder: CollectionSchema.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CreateCollectionOp clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CreateCollectionOp copyWith(void Function(CreateCollectionOp) updates) =>
      super.copyWith((message) => updates(message as CreateCollectionOp))
          as CreateCollectionOp;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static CreateCollectionOp create() => CreateCollectionOp._();
  @$core.override
  CreateCollectionOp createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static CreateCollectionOp getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<CreateCollectionOp>(create);
  static CreateCollectionOp? _defaultInstance;

  @$pb.TagNumber(3)
  CollectionSchema get schema => $_getN(0);
  @$pb.TagNumber(3)
  set schema(CollectionSchema value) => $_setField(3, value);
  @$pb.TagNumber(3)
  $core.bool hasSchema() => $_has(0);
  @$pb.TagNumber(3)
  void clearSchema() => $_clearField(3);
  @$pb.TagNumber(3)
  CollectionSchema ensureSchema() => $_ensure(0);
}

class DeleteCollectionOp extends $pb.GeneratedMessage {
  factory DeleteCollectionOp() => create();

  DeleteCollectionOp._();

  factory DeleteCollectionOp.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory DeleteCollectionOp.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'DeleteCollectionOp',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'hyperspace'),
      createEmptyInstance: create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteCollectionOp clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteCollectionOp copyWith(void Function(DeleteCollectionOp) updates) =>
      super.copyWith((message) => updates(message as DeleteCollectionOp))
          as DeleteCollectionOp;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static DeleteCollectionOp create() => DeleteCollectionOp._();
  @$core.override
  DeleteCollectionOp createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static DeleteCollectionOp getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<DeleteCollectionOp>(create);
  static DeleteCollectionOp? _defaultInstance;
}

class DeleteOp extends $pb.GeneratedMessage {
  factory DeleteOp({
    $core.int? id,
  }) {
    final result = create();
    if (id != null) result.id = id;
    return result;
  }

  DeleteOp._();

  factory DeleteOp.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory DeleteOp.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'DeleteOp',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'hyperspace'),
      createEmptyInstance: create)
    ..aI(1, _omitFieldNames ? '' : 'id', fieldType: $pb.PbFieldType.OU3)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteOp clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteOp copyWith(void Function(DeleteOp) updates) =>
      super.copyWith((message) => updates(message as DeleteOp)) as DeleteOp;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static DeleteOp create() => DeleteOp._();
  @$core.override
  DeleteOp createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static DeleteOp getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<DeleteOp>(create);
  static DeleteOp? _defaultInstance;

  @$pb.TagNumber(1)
  $core.int get id => $_getIZ(0);
  @$pb.TagNumber(1)
  set id($core.int value) => $_setUnsignedInt32(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);
}

class QuantizationConfig extends $pb.GeneratedMessage {
  factory QuantizationConfig({
    QuantizationMode? mode,
  }) {
    final result = create();
    if (mode != null) result.mode = mode;
    return result;
  }

  QuantizationConfig._();

  factory QuantizationConfig.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory QuantizationConfig.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'QuantizationConfig',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'hyperspace'),
      createEmptyInstance: create)
    ..aE<QuantizationMode>(1, _omitFieldNames ? '' : 'mode',
        enumValues: QuantizationMode.values)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  QuantizationConfig clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  QuantizationConfig copyWith(void Function(QuantizationConfig) updates) =>
      super.copyWith((message) => updates(message as QuantizationConfig))
          as QuantizationConfig;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static QuantizationConfig create() => QuantizationConfig._();
  @$core.override
  QuantizationConfig createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static QuantizationConfig getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<QuantizationConfig>(create);
  static QuantizationConfig? _defaultInstance;

  @$pb.TagNumber(1)
  QuantizationMode get mode => $_getN(0);
  @$pb.TagNumber(1)
  set mode(QuantizationMode value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasMode() => $_has(0);
  @$pb.TagNumber(1)
  void clearMode() => $_clearField(1);
}

class CreateCollectionRequest extends $pb.GeneratedMessage {
  factory CreateCollectionRequest({
    $core.String? name,
    CollectionSchema? schema,
  }) {
    final result = create();
    if (name != null) result.name = name;
    if (schema != null) result.schema = schema;
    return result;
  }

  CreateCollectionRequest._();

  factory CreateCollectionRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory CreateCollectionRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'CreateCollectionRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'hyperspace'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'name')
    ..aOM<CollectionSchema>(5, _omitFieldNames ? '' : 'schema',
        subBuilder: CollectionSchema.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CreateCollectionRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CreateCollectionRequest copyWith(
          void Function(CreateCollectionRequest) updates) =>
      super.copyWith((message) => updates(message as CreateCollectionRequest))
          as CreateCollectionRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static CreateCollectionRequest create() => CreateCollectionRequest._();
  @$core.override
  CreateCollectionRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static CreateCollectionRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<CreateCollectionRequest>(create);
  static CreateCollectionRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get name => $_getSZ(0);
  @$pb.TagNumber(1)
  set name($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasName() => $_has(0);
  @$pb.TagNumber(1)
  void clearName() => $_clearField(1);

  @$pb.TagNumber(5)
  CollectionSchema get schema => $_getN(1);
  @$pb.TagNumber(5)
  set schema(CollectionSchema value) => $_setField(5, value);
  @$pb.TagNumber(5)
  $core.bool hasSchema() => $_has(1);
  @$pb.TagNumber(5)
  void clearSchema() => $_clearField(5);
  @$pb.TagNumber(5)
  CollectionSchema ensureSchema() => $_ensure(1);
}

class VectorComponent extends $pb.GeneratedMessage {
  factory VectorComponent({
    $core.String? name,
    $core.String? metric,
    $core.int? fullDimension,
    $core.double? weight,
  }) {
    final result = create();
    if (name != null) result.name = name;
    if (metric != null) result.metric = metric;
    if (fullDimension != null) result.fullDimension = fullDimension;
    if (weight != null) result.weight = weight;
    return result;
  }

  VectorComponent._();

  factory VectorComponent.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory VectorComponent.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'VectorComponent',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'hyperspace'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'name')
    ..aOS(2, _omitFieldNames ? '' : 'metric')
    ..aI(3, _omitFieldNames ? '' : 'fullDimension',
        fieldType: $pb.PbFieldType.OU3)
    ..aD(4, _omitFieldNames ? '' : 'weight', fieldType: $pb.PbFieldType.OF)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  VectorComponent clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  VectorComponent copyWith(void Function(VectorComponent) updates) =>
      super.copyWith((message) => updates(message as VectorComponent))
          as VectorComponent;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static VectorComponent create() => VectorComponent._();
  @$core.override
  VectorComponent createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static VectorComponent getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<VectorComponent>(create);
  static VectorComponent? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get name => $_getSZ(0);
  @$pb.TagNumber(1)
  set name($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasName() => $_has(0);
  @$pb.TagNumber(1)
  void clearName() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get metric => $_getSZ(1);
  @$pb.TagNumber(2)
  set metric($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasMetric() => $_has(1);
  @$pb.TagNumber(2)
  void clearMetric() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.int get fullDimension => $_getIZ(2);
  @$pb.TagNumber(3)
  set fullDimension($core.int value) => $_setUnsignedInt32(2, value);
  @$pb.TagNumber(3)
  $core.bool hasFullDimension() => $_has(2);
  @$pb.TagNumber(3)
  void clearFullDimension() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.double get weight => $_getN(3);
  @$pb.TagNumber(4)
  set weight($core.double value) => $_setFloat(3, value);
  @$pb.TagNumber(4)
  $core.bool hasWeight() => $_has(3);
  @$pb.TagNumber(4)
  void clearWeight() => $_clearField(4);
}

class MrlLayer extends $pb.GeneratedMessage {
  factory MrlLayer({
    $core.String? componentName,
    $core.int? cutoffDimension,
    $core.bool? storeInRam,
    $core.int? rerankTopK,
  }) {
    final result = create();
    if (componentName != null) result.componentName = componentName;
    if (cutoffDimension != null) result.cutoffDimension = cutoffDimension;
    if (storeInRam != null) result.storeInRam = storeInRam;
    if (rerankTopK != null) result.rerankTopK = rerankTopK;
    return result;
  }

  MrlLayer._();

  factory MrlLayer.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory MrlLayer.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'MrlLayer',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'hyperspace'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'componentName')
    ..aI(2, _omitFieldNames ? '' : 'cutoffDimension',
        fieldType: $pb.PbFieldType.OU3)
    ..aOB(3, _omitFieldNames ? '' : 'storeInRam')
    ..aI(4, _omitFieldNames ? '' : 'rerankTopK', fieldType: $pb.PbFieldType.OU3)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  MrlLayer clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  MrlLayer copyWith(void Function(MrlLayer) updates) =>
      super.copyWith((message) => updates(message as MrlLayer)) as MrlLayer;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static MrlLayer create() => MrlLayer._();
  @$core.override
  MrlLayer createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static MrlLayer getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<MrlLayer>(create);
  static MrlLayer? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get componentName => $_getSZ(0);
  @$pb.TagNumber(1)
  set componentName($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasComponentName() => $_has(0);
  @$pb.TagNumber(1)
  void clearComponentName() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.int get cutoffDimension => $_getIZ(1);
  @$pb.TagNumber(2)
  set cutoffDimension($core.int value) => $_setUnsignedInt32(1, value);
  @$pb.TagNumber(2)
  $core.bool hasCutoffDimension() => $_has(1);
  @$pb.TagNumber(2)
  void clearCutoffDimension() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.bool get storeInRam => $_getBF(2);
  @$pb.TagNumber(3)
  set storeInRam($core.bool value) => $_setBool(2, value);
  @$pb.TagNumber(3)
  $core.bool hasStoreInRam() => $_has(2);
  @$pb.TagNumber(3)
  void clearStoreInRam() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.int get rerankTopK => $_getIZ(3);
  @$pb.TagNumber(4)
  set rerankTopK($core.int value) => $_setUnsignedInt32(3, value);
  @$pb.TagNumber(4)
  $core.bool hasRerankTopK() => $_has(3);
  @$pb.TagNumber(4)
  void clearRerankTopK() => $_clearField(4);
}

class CollectionSchema extends $pb.GeneratedMessage {
  factory CollectionSchema({
    $core.Iterable<VectorComponent>? components,
    $core.Iterable<MrlLayer>? cascadePipeline,
  }) {
    final result = create();
    if (components != null) result.components.addAll(components);
    if (cascadePipeline != null) result.cascadePipeline.addAll(cascadePipeline);
    return result;
  }

  CollectionSchema._();

  factory CollectionSchema.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory CollectionSchema.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'CollectionSchema',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'hyperspace'),
      createEmptyInstance: create)
    ..pPM<VectorComponent>(1, _omitFieldNames ? '' : 'components',
        subBuilder: VectorComponent.create)
    ..pPM<MrlLayer>(2, _omitFieldNames ? '' : 'cascadePipeline',
        subBuilder: MrlLayer.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CollectionSchema clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CollectionSchema copyWith(void Function(CollectionSchema) updates) =>
      super.copyWith((message) => updates(message as CollectionSchema))
          as CollectionSchema;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static CollectionSchema create() => CollectionSchema._();
  @$core.override
  CollectionSchema createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static CollectionSchema getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<CollectionSchema>(create);
  static CollectionSchema? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<VectorComponent> get components => $_getList(0);

  @$pb.TagNumber(2)
  $pb.PbList<MrlLayer> get cascadePipeline => $_getList(1);
}

class CollectionComponent extends $pb.GeneratedMessage {
  factory CollectionComponent({
    $core.String? space,
    $core.int? dimension,
    $core.String? metric,
    $core.double? weight,
  }) {
    final result = create();
    if (space != null) result.space = space;
    if (dimension != null) result.dimension = dimension;
    if (metric != null) result.metric = metric;
    if (weight != null) result.weight = weight;
    return result;
  }

  CollectionComponent._();

  factory CollectionComponent.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory CollectionComponent.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'CollectionComponent',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'hyperspace'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'space')
    ..aI(2, _omitFieldNames ? '' : 'dimension', fieldType: $pb.PbFieldType.OU3)
    ..aOS(3, _omitFieldNames ? '' : 'metric')
    ..aD(4, _omitFieldNames ? '' : 'weight', fieldType: $pb.PbFieldType.OF)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CollectionComponent clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CollectionComponent copyWith(void Function(CollectionComponent) updates) =>
      super.copyWith((message) => updates(message as CollectionComponent))
          as CollectionComponent;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static CollectionComponent create() => CollectionComponent._();
  @$core.override
  CollectionComponent createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static CollectionComponent getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<CollectionComponent>(create);
  static CollectionComponent? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get space => $_getSZ(0);
  @$pb.TagNumber(1)
  set space($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasSpace() => $_has(0);
  @$pb.TagNumber(1)
  void clearSpace() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.int get dimension => $_getIZ(1);
  @$pb.TagNumber(2)
  set dimension($core.int value) => $_setUnsignedInt32(1, value);
  @$pb.TagNumber(2)
  $core.bool hasDimension() => $_has(1);
  @$pb.TagNumber(2)
  void clearDimension() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get metric => $_getSZ(2);
  @$pb.TagNumber(3)
  set metric($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasMetric() => $_has(2);
  @$pb.TagNumber(3)
  void clearMetric() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.double get weight => $_getN(3);
  @$pb.TagNumber(4)
  set weight($core.double value) => $_setFloat(3, value);
  @$pb.TagNumber(4)
  $core.bool hasWeight() => $_has(3);
  @$pb.TagNumber(4)
  void clearWeight() => $_clearField(4);
}

class DeleteCollectionRequest extends $pb.GeneratedMessage {
  factory DeleteCollectionRequest({
    $core.String? name,
  }) {
    final result = create();
    if (name != null) result.name = name;
    return result;
  }

  DeleteCollectionRequest._();

  factory DeleteCollectionRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory DeleteCollectionRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'DeleteCollectionRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'hyperspace'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'name')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteCollectionRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteCollectionRequest copyWith(
          void Function(DeleteCollectionRequest) updates) =>
      super.copyWith((message) => updates(message as DeleteCollectionRequest))
          as DeleteCollectionRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static DeleteCollectionRequest create() => DeleteCollectionRequest._();
  @$core.override
  DeleteCollectionRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static DeleteCollectionRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<DeleteCollectionRequest>(create);
  static DeleteCollectionRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get name => $_getSZ(0);
  @$pb.TagNumber(1)
  set name($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasName() => $_has(0);
  @$pb.TagNumber(1)
  void clearName() => $_clearField(1);
}

class CollectionSummary extends $pb.GeneratedMessage {
  factory CollectionSummary({
    $core.String? name,
    $fixnum.Int64? count,
    CollectionSchema? schema,
  }) {
    final result = create();
    if (name != null) result.name = name;
    if (count != null) result.count = count;
    if (schema != null) result.schema = schema;
    return result;
  }

  CollectionSummary._();

  factory CollectionSummary.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory CollectionSummary.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'CollectionSummary',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'hyperspace'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'name')
    ..a<$fixnum.Int64>(2, _omitFieldNames ? '' : 'count', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..aOM<CollectionSchema>(5, _omitFieldNames ? '' : 'schema',
        subBuilder: CollectionSchema.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CollectionSummary clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CollectionSummary copyWith(void Function(CollectionSummary) updates) =>
      super.copyWith((message) => updates(message as CollectionSummary))
          as CollectionSummary;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static CollectionSummary create() => CollectionSummary._();
  @$core.override
  CollectionSummary createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static CollectionSummary getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<CollectionSummary>(create);
  static CollectionSummary? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get name => $_getSZ(0);
  @$pb.TagNumber(1)
  set name($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasName() => $_has(0);
  @$pb.TagNumber(1)
  void clearName() => $_clearField(1);

  @$pb.TagNumber(2)
  $fixnum.Int64 get count => $_getI64(1);
  @$pb.TagNumber(2)
  set count($fixnum.Int64 value) => $_setInt64(1, value);
  @$pb.TagNumber(2)
  $core.bool hasCount() => $_has(1);
  @$pb.TagNumber(2)
  void clearCount() => $_clearField(2);

  @$pb.TagNumber(5)
  CollectionSchema get schema => $_getN(2);
  @$pb.TagNumber(5)
  set schema(CollectionSchema value) => $_setField(5, value);
  @$pb.TagNumber(5)
  $core.bool hasSchema() => $_has(2);
  @$pb.TagNumber(5)
  void clearSchema() => $_clearField(5);
  @$pb.TagNumber(5)
  CollectionSchema ensureSchema() => $_ensure(2);
}

class ListCollectionsResponse extends $pb.GeneratedMessage {
  factory ListCollectionsResponse({
    $core.Iterable<CollectionSummary>? collections,
  }) {
    final result = create();
    if (collections != null) result.collections.addAll(collections);
    return result;
  }

  ListCollectionsResponse._();

  factory ListCollectionsResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ListCollectionsResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListCollectionsResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'hyperspace'),
      createEmptyInstance: create)
    ..pPM<CollectionSummary>(1, _omitFieldNames ? '' : 'collections',
        subBuilder: CollectionSummary.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListCollectionsResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListCollectionsResponse copyWith(
          void Function(ListCollectionsResponse) updates) =>
      super.copyWith((message) => updates(message as ListCollectionsResponse))
          as ListCollectionsResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ListCollectionsResponse create() => ListCollectionsResponse._();
  @$core.override
  ListCollectionsResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ListCollectionsResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListCollectionsResponse>(create);
  static ListCollectionsResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<CollectionSummary> get collections => $_getList(0);
}

class CollectionStatsRequest extends $pb.GeneratedMessage {
  factory CollectionStatsRequest({
    $core.String? name,
  }) {
    final result = create();
    if (name != null) result.name = name;
    return result;
  }

  CollectionStatsRequest._();

  factory CollectionStatsRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory CollectionStatsRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'CollectionStatsRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'hyperspace'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'name')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CollectionStatsRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CollectionStatsRequest copyWith(
          void Function(CollectionStatsRequest) updates) =>
      super.copyWith((message) => updates(message as CollectionStatsRequest))
          as CollectionStatsRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static CollectionStatsRequest create() => CollectionStatsRequest._();
  @$core.override
  CollectionStatsRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static CollectionStatsRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<CollectionStatsRequest>(create);
  static CollectionStatsRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get name => $_getSZ(0);
  @$pb.TagNumber(1)
  set name($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasName() => $_has(0);
  @$pb.TagNumber(1)
  void clearName() => $_clearField(1);
}

class CollectionStatsResponse extends $pb.GeneratedMessage {
  factory CollectionStatsResponse({
    $fixnum.Int64? count,
    $fixnum.Int64? indexingQueue,
    $fixnum.Int64? diskUsageBytes,
    $fixnum.Int64? ramUsageBytes,
    $fixnum.Int64? activeTasks,
    CollectionSchema? schema,
  }) {
    final result = create();
    if (count != null) result.count = count;
    if (indexingQueue != null) result.indexingQueue = indexingQueue;
    if (diskUsageBytes != null) result.diskUsageBytes = diskUsageBytes;
    if (ramUsageBytes != null) result.ramUsageBytes = ramUsageBytes;
    if (activeTasks != null) result.activeTasks = activeTasks;
    if (schema != null) result.schema = schema;
    return result;
  }

  CollectionStatsResponse._();

  factory CollectionStatsResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory CollectionStatsResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'CollectionStatsResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'hyperspace'),
      createEmptyInstance: create)
    ..a<$fixnum.Int64>(1, _omitFieldNames ? '' : 'count', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..a<$fixnum.Int64>(
        4, _omitFieldNames ? '' : 'indexingQueue', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..a<$fixnum.Int64>(
        5, _omitFieldNames ? '' : 'diskUsageBytes', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..a<$fixnum.Int64>(
        6, _omitFieldNames ? '' : 'ramUsageBytes', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..a<$fixnum.Int64>(
        7, _omitFieldNames ? '' : 'activeTasks', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..aOM<CollectionSchema>(8, _omitFieldNames ? '' : 'schema',
        subBuilder: CollectionSchema.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CollectionStatsResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CollectionStatsResponse copyWith(
          void Function(CollectionStatsResponse) updates) =>
      super.copyWith((message) => updates(message as CollectionStatsResponse))
          as CollectionStatsResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static CollectionStatsResponse create() => CollectionStatsResponse._();
  @$core.override
  CollectionStatsResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static CollectionStatsResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<CollectionStatsResponse>(create);
  static CollectionStatsResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $fixnum.Int64 get count => $_getI64(0);
  @$pb.TagNumber(1)
  set count($fixnum.Int64 value) => $_setInt64(0, value);
  @$pb.TagNumber(1)
  $core.bool hasCount() => $_has(0);
  @$pb.TagNumber(1)
  void clearCount() => $_clearField(1);

  @$pb.TagNumber(4)
  $fixnum.Int64 get indexingQueue => $_getI64(1);
  @$pb.TagNumber(4)
  set indexingQueue($fixnum.Int64 value) => $_setInt64(1, value);
  @$pb.TagNumber(4)
  $core.bool hasIndexingQueue() => $_has(1);
  @$pb.TagNumber(4)
  void clearIndexingQueue() => $_clearField(4);

  @$pb.TagNumber(5)
  $fixnum.Int64 get diskUsageBytes => $_getI64(2);
  @$pb.TagNumber(5)
  set diskUsageBytes($fixnum.Int64 value) => $_setInt64(2, value);
  @$pb.TagNumber(5)
  $core.bool hasDiskUsageBytes() => $_has(2);
  @$pb.TagNumber(5)
  void clearDiskUsageBytes() => $_clearField(5);

  @$pb.TagNumber(6)
  $fixnum.Int64 get ramUsageBytes => $_getI64(3);
  @$pb.TagNumber(6)
  set ramUsageBytes($fixnum.Int64 value) => $_setInt64(3, value);
  @$pb.TagNumber(6)
  $core.bool hasRamUsageBytes() => $_has(3);
  @$pb.TagNumber(6)
  void clearRamUsageBytes() => $_clearField(6);

  @$pb.TagNumber(7)
  $fixnum.Int64 get activeTasks => $_getI64(4);
  @$pb.TagNumber(7)
  set activeTasks($fixnum.Int64 value) => $_setInt64(4, value);
  @$pb.TagNumber(7)
  $core.bool hasActiveTasks() => $_has(4);
  @$pb.TagNumber(7)
  void clearActiveTasks() => $_clearField(7);

  @$pb.TagNumber(8)
  CollectionSchema get schema => $_getN(5);
  @$pb.TagNumber(8)
  set schema(CollectionSchema value) => $_setField(8, value);
  @$pb.TagNumber(8)
  $core.bool hasSchema() => $_has(5);
  @$pb.TagNumber(8)
  void clearSchema() => $_clearField(8);
  @$pb.TagNumber(8)
  CollectionSchema ensureSchema() => $_ensure(5);
}

class RebuildIndexRequest extends $pb.GeneratedMessage {
  factory RebuildIndexRequest({
    $core.String? name,
    VacuumFilterQuery? filterQuery,
  }) {
    final result = create();
    if (name != null) result.name = name;
    if (filterQuery != null) result.filterQuery = filterQuery;
    return result;
  }

  RebuildIndexRequest._();

  factory RebuildIndexRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory RebuildIndexRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'RebuildIndexRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'hyperspace'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'name')
    ..aOM<VacuumFilterQuery>(2, _omitFieldNames ? '' : 'filterQuery',
        subBuilder: VacuumFilterQuery.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  RebuildIndexRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  RebuildIndexRequest copyWith(void Function(RebuildIndexRequest) updates) =>
      super.copyWith((message) => updates(message as RebuildIndexRequest))
          as RebuildIndexRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static RebuildIndexRequest create() => RebuildIndexRequest._();
  @$core.override
  RebuildIndexRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static RebuildIndexRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<RebuildIndexRequest>(create);
  static RebuildIndexRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get name => $_getSZ(0);
  @$pb.TagNumber(1)
  set name($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasName() => $_has(0);
  @$pb.TagNumber(1)
  void clearName() => $_clearField(1);

  @$pb.TagNumber(2)
  VacuumFilterQuery get filterQuery => $_getN(1);
  @$pb.TagNumber(2)
  set filterQuery(VacuumFilterQuery value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasFilterQuery() => $_has(1);
  @$pb.TagNumber(2)
  void clearFilterQuery() => $_clearField(2);
  @$pb.TagNumber(2)
  VacuumFilterQuery ensureFilterQuery() => $_ensure(1);
}

class ConfigUpdate extends $pb.GeneratedMessage {
  factory ConfigUpdate({
    $core.String? collection,
    $core.int? efSearch,
    $core.int? efConstruction,
    $core.int? m,
  }) {
    final result = create();
    if (collection != null) result.collection = collection;
    if (efSearch != null) result.efSearch = efSearch;
    if (efConstruction != null) result.efConstruction = efConstruction;
    if (m != null) result.m = m;
    return result;
  }

  ConfigUpdate._();

  factory ConfigUpdate.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ConfigUpdate.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ConfigUpdate',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'hyperspace'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'collection')
    ..aI(2, _omitFieldNames ? '' : 'efSearch', fieldType: $pb.PbFieldType.OU3)
    ..aI(3, _omitFieldNames ? '' : 'efConstruction',
        fieldType: $pb.PbFieldType.OU3)
    ..aI(4, _omitFieldNames ? '' : 'm', fieldType: $pb.PbFieldType.OU3)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ConfigUpdate clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ConfigUpdate copyWith(void Function(ConfigUpdate) updates) =>
      super.copyWith((message) => updates(message as ConfigUpdate))
          as ConfigUpdate;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ConfigUpdate create() => ConfigUpdate._();
  @$core.override
  ConfigUpdate createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ConfigUpdate getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ConfigUpdate>(create);
  static ConfigUpdate? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get collection => $_getSZ(0);
  @$pb.TagNumber(1)
  set collection($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasCollection() => $_has(0);
  @$pb.TagNumber(1)
  void clearCollection() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.int get efSearch => $_getIZ(1);
  @$pb.TagNumber(2)
  set efSearch($core.int value) => $_setUnsignedInt32(1, value);
  @$pb.TagNumber(2)
  $core.bool hasEfSearch() => $_has(1);
  @$pb.TagNumber(2)
  void clearEfSearch() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.int get efConstruction => $_getIZ(2);
  @$pb.TagNumber(3)
  set efConstruction($core.int value) => $_setUnsignedInt32(2, value);
  @$pb.TagNumber(3)
  $core.bool hasEfConstruction() => $_has(2);
  @$pb.TagNumber(3)
  void clearEfConstruction() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.int get m => $_getIZ(3);
  @$pb.TagNumber(4)
  set m($core.int value) => $_setUnsignedInt32(3, value);
  @$pb.TagNumber(4)
  $core.bool hasM() => $_has(3);
  @$pb.TagNumber(4)
  void clearM() => $_clearField(4);
}

class VacuumFilterQuery extends $pb.GeneratedMessage {
  factory VacuumFilterQuery({
    $core.String? key,
    $core.String? op,
    $core.double? value,
  }) {
    final result = create();
    if (key != null) result.key = key;
    if (op != null) result.op = op;
    if (value != null) result.value = value;
    return result;
  }

  VacuumFilterQuery._();

  factory VacuumFilterQuery.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory VacuumFilterQuery.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'VacuumFilterQuery',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'hyperspace'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'key')
    ..aOS(2, _omitFieldNames ? '' : 'op')
    ..aD(3, _omitFieldNames ? '' : 'value')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  VacuumFilterQuery clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  VacuumFilterQuery copyWith(void Function(VacuumFilterQuery) updates) =>
      super.copyWith((message) => updates(message as VacuumFilterQuery))
          as VacuumFilterQuery;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static VacuumFilterQuery create() => VacuumFilterQuery._();
  @$core.override
  VacuumFilterQuery createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static VacuumFilterQuery getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<VacuumFilterQuery>(create);
  static VacuumFilterQuery? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get key => $_getSZ(0);
  @$pb.TagNumber(1)
  set key($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasKey() => $_has(0);
  @$pb.TagNumber(1)
  void clearKey() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get op => $_getSZ(1);
  @$pb.TagNumber(2)
  set op($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasOp() => $_has(1);
  @$pb.TagNumber(2)
  void clearOp() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.double get value => $_getN(2);
  @$pb.TagNumber(3)
  set value($core.double value) => $_setDouble(2, value);
  @$pb.TagNumber(3)
  $core.bool hasValue() => $_has(2);
  @$pb.TagNumber(3)
  void clearValue() => $_clearField(3);
}

class ReconsolidationRequest extends $pb.GeneratedMessage {
  factory ReconsolidationRequest({
    $core.String? collection,
    $core.Iterable<$core.double>? targetVector,
    $core.double? learningRate,
  }) {
    final result = create();
    if (collection != null) result.collection = collection;
    if (targetVector != null) result.targetVector.addAll(targetVector);
    if (learningRate != null) result.learningRate = learningRate;
    return result;
  }

  ReconsolidationRequest._();

  factory ReconsolidationRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ReconsolidationRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ReconsolidationRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'hyperspace'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'collection')
    ..p<$core.double>(
        2, _omitFieldNames ? '' : 'targetVector', $pb.PbFieldType.KD)
    ..aD(3, _omitFieldNames ? '' : 'learningRate')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ReconsolidationRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ReconsolidationRequest copyWith(
          void Function(ReconsolidationRequest) updates) =>
      super.copyWith((message) => updates(message as ReconsolidationRequest))
          as ReconsolidationRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ReconsolidationRequest create() => ReconsolidationRequest._();
  @$core.override
  ReconsolidationRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ReconsolidationRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ReconsolidationRequest>(create);
  static ReconsolidationRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get collection => $_getSZ(0);
  @$pb.TagNumber(1)
  set collection($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasCollection() => $_has(0);
  @$pb.TagNumber(1)
  void clearCollection() => $_clearField(1);

  @$pb.TagNumber(2)
  $pb.PbList<$core.double> get targetVector => $_getList(1);

  @$pb.TagNumber(3)
  $core.double get learningRate => $_getN(2);
  @$pb.TagNumber(3)
  set learningRate($core.double value) => $_setDouble(2, value);
  @$pb.TagNumber(3)
  $core.bool hasLearningRate() => $_has(2);
  @$pb.TagNumber(3)
  void clearLearningRate() => $_clearField(3);
}

class InsertRequest extends $pb.GeneratedMessage {
  factory InsertRequest({
    $core.String? collection,
    $core.Iterable<$core.double>? vector,
    $core.int? id,
    $core.Iterable<$core.MapEntry<$core.String, $core.String>>? metadata,
    $core.String? originNodeId,
    $fixnum.Int64? logicalClock,
    DurabilityLevel? durability,
    $core.Iterable<$core.MapEntry<$core.String, MetadataValue>>? typedMetadata,
    $core.List<$core.int>? payload,
  }) {
    final result = create();
    if (collection != null) result.collection = collection;
    if (vector != null) result.vector.addAll(vector);
    if (id != null) result.id = id;
    if (metadata != null) result.metadata.addEntries(metadata);
    if (originNodeId != null) result.originNodeId = originNodeId;
    if (logicalClock != null) result.logicalClock = logicalClock;
    if (durability != null) result.durability = durability;
    if (typedMetadata != null) result.typedMetadata.addEntries(typedMetadata);
    if (payload != null) result.payload = payload;
    return result;
  }

  InsertRequest._();

  factory InsertRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory InsertRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'InsertRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'hyperspace'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'collection')
    ..p<$core.double>(2, _omitFieldNames ? '' : 'vector', $pb.PbFieldType.KD)
    ..aI(3, _omitFieldNames ? '' : 'id', fieldType: $pb.PbFieldType.OU3)
    ..m<$core.String, $core.String>(4, _omitFieldNames ? '' : 'metadata',
        entryClassName: 'InsertRequest.MetadataEntry',
        keyFieldType: $pb.PbFieldType.OS,
        valueFieldType: $pb.PbFieldType.OS,
        packageName: const $pb.PackageName('hyperspace'))
    ..aOS(5, _omitFieldNames ? '' : 'originNodeId')
    ..a<$fixnum.Int64>(
        6, _omitFieldNames ? '' : 'logicalClock', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..aE<DurabilityLevel>(7, _omitFieldNames ? '' : 'durability',
        enumValues: DurabilityLevel.values)
    ..m<$core.String, MetadataValue>(8, _omitFieldNames ? '' : 'typedMetadata',
        entryClassName: 'InsertRequest.TypedMetadataEntry',
        keyFieldType: $pb.PbFieldType.OS,
        valueFieldType: $pb.PbFieldType.OM,
        valueCreator: MetadataValue.create,
        valueDefaultOrMaker: MetadataValue.getDefault,
        packageName: const $pb.PackageName('hyperspace'))
    ..a<$core.List<$core.int>>(
        9, _omitFieldNames ? '' : 'payload', $pb.PbFieldType.OY)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  InsertRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  InsertRequest copyWith(void Function(InsertRequest) updates) =>
      super.copyWith((message) => updates(message as InsertRequest))
          as InsertRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static InsertRequest create() => InsertRequest._();
  @$core.override
  InsertRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static InsertRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<InsertRequest>(create);
  static InsertRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get collection => $_getSZ(0);
  @$pb.TagNumber(1)
  set collection($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasCollection() => $_has(0);
  @$pb.TagNumber(1)
  void clearCollection() => $_clearField(1);

  @$pb.TagNumber(2)
  $pb.PbList<$core.double> get vector => $_getList(1);

  @$pb.TagNumber(3)
  $core.int get id => $_getIZ(2);
  @$pb.TagNumber(3)
  set id($core.int value) => $_setUnsignedInt32(2, value);
  @$pb.TagNumber(3)
  $core.bool hasId() => $_has(2);
  @$pb.TagNumber(3)
  void clearId() => $_clearField(3);

  @$pb.TagNumber(4)
  $pb.PbMap<$core.String, $core.String> get metadata => $_getMap(3);

  /// Replication fields
  @$pb.TagNumber(5)
  $core.String get originNodeId => $_getSZ(4);
  @$pb.TagNumber(5)
  set originNodeId($core.String value) => $_setString(4, value);
  @$pb.TagNumber(5)
  $core.bool hasOriginNodeId() => $_has(4);
  @$pb.TagNumber(5)
  void clearOriginNodeId() => $_clearField(5);

  @$pb.TagNumber(6)
  $fixnum.Int64 get logicalClock => $_getI64(5);
  @$pb.TagNumber(6)
  set logicalClock($fixnum.Int64 value) => $_setInt64(5, value);
  @$pb.TagNumber(6)
  $core.bool hasLogicalClock() => $_has(5);
  @$pb.TagNumber(6)
  void clearLogicalClock() => $_clearField(6);

  @$pb.TagNumber(7)
  DurabilityLevel get durability => $_getN(6);
  @$pb.TagNumber(7)
  set durability(DurabilityLevel value) => $_setField(7, value);
  @$pb.TagNumber(7)
  $core.bool hasDurability() => $_has(6);
  @$pb.TagNumber(7)
  void clearDurability() => $_clearField(7);

  @$pb.TagNumber(8)
  $pb.PbMap<$core.String, MetadataValue> get typedMetadata => $_getMap(7);

  /// Sidecar Payload Storage (v3.2): heavy documents stored on disk, NOT in RAM.
  /// The payload is zstd-compressed and written to the chunk file's Payload Layer.
  /// It is NEVER loaded into memory during HNSW traversal.
  @$pb.TagNumber(9)
  $core.List<$core.int> get payload => $_getN(8);
  @$pb.TagNumber(9)
  set payload($core.List<$core.int> value) => $_setBytes(8, value);
  @$pb.TagNumber(9)
  $core.bool hasPayload() => $_has(8);
  @$pb.TagNumber(9)
  void clearPayload() => $_clearField(9);
}

class VectorData extends $pb.GeneratedMessage {
  factory VectorData({
    $core.Iterable<$core.double>? vector,
    $core.int? id,
    $core.Iterable<$core.MapEntry<$core.String, $core.String>>? metadata,
    $core.Iterable<$core.MapEntry<$core.String, MetadataValue>>? typedMetadata,
  }) {
    final result = create();
    if (vector != null) result.vector.addAll(vector);
    if (id != null) result.id = id;
    if (metadata != null) result.metadata.addEntries(metadata);
    if (typedMetadata != null) result.typedMetadata.addEntries(typedMetadata);
    return result;
  }

  VectorData._();

  factory VectorData.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory VectorData.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'VectorData',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'hyperspace'),
      createEmptyInstance: create)
    ..p<$core.double>(1, _omitFieldNames ? '' : 'vector', $pb.PbFieldType.KD)
    ..aI(2, _omitFieldNames ? '' : 'id', fieldType: $pb.PbFieldType.OU3)
    ..m<$core.String, $core.String>(3, _omitFieldNames ? '' : 'metadata',
        entryClassName: 'VectorData.MetadataEntry',
        keyFieldType: $pb.PbFieldType.OS,
        valueFieldType: $pb.PbFieldType.OS,
        packageName: const $pb.PackageName('hyperspace'))
    ..m<$core.String, MetadataValue>(4, _omitFieldNames ? '' : 'typedMetadata',
        entryClassName: 'VectorData.TypedMetadataEntry',
        keyFieldType: $pb.PbFieldType.OS,
        valueFieldType: $pb.PbFieldType.OM,
        valueCreator: MetadataValue.create,
        valueDefaultOrMaker: MetadataValue.getDefault,
        packageName: const $pb.PackageName('hyperspace'))
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  VectorData clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  VectorData copyWith(void Function(VectorData) updates) =>
      super.copyWith((message) => updates(message as VectorData)) as VectorData;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static VectorData create() => VectorData._();
  @$core.override
  VectorData createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static VectorData getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<VectorData>(create);
  static VectorData? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<$core.double> get vector => $_getList(0);

  @$pb.TagNumber(2)
  $core.int get id => $_getIZ(1);
  @$pb.TagNumber(2)
  set id($core.int value) => $_setUnsignedInt32(1, value);
  @$pb.TagNumber(2)
  $core.bool hasId() => $_has(1);
  @$pb.TagNumber(2)
  void clearId() => $_clearField(2);

  @$pb.TagNumber(3)
  $pb.PbMap<$core.String, $core.String> get metadata => $_getMap(2);

  @$pb.TagNumber(4)
  $pb.PbMap<$core.String, MetadataValue> get typedMetadata => $_getMap(3);
}

class BatchInsertRequest extends $pb.GeneratedMessage {
  factory BatchInsertRequest({
    $core.String? collection,
    $core.Iterable<VectorData>? vectors,
    $core.String? originNodeId,
    $fixnum.Int64? logicalClock,
    DurabilityLevel? durability,
  }) {
    final result = create();
    if (collection != null) result.collection = collection;
    if (vectors != null) result.vectors.addAll(vectors);
    if (originNodeId != null) result.originNodeId = originNodeId;
    if (logicalClock != null) result.logicalClock = logicalClock;
    if (durability != null) result.durability = durability;
    return result;
  }

  BatchInsertRequest._();

  factory BatchInsertRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory BatchInsertRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'BatchInsertRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'hyperspace'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'collection')
    ..pPM<VectorData>(2, _omitFieldNames ? '' : 'vectors',
        subBuilder: VectorData.create)
    ..aOS(3, _omitFieldNames ? '' : 'originNodeId')
    ..a<$fixnum.Int64>(
        4, _omitFieldNames ? '' : 'logicalClock', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..aE<DurabilityLevel>(5, _omitFieldNames ? '' : 'durability',
        enumValues: DurabilityLevel.values)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  BatchInsertRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  BatchInsertRequest copyWith(void Function(BatchInsertRequest) updates) =>
      super.copyWith((message) => updates(message as BatchInsertRequest))
          as BatchInsertRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static BatchInsertRequest create() => BatchInsertRequest._();
  @$core.override
  BatchInsertRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static BatchInsertRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<BatchInsertRequest>(create);
  static BatchInsertRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get collection => $_getSZ(0);
  @$pb.TagNumber(1)
  set collection($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasCollection() => $_has(0);
  @$pb.TagNumber(1)
  void clearCollection() => $_clearField(1);

  @$pb.TagNumber(2)
  $pb.PbList<VectorData> get vectors => $_getList(1);

  @$pb.TagNumber(3)
  $core.String get originNodeId => $_getSZ(2);
  @$pb.TagNumber(3)
  set originNodeId($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasOriginNodeId() => $_has(2);
  @$pb.TagNumber(3)
  void clearOriginNodeId() => $_clearField(3);

  @$pb.TagNumber(4)
  $fixnum.Int64 get logicalClock => $_getI64(3);
  @$pb.TagNumber(4)
  set logicalClock($fixnum.Int64 value) => $_setInt64(3, value);
  @$pb.TagNumber(4)
  $core.bool hasLogicalClock() => $_has(3);
  @$pb.TagNumber(4)
  void clearLogicalClock() => $_clearField(4);

  @$pb.TagNumber(5)
  DurabilityLevel get durability => $_getN(4);
  @$pb.TagNumber(5)
  set durability(DurabilityLevel value) => $_setField(5, value);
  @$pb.TagNumber(5)
  $core.bool hasDurability() => $_has(4);
  @$pb.TagNumber(5)
  void clearDurability() => $_clearField(5);
}

class InsertTextRequest extends $pb.GeneratedMessage {
  factory InsertTextRequest({
    $core.String? collection,
    $core.int? id,
    $core.String? text,
    $core.Iterable<$core.MapEntry<$core.String, $core.String>>? metadata,
    DurabilityLevel? durability,
  }) {
    final result = create();
    if (collection != null) result.collection = collection;
    if (id != null) result.id = id;
    if (text != null) result.text = text;
    if (metadata != null) result.metadata.addEntries(metadata);
    if (durability != null) result.durability = durability;
    return result;
  }

  InsertTextRequest._();

  factory InsertTextRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory InsertTextRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'InsertTextRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'hyperspace'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'collection')
    ..aI(2, _omitFieldNames ? '' : 'id', fieldType: $pb.PbFieldType.OU3)
    ..aOS(3, _omitFieldNames ? '' : 'text')
    ..m<$core.String, $core.String>(4, _omitFieldNames ? '' : 'metadata',
        entryClassName: 'InsertTextRequest.MetadataEntry',
        keyFieldType: $pb.PbFieldType.OS,
        valueFieldType: $pb.PbFieldType.OS,
        packageName: const $pb.PackageName('hyperspace'))
    ..aE<DurabilityLevel>(5, _omitFieldNames ? '' : 'durability',
        enumValues: DurabilityLevel.values)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  InsertTextRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  InsertTextRequest copyWith(void Function(InsertTextRequest) updates) =>
      super.copyWith((message) => updates(message as InsertTextRequest))
          as InsertTextRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static InsertTextRequest create() => InsertTextRequest._();
  @$core.override
  InsertTextRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static InsertTextRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<InsertTextRequest>(create);
  static InsertTextRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get collection => $_getSZ(0);
  @$pb.TagNumber(1)
  set collection($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasCollection() => $_has(0);
  @$pb.TagNumber(1)
  void clearCollection() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.int get id => $_getIZ(1);
  @$pb.TagNumber(2)
  set id($core.int value) => $_setUnsignedInt32(1, value);
  @$pb.TagNumber(2)
  $core.bool hasId() => $_has(1);
  @$pb.TagNumber(2)
  void clearId() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get text => $_getSZ(2);
  @$pb.TagNumber(3)
  set text($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasText() => $_has(2);
  @$pb.TagNumber(3)
  void clearText() => $_clearField(3);

  @$pb.TagNumber(4)
  $pb.PbMap<$core.String, $core.String> get metadata => $_getMap(3);

  @$pb.TagNumber(5)
  DurabilityLevel get durability => $_getN(4);
  @$pb.TagNumber(5)
  set durability(DurabilityLevel value) => $_setField(5, value);
  @$pb.TagNumber(5)
  $core.bool hasDurability() => $_has(4);
  @$pb.TagNumber(5)
  void clearDurability() => $_clearField(5);
}

class VectorizeRequest extends $pb.GeneratedMessage {
  factory VectorizeRequest({
    $core.String? text,
    $core.String? metric,
  }) {
    final result = create();
    if (text != null) result.text = text;
    if (metric != null) result.metric = metric;
    return result;
  }

  VectorizeRequest._();

  factory VectorizeRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory VectorizeRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'VectorizeRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'hyperspace'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'text')
    ..aOS(2, _omitFieldNames ? '' : 'metric')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  VectorizeRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  VectorizeRequest copyWith(void Function(VectorizeRequest) updates) =>
      super.copyWith((message) => updates(message as VectorizeRequest))
          as VectorizeRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static VectorizeRequest create() => VectorizeRequest._();
  @$core.override
  VectorizeRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static VectorizeRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<VectorizeRequest>(create);
  static VectorizeRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get text => $_getSZ(0);
  @$pb.TagNumber(1)
  set text($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasText() => $_has(0);
  @$pb.TagNumber(1)
  void clearText() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get metric => $_getSZ(1);
  @$pb.TagNumber(2)
  set metric($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasMetric() => $_has(1);
  @$pb.TagNumber(2)
  void clearMetric() => $_clearField(2);
}

class VectorizeResponse extends $pb.GeneratedMessage {
  factory VectorizeResponse({
    $core.Iterable<$core.double>? vector,
  }) {
    final result = create();
    if (vector != null) result.vector.addAll(vector);
    return result;
  }

  VectorizeResponse._();

  factory VectorizeResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory VectorizeResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'VectorizeResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'hyperspace'),
      createEmptyInstance: create)
    ..p<$core.double>(1, _omitFieldNames ? '' : 'vector', $pb.PbFieldType.KD)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  VectorizeResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  VectorizeResponse copyWith(void Function(VectorizeResponse) updates) =>
      super.copyWith((message) => updates(message as VectorizeResponse))
          as VectorizeResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static VectorizeResponse create() => VectorizeResponse._();
  @$core.override
  VectorizeResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static VectorizeResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<VectorizeResponse>(create);
  static VectorizeResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<$core.double> get vector => $_getList(0);
}

class SearchTextRequest extends $pb.GeneratedMessage {
  factory SearchTextRequest({
    $core.String? collection,
    $core.String? text,
    $core.int? topK,
    $core.Iterable<$core.MapEntry<$core.String, $core.String>>? filter,
    $core.Iterable<Filter>? filters,
    Bm25Options? bm25Options,
    $core.double? hybridAlpha,
    $core.bool? includePayload,
    $core.Iterable<$core.MapEntry<$core.String, $core.double>>?
        componentWeights,
  }) {
    final result = create();
    if (collection != null) result.collection = collection;
    if (text != null) result.text = text;
    if (topK != null) result.topK = topK;
    if (filter != null) result.filter.addEntries(filter);
    if (filters != null) result.filters.addAll(filters);
    if (bm25Options != null) result.bm25Options = bm25Options;
    if (hybridAlpha != null) result.hybridAlpha = hybridAlpha;
    if (includePayload != null) result.includePayload = includePayload;
    if (componentWeights != null)
      result.componentWeights.addEntries(componentWeights);
    return result;
  }

  SearchTextRequest._();

  factory SearchTextRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory SearchTextRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'SearchTextRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'hyperspace'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'collection')
    ..aOS(2, _omitFieldNames ? '' : 'text')
    ..aI(3, _omitFieldNames ? '' : 'topK', fieldType: $pb.PbFieldType.OU3)
    ..m<$core.String, $core.String>(4, _omitFieldNames ? '' : 'filter',
        entryClassName: 'SearchTextRequest.FilterEntry',
        keyFieldType: $pb.PbFieldType.OS,
        valueFieldType: $pb.PbFieldType.OS,
        packageName: const $pb.PackageName('hyperspace'))
    ..pPM<Filter>(5, _omitFieldNames ? '' : 'filters',
        subBuilder: Filter.create)
    ..aOM<Bm25Options>(6, _omitFieldNames ? '' : 'bm25Options',
        subBuilder: Bm25Options.create)
    ..aD(7, _omitFieldNames ? '' : 'hybridAlpha', fieldType: $pb.PbFieldType.OF)
    ..aOB(8, _omitFieldNames ? '' : 'includePayload')
    ..m<$core.String, $core.double>(
        9, _omitFieldNames ? '' : 'componentWeights',
        entryClassName: 'SearchTextRequest.ComponentWeightsEntry',
        keyFieldType: $pb.PbFieldType.OS,
        valueFieldType: $pb.PbFieldType.OF,
        packageName: const $pb.PackageName('hyperspace'))
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SearchTextRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SearchTextRequest copyWith(void Function(SearchTextRequest) updates) =>
      super.copyWith((message) => updates(message as SearchTextRequest))
          as SearchTextRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static SearchTextRequest create() => SearchTextRequest._();
  @$core.override
  SearchTextRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static SearchTextRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<SearchTextRequest>(create);
  static SearchTextRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get collection => $_getSZ(0);
  @$pb.TagNumber(1)
  set collection($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasCollection() => $_has(0);
  @$pb.TagNumber(1)
  void clearCollection() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get text => $_getSZ(1);
  @$pb.TagNumber(2)
  set text($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasText() => $_has(1);
  @$pb.TagNumber(2)
  void clearText() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.int get topK => $_getIZ(2);
  @$pb.TagNumber(3)
  set topK($core.int value) => $_setUnsignedInt32(2, value);
  @$pb.TagNumber(3)
  $core.bool hasTopK() => $_has(2);
  @$pb.TagNumber(3)
  void clearTopK() => $_clearField(3);

  @$pb.TagNumber(4)
  $pb.PbMap<$core.String, $core.String> get filter => $_getMap(3);

  @$pb.TagNumber(5)
  $pb.PbList<Filter> get filters => $_getList(4);

  @$pb.TagNumber(6)
  Bm25Options get bm25Options => $_getN(5);
  @$pb.TagNumber(6)
  set bm25Options(Bm25Options value) => $_setField(6, value);
  @$pb.TagNumber(6)
  $core.bool hasBm25Options() => $_has(5);
  @$pb.TagNumber(6)
  void clearBm25Options() => $_clearField(6);
  @$pb.TagNumber(6)
  Bm25Options ensureBm25Options() => $_ensure(5);

  @$pb.TagNumber(7)
  $core.double get hybridAlpha => $_getN(6);
  @$pb.TagNumber(7)
  set hybridAlpha($core.double value) => $_setFloat(6, value);
  @$pb.TagNumber(7)
  $core.bool hasHybridAlpha() => $_has(6);
  @$pb.TagNumber(7)
  void clearHybridAlpha() => $_clearField(7);

  @$pb.TagNumber(8)
  $core.bool get includePayload => $_getBF(7);
  @$pb.TagNumber(8)
  set includePayload($core.bool value) => $_setBool(7, value);
  @$pb.TagNumber(8)
  $core.bool hasIncludePayload() => $_has(7);
  @$pb.TagNumber(8)
  void clearIncludePayload() => $_clearField(8);

  @$pb.TagNumber(9)
  $pb.PbMap<$core.String, $core.double> get componentWeights => $_getMap(8);
}

class Bm25Options extends $pb.GeneratedMessage {
  factory Bm25Options({
    $core.String? method,
    $core.double? k1,
    $core.double? b,
    $core.double? delta,
    $core.String? language,
    $core.int? ngrams,
    $core.String? fusionMethod,
  }) {
    final result = create();
    if (method != null) result.method = method;
    if (k1 != null) result.k1 = k1;
    if (b != null) result.b = b;
    if (delta != null) result.delta = delta;
    if (language != null) result.language = language;
    if (ngrams != null) result.ngrams = ngrams;
    if (fusionMethod != null) result.fusionMethod = fusionMethod;
    return result;
  }

  Bm25Options._();

  factory Bm25Options.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory Bm25Options.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'Bm25Options',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'hyperspace'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'method')
    ..aD(2, _omitFieldNames ? '' : 'k1', fieldType: $pb.PbFieldType.OF)
    ..aD(3, _omitFieldNames ? '' : 'b', fieldType: $pb.PbFieldType.OF)
    ..aD(4, _omitFieldNames ? '' : 'delta', fieldType: $pb.PbFieldType.OF)
    ..aOS(5, _omitFieldNames ? '' : 'language')
    ..aI(6, _omitFieldNames ? '' : 'ngrams', fieldType: $pb.PbFieldType.OU3)
    ..aOS(7, _omitFieldNames ? '' : 'fusionMethod')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Bm25Options clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Bm25Options copyWith(void Function(Bm25Options) updates) =>
      super.copyWith((message) => updates(message as Bm25Options))
          as Bm25Options;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static Bm25Options create() => Bm25Options._();
  @$core.override
  Bm25Options createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static Bm25Options getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<Bm25Options>(create);
  static Bm25Options? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get method => $_getSZ(0);
  @$pb.TagNumber(1)
  set method($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasMethod() => $_has(0);
  @$pb.TagNumber(1)
  void clearMethod() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.double get k1 => $_getN(1);
  @$pb.TagNumber(2)
  set k1($core.double value) => $_setFloat(1, value);
  @$pb.TagNumber(2)
  $core.bool hasK1() => $_has(1);
  @$pb.TagNumber(2)
  void clearK1() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.double get b => $_getN(2);
  @$pb.TagNumber(3)
  set b($core.double value) => $_setFloat(2, value);
  @$pb.TagNumber(3)
  $core.bool hasB() => $_has(2);
  @$pb.TagNumber(3)
  void clearB() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.double get delta => $_getN(3);
  @$pb.TagNumber(4)
  set delta($core.double value) => $_setFloat(3, value);
  @$pb.TagNumber(4)
  $core.bool hasDelta() => $_has(3);
  @$pb.TagNumber(4)
  void clearDelta() => $_clearField(4);

  @$pb.TagNumber(5)
  $core.String get language => $_getSZ(4);
  @$pb.TagNumber(5)
  set language($core.String value) => $_setString(4, value);
  @$pb.TagNumber(5)
  $core.bool hasLanguage() => $_has(4);
  @$pb.TagNumber(5)
  void clearLanguage() => $_clearField(5);

  @$pb.TagNumber(6)
  $core.int get ngrams => $_getIZ(5);
  @$pb.TagNumber(6)
  set ngrams($core.int value) => $_setUnsignedInt32(5, value);
  @$pb.TagNumber(6)
  $core.bool hasNgrams() => $_has(5);
  @$pb.TagNumber(6)
  void clearNgrams() => $_clearField(6);

  @$pb.TagNumber(7)
  $core.String get fusionMethod => $_getSZ(6);
  @$pb.TagNumber(7)
  set fusionMethod($core.String value) => $_setString(6, value);
  @$pb.TagNumber(7)
  $core.bool hasFusionMethod() => $_has(6);
  @$pb.TagNumber(7)
  void clearFusionMethod() => $_clearField(7);
}

class InsertResponse extends $pb.GeneratedMessage {
  factory InsertResponse({
    $core.bool? success,
  }) {
    final result = create();
    if (success != null) result.success = success;
    return result;
  }

  InsertResponse._();

  factory InsertResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory InsertResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'InsertResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'hyperspace'),
      createEmptyInstance: create)
    ..aOB(1, _omitFieldNames ? '' : 'success')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  InsertResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  InsertResponse copyWith(void Function(InsertResponse) updates) =>
      super.copyWith((message) => updates(message as InsertResponse))
          as InsertResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static InsertResponse create() => InsertResponse._();
  @$core.override
  InsertResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static InsertResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<InsertResponse>(create);
  static InsertResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $core.bool get success => $_getBF(0);
  @$pb.TagNumber(1)
  set success($core.bool value) => $_setBool(0, value);
  @$pb.TagNumber(1)
  $core.bool hasSuccess() => $_has(0);
  @$pb.TagNumber(1)
  void clearSuccess() => $_clearField(1);
}

class DeleteRequest extends $pb.GeneratedMessage {
  factory DeleteRequest({
    $core.String? collection,
    $core.int? id,
  }) {
    final result = create();
    if (collection != null) result.collection = collection;
    if (id != null) result.id = id;
    return result;
  }

  DeleteRequest._();

  factory DeleteRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory DeleteRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'DeleteRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'hyperspace'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'collection')
    ..aI(2, _omitFieldNames ? '' : 'id', fieldType: $pb.PbFieldType.OU3)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteRequest copyWith(void Function(DeleteRequest) updates) =>
      super.copyWith((message) => updates(message as DeleteRequest))
          as DeleteRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static DeleteRequest create() => DeleteRequest._();
  @$core.override
  DeleteRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static DeleteRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<DeleteRequest>(create);
  static DeleteRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get collection => $_getSZ(0);
  @$pb.TagNumber(1)
  set collection($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasCollection() => $_has(0);
  @$pb.TagNumber(1)
  void clearCollection() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.int get id => $_getIZ(1);
  @$pb.TagNumber(2)
  set id($core.int value) => $_setUnsignedInt32(1, value);
  @$pb.TagNumber(2)
  $core.bool hasId() => $_has(1);
  @$pb.TagNumber(2)
  void clearId() => $_clearField(2);
}

class DeleteResponse extends $pb.GeneratedMessage {
  factory DeleteResponse({
    $core.bool? success,
  }) {
    final result = create();
    if (success != null) result.success = success;
    return result;
  }

  DeleteResponse._();

  factory DeleteResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory DeleteResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'DeleteResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'hyperspace'),
      createEmptyInstance: create)
    ..aOB(1, _omitFieldNames ? '' : 'success')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteResponse copyWith(void Function(DeleteResponse) updates) =>
      super.copyWith((message) => updates(message as DeleteResponse))
          as DeleteResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static DeleteResponse create() => DeleteResponse._();
  @$core.override
  DeleteResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static DeleteResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<DeleteResponse>(create);
  static DeleteResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $core.bool get success => $_getBF(0);
  @$pb.TagNumber(1)
  set success($core.bool value) => $_setBool(0, value);
  @$pb.TagNumber(1)
  $core.bool hasSuccess() => $_has(0);
  @$pb.TagNumber(1)
  void clearSuccess() => $_clearField(1);
}

class GetPointsRequest extends $pb.GeneratedMessage {
  factory GetPointsRequest({
    $core.String? collection,
    $core.Iterable<$core.int>? ids,
  }) {
    final result = create();
    if (collection != null) result.collection = collection;
    if (ids != null) result.ids.addAll(ids);
    return result;
  }

  GetPointsRequest._();

  factory GetPointsRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory GetPointsRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetPointsRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'hyperspace'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'collection')
    ..p<$core.int>(2, _omitFieldNames ? '' : 'ids', $pb.PbFieldType.KU3)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetPointsRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetPointsRequest copyWith(void Function(GetPointsRequest) updates) =>
      super.copyWith((message) => updates(message as GetPointsRequest))
          as GetPointsRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static GetPointsRequest create() => GetPointsRequest._();
  @$core.override
  GetPointsRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static GetPointsRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GetPointsRequest>(create);
  static GetPointsRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get collection => $_getSZ(0);
  @$pb.TagNumber(1)
  set collection($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasCollection() => $_has(0);
  @$pb.TagNumber(1)
  void clearCollection() => $_clearField(1);

  @$pb.TagNumber(2)
  $pb.PbList<$core.int> get ids => $_getList(1);
}

class GetPointsResponse extends $pb.GeneratedMessage {
  factory GetPointsResponse({
    $core.Iterable<VectorData>? points,
  }) {
    final result = create();
    if (points != null) result.points.addAll(points);
    return result;
  }

  GetPointsResponse._();

  factory GetPointsResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory GetPointsResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetPointsResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'hyperspace'),
      createEmptyInstance: create)
    ..pPM<VectorData>(1, _omitFieldNames ? '' : 'points',
        subBuilder: VectorData.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetPointsResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetPointsResponse copyWith(void Function(GetPointsResponse) updates) =>
      super.copyWith((message) => updates(message as GetPointsResponse))
          as GetPointsResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static GetPointsResponse create() => GetPointsResponse._();
  @$core.override
  GetPointsResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static GetPointsResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GetPointsResponse>(create);
  static GetPointsResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<VectorData> get points => $_getList(0);
}

class UpdatePayloadRequest extends $pb.GeneratedMessage {
  factory UpdatePayloadRequest({
    $core.String? collection,
    $core.int? id,
    $core.Iterable<$core.MapEntry<$core.String, $core.String>>? metadata,
    $core.Iterable<$core.MapEntry<$core.String, MetadataValue>>? typedMetadata,
  }) {
    final result = create();
    if (collection != null) result.collection = collection;
    if (id != null) result.id = id;
    if (metadata != null) result.metadata.addEntries(metadata);
    if (typedMetadata != null) result.typedMetadata.addEntries(typedMetadata);
    return result;
  }

  UpdatePayloadRequest._();

  factory UpdatePayloadRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory UpdatePayloadRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'UpdatePayloadRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'hyperspace'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'collection')
    ..aI(2, _omitFieldNames ? '' : 'id', fieldType: $pb.PbFieldType.OU3)
    ..m<$core.String, $core.String>(3, _omitFieldNames ? '' : 'metadata',
        entryClassName: 'UpdatePayloadRequest.MetadataEntry',
        keyFieldType: $pb.PbFieldType.OS,
        valueFieldType: $pb.PbFieldType.OS,
        packageName: const $pb.PackageName('hyperspace'))
    ..m<$core.String, MetadataValue>(4, _omitFieldNames ? '' : 'typedMetadata',
        entryClassName: 'UpdatePayloadRequest.TypedMetadataEntry',
        keyFieldType: $pb.PbFieldType.OS,
        valueFieldType: $pb.PbFieldType.OM,
        valueCreator: MetadataValue.create,
        valueDefaultOrMaker: MetadataValue.getDefault,
        packageName: const $pb.PackageName('hyperspace'))
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpdatePayloadRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpdatePayloadRequest copyWith(void Function(UpdatePayloadRequest) updates) =>
      super.copyWith((message) => updates(message as UpdatePayloadRequest))
          as UpdatePayloadRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static UpdatePayloadRequest create() => UpdatePayloadRequest._();
  @$core.override
  UpdatePayloadRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static UpdatePayloadRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<UpdatePayloadRequest>(create);
  static UpdatePayloadRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get collection => $_getSZ(0);
  @$pb.TagNumber(1)
  set collection($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasCollection() => $_has(0);
  @$pb.TagNumber(1)
  void clearCollection() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.int get id => $_getIZ(1);
  @$pb.TagNumber(2)
  set id($core.int value) => $_setUnsignedInt32(1, value);
  @$pb.TagNumber(2)
  $core.bool hasId() => $_has(1);
  @$pb.TagNumber(2)
  void clearId() => $_clearField(2);

  @$pb.TagNumber(3)
  $pb.PbMap<$core.String, $core.String> get metadata => $_getMap(2);

  @$pb.TagNumber(4)
  $pb.PbMap<$core.String, MetadataValue> get typedMetadata => $_getMap(3);
}

class ScrollRequest extends $pb.GeneratedMessage {
  factory ScrollRequest({
    $core.String? collection,
    $core.int? limit,
    $core.int? offset,
    $core.Iterable<Filter>? filters,
  }) {
    final result = create();
    if (collection != null) result.collection = collection;
    if (limit != null) result.limit = limit;
    if (offset != null) result.offset = offset;
    if (filters != null) result.filters.addAll(filters);
    return result;
  }

  ScrollRequest._();

  factory ScrollRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ScrollRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ScrollRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'hyperspace'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'collection')
    ..aI(2, _omitFieldNames ? '' : 'limit', fieldType: $pb.PbFieldType.OU3)
    ..aI(3, _omitFieldNames ? '' : 'offset', fieldType: $pb.PbFieldType.OU3)
    ..pPM<Filter>(4, _omitFieldNames ? '' : 'filters',
        subBuilder: Filter.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ScrollRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ScrollRequest copyWith(void Function(ScrollRequest) updates) =>
      super.copyWith((message) => updates(message as ScrollRequest))
          as ScrollRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ScrollRequest create() => ScrollRequest._();
  @$core.override
  ScrollRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ScrollRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ScrollRequest>(create);
  static ScrollRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get collection => $_getSZ(0);
  @$pb.TagNumber(1)
  set collection($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasCollection() => $_has(0);
  @$pb.TagNumber(1)
  void clearCollection() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.int get limit => $_getIZ(1);
  @$pb.TagNumber(2)
  set limit($core.int value) => $_setUnsignedInt32(1, value);
  @$pb.TagNumber(2)
  $core.bool hasLimit() => $_has(1);
  @$pb.TagNumber(2)
  void clearLimit() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.int get offset => $_getIZ(2);
  @$pb.TagNumber(3)
  set offset($core.int value) => $_setUnsignedInt32(2, value);
  @$pb.TagNumber(3)
  $core.bool hasOffset() => $_has(2);
  @$pb.TagNumber(3)
  void clearOffset() => $_clearField(3);

  @$pb.TagNumber(4)
  $pb.PbList<Filter> get filters => $_getList(3);
}

class ScrollResponse extends $pb.GeneratedMessage {
  factory ScrollResponse({
    $core.Iterable<VectorData>? points,
  }) {
    final result = create();
    if (points != null) result.points.addAll(points);
    return result;
  }

  ScrollResponse._();

  factory ScrollResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory ScrollResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ScrollResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'hyperspace'),
      createEmptyInstance: create)
    ..pPM<VectorData>(1, _omitFieldNames ? '' : 'points',
        subBuilder: VectorData.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ScrollResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ScrollResponse copyWith(void Function(ScrollResponse) updates) =>
      super.copyWith((message) => updates(message as ScrollResponse))
          as ScrollResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static ScrollResponse create() => ScrollResponse._();
  @$core.override
  ScrollResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static ScrollResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ScrollResponse>(create);
  static ScrollResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<VectorData> get points => $_getList(0);
}

class CountRequest extends $pb.GeneratedMessage {
  factory CountRequest({
    $core.String? collection,
    $core.Iterable<Filter>? filters,
  }) {
    final result = create();
    if (collection != null) result.collection = collection;
    if (filters != null) result.filters.addAll(filters);
    return result;
  }

  CountRequest._();

  factory CountRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory CountRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'CountRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'hyperspace'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'collection')
    ..pPM<Filter>(2, _omitFieldNames ? '' : 'filters',
        subBuilder: Filter.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CountRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CountRequest copyWith(void Function(CountRequest) updates) =>
      super.copyWith((message) => updates(message as CountRequest))
          as CountRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static CountRequest create() => CountRequest._();
  @$core.override
  CountRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static CountRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<CountRequest>(create);
  static CountRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get collection => $_getSZ(0);
  @$pb.TagNumber(1)
  set collection($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasCollection() => $_has(0);
  @$pb.TagNumber(1)
  void clearCollection() => $_clearField(1);

  @$pb.TagNumber(2)
  $pb.PbList<Filter> get filters => $_getList(1);
}

class CountResponse extends $pb.GeneratedMessage {
  factory CountResponse({
    $fixnum.Int64? count,
  }) {
    final result = create();
    if (count != null) result.count = count;
    return result;
  }

  CountResponse._();

  factory CountResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory CountResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'CountResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'hyperspace'),
      createEmptyInstance: create)
    ..a<$fixnum.Int64>(1, _omitFieldNames ? '' : 'count', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CountResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CountResponse copyWith(void Function(CountResponse) updates) =>
      super.copyWith((message) => updates(message as CountResponse))
          as CountResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static CountResponse create() => CountResponse._();
  @$core.override
  CountResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static CountResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<CountResponse>(create);
  static CountResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $fixnum.Int64 get count => $_getI64(0);
  @$pb.TagNumber(1)
  set count($fixnum.Int64 value) => $_setInt64(0, value);
  @$pb.TagNumber(1)
  $core.bool hasCount() => $_has(0);
  @$pb.TagNumber(1)
  void clearCount() => $_clearField(1);
}

class HealthCheckResponse extends $pb.GeneratedMessage {
  factory HealthCheckResponse({
    $core.String? status,
  }) {
    final result = create();
    if (status != null) result.status = status;
    return result;
  }

  HealthCheckResponse._();

  factory HealthCheckResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory HealthCheckResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'HealthCheckResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'hyperspace'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'status')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  HealthCheckResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  HealthCheckResponse copyWith(void Function(HealthCheckResponse) updates) =>
      super.copyWith((message) => updates(message as HealthCheckResponse))
          as HealthCheckResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static HealthCheckResponse create() => HealthCheckResponse._();
  @$core.override
  HealthCheckResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static HealthCheckResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<HealthCheckResponse>(create);
  static HealthCheckResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get status => $_getSZ(0);
  @$pb.TagNumber(1)
  set status($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasStatus() => $_has(0);
  @$pb.TagNumber(1)
  void clearStatus() => $_clearField(1);
}

class SearchRequest extends $pb.GeneratedMessage {
  factory SearchRequest({
    $core.String? collection,
    $core.Iterable<$core.double>? vector,
    $core.int? topK,
    $core.Iterable<$core.MapEntry<$core.String, $core.String>>? filter,
    $core.Iterable<Filter>? filters,
    $core.String? hybridQuery,
    $core.double? hybridAlpha,
    $core.bool? useWasserstein,
    Bm25Options? bm25Options,
    $core.int? mrlDimension,
    $core.bool? includePayload,
    $core.Iterable<$core.MapEntry<$core.String, $core.double>>?
        componentWeights,
    $core.bool? useWave,
  }) {
    final result = create();
    if (collection != null) result.collection = collection;
    if (vector != null) result.vector.addAll(vector);
    if (topK != null) result.topK = topK;
    if (filter != null) result.filter.addEntries(filter);
    if (filters != null) result.filters.addAll(filters);
    if (hybridQuery != null) result.hybridQuery = hybridQuery;
    if (hybridAlpha != null) result.hybridAlpha = hybridAlpha;
    if (useWasserstein != null) result.useWasserstein = useWasserstein;
    if (bm25Options != null) result.bm25Options = bm25Options;
    if (mrlDimension != null) result.mrlDimension = mrlDimension;
    if (includePayload != null) result.includePayload = includePayload;
    if (componentWeights != null)
      result.componentWeights.addEntries(componentWeights);
    if (useWave != null) result.useWave = useWave;
    return result;
  }

  SearchRequest._();

  factory SearchRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory SearchRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'SearchRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'hyperspace'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'collection')
    ..p<$core.double>(2, _omitFieldNames ? '' : 'vector', $pb.PbFieldType.KD)
    ..aI(3, _omitFieldNames ? '' : 'topK', fieldType: $pb.PbFieldType.OU3)
    ..m<$core.String, $core.String>(4, _omitFieldNames ? '' : 'filter',
        entryClassName: 'SearchRequest.FilterEntry',
        keyFieldType: $pb.PbFieldType.OS,
        valueFieldType: $pb.PbFieldType.OS,
        packageName: const $pb.PackageName('hyperspace'))
    ..pPM<Filter>(5, _omitFieldNames ? '' : 'filters',
        subBuilder: Filter.create)
    ..aOS(6, _omitFieldNames ? '' : 'hybridQuery')
    ..aD(7, _omitFieldNames ? '' : 'hybridAlpha', fieldType: $pb.PbFieldType.OF)
    ..aOB(8, _omitFieldNames ? '' : 'useWasserstein')
    ..aOM<Bm25Options>(9, _omitFieldNames ? '' : 'bm25Options',
        subBuilder: Bm25Options.create)
    ..aI(10, _omitFieldNames ? '' : 'mrlDimension',
        fieldType: $pb.PbFieldType.OU3)
    ..aOB(11, _omitFieldNames ? '' : 'includePayload')
    ..m<$core.String, $core.double>(
        12, _omitFieldNames ? '' : 'componentWeights',
        entryClassName: 'SearchRequest.ComponentWeightsEntry',
        keyFieldType: $pb.PbFieldType.OS,
        valueFieldType: $pb.PbFieldType.OF,
        packageName: const $pb.PackageName('hyperspace'))
    ..aOB(13, _omitFieldNames ? '' : 'useWave')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SearchRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SearchRequest copyWith(void Function(SearchRequest) updates) =>
      super.copyWith((message) => updates(message as SearchRequest))
          as SearchRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static SearchRequest create() => SearchRequest._();
  @$core.override
  SearchRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static SearchRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<SearchRequest>(create);
  static SearchRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get collection => $_getSZ(0);
  @$pb.TagNumber(1)
  set collection($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasCollection() => $_has(0);
  @$pb.TagNumber(1)
  void clearCollection() => $_clearField(1);

  @$pb.TagNumber(2)
  $pb.PbList<$core.double> get vector => $_getList(1);

  @$pb.TagNumber(3)
  $core.int get topK => $_getIZ(2);
  @$pb.TagNumber(3)
  set topK($core.int value) => $_setUnsignedInt32(2, value);
  @$pb.TagNumber(3)
  $core.bool hasTopK() => $_has(2);
  @$pb.TagNumber(3)
  void clearTopK() => $_clearField(3);

  @$pb.TagNumber(4)
  $pb.PbMap<$core.String, $core.String> get filter => $_getMap(3);

  @$pb.TagNumber(5)
  $pb.PbList<Filter> get filters => $_getList(4);

  @$pb.TagNumber(6)
  $core.String get hybridQuery => $_getSZ(5);
  @$pb.TagNumber(6)
  set hybridQuery($core.String value) => $_setString(5, value);
  @$pb.TagNumber(6)
  $core.bool hasHybridQuery() => $_has(5);
  @$pb.TagNumber(6)
  void clearHybridQuery() => $_clearField(6);

  @$pb.TagNumber(7)
  $core.double get hybridAlpha => $_getN(6);
  @$pb.TagNumber(7)
  set hybridAlpha($core.double value) => $_setFloat(6, value);
  @$pb.TagNumber(7)
  $core.bool hasHybridAlpha() => $_has(6);
  @$pb.TagNumber(7)
  void clearHybridAlpha() => $_clearField(7);

  @$pb.TagNumber(8)
  $core.bool get useWasserstein => $_getBF(7);
  @$pb.TagNumber(8)
  set useWasserstein($core.bool value) => $_setBool(7, value);
  @$pb.TagNumber(8)
  $core.bool hasUseWasserstein() => $_has(7);
  @$pb.TagNumber(8)
  void clearUseWasserstein() => $_clearField(8);

  @$pb.TagNumber(9)
  Bm25Options get bm25Options => $_getN(8);
  @$pb.TagNumber(9)
  set bm25Options(Bm25Options value) => $_setField(9, value);
  @$pb.TagNumber(9)
  $core.bool hasBm25Options() => $_has(8);
  @$pb.TagNumber(9)
  void clearBm25Options() => $_clearField(9);
  @$pb.TagNumber(9)
  Bm25Options ensureBm25Options() => $_ensure(8);

  @$pb.TagNumber(10)
  $core.int get mrlDimension => $_getIZ(9);
  @$pb.TagNumber(10)
  set mrlDimension($core.int value) => $_setUnsignedInt32(9, value);
  @$pb.TagNumber(10)
  $core.bool hasMrlDimension() => $_has(9);
  @$pb.TagNumber(10)
  void clearMrlDimension() => $_clearField(10);

  /// Sidecar Payload Storage (v3.2): if true, the server performs lazy disk I/O
  /// ONLY for the final Top-K results to attach their stored payload blobs.
  /// Default is false — zero extra I/O for standard searches.
  @$pb.TagNumber(11)
  $core.bool get includePayload => $_getBF(10);
  @$pb.TagNumber(11)
  set includePayload($core.bool value) => $_setBool(10, value);
  @$pb.TagNumber(11)
  $core.bool hasIncludePayload() => $_has(10);
  @$pb.TagNumber(11)
  void clearIncludePayload() => $_clearField(11);

  @$pb.TagNumber(12)
  $pb.PbMap<$core.String, $core.double> get componentWeights => $_getMap(11);

  @$pb.TagNumber(13)
  $core.bool get useWave => $_getBF(12);
  @$pb.TagNumber(13)
  set useWave($core.bool value) => $_setBool(12, value);
  @$pb.TagNumber(13)
  $core.bool hasUseWave() => $_has(12);
  @$pb.TagNumber(13)
  void clearUseWave() => $_clearField(13);
}

enum Filter_Condition {
  match,
  range,
  inCone,
  inBox,
  inBall,
  andOp,
  orOp,
  notOp,
  prefix,
  notSet
}

class Filter extends $pb.GeneratedMessage {
  factory Filter({
    Match? match,
    Range? range,
    InCone? inCone,
    InBox? inBox,
    InBall? inBall,
    FilterAnd? andOp,
    FilterOr? orOp,
    FilterNot? notOp,
    Prefix? prefix,
  }) {
    final result = create();
    if (match != null) result.match = match;
    if (range != null) result.range = range;
    if (inCone != null) result.inCone = inCone;
    if (inBox != null) result.inBox = inBox;
    if (inBall != null) result.inBall = inBall;
    if (andOp != null) result.andOp = andOp;
    if (orOp != null) result.orOp = orOp;
    if (notOp != null) result.notOp = notOp;
    if (prefix != null) result.prefix = prefix;
    return result;
  }

  Filter._();

  factory Filter.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory Filter.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static const $core.Map<$core.int, Filter_Condition> _Filter_ConditionByTag = {
    1: Filter_Condition.match,
    2: Filter_Condition.range,
    3: Filter_Condition.inCone,
    4: Filter_Condition.inBox,
    5: Filter_Condition.inBall,
    6: Filter_Condition.andOp,
    7: Filter_Condition.orOp,
    8: Filter_Condition.notOp,
    9: Filter_Condition.prefix,
    0: Filter_Condition.notSet
  };
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'Filter',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'hyperspace'),
      createEmptyInstance: create)
    ..oo(0, [1, 2, 3, 4, 5, 6, 7, 8, 9])
    ..aOM<Match>(1, _omitFieldNames ? '' : 'match', subBuilder: Match.create)
    ..aOM<Range>(2, _omitFieldNames ? '' : 'range', subBuilder: Range.create)
    ..aOM<InCone>(3, _omitFieldNames ? '' : 'inCone', subBuilder: InCone.create)
    ..aOM<InBox>(4, _omitFieldNames ? '' : 'inBox', subBuilder: InBox.create)
    ..aOM<InBall>(5, _omitFieldNames ? '' : 'inBall', subBuilder: InBall.create)
    ..aOM<FilterAnd>(6, _omitFieldNames ? '' : 'andOp',
        subBuilder: FilterAnd.create)
    ..aOM<FilterOr>(7, _omitFieldNames ? '' : 'orOp',
        subBuilder: FilterOr.create)
    ..aOM<FilterNot>(8, _omitFieldNames ? '' : 'notOp',
        subBuilder: FilterNot.create)
    ..aOM<Prefix>(9, _omitFieldNames ? '' : 'prefix', subBuilder: Prefix.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Filter clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Filter copyWith(void Function(Filter) updates) =>
      super.copyWith((message) => updates(message as Filter)) as Filter;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static Filter create() => Filter._();
  @$core.override
  Filter createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static Filter getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<Filter>(create);
  static Filter? _defaultInstance;

  @$pb.TagNumber(1)
  @$pb.TagNumber(2)
  @$pb.TagNumber(3)
  @$pb.TagNumber(4)
  @$pb.TagNumber(5)
  @$pb.TagNumber(6)
  @$pb.TagNumber(7)
  @$pb.TagNumber(8)
  @$pb.TagNumber(9)
  Filter_Condition whichCondition() => _Filter_ConditionByTag[$_whichOneof(0)]!;
  @$pb.TagNumber(1)
  @$pb.TagNumber(2)
  @$pb.TagNumber(3)
  @$pb.TagNumber(4)
  @$pb.TagNumber(5)
  @$pb.TagNumber(6)
  @$pb.TagNumber(7)
  @$pb.TagNumber(8)
  @$pb.TagNumber(9)
  void clearCondition() => $_clearField($_whichOneof(0));

  @$pb.TagNumber(1)
  Match get match => $_getN(0);
  @$pb.TagNumber(1)
  set match(Match value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasMatch() => $_has(0);
  @$pb.TagNumber(1)
  void clearMatch() => $_clearField(1);
  @$pb.TagNumber(1)
  Match ensureMatch() => $_ensure(0);

  @$pb.TagNumber(2)
  Range get range => $_getN(1);
  @$pb.TagNumber(2)
  set range(Range value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasRange() => $_has(1);
  @$pb.TagNumber(2)
  void clearRange() => $_clearField(2);
  @$pb.TagNumber(2)
  Range ensureRange() => $_ensure(1);

  @$pb.TagNumber(3)
  InCone get inCone => $_getN(2);
  @$pb.TagNumber(3)
  set inCone(InCone value) => $_setField(3, value);
  @$pb.TagNumber(3)
  $core.bool hasInCone() => $_has(2);
  @$pb.TagNumber(3)
  void clearInCone() => $_clearField(3);
  @$pb.TagNumber(3)
  InCone ensureInCone() => $_ensure(2);

  @$pb.TagNumber(4)
  InBox get inBox => $_getN(3);
  @$pb.TagNumber(4)
  set inBox(InBox value) => $_setField(4, value);
  @$pb.TagNumber(4)
  $core.bool hasInBox() => $_has(3);
  @$pb.TagNumber(4)
  void clearInBox() => $_clearField(4);
  @$pb.TagNumber(4)
  InBox ensureInBox() => $_ensure(3);

  @$pb.TagNumber(5)
  InBall get inBall => $_getN(4);
  @$pb.TagNumber(5)
  set inBall(InBall value) => $_setField(5, value);
  @$pb.TagNumber(5)
  $core.bool hasInBall() => $_has(4);
  @$pb.TagNumber(5)
  void clearInBall() => $_clearField(5);
  @$pb.TagNumber(5)
  InBall ensureInBall() => $_ensure(4);

  @$pb.TagNumber(6)
  FilterAnd get andOp => $_getN(5);
  @$pb.TagNumber(6)
  set andOp(FilterAnd value) => $_setField(6, value);
  @$pb.TagNumber(6)
  $core.bool hasAndOp() => $_has(5);
  @$pb.TagNumber(6)
  void clearAndOp() => $_clearField(6);
  @$pb.TagNumber(6)
  FilterAnd ensureAndOp() => $_ensure(5);

  @$pb.TagNumber(7)
  FilterOr get orOp => $_getN(6);
  @$pb.TagNumber(7)
  set orOp(FilterOr value) => $_setField(7, value);
  @$pb.TagNumber(7)
  $core.bool hasOrOp() => $_has(6);
  @$pb.TagNumber(7)
  void clearOrOp() => $_clearField(7);
  @$pb.TagNumber(7)
  FilterOr ensureOrOp() => $_ensure(6);

  @$pb.TagNumber(8)
  FilterNot get notOp => $_getN(7);
  @$pb.TagNumber(8)
  set notOp(FilterNot value) => $_setField(8, value);
  @$pb.TagNumber(8)
  $core.bool hasNotOp() => $_has(7);
  @$pb.TagNumber(8)
  void clearNotOp() => $_clearField(8);
  @$pb.TagNumber(8)
  FilterNot ensureNotOp() => $_ensure(7);

  @$pb.TagNumber(9)
  Prefix get prefix => $_getN(8);
  @$pb.TagNumber(9)
  set prefix(Prefix value) => $_setField(9, value);
  @$pb.TagNumber(9)
  $core.bool hasPrefix() => $_has(8);
  @$pb.TagNumber(9)
  void clearPrefix() => $_clearField(9);
  @$pb.TagNumber(9)
  Prefix ensurePrefix() => $_ensure(8);
}

class FilterAnd extends $pb.GeneratedMessage {
  factory FilterAnd({
    $core.Iterable<Filter>? conditions,
  }) {
    final result = create();
    if (conditions != null) result.conditions.addAll(conditions);
    return result;
  }

  FilterAnd._();

  factory FilterAnd.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory FilterAnd.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'FilterAnd',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'hyperspace'),
      createEmptyInstance: create)
    ..pPM<Filter>(1, _omitFieldNames ? '' : 'conditions',
        subBuilder: Filter.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  FilterAnd clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  FilterAnd copyWith(void Function(FilterAnd) updates) =>
      super.copyWith((message) => updates(message as FilterAnd)) as FilterAnd;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static FilterAnd create() => FilterAnd._();
  @$core.override
  FilterAnd createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static FilterAnd getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<FilterAnd>(create);
  static FilterAnd? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<Filter> get conditions => $_getList(0);
}

class FilterOr extends $pb.GeneratedMessage {
  factory FilterOr({
    $core.Iterable<Filter>? conditions,
  }) {
    final result = create();
    if (conditions != null) result.conditions.addAll(conditions);
    return result;
  }

  FilterOr._();

  factory FilterOr.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory FilterOr.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'FilterOr',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'hyperspace'),
      createEmptyInstance: create)
    ..pPM<Filter>(1, _omitFieldNames ? '' : 'conditions',
        subBuilder: Filter.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  FilterOr clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  FilterOr copyWith(void Function(FilterOr) updates) =>
      super.copyWith((message) => updates(message as FilterOr)) as FilterOr;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static FilterOr create() => FilterOr._();
  @$core.override
  FilterOr createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static FilterOr getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<FilterOr>(create);
  static FilterOr? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<Filter> get conditions => $_getList(0);
}

class FilterNot extends $pb.GeneratedMessage {
  factory FilterNot({
    Filter? condition,
  }) {
    final result = create();
    if (condition != null) result.condition = condition;
    return result;
  }

  FilterNot._();

  factory FilterNot.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory FilterNot.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'FilterNot',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'hyperspace'),
      createEmptyInstance: create)
    ..aOM<Filter>(1, _omitFieldNames ? '' : 'condition',
        subBuilder: Filter.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  FilterNot clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  FilterNot copyWith(void Function(FilterNot) updates) =>
      super.copyWith((message) => updates(message as FilterNot)) as FilterNot;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static FilterNot create() => FilterNot._();
  @$core.override
  FilterNot createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static FilterNot getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<FilterNot>(create);
  static FilterNot? _defaultInstance;

  @$pb.TagNumber(1)
  Filter get condition => $_getN(0);
  @$pb.TagNumber(1)
  set condition(Filter value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasCondition() => $_has(0);
  @$pb.TagNumber(1)
  void clearCondition() => $_clearField(1);
  @$pb.TagNumber(1)
  Filter ensureCondition() => $_ensure(0);
}

class Match extends $pb.GeneratedMessage {
  factory Match({
    $core.String? key,
    $core.String? value,
  }) {
    final result = create();
    if (key != null) result.key = key;
    if (value != null) result.value = value;
    return result;
  }

  Match._();

  factory Match.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory Match.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'Match',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'hyperspace'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'key')
    ..aOS(2, _omitFieldNames ? '' : 'value')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Match clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Match copyWith(void Function(Match) updates) =>
      super.copyWith((message) => updates(message as Match)) as Match;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static Match create() => Match._();
  @$core.override
  Match createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static Match getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<Match>(create);
  static Match? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get key => $_getSZ(0);
  @$pb.TagNumber(1)
  set key($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasKey() => $_has(0);
  @$pb.TagNumber(1)
  void clearKey() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get value => $_getSZ(1);
  @$pb.TagNumber(2)
  set value($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasValue() => $_has(1);
  @$pb.TagNumber(2)
  void clearValue() => $_clearField(2);
}

class Prefix extends $pb.GeneratedMessage {
  factory Prefix({
    $core.String? key,
    $core.String? prefix,
  }) {
    final result = create();
    if (key != null) result.key = key;
    if (prefix != null) result.prefix = prefix;
    return result;
  }

  Prefix._();

  factory Prefix.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory Prefix.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'Prefix',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'hyperspace'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'key')
    ..aOS(2, _omitFieldNames ? '' : 'prefix')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Prefix clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Prefix copyWith(void Function(Prefix) updates) =>
      super.copyWith((message) => updates(message as Prefix)) as Prefix;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static Prefix create() => Prefix._();
  @$core.override
  Prefix createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static Prefix getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<Prefix>(create);
  static Prefix? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get key => $_getSZ(0);
  @$pb.TagNumber(1)
  set key($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasKey() => $_has(0);
  @$pb.TagNumber(1)
  void clearKey() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get prefix => $_getSZ(1);
  @$pb.TagNumber(2)
  set prefix($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasPrefix() => $_has(1);
  @$pb.TagNumber(2)
  void clearPrefix() => $_clearField(2);
}

class Range extends $pb.GeneratedMessage {
  factory Range({
    $core.String? key,
    $fixnum.Int64? gte,
    $fixnum.Int64? lte,
    $core.double? gteF64,
    $core.double? lteF64,
  }) {
    final result = create();
    if (key != null) result.key = key;
    if (gte != null) result.gte = gte;
    if (lte != null) result.lte = lte;
    if (gteF64 != null) result.gteF64 = gteF64;
    if (lteF64 != null) result.lteF64 = lteF64;
    return result;
  }

  Range._();

  factory Range.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory Range.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'Range',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'hyperspace'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'key')
    ..aInt64(2, _omitFieldNames ? '' : 'gte')
    ..aInt64(3, _omitFieldNames ? '' : 'lte')
    ..aD(4, _omitFieldNames ? '' : 'gteF64')
    ..aD(5, _omitFieldNames ? '' : 'lteF64')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Range clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Range copyWith(void Function(Range) updates) =>
      super.copyWith((message) => updates(message as Range)) as Range;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static Range create() => Range._();
  @$core.override
  Range createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static Range getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<Range>(create);
  static Range? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get key => $_getSZ(0);
  @$pb.TagNumber(1)
  set key($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasKey() => $_has(0);
  @$pb.TagNumber(1)
  void clearKey() => $_clearField(1);

  @$pb.TagNumber(2)
  $fixnum.Int64 get gte => $_getI64(1);
  @$pb.TagNumber(2)
  set gte($fixnum.Int64 value) => $_setInt64(1, value);
  @$pb.TagNumber(2)
  $core.bool hasGte() => $_has(1);
  @$pb.TagNumber(2)
  void clearGte() => $_clearField(2);

  @$pb.TagNumber(3)
  $fixnum.Int64 get lte => $_getI64(2);
  @$pb.TagNumber(3)
  set lte($fixnum.Int64 value) => $_setInt64(2, value);
  @$pb.TagNumber(3)
  $core.bool hasLte() => $_has(2);
  @$pb.TagNumber(3)
  void clearLte() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.double get gteF64 => $_getN(3);
  @$pb.TagNumber(4)
  set gteF64($core.double value) => $_setDouble(3, value);
  @$pb.TagNumber(4)
  $core.bool hasGteF64() => $_has(3);
  @$pb.TagNumber(4)
  void clearGteF64() => $_clearField(4);

  @$pb.TagNumber(5)
  $core.double get lteF64 => $_getN(4);
  @$pb.TagNumber(5)
  set lteF64($core.double value) => $_setDouble(4, value);
  @$pb.TagNumber(5)
  $core.bool hasLteF64() => $_has(4);
  @$pb.TagNumber(5)
  void clearLteF64() => $_clearField(5);
}

class InCone extends $pb.GeneratedMessage {
  factory InCone({
    $core.Iterable<$core.double>? axes,
    $core.Iterable<$core.double>? apertures,
    $core.double? cen,
  }) {
    final result = create();
    if (axes != null) result.axes.addAll(axes);
    if (apertures != null) result.apertures.addAll(apertures);
    if (cen != null) result.cen = cen;
    return result;
  }

  InCone._();

  factory InCone.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory InCone.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'InCone',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'hyperspace'),
      createEmptyInstance: create)
    ..p<$core.double>(1, _omitFieldNames ? '' : 'axes', $pb.PbFieldType.KD)
    ..p<$core.double>(2, _omitFieldNames ? '' : 'apertures', $pb.PbFieldType.KD)
    ..aD(3, _omitFieldNames ? '' : 'cen')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  InCone clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  InCone copyWith(void Function(InCone) updates) =>
      super.copyWith((message) => updates(message as InCone)) as InCone;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static InCone create() => InCone._();
  @$core.override
  InCone createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static InCone getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<InCone>(create);
  static InCone? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<$core.double> get axes => $_getList(0);

  @$pb.TagNumber(2)
  $pb.PbList<$core.double> get apertures => $_getList(1);

  @$pb.TagNumber(3)
  $core.double get cen => $_getN(2);
  @$pb.TagNumber(3)
  set cen($core.double value) => $_setDouble(2, value);
  @$pb.TagNumber(3)
  $core.bool hasCen() => $_has(2);
  @$pb.TagNumber(3)
  void clearCen() => $_clearField(3);
}

class InBox extends $pb.GeneratedMessage {
  factory InBox({
    $core.Iterable<$core.double>? minBounds,
    $core.Iterable<$core.double>? maxBounds,
  }) {
    final result = create();
    if (minBounds != null) result.minBounds.addAll(minBounds);
    if (maxBounds != null) result.maxBounds.addAll(maxBounds);
    return result;
  }

  InBox._();

  factory InBox.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory InBox.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'InBox',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'hyperspace'),
      createEmptyInstance: create)
    ..p<$core.double>(1, _omitFieldNames ? '' : 'minBounds', $pb.PbFieldType.KD)
    ..p<$core.double>(2, _omitFieldNames ? '' : 'maxBounds', $pb.PbFieldType.KD)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  InBox clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  InBox copyWith(void Function(InBox) updates) =>
      super.copyWith((message) => updates(message as InBox)) as InBox;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static InBox create() => InBox._();
  @$core.override
  InBox createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static InBox getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<InBox>(create);
  static InBox? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<$core.double> get minBounds => $_getList(0);

  @$pb.TagNumber(2)
  $pb.PbList<$core.double> get maxBounds => $_getList(1);
}

class InBall extends $pb.GeneratedMessage {
  factory InBall({
    $core.Iterable<$core.double>? center,
    $core.double? radius,
  }) {
    final result = create();
    if (center != null) result.center.addAll(center);
    if (radius != null) result.radius = radius;
    return result;
  }

  InBall._();

  factory InBall.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory InBall.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'InBall',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'hyperspace'),
      createEmptyInstance: create)
    ..p<$core.double>(1, _omitFieldNames ? '' : 'center', $pb.PbFieldType.KD)
    ..aD(2, _omitFieldNames ? '' : 'radius')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  InBall clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  InBall copyWith(void Function(InBall) updates) =>
      super.copyWith((message) => updates(message as InBall)) as InBall;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static InBall create() => InBall._();
  @$core.override
  InBall createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static InBall getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<InBall>(create);
  static InBall? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<$core.double> get center => $_getList(0);

  @$pb.TagNumber(2)
  $core.double get radius => $_getN(1);
  @$pb.TagNumber(2)
  set radius($core.double value) => $_setDouble(1, value);
  @$pb.TagNumber(2)
  $core.bool hasRadius() => $_has(1);
  @$pb.TagNumber(2)
  void clearRadius() => $_clearField(2);
}

class SearchResponse extends $pb.GeneratedMessage {
  factory SearchResponse({
    $core.Iterable<SearchResult>? results,
  }) {
    final result = create();
    if (results != null) result.results.addAll(results);
    return result;
  }

  SearchResponse._();

  factory SearchResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory SearchResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'SearchResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'hyperspace'),
      createEmptyInstance: create)
    ..pPM<SearchResult>(1, _omitFieldNames ? '' : 'results',
        subBuilder: SearchResult.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SearchResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SearchResponse copyWith(void Function(SearchResponse) updates) =>
      super.copyWith((message) => updates(message as SearchResponse))
          as SearchResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static SearchResponse create() => SearchResponse._();
  @$core.override
  SearchResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static SearchResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<SearchResponse>(create);
  static SearchResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<SearchResult> get results => $_getList(0);
}

class BatchSearchRequest extends $pb.GeneratedMessage {
  factory BatchSearchRequest({
    $core.Iterable<SearchRequest>? searches,
  }) {
    final result = create();
    if (searches != null) result.searches.addAll(searches);
    return result;
  }

  BatchSearchRequest._();

  factory BatchSearchRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory BatchSearchRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'BatchSearchRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'hyperspace'),
      createEmptyInstance: create)
    ..pPM<SearchRequest>(1, _omitFieldNames ? '' : 'searches',
        subBuilder: SearchRequest.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  BatchSearchRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  BatchSearchRequest copyWith(void Function(BatchSearchRequest) updates) =>
      super.copyWith((message) => updates(message as BatchSearchRequest))
          as BatchSearchRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static BatchSearchRequest create() => BatchSearchRequest._();
  @$core.override
  BatchSearchRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static BatchSearchRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<BatchSearchRequest>(create);
  static BatchSearchRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<SearchRequest> get searches => $_getList(0);
}

class BatchSearchResponse extends $pb.GeneratedMessage {
  factory BatchSearchResponse({
    $core.Iterable<SearchResponse>? responses,
  }) {
    final result = create();
    if (responses != null) result.responses.addAll(responses);
    return result;
  }

  BatchSearchResponse._();

  factory BatchSearchResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory BatchSearchResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'BatchSearchResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'hyperspace'),
      createEmptyInstance: create)
    ..pPM<SearchResponse>(1, _omitFieldNames ? '' : 'responses',
        subBuilder: SearchResponse.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  BatchSearchResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  BatchSearchResponse copyWith(void Function(BatchSearchResponse) updates) =>
      super.copyWith((message) => updates(message as BatchSearchResponse))
          as BatchSearchResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static BatchSearchResponse create() => BatchSearchResponse._();
  @$core.override
  BatchSearchResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static BatchSearchResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<BatchSearchResponse>(create);
  static BatchSearchResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<SearchResponse> get responses => $_getList(0);
}

class SearchMultiCollectionRequest extends $pb.GeneratedMessage {
  factory SearchMultiCollectionRequest({
    $core.Iterable<$core.String>? collections,
    $core.Iterable<$core.double>? vector,
    $core.int? topK,
  }) {
    final result = create();
    if (collections != null) result.collections.addAll(collections);
    if (vector != null) result.vector.addAll(vector);
    if (topK != null) result.topK = topK;
    return result;
  }

  SearchMultiCollectionRequest._();

  factory SearchMultiCollectionRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory SearchMultiCollectionRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'SearchMultiCollectionRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'hyperspace'),
      createEmptyInstance: create)
    ..pPS(1, _omitFieldNames ? '' : 'collections')
    ..p<$core.double>(2, _omitFieldNames ? '' : 'vector', $pb.PbFieldType.KD)
    ..aI(3, _omitFieldNames ? '' : 'topK', fieldType: $pb.PbFieldType.OU3)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SearchMultiCollectionRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SearchMultiCollectionRequest copyWith(
          void Function(SearchMultiCollectionRequest) updates) =>
      super.copyWith(
              (message) => updates(message as SearchMultiCollectionRequest))
          as SearchMultiCollectionRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static SearchMultiCollectionRequest create() =>
      SearchMultiCollectionRequest._();
  @$core.override
  SearchMultiCollectionRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static SearchMultiCollectionRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<SearchMultiCollectionRequest>(create);
  static SearchMultiCollectionRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<$core.String> get collections => $_getList(0);

  @$pb.TagNumber(2)
  $pb.PbList<$core.double> get vector => $_getList(1);

  @$pb.TagNumber(3)
  $core.int get topK => $_getIZ(2);
  @$pb.TagNumber(3)
  set topK($core.int value) => $_setUnsignedInt32(2, value);
  @$pb.TagNumber(3)
  $core.bool hasTopK() => $_has(2);
  @$pb.TagNumber(3)
  void clearTopK() => $_clearField(3);
}

class SearchMultiCollectionResponse extends $pb.GeneratedMessage {
  factory SearchMultiCollectionResponse({
    $core.Iterable<$core.MapEntry<$core.String, SearchResponse>>? responses,
  }) {
    final result = create();
    if (responses != null) result.responses.addEntries(responses);
    return result;
  }

  SearchMultiCollectionResponse._();

  factory SearchMultiCollectionResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory SearchMultiCollectionResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'SearchMultiCollectionResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'hyperspace'),
      createEmptyInstance: create)
    ..m<$core.String, SearchResponse>(1, _omitFieldNames ? '' : 'responses',
        entryClassName: 'SearchMultiCollectionResponse.ResponsesEntry',
        keyFieldType: $pb.PbFieldType.OS,
        valueFieldType: $pb.PbFieldType.OM,
        valueCreator: SearchResponse.create,
        valueDefaultOrMaker: SearchResponse.getDefault,
        packageName: const $pb.PackageName('hyperspace'))
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SearchMultiCollectionResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SearchMultiCollectionResponse copyWith(
          void Function(SearchMultiCollectionResponse) updates) =>
      super.copyWith(
              (message) => updates(message as SearchMultiCollectionResponse))
          as SearchMultiCollectionResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static SearchMultiCollectionResponse create() =>
      SearchMultiCollectionResponse._();
  @$core.override
  SearchMultiCollectionResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static SearchMultiCollectionResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<SearchMultiCollectionResponse>(create);
  static SearchMultiCollectionResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbMap<$core.String, SearchResponse> get responses => $_getMap(0);
}

class SearchResult extends $pb.GeneratedMessage {
  factory SearchResult({
    $core.int? id,
    $core.double? distance,
    $core.Iterable<$core.MapEntry<$core.String, $core.String>>? metadata,
    $core.Iterable<$core.MapEntry<$core.String, MetadataValue>>? typedMetadata,
    $core.List<$core.int>? payload,
  }) {
    final result = create();
    if (id != null) result.id = id;
    if (distance != null) result.distance = distance;
    if (metadata != null) result.metadata.addEntries(metadata);
    if (typedMetadata != null) result.typedMetadata.addEntries(typedMetadata);
    if (payload != null) result.payload = payload;
    return result;
  }

  SearchResult._();

  factory SearchResult.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory SearchResult.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'SearchResult',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'hyperspace'),
      createEmptyInstance: create)
    ..aI(1, _omitFieldNames ? '' : 'id', fieldType: $pb.PbFieldType.OU3)
    ..aD(2, _omitFieldNames ? '' : 'distance')
    ..m<$core.String, $core.String>(3, _omitFieldNames ? '' : 'metadata',
        entryClassName: 'SearchResult.MetadataEntry',
        keyFieldType: $pb.PbFieldType.OS,
        valueFieldType: $pb.PbFieldType.OS,
        packageName: const $pb.PackageName('hyperspace'))
    ..m<$core.String, MetadataValue>(4, _omitFieldNames ? '' : 'typedMetadata',
        entryClassName: 'SearchResult.TypedMetadataEntry',
        keyFieldType: $pb.PbFieldType.OS,
        valueFieldType: $pb.PbFieldType.OM,
        valueCreator: MetadataValue.create,
        valueDefaultOrMaker: MetadataValue.getDefault,
        packageName: const $pb.PackageName('hyperspace'))
    ..a<$core.List<$core.int>>(
        5, _omitFieldNames ? '' : 'payload', $pb.PbFieldType.OY)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SearchResult clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SearchResult copyWith(void Function(SearchResult) updates) =>
      super.copyWith((message) => updates(message as SearchResult))
          as SearchResult;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static SearchResult create() => SearchResult._();
  @$core.override
  SearchResult createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static SearchResult getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<SearchResult>(create);
  static SearchResult? _defaultInstance;

  @$pb.TagNumber(1)
  $core.int get id => $_getIZ(0);
  @$pb.TagNumber(1)
  set id($core.int value) => $_setUnsignedInt32(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.double get distance => $_getN(1);
  @$pb.TagNumber(2)
  set distance($core.double value) => $_setDouble(1, value);
  @$pb.TagNumber(2)
  $core.bool hasDistance() => $_has(1);
  @$pb.TagNumber(2)
  void clearDistance() => $_clearField(2);

  @$pb.TagNumber(3)
  $pb.PbMap<$core.String, $core.String> get metadata => $_getMap(2);

  @$pb.TagNumber(4)
  $pb.PbMap<$core.String, MetadataValue> get typedMetadata => $_getMap(3);

  /// Sidecar Payload Storage (v3.2): populated ONLY when include_payload=true
  /// in the SearchRequest AND the vector was inserted with a payload.
  /// The bytes are the original uncompressed payload — decompression is server-side.
  @$pb.TagNumber(5)
  $core.List<$core.int> get payload => $_getN(4);
  @$pb.TagNumber(5)
  set payload($core.List<$core.int> value) => $_setBytes(4, value);
  @$pb.TagNumber(5)
  $core.bool hasPayload() => $_has(4);
  @$pb.TagNumber(5)
  void clearPayload() => $_clearField(5);
}

class GetNodeRequest extends $pb.GeneratedMessage {
  factory GetNodeRequest({
    $core.String? collection,
    $core.int? id,
    $core.int? layer,
  }) {
    final result = create();
    if (collection != null) result.collection = collection;
    if (id != null) result.id = id;
    if (layer != null) result.layer = layer;
    return result;
  }

  GetNodeRequest._();

  factory GetNodeRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory GetNodeRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetNodeRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'hyperspace'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'collection')
    ..aI(2, _omitFieldNames ? '' : 'id', fieldType: $pb.PbFieldType.OU3)
    ..aI(3, _omitFieldNames ? '' : 'layer', fieldType: $pb.PbFieldType.OU3)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetNodeRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetNodeRequest copyWith(void Function(GetNodeRequest) updates) =>
      super.copyWith((message) => updates(message as GetNodeRequest))
          as GetNodeRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static GetNodeRequest create() => GetNodeRequest._();
  @$core.override
  GetNodeRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static GetNodeRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GetNodeRequest>(create);
  static GetNodeRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get collection => $_getSZ(0);
  @$pb.TagNumber(1)
  set collection($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasCollection() => $_has(0);
  @$pb.TagNumber(1)
  void clearCollection() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.int get id => $_getIZ(1);
  @$pb.TagNumber(2)
  set id($core.int value) => $_setUnsignedInt32(1, value);
  @$pb.TagNumber(2)
  $core.bool hasId() => $_has(1);
  @$pb.TagNumber(2)
  void clearId() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.int get layer => $_getIZ(2);
  @$pb.TagNumber(3)
  set layer($core.int value) => $_setUnsignedInt32(2, value);
  @$pb.TagNumber(3)
  $core.bool hasLayer() => $_has(2);
  @$pb.TagNumber(3)
  void clearLayer() => $_clearField(3);
}

class GraphNode extends $pb.GeneratedMessage {
  factory GraphNode({
    $core.int? id,
    $core.int? layer,
    $core.Iterable<$core.int>? neighbors,
    $core.Iterable<$core.MapEntry<$core.String, $core.String>>? metadata,
    $core.Iterable<$core.MapEntry<$core.String, MetadataValue>>? typedMetadata,
    $core.Iterable<EdgeType>? edgeTypes,
  }) {
    final result = create();
    if (id != null) result.id = id;
    if (layer != null) result.layer = layer;
    if (neighbors != null) result.neighbors.addAll(neighbors);
    if (metadata != null) result.metadata.addEntries(metadata);
    if (typedMetadata != null) result.typedMetadata.addEntries(typedMetadata);
    if (edgeTypes != null) result.edgeTypes.addAll(edgeTypes);
    return result;
  }

  GraphNode._();

  factory GraphNode.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory GraphNode.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GraphNode',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'hyperspace'),
      createEmptyInstance: create)
    ..aI(1, _omitFieldNames ? '' : 'id', fieldType: $pb.PbFieldType.OU3)
    ..aI(2, _omitFieldNames ? '' : 'layer', fieldType: $pb.PbFieldType.OU3)
    ..p<$core.int>(3, _omitFieldNames ? '' : 'neighbors', $pb.PbFieldType.KU3)
    ..m<$core.String, $core.String>(4, _omitFieldNames ? '' : 'metadata',
        entryClassName: 'GraphNode.MetadataEntry',
        keyFieldType: $pb.PbFieldType.OS,
        valueFieldType: $pb.PbFieldType.OS,
        packageName: const $pb.PackageName('hyperspace'))
    ..m<$core.String, MetadataValue>(5, _omitFieldNames ? '' : 'typedMetadata',
        entryClassName: 'GraphNode.TypedMetadataEntry',
        keyFieldType: $pb.PbFieldType.OS,
        valueFieldType: $pb.PbFieldType.OM,
        valueCreator: MetadataValue.create,
        valueDefaultOrMaker: MetadataValue.getDefault,
        packageName: const $pb.PackageName('hyperspace'))
    ..pc<EdgeType>(6, _omitFieldNames ? '' : 'edgeTypes', $pb.PbFieldType.KE,
        valueOf: EdgeType.valueOf,
        enumValues: EdgeType.values,
        defaultEnumValue: EdgeType.UNKNOWN)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GraphNode clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GraphNode copyWith(void Function(GraphNode) updates) =>
      super.copyWith((message) => updates(message as GraphNode)) as GraphNode;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static GraphNode create() => GraphNode._();
  @$core.override
  GraphNode createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static GraphNode getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<GraphNode>(create);
  static GraphNode? _defaultInstance;

  @$pb.TagNumber(1)
  $core.int get id => $_getIZ(0);
  @$pb.TagNumber(1)
  set id($core.int value) => $_setUnsignedInt32(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.int get layer => $_getIZ(1);
  @$pb.TagNumber(2)
  set layer($core.int value) => $_setUnsignedInt32(1, value);
  @$pb.TagNumber(2)
  $core.bool hasLayer() => $_has(1);
  @$pb.TagNumber(2)
  void clearLayer() => $_clearField(2);

  @$pb.TagNumber(3)
  $pb.PbList<$core.int> get neighbors => $_getList(2);

  @$pb.TagNumber(4)
  $pb.PbMap<$core.String, $core.String> get metadata => $_getMap(3);

  @$pb.TagNumber(5)
  $pb.PbMap<$core.String, MetadataValue> get typedMetadata => $_getMap(4);

  @$pb.TagNumber(6)
  $pb.PbList<EdgeType> get edgeTypes => $_getList(5);
}

class GetNeighborsRequest extends $pb.GeneratedMessage {
  factory GetNeighborsRequest({
    $core.String? collection,
    $core.int? id,
    $core.int? layer,
    $core.int? limit,
    $core.int? offset,
  }) {
    final result = create();
    if (collection != null) result.collection = collection;
    if (id != null) result.id = id;
    if (layer != null) result.layer = layer;
    if (limit != null) result.limit = limit;
    if (offset != null) result.offset = offset;
    return result;
  }

  GetNeighborsRequest._();

  factory GetNeighborsRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory GetNeighborsRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetNeighborsRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'hyperspace'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'collection')
    ..aI(2, _omitFieldNames ? '' : 'id', fieldType: $pb.PbFieldType.OU3)
    ..aI(3, _omitFieldNames ? '' : 'layer', fieldType: $pb.PbFieldType.OU3)
    ..aI(4, _omitFieldNames ? '' : 'limit', fieldType: $pb.PbFieldType.OU3)
    ..aI(5, _omitFieldNames ? '' : 'offset', fieldType: $pb.PbFieldType.OU3)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetNeighborsRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetNeighborsRequest copyWith(void Function(GetNeighborsRequest) updates) =>
      super.copyWith((message) => updates(message as GetNeighborsRequest))
          as GetNeighborsRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static GetNeighborsRequest create() => GetNeighborsRequest._();
  @$core.override
  GetNeighborsRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static GetNeighborsRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GetNeighborsRequest>(create);
  static GetNeighborsRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get collection => $_getSZ(0);
  @$pb.TagNumber(1)
  set collection($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasCollection() => $_has(0);
  @$pb.TagNumber(1)
  void clearCollection() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.int get id => $_getIZ(1);
  @$pb.TagNumber(2)
  set id($core.int value) => $_setUnsignedInt32(1, value);
  @$pb.TagNumber(2)
  $core.bool hasId() => $_has(1);
  @$pb.TagNumber(2)
  void clearId() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.int get layer => $_getIZ(2);
  @$pb.TagNumber(3)
  set layer($core.int value) => $_setUnsignedInt32(2, value);
  @$pb.TagNumber(3)
  $core.bool hasLayer() => $_has(2);
  @$pb.TagNumber(3)
  void clearLayer() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.int get limit => $_getIZ(3);
  @$pb.TagNumber(4)
  set limit($core.int value) => $_setUnsignedInt32(3, value);
  @$pb.TagNumber(4)
  $core.bool hasLimit() => $_has(3);
  @$pb.TagNumber(4)
  void clearLimit() => $_clearField(4);

  @$pb.TagNumber(5)
  $core.int get offset => $_getIZ(4);
  @$pb.TagNumber(5)
  set offset($core.int value) => $_setUnsignedInt32(4, value);
  @$pb.TagNumber(5)
  $core.bool hasOffset() => $_has(4);
  @$pb.TagNumber(5)
  void clearOffset() => $_clearField(5);
}

class GetNeighborsResponse extends $pb.GeneratedMessage {
  factory GetNeighborsResponse({
    $core.Iterable<GraphNode>? neighbors,
    $core.Iterable<$core.double>? edgeWeights,
  }) {
    final result = create();
    if (neighbors != null) result.neighbors.addAll(neighbors);
    if (edgeWeights != null) result.edgeWeights.addAll(edgeWeights);
    return result;
  }

  GetNeighborsResponse._();

  factory GetNeighborsResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory GetNeighborsResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetNeighborsResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'hyperspace'),
      createEmptyInstance: create)
    ..pPM<GraphNode>(1, _omitFieldNames ? '' : 'neighbors',
        subBuilder: GraphNode.create)
    ..p<$core.double>(
        2, _omitFieldNames ? '' : 'edgeWeights', $pb.PbFieldType.KD)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetNeighborsResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetNeighborsResponse copyWith(void Function(GetNeighborsResponse) updates) =>
      super.copyWith((message) => updates(message as GetNeighborsResponse))
          as GetNeighborsResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static GetNeighborsResponse create() => GetNeighborsResponse._();
  @$core.override
  GetNeighborsResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static GetNeighborsResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GetNeighborsResponse>(create);
  static GetNeighborsResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<GraphNode> get neighbors => $_getList(0);

  @$pb.TagNumber(2)
  $pb.PbList<$core.double> get edgeWeights => $_getList(1);
}

class TraverseRequest extends $pb.GeneratedMessage {
  factory TraverseRequest({
    $core.String? collection,
    $core.int? startId,
    $core.int? maxDepth,
    $core.int? maxNodes,
    $core.int? layer,
    $core.Iterable<$core.MapEntry<$core.String, $core.String>>? filter,
    $core.Iterable<Filter>? filters,
    TraversalMode? traversalMode,
    $core.int? breadthLimit,
  }) {
    final result = create();
    if (collection != null) result.collection = collection;
    if (startId != null) result.startId = startId;
    if (maxDepth != null) result.maxDepth = maxDepth;
    if (maxNodes != null) result.maxNodes = maxNodes;
    if (layer != null) result.layer = layer;
    if (filter != null) result.filter.addEntries(filter);
    if (filters != null) result.filters.addAll(filters);
    if (traversalMode != null) result.traversalMode = traversalMode;
    if (breadthLimit != null) result.breadthLimit = breadthLimit;
    return result;
  }

  TraverseRequest._();

  factory TraverseRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory TraverseRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'TraverseRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'hyperspace'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'collection')
    ..aI(2, _omitFieldNames ? '' : 'startId', fieldType: $pb.PbFieldType.OU3)
    ..aI(3, _omitFieldNames ? '' : 'maxDepth', fieldType: $pb.PbFieldType.OU3)
    ..aI(4, _omitFieldNames ? '' : 'maxNodes', fieldType: $pb.PbFieldType.OU3)
    ..aI(5, _omitFieldNames ? '' : 'layer', fieldType: $pb.PbFieldType.OU3)
    ..m<$core.String, $core.String>(6, _omitFieldNames ? '' : 'filter',
        entryClassName: 'TraverseRequest.FilterEntry',
        keyFieldType: $pb.PbFieldType.OS,
        valueFieldType: $pb.PbFieldType.OS,
        packageName: const $pb.PackageName('hyperspace'))
    ..pPM<Filter>(7, _omitFieldNames ? '' : 'filters',
        subBuilder: Filter.create)
    ..aE<TraversalMode>(8, _omitFieldNames ? '' : 'traversalMode',
        enumValues: TraversalMode.values)
    ..aI(9, _omitFieldNames ? '' : 'breadthLimit',
        fieldType: $pb.PbFieldType.OU3)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  TraverseRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  TraverseRequest copyWith(void Function(TraverseRequest) updates) =>
      super.copyWith((message) => updates(message as TraverseRequest))
          as TraverseRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static TraverseRequest create() => TraverseRequest._();
  @$core.override
  TraverseRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static TraverseRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<TraverseRequest>(create);
  static TraverseRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get collection => $_getSZ(0);
  @$pb.TagNumber(1)
  set collection($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasCollection() => $_has(0);
  @$pb.TagNumber(1)
  void clearCollection() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.int get startId => $_getIZ(1);
  @$pb.TagNumber(2)
  set startId($core.int value) => $_setUnsignedInt32(1, value);
  @$pb.TagNumber(2)
  $core.bool hasStartId() => $_has(1);
  @$pb.TagNumber(2)
  void clearStartId() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.int get maxDepth => $_getIZ(2);
  @$pb.TagNumber(3)
  set maxDepth($core.int value) => $_setUnsignedInt32(2, value);
  @$pb.TagNumber(3)
  $core.bool hasMaxDepth() => $_has(2);
  @$pb.TagNumber(3)
  void clearMaxDepth() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.int get maxNodes => $_getIZ(3);
  @$pb.TagNumber(4)
  set maxNodes($core.int value) => $_setUnsignedInt32(3, value);
  @$pb.TagNumber(4)
  $core.bool hasMaxNodes() => $_has(3);
  @$pb.TagNumber(4)
  void clearMaxNodes() => $_clearField(4);

  @$pb.TagNumber(5)
  $core.int get layer => $_getIZ(4);
  @$pb.TagNumber(5)
  set layer($core.int value) => $_setUnsignedInt32(4, value);
  @$pb.TagNumber(5)
  $core.bool hasLayer() => $_has(4);
  @$pb.TagNumber(5)
  void clearLayer() => $_clearField(5);

  @$pb.TagNumber(6)
  $pb.PbMap<$core.String, $core.String> get filter => $_getMap(5);

  @$pb.TagNumber(7)
  $pb.PbList<Filter> get filters => $_getList(6);

  @$pb.TagNumber(8)
  TraversalMode get traversalMode => $_getN(7);
  @$pb.TagNumber(8)
  set traversalMode(TraversalMode value) => $_setField(8, value);
  @$pb.TagNumber(8)
  $core.bool hasTraversalMode() => $_has(7);
  @$pb.TagNumber(8)
  void clearTraversalMode() => $_clearField(8);

  @$pb.TagNumber(9)
  $core.int get breadthLimit => $_getIZ(8);
  @$pb.TagNumber(9)
  set breadthLimit($core.int value) => $_setUnsignedInt32(8, value);
  @$pb.TagNumber(9)
  $core.bool hasBreadthLimit() => $_has(8);
  @$pb.TagNumber(9)
  void clearBreadthLimit() => $_clearField(9);
}

class TraverseResponse extends $pb.GeneratedMessage {
  factory TraverseResponse({
    $core.Iterable<GraphNode>? nodes,
  }) {
    final result = create();
    if (nodes != null) result.nodes.addAll(nodes);
    return result;
  }

  TraverseResponse._();

  factory TraverseResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory TraverseResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'TraverseResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'hyperspace'),
      createEmptyInstance: create)
    ..pPM<GraphNode>(1, _omitFieldNames ? '' : 'nodes',
        subBuilder: GraphNode.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  TraverseResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  TraverseResponse copyWith(void Function(TraverseResponse) updates) =>
      super.copyWith((message) => updates(message as TraverseResponse))
          as TraverseResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static TraverseResponse create() => TraverseResponse._();
  @$core.override
  TraverseResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static TraverseResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<TraverseResponse>(create);
  static TraverseResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<GraphNode> get nodes => $_getList(0);
}

class GetSubsumptionTreeRequest extends $pb.GeneratedMessage {
  factory GetSubsumptionTreeRequest({
    $core.String? collection,
    $core.int? rootId,
    $core.int? maxDepth,
  }) {
    final result = create();
    if (collection != null) result.collection = collection;
    if (rootId != null) result.rootId = rootId;
    if (maxDepth != null) result.maxDepth = maxDepth;
    return result;
  }

  GetSubsumptionTreeRequest._();

  factory GetSubsumptionTreeRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory GetSubsumptionTreeRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetSubsumptionTreeRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'hyperspace'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'collection')
    ..aI(2, _omitFieldNames ? '' : 'rootId', fieldType: $pb.PbFieldType.OU3)
    ..aI(3, _omitFieldNames ? '' : 'maxDepth', fieldType: $pb.PbFieldType.OU3)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetSubsumptionTreeRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetSubsumptionTreeRequest copyWith(
          void Function(GetSubsumptionTreeRequest) updates) =>
      super.copyWith((message) => updates(message as GetSubsumptionTreeRequest))
          as GetSubsumptionTreeRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static GetSubsumptionTreeRequest create() => GetSubsumptionTreeRequest._();
  @$core.override
  GetSubsumptionTreeRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static GetSubsumptionTreeRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GetSubsumptionTreeRequest>(create);
  static GetSubsumptionTreeRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get collection => $_getSZ(0);
  @$pb.TagNumber(1)
  set collection($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasCollection() => $_has(0);
  @$pb.TagNumber(1)
  void clearCollection() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.int get rootId => $_getIZ(1);
  @$pb.TagNumber(2)
  set rootId($core.int value) => $_setUnsignedInt32(1, value);
  @$pb.TagNumber(2)
  $core.bool hasRootId() => $_has(1);
  @$pb.TagNumber(2)
  void clearRootId() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.int get maxDepth => $_getIZ(2);
  @$pb.TagNumber(3)
  set maxDepth($core.int value) => $_setUnsignedInt32(2, value);
  @$pb.TagNumber(3)
  $core.bool hasMaxDepth() => $_has(2);
  @$pb.TagNumber(3)
  void clearMaxDepth() => $_clearField(3);
}

class GetSubsumptionTreeResponse extends $pb.GeneratedMessage {
  factory GetSubsumptionTreeResponse({
    $core.Iterable<GraphNode>? nodes,
  }) {
    final result = create();
    if (nodes != null) result.nodes.addAll(nodes);
    return result;
  }

  GetSubsumptionTreeResponse._();

  factory GetSubsumptionTreeResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory GetSubsumptionTreeResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetSubsumptionTreeResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'hyperspace'),
      createEmptyInstance: create)
    ..pPM<GraphNode>(1, _omitFieldNames ? '' : 'nodes',
        subBuilder: GraphNode.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetSubsumptionTreeResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetSubsumptionTreeResponse copyWith(
          void Function(GetSubsumptionTreeResponse) updates) =>
      super.copyWith(
              (message) => updates(message as GetSubsumptionTreeResponse))
          as GetSubsumptionTreeResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static GetSubsumptionTreeResponse create() => GetSubsumptionTreeResponse._();
  @$core.override
  GetSubsumptionTreeResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static GetSubsumptionTreeResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GetSubsumptionTreeResponse>(create);
  static GetSubsumptionTreeResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<GraphNode> get nodes => $_getList(0);
}

class FindSemanticClustersRequest extends $pb.GeneratedMessage {
  factory FindSemanticClustersRequest({
    $core.String? collection,
    $core.int? layer,
    $core.int? minClusterSize,
    $core.int? maxClusters,
    $core.int? maxNodes,
  }) {
    final result = create();
    if (collection != null) result.collection = collection;
    if (layer != null) result.layer = layer;
    if (minClusterSize != null) result.minClusterSize = minClusterSize;
    if (maxClusters != null) result.maxClusters = maxClusters;
    if (maxNodes != null) result.maxNodes = maxNodes;
    return result;
  }

  FindSemanticClustersRequest._();

  factory FindSemanticClustersRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory FindSemanticClustersRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'FindSemanticClustersRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'hyperspace'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'collection')
    ..aI(2, _omitFieldNames ? '' : 'layer', fieldType: $pb.PbFieldType.OU3)
    ..aI(3, _omitFieldNames ? '' : 'minClusterSize',
        fieldType: $pb.PbFieldType.OU3)
    ..aI(4, _omitFieldNames ? '' : 'maxClusters',
        fieldType: $pb.PbFieldType.OU3)
    ..aI(5, _omitFieldNames ? '' : 'maxNodes', fieldType: $pb.PbFieldType.OU3)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  FindSemanticClustersRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  FindSemanticClustersRequest copyWith(
          void Function(FindSemanticClustersRequest) updates) =>
      super.copyWith(
              (message) => updates(message as FindSemanticClustersRequest))
          as FindSemanticClustersRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static FindSemanticClustersRequest create() =>
      FindSemanticClustersRequest._();
  @$core.override
  FindSemanticClustersRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static FindSemanticClustersRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<FindSemanticClustersRequest>(create);
  static FindSemanticClustersRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get collection => $_getSZ(0);
  @$pb.TagNumber(1)
  set collection($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasCollection() => $_has(0);
  @$pb.TagNumber(1)
  void clearCollection() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.int get layer => $_getIZ(1);
  @$pb.TagNumber(2)
  set layer($core.int value) => $_setUnsignedInt32(1, value);
  @$pb.TagNumber(2)
  $core.bool hasLayer() => $_has(1);
  @$pb.TagNumber(2)
  void clearLayer() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.int get minClusterSize => $_getIZ(2);
  @$pb.TagNumber(3)
  set minClusterSize($core.int value) => $_setUnsignedInt32(2, value);
  @$pb.TagNumber(3)
  $core.bool hasMinClusterSize() => $_has(2);
  @$pb.TagNumber(3)
  void clearMinClusterSize() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.int get maxClusters => $_getIZ(3);
  @$pb.TagNumber(4)
  set maxClusters($core.int value) => $_setUnsignedInt32(3, value);
  @$pb.TagNumber(4)
  $core.bool hasMaxClusters() => $_has(3);
  @$pb.TagNumber(4)
  void clearMaxClusters() => $_clearField(4);

  @$pb.TagNumber(5)
  $core.int get maxNodes => $_getIZ(4);
  @$pb.TagNumber(5)
  set maxNodes($core.int value) => $_setUnsignedInt32(4, value);
  @$pb.TagNumber(5)
  $core.bool hasMaxNodes() => $_has(4);
  @$pb.TagNumber(5)
  void clearMaxNodes() => $_clearField(5);
}

class GetConceptParentsRequest extends $pb.GeneratedMessage {
  factory GetConceptParentsRequest({
    $core.String? collection,
    $core.int? id,
    $core.int? layer,
    $core.int? limit,
  }) {
    final result = create();
    if (collection != null) result.collection = collection;
    if (id != null) result.id = id;
    if (layer != null) result.layer = layer;
    if (limit != null) result.limit = limit;
    return result;
  }

  GetConceptParentsRequest._();

  factory GetConceptParentsRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory GetConceptParentsRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetConceptParentsRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'hyperspace'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'collection')
    ..aI(2, _omitFieldNames ? '' : 'id', fieldType: $pb.PbFieldType.OU3)
    ..aI(3, _omitFieldNames ? '' : 'layer', fieldType: $pb.PbFieldType.OU3)
    ..aI(4, _omitFieldNames ? '' : 'limit', fieldType: $pb.PbFieldType.OU3)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetConceptParentsRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetConceptParentsRequest copyWith(
          void Function(GetConceptParentsRequest) updates) =>
      super.copyWith((message) => updates(message as GetConceptParentsRequest))
          as GetConceptParentsRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static GetConceptParentsRequest create() => GetConceptParentsRequest._();
  @$core.override
  GetConceptParentsRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static GetConceptParentsRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GetConceptParentsRequest>(create);
  static GetConceptParentsRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get collection => $_getSZ(0);
  @$pb.TagNumber(1)
  set collection($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasCollection() => $_has(0);
  @$pb.TagNumber(1)
  void clearCollection() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.int get id => $_getIZ(1);
  @$pb.TagNumber(2)
  set id($core.int value) => $_setUnsignedInt32(1, value);
  @$pb.TagNumber(2)
  $core.bool hasId() => $_has(1);
  @$pb.TagNumber(2)
  void clearId() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.int get layer => $_getIZ(2);
  @$pb.TagNumber(3)
  set layer($core.int value) => $_setUnsignedInt32(2, value);
  @$pb.TagNumber(3)
  $core.bool hasLayer() => $_has(2);
  @$pb.TagNumber(3)
  void clearLayer() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.int get limit => $_getIZ(3);
  @$pb.TagNumber(4)
  set limit($core.int value) => $_setUnsignedInt32(3, value);
  @$pb.TagNumber(4)
  $core.bool hasLimit() => $_has(3);
  @$pb.TagNumber(4)
  void clearLimit() => $_clearField(4);
}

class GetConceptParentsResponse extends $pb.GeneratedMessage {
  factory GetConceptParentsResponse({
    $core.Iterable<GraphNode>? parents,
  }) {
    final result = create();
    if (parents != null) result.parents.addAll(parents);
    return result;
  }

  GetConceptParentsResponse._();

  factory GetConceptParentsResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory GetConceptParentsResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetConceptParentsResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'hyperspace'),
      createEmptyInstance: create)
    ..pPM<GraphNode>(1, _omitFieldNames ? '' : 'parents',
        subBuilder: GraphNode.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetConceptParentsResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetConceptParentsResponse copyWith(
          void Function(GetConceptParentsResponse) updates) =>
      super.copyWith((message) => updates(message as GetConceptParentsResponse))
          as GetConceptParentsResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static GetConceptParentsResponse create() => GetConceptParentsResponse._();
  @$core.override
  GetConceptParentsResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static GetConceptParentsResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GetConceptParentsResponse>(create);
  static GetConceptParentsResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<GraphNode> get parents => $_getList(0);
}

class GraphCluster extends $pb.GeneratedMessage {
  factory GraphCluster({
    $core.Iterable<$core.int>? nodeIds,
  }) {
    final result = create();
    if (nodeIds != null) result.nodeIds.addAll(nodeIds);
    return result;
  }

  GraphCluster._();

  factory GraphCluster.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory GraphCluster.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GraphCluster',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'hyperspace'),
      createEmptyInstance: create)
    ..p<$core.int>(1, _omitFieldNames ? '' : 'nodeIds', $pb.PbFieldType.KU3)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GraphCluster clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GraphCluster copyWith(void Function(GraphCluster) updates) =>
      super.copyWith((message) => updates(message as GraphCluster))
          as GraphCluster;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static GraphCluster create() => GraphCluster._();
  @$core.override
  GraphCluster createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static GraphCluster getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GraphCluster>(create);
  static GraphCluster? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<$core.int> get nodeIds => $_getList(0);
}

class FindSemanticClustersResponse extends $pb.GeneratedMessage {
  factory FindSemanticClustersResponse({
    $core.Iterable<GraphCluster>? clusters,
  }) {
    final result = create();
    if (clusters != null) result.clusters.addAll(clusters);
    return result;
  }

  FindSemanticClustersResponse._();

  factory FindSemanticClustersResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory FindSemanticClustersResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'FindSemanticClustersResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'hyperspace'),
      createEmptyInstance: create)
    ..pPM<GraphCluster>(1, _omitFieldNames ? '' : 'clusters',
        subBuilder: GraphCluster.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  FindSemanticClustersResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  FindSemanticClustersResponse copyWith(
          void Function(FindSemanticClustersResponse) updates) =>
      super.copyWith(
              (message) => updates(message as FindSemanticClustersResponse))
          as FindSemanticClustersResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static FindSemanticClustersResponse create() =>
      FindSemanticClustersResponse._();
  @$core.override
  FindSemanticClustersResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static FindSemanticClustersResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<FindSemanticClustersResponse>(create);
  static FindSemanticClustersResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<GraphCluster> get clusters => $_getList(0);
}

enum MetadataValue_Kind {
  stringValue,
  intValue,
  doubleValue,
  boolValue,
  notSet
}

class MetadataValue extends $pb.GeneratedMessage {
  factory MetadataValue({
    $core.String? stringValue,
    $fixnum.Int64? intValue,
    $core.double? doubleValue,
    $core.bool? boolValue,
  }) {
    final result = create();
    if (stringValue != null) result.stringValue = stringValue;
    if (intValue != null) result.intValue = intValue;
    if (doubleValue != null) result.doubleValue = doubleValue;
    if (boolValue != null) result.boolValue = boolValue;
    return result;
  }

  MetadataValue._();

  factory MetadataValue.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory MetadataValue.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static const $core.Map<$core.int, MetadataValue_Kind>
      _MetadataValue_KindByTag = {
    1: MetadataValue_Kind.stringValue,
    2: MetadataValue_Kind.intValue,
    3: MetadataValue_Kind.doubleValue,
    4: MetadataValue_Kind.boolValue,
    0: MetadataValue_Kind.notSet
  };
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'MetadataValue',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'hyperspace'),
      createEmptyInstance: create)
    ..oo(0, [1, 2, 3, 4])
    ..aOS(1, _omitFieldNames ? '' : 'stringValue')
    ..aInt64(2, _omitFieldNames ? '' : 'intValue')
    ..aD(3, _omitFieldNames ? '' : 'doubleValue')
    ..aOB(4, _omitFieldNames ? '' : 'boolValue')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  MetadataValue clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  MetadataValue copyWith(void Function(MetadataValue) updates) =>
      super.copyWith((message) => updates(message as MetadataValue))
          as MetadataValue;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static MetadataValue create() => MetadataValue._();
  @$core.override
  MetadataValue createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static MetadataValue getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<MetadataValue>(create);
  static MetadataValue? _defaultInstance;

  @$pb.TagNumber(1)
  @$pb.TagNumber(2)
  @$pb.TagNumber(3)
  @$pb.TagNumber(4)
  MetadataValue_Kind whichKind() => _MetadataValue_KindByTag[$_whichOneof(0)]!;
  @$pb.TagNumber(1)
  @$pb.TagNumber(2)
  @$pb.TagNumber(3)
  @$pb.TagNumber(4)
  void clearKind() => $_clearField($_whichOneof(0));

  @$pb.TagNumber(1)
  $core.String get stringValue => $_getSZ(0);
  @$pb.TagNumber(1)
  set stringValue($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasStringValue() => $_has(0);
  @$pb.TagNumber(1)
  void clearStringValue() => $_clearField(1);

  @$pb.TagNumber(2)
  $fixnum.Int64 get intValue => $_getI64(1);
  @$pb.TagNumber(2)
  set intValue($fixnum.Int64 value) => $_setInt64(1, value);
  @$pb.TagNumber(2)
  $core.bool hasIntValue() => $_has(1);
  @$pb.TagNumber(2)
  void clearIntValue() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.double get doubleValue => $_getN(2);
  @$pb.TagNumber(3)
  set doubleValue($core.double value) => $_setDouble(2, value);
  @$pb.TagNumber(3)
  $core.bool hasDoubleValue() => $_has(2);
  @$pb.TagNumber(3)
  void clearDoubleValue() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.bool get boolValue => $_getBF(3);
  @$pb.TagNumber(4)
  set boolValue($core.bool value) => $_setBool(3, value);
  @$pb.TagNumber(4)
  $core.bool hasBoolValue() => $_has(3);
  @$pb.TagNumber(4)
  void clearBoolValue() => $_clearField(4);
}

class EventSubscriptionRequest extends $pb.GeneratedMessage {
  factory EventSubscriptionRequest({
    $core.Iterable<EventType>? types,
    $core.String? collection,
  }) {
    final result = create();
    if (types != null) result.types.addAll(types);
    if (collection != null) result.collection = collection;
    return result;
  }

  EventSubscriptionRequest._();

  factory EventSubscriptionRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory EventSubscriptionRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'EventSubscriptionRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'hyperspace'),
      createEmptyInstance: create)
    ..pc<EventType>(1, _omitFieldNames ? '' : 'types', $pb.PbFieldType.KE,
        valueOf: EventType.valueOf,
        enumValues: EventType.values,
        defaultEnumValue: EventType.EVENT_UNKNOWN)
    ..aOS(2, _omitFieldNames ? '' : 'collection')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  EventSubscriptionRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  EventSubscriptionRequest copyWith(
          void Function(EventSubscriptionRequest) updates) =>
      super.copyWith((message) => updates(message as EventSubscriptionRequest))
          as EventSubscriptionRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static EventSubscriptionRequest create() => EventSubscriptionRequest._();
  @$core.override
  EventSubscriptionRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static EventSubscriptionRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<EventSubscriptionRequest>(create);
  static EventSubscriptionRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<EventType> get types => $_getList(0);

  @$pb.TagNumber(2)
  $core.String get collection => $_getSZ(1);
  @$pb.TagNumber(2)
  set collection($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasCollection() => $_has(1);
  @$pb.TagNumber(2)
  void clearCollection() => $_clearField(2);
}

class VectorInsertedEvent extends $pb.GeneratedMessage {
  factory VectorInsertedEvent({
    $core.int? id,
    $core.String? collection,
    $fixnum.Int64? logicalClock,
    $core.String? originNodeId,
    $core.Iterable<$core.MapEntry<$core.String, $core.String>>? metadata,
    $core.Iterable<$core.MapEntry<$core.String, MetadataValue>>? typedMetadata,
  }) {
    final result = create();
    if (id != null) result.id = id;
    if (collection != null) result.collection = collection;
    if (logicalClock != null) result.logicalClock = logicalClock;
    if (originNodeId != null) result.originNodeId = originNodeId;
    if (metadata != null) result.metadata.addEntries(metadata);
    if (typedMetadata != null) result.typedMetadata.addEntries(typedMetadata);
    return result;
  }

  VectorInsertedEvent._();

  factory VectorInsertedEvent.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory VectorInsertedEvent.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'VectorInsertedEvent',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'hyperspace'),
      createEmptyInstance: create)
    ..aI(1, _omitFieldNames ? '' : 'id', fieldType: $pb.PbFieldType.OU3)
    ..aOS(2, _omitFieldNames ? '' : 'collection')
    ..a<$fixnum.Int64>(
        3, _omitFieldNames ? '' : 'logicalClock', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..aOS(4, _omitFieldNames ? '' : 'originNodeId')
    ..m<$core.String, $core.String>(5, _omitFieldNames ? '' : 'metadata',
        entryClassName: 'VectorInsertedEvent.MetadataEntry',
        keyFieldType: $pb.PbFieldType.OS,
        valueFieldType: $pb.PbFieldType.OS,
        packageName: const $pb.PackageName('hyperspace'))
    ..m<$core.String, MetadataValue>(6, _omitFieldNames ? '' : 'typedMetadata',
        entryClassName: 'VectorInsertedEvent.TypedMetadataEntry',
        keyFieldType: $pb.PbFieldType.OS,
        valueFieldType: $pb.PbFieldType.OM,
        valueCreator: MetadataValue.create,
        valueDefaultOrMaker: MetadataValue.getDefault,
        packageName: const $pb.PackageName('hyperspace'))
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  VectorInsertedEvent clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  VectorInsertedEvent copyWith(void Function(VectorInsertedEvent) updates) =>
      super.copyWith((message) => updates(message as VectorInsertedEvent))
          as VectorInsertedEvent;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static VectorInsertedEvent create() => VectorInsertedEvent._();
  @$core.override
  VectorInsertedEvent createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static VectorInsertedEvent getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<VectorInsertedEvent>(create);
  static VectorInsertedEvent? _defaultInstance;

  @$pb.TagNumber(1)
  $core.int get id => $_getIZ(0);
  @$pb.TagNumber(1)
  set id($core.int value) => $_setUnsignedInt32(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get collection => $_getSZ(1);
  @$pb.TagNumber(2)
  set collection($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasCollection() => $_has(1);
  @$pb.TagNumber(2)
  void clearCollection() => $_clearField(2);

  @$pb.TagNumber(3)
  $fixnum.Int64 get logicalClock => $_getI64(2);
  @$pb.TagNumber(3)
  set logicalClock($fixnum.Int64 value) => $_setInt64(2, value);
  @$pb.TagNumber(3)
  $core.bool hasLogicalClock() => $_has(2);
  @$pb.TagNumber(3)
  void clearLogicalClock() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.String get originNodeId => $_getSZ(3);
  @$pb.TagNumber(4)
  set originNodeId($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasOriginNodeId() => $_has(3);
  @$pb.TagNumber(4)
  void clearOriginNodeId() => $_clearField(4);

  @$pb.TagNumber(5)
  $pb.PbMap<$core.String, $core.String> get metadata => $_getMap(4);

  @$pb.TagNumber(6)
  $pb.PbMap<$core.String, MetadataValue> get typedMetadata => $_getMap(5);
}

class TrajectoryStepEvent extends $pb.GeneratedMessage {
  factory TrajectoryStepEvent({
    $core.int? id,
    $core.String? collection,
    $core.double? x,
    $core.double? y,
    $core.Iterable<$core.MapEntry<$core.String, $core.String>>? metadata,
  }) {
    final result = create();
    if (id != null) result.id = id;
    if (collection != null) result.collection = collection;
    if (x != null) result.x = x;
    if (y != null) result.y = y;
    if (metadata != null) result.metadata.addEntries(metadata);
    return result;
  }

  TrajectoryStepEvent._();

  factory TrajectoryStepEvent.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory TrajectoryStepEvent.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'TrajectoryStepEvent',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'hyperspace'),
      createEmptyInstance: create)
    ..aI(1, _omitFieldNames ? '' : 'id', fieldType: $pb.PbFieldType.OU3)
    ..aOS(2, _omitFieldNames ? '' : 'collection')
    ..aD(3, _omitFieldNames ? '' : 'x', fieldType: $pb.PbFieldType.OF)
    ..aD(4, _omitFieldNames ? '' : 'y', fieldType: $pb.PbFieldType.OF)
    ..m<$core.String, $core.String>(5, _omitFieldNames ? '' : 'metadata',
        entryClassName: 'TrajectoryStepEvent.MetadataEntry',
        keyFieldType: $pb.PbFieldType.OS,
        valueFieldType: $pb.PbFieldType.OS,
        packageName: const $pb.PackageName('hyperspace'))
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  TrajectoryStepEvent clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  TrajectoryStepEvent copyWith(void Function(TrajectoryStepEvent) updates) =>
      super.copyWith((message) => updates(message as TrajectoryStepEvent))
          as TrajectoryStepEvent;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static TrajectoryStepEvent create() => TrajectoryStepEvent._();
  @$core.override
  TrajectoryStepEvent createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static TrajectoryStepEvent getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<TrajectoryStepEvent>(create);
  static TrajectoryStepEvent? _defaultInstance;

  @$pb.TagNumber(1)
  $core.int get id => $_getIZ(0);
  @$pb.TagNumber(1)
  set id($core.int value) => $_setUnsignedInt32(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get collection => $_getSZ(1);
  @$pb.TagNumber(2)
  set collection($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasCollection() => $_has(1);
  @$pb.TagNumber(2)
  void clearCollection() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.double get x => $_getN(2);
  @$pb.TagNumber(3)
  set x($core.double value) => $_setFloat(2, value);
  @$pb.TagNumber(3)
  $core.bool hasX() => $_has(2);
  @$pb.TagNumber(3)
  void clearX() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.double get y => $_getN(3);
  @$pb.TagNumber(4)
  set y($core.double value) => $_setFloat(3, value);
  @$pb.TagNumber(4)
  $core.bool hasY() => $_has(3);
  @$pb.TagNumber(4)
  void clearY() => $_clearField(4);

  @$pb.TagNumber(5)
  $pb.PbMap<$core.String, $core.String> get metadata => $_getMap(4);
}

class VectorDeletedEvent extends $pb.GeneratedMessage {
  factory VectorDeletedEvent({
    $core.int? id,
    $core.String? collection,
    $fixnum.Int64? logicalClock,
    $core.String? originNodeId,
  }) {
    final result = create();
    if (id != null) result.id = id;
    if (collection != null) result.collection = collection;
    if (logicalClock != null) result.logicalClock = logicalClock;
    if (originNodeId != null) result.originNodeId = originNodeId;
    return result;
  }

  VectorDeletedEvent._();

  factory VectorDeletedEvent.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory VectorDeletedEvent.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'VectorDeletedEvent',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'hyperspace'),
      createEmptyInstance: create)
    ..aI(1, _omitFieldNames ? '' : 'id', fieldType: $pb.PbFieldType.OU3)
    ..aOS(2, _omitFieldNames ? '' : 'collection')
    ..a<$fixnum.Int64>(
        3, _omitFieldNames ? '' : 'logicalClock', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..aOS(4, _omitFieldNames ? '' : 'originNodeId')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  VectorDeletedEvent clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  VectorDeletedEvent copyWith(void Function(VectorDeletedEvent) updates) =>
      super.copyWith((message) => updates(message as VectorDeletedEvent))
          as VectorDeletedEvent;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static VectorDeletedEvent create() => VectorDeletedEvent._();
  @$core.override
  VectorDeletedEvent createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static VectorDeletedEvent getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<VectorDeletedEvent>(create);
  static VectorDeletedEvent? _defaultInstance;

  @$pb.TagNumber(1)
  $core.int get id => $_getIZ(0);
  @$pb.TagNumber(1)
  set id($core.int value) => $_setUnsignedInt32(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get collection => $_getSZ(1);
  @$pb.TagNumber(2)
  set collection($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasCollection() => $_has(1);
  @$pb.TagNumber(2)
  void clearCollection() => $_clearField(2);

  @$pb.TagNumber(3)
  $fixnum.Int64 get logicalClock => $_getI64(2);
  @$pb.TagNumber(3)
  set logicalClock($fixnum.Int64 value) => $_setInt64(2, value);
  @$pb.TagNumber(3)
  $core.bool hasLogicalClock() => $_has(2);
  @$pb.TagNumber(3)
  void clearLogicalClock() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.String get originNodeId => $_getSZ(3);
  @$pb.TagNumber(4)
  set originNodeId($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasOriginNodeId() => $_has(3);
  @$pb.TagNumber(4)
  void clearOriginNodeId() => $_clearField(4);
}

enum EventMessage_Payload {
  vectorInserted,
  vectorDeleted,
  trajectoryStep,
  notSet
}

class EventMessage extends $pb.GeneratedMessage {
  factory EventMessage({
    EventType? type,
    VectorInsertedEvent? vectorInserted,
    VectorDeletedEvent? vectorDeleted,
    TrajectoryStepEvent? trajectoryStep,
  }) {
    final result = create();
    if (type != null) result.type = type;
    if (vectorInserted != null) result.vectorInserted = vectorInserted;
    if (vectorDeleted != null) result.vectorDeleted = vectorDeleted;
    if (trajectoryStep != null) result.trajectoryStep = trajectoryStep;
    return result;
  }

  EventMessage._();

  factory EventMessage.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory EventMessage.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static const $core.Map<$core.int, EventMessage_Payload>
      _EventMessage_PayloadByTag = {
    2: EventMessage_Payload.vectorInserted,
    3: EventMessage_Payload.vectorDeleted,
    4: EventMessage_Payload.trajectoryStep,
    0: EventMessage_Payload.notSet
  };
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'EventMessage',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'hyperspace'),
      createEmptyInstance: create)
    ..oo(0, [2, 3, 4])
    ..aE<EventType>(1, _omitFieldNames ? '' : 'type',
        enumValues: EventType.values)
    ..aOM<VectorInsertedEvent>(2, _omitFieldNames ? '' : 'vectorInserted',
        subBuilder: VectorInsertedEvent.create)
    ..aOM<VectorDeletedEvent>(3, _omitFieldNames ? '' : 'vectorDeleted',
        subBuilder: VectorDeletedEvent.create)
    ..aOM<TrajectoryStepEvent>(4, _omitFieldNames ? '' : 'trajectoryStep',
        subBuilder: TrajectoryStepEvent.create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  EventMessage clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  EventMessage copyWith(void Function(EventMessage) updates) =>
      super.copyWith((message) => updates(message as EventMessage))
          as EventMessage;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static EventMessage create() => EventMessage._();
  @$core.override
  EventMessage createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static EventMessage getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<EventMessage>(create);
  static EventMessage? _defaultInstance;

  @$pb.TagNumber(2)
  @$pb.TagNumber(3)
  @$pb.TagNumber(4)
  EventMessage_Payload whichPayload() =>
      _EventMessage_PayloadByTag[$_whichOneof(0)]!;
  @$pb.TagNumber(2)
  @$pb.TagNumber(3)
  @$pb.TagNumber(4)
  void clearPayload() => $_clearField($_whichOneof(0));

  @$pb.TagNumber(1)
  EventType get type => $_getN(0);
  @$pb.TagNumber(1)
  set type(EventType value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasType() => $_has(0);
  @$pb.TagNumber(1)
  void clearType() => $_clearField(1);

  @$pb.TagNumber(2)
  VectorInsertedEvent get vectorInserted => $_getN(1);
  @$pb.TagNumber(2)
  set vectorInserted(VectorInsertedEvent value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasVectorInserted() => $_has(1);
  @$pb.TagNumber(2)
  void clearVectorInserted() => $_clearField(2);
  @$pb.TagNumber(2)
  VectorInsertedEvent ensureVectorInserted() => $_ensure(1);

  @$pb.TagNumber(3)
  VectorDeletedEvent get vectorDeleted => $_getN(2);
  @$pb.TagNumber(3)
  set vectorDeleted(VectorDeletedEvent value) => $_setField(3, value);
  @$pb.TagNumber(3)
  $core.bool hasVectorDeleted() => $_has(2);
  @$pb.TagNumber(3)
  void clearVectorDeleted() => $_clearField(3);
  @$pb.TagNumber(3)
  VectorDeletedEvent ensureVectorDeleted() => $_ensure(2);

  @$pb.TagNumber(4)
  TrajectoryStepEvent get trajectoryStep => $_getN(3);
  @$pb.TagNumber(4)
  set trajectoryStep(TrajectoryStepEvent value) => $_setField(4, value);
  @$pb.TagNumber(4)
  $core.bool hasTrajectoryStep() => $_has(3);
  @$pb.TagNumber(4)
  void clearTrajectoryStep() => $_clearField(4);
  @$pb.TagNumber(4)
  TrajectoryStepEvent ensureTrajectoryStep() => $_ensure(3);
}

class Empty extends $pb.GeneratedMessage {
  factory Empty() => create();

  Empty._();

  factory Empty.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory Empty.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'Empty',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'hyperspace'),
      createEmptyInstance: create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Empty clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Empty copyWith(void Function(Empty) updates) =>
      super.copyWith((message) => updates(message as Empty)) as Empty;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static Empty create() => Empty._();
  @$core.override
  Empty createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static Empty getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<Empty>(create);
  static Empty? _defaultInstance;
}

class StatusResponse extends $pb.GeneratedMessage {
  factory StatusResponse({
    $core.String? status,
  }) {
    final result = create();
    if (status != null) result.status = status;
    return result;
  }

  StatusResponse._();

  factory StatusResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory StatusResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'StatusResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'hyperspace'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'status')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  StatusResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  StatusResponse copyWith(void Function(StatusResponse) updates) =>
      super.copyWith((message) => updates(message as StatusResponse))
          as StatusResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static StatusResponse create() => StatusResponse._();
  @$core.override
  StatusResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static StatusResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<StatusResponse>(create);
  static StatusResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get status => $_getSZ(0);
  @$pb.TagNumber(1)
  set status($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasStatus() => $_has(0);
  @$pb.TagNumber(1)
  void clearStatus() => $_clearField(1);
}

class MonitorRequest extends $pb.GeneratedMessage {
  factory MonitorRequest() => create();

  MonitorRequest._();

  factory MonitorRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory MonitorRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'MonitorRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'hyperspace'),
      createEmptyInstance: create)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  MonitorRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  MonitorRequest copyWith(void Function(MonitorRequest) updates) =>
      super.copyWith((message) => updates(message as MonitorRequest))
          as MonitorRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static MonitorRequest create() => MonitorRequest._();
  @$core.override
  MonitorRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static MonitorRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<MonitorRequest>(create);
  static MonitorRequest? _defaultInstance;
}

class SystemStats extends $pb.GeneratedMessage {
  factory SystemStats({
    $fixnum.Int64? totalCollections,
    $fixnum.Int64? totalVectors,
    $core.double? totalMemoryMb,
    $core.double? qps,
  }) {
    final result = create();
    if (totalCollections != null) result.totalCollections = totalCollections;
    if (totalVectors != null) result.totalVectors = totalVectors;
    if (totalMemoryMb != null) result.totalMemoryMb = totalMemoryMb;
    if (qps != null) result.qps = qps;
    return result;
  }

  SystemStats._();

  factory SystemStats.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory SystemStats.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'SystemStats',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'hyperspace'),
      createEmptyInstance: create)
    ..a<$fixnum.Int64>(
        1, _omitFieldNames ? '' : 'totalCollections', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..a<$fixnum.Int64>(
        2, _omitFieldNames ? '' : 'totalVectors', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..aD(3, _omitFieldNames ? '' : 'totalMemoryMb')
    ..aD(4, _omitFieldNames ? '' : 'qps')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SystemStats clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SystemStats copyWith(void Function(SystemStats) updates) =>
      super.copyWith((message) => updates(message as SystemStats))
          as SystemStats;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static SystemStats create() => SystemStats._();
  @$core.override
  SystemStats createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static SystemStats getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<SystemStats>(create);
  static SystemStats? _defaultInstance;

  @$pb.TagNumber(1)
  $fixnum.Int64 get totalCollections => $_getI64(0);
  @$pb.TagNumber(1)
  set totalCollections($fixnum.Int64 value) => $_setInt64(0, value);
  @$pb.TagNumber(1)
  $core.bool hasTotalCollections() => $_has(0);
  @$pb.TagNumber(1)
  void clearTotalCollections() => $_clearField(1);

  @$pb.TagNumber(2)
  $fixnum.Int64 get totalVectors => $_getI64(1);
  @$pb.TagNumber(2)
  set totalVectors($fixnum.Int64 value) => $_setInt64(1, value);
  @$pb.TagNumber(2)
  $core.bool hasTotalVectors() => $_has(1);
  @$pb.TagNumber(2)
  void clearTotalVectors() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.double get totalMemoryMb => $_getN(2);
  @$pb.TagNumber(3)
  set totalMemoryMb($core.double value) => $_setDouble(2, value);
  @$pb.TagNumber(3)
  $core.bool hasTotalMemoryMb() => $_has(2);
  @$pb.TagNumber(3)
  void clearTotalMemoryMb() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.double get qps => $_getN(3);
  @$pb.TagNumber(4)
  set qps($core.double value) => $_setDouble(3, value);
  @$pb.TagNumber(4)
  $core.bool hasQps() => $_has(3);
  @$pb.TagNumber(4)
  void clearQps() => $_clearField(4);
}

class DigestRequest extends $pb.GeneratedMessage {
  factory DigestRequest({
    $core.String? collection,
  }) {
    final result = create();
    if (collection != null) result.collection = collection;
    return result;
  }

  DigestRequest._();

  factory DigestRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory DigestRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'DigestRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'hyperspace'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'collection')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DigestRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DigestRequest copyWith(void Function(DigestRequest) updates) =>
      super.copyWith((message) => updates(message as DigestRequest))
          as DigestRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static DigestRequest create() => DigestRequest._();
  @$core.override
  DigestRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static DigestRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<DigestRequest>(create);
  static DigestRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get collection => $_getSZ(0);
  @$pb.TagNumber(1)
  set collection($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasCollection() => $_has(0);
  @$pb.TagNumber(1)
  void clearCollection() => $_clearField(1);
}

class DigestResponse extends $pb.GeneratedMessage {
  factory DigestResponse({
    $fixnum.Int64? logicalClock,
    $fixnum.Int64? stateHash,
    $core.Iterable<$fixnum.Int64>? buckets,
    $fixnum.Int64? count,
  }) {
    final result = create();
    if (logicalClock != null) result.logicalClock = logicalClock;
    if (stateHash != null) result.stateHash = stateHash;
    if (buckets != null) result.buckets.addAll(buckets);
    if (count != null) result.count = count;
    return result;
  }

  DigestResponse._();

  factory DigestResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory DigestResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'DigestResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'hyperspace'),
      createEmptyInstance: create)
    ..a<$fixnum.Int64>(
        1, _omitFieldNames ? '' : 'logicalClock', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..a<$fixnum.Int64>(
        2, _omitFieldNames ? '' : 'stateHash', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..p<$fixnum.Int64>(3, _omitFieldNames ? '' : 'buckets', $pb.PbFieldType.KU6)
    ..a<$fixnum.Int64>(4, _omitFieldNames ? '' : 'count', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DigestResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DigestResponse copyWith(void Function(DigestResponse) updates) =>
      super.copyWith((message) => updates(message as DigestResponse))
          as DigestResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static DigestResponse create() => DigestResponse._();
  @$core.override
  DigestResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static DigestResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<DigestResponse>(create);
  static DigestResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $fixnum.Int64 get logicalClock => $_getI64(0);
  @$pb.TagNumber(1)
  set logicalClock($fixnum.Int64 value) => $_setInt64(0, value);
  @$pb.TagNumber(1)
  $core.bool hasLogicalClock() => $_has(0);
  @$pb.TagNumber(1)
  void clearLogicalClock() => $_clearField(1);

  @$pb.TagNumber(2)
  $fixnum.Int64 get stateHash => $_getI64(1);
  @$pb.TagNumber(2)
  set stateHash($fixnum.Int64 value) => $_setInt64(1, value);
  @$pb.TagNumber(2)
  $core.bool hasStateHash() => $_has(1);
  @$pb.TagNumber(2)
  void clearStateHash() => $_clearField(2);

  @$pb.TagNumber(3)
  $pb.PbList<$fixnum.Int64> get buckets => $_getList(2);

  @$pb.TagNumber(4)
  $fixnum.Int64 get count => $_getI64(3);
  @$pb.TagNumber(4)
  set count($fixnum.Int64 value) => $_setInt64(3, value);
  @$pb.TagNumber(4)
  $core.bool hasCount() => $_has(3);
  @$pb.TagNumber(4)
  void clearCount() => $_clearField(4);
}

class SyncHandshakeRequest extends $pb.GeneratedMessage {
  factory SyncHandshakeRequest({
    $core.String? collection,
    $core.Iterable<$fixnum.Int64>? clientBuckets,
    $fixnum.Int64? clientLogicalClock,
    $fixnum.Int64? clientCount,
  }) {
    final result = create();
    if (collection != null) result.collection = collection;
    if (clientBuckets != null) result.clientBuckets.addAll(clientBuckets);
    if (clientLogicalClock != null)
      result.clientLogicalClock = clientLogicalClock;
    if (clientCount != null) result.clientCount = clientCount;
    return result;
  }

  SyncHandshakeRequest._();

  factory SyncHandshakeRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory SyncHandshakeRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'SyncHandshakeRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'hyperspace'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'collection')
    ..p<$fixnum.Int64>(
        2, _omitFieldNames ? '' : 'clientBuckets', $pb.PbFieldType.KU6)
    ..a<$fixnum.Int64>(
        3, _omitFieldNames ? '' : 'clientLogicalClock', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..a<$fixnum.Int64>(
        4, _omitFieldNames ? '' : 'clientCount', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SyncHandshakeRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SyncHandshakeRequest copyWith(void Function(SyncHandshakeRequest) updates) =>
      super.copyWith((message) => updates(message as SyncHandshakeRequest))
          as SyncHandshakeRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static SyncHandshakeRequest create() => SyncHandshakeRequest._();
  @$core.override
  SyncHandshakeRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static SyncHandshakeRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<SyncHandshakeRequest>(create);
  static SyncHandshakeRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get collection => $_getSZ(0);
  @$pb.TagNumber(1)
  set collection($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasCollection() => $_has(0);
  @$pb.TagNumber(1)
  void clearCollection() => $_clearField(1);

  /// Client's current bucket hashes (256 entries)
  @$pb.TagNumber(2)
  $pb.PbList<$fixnum.Int64> get clientBuckets => $_getList(1);

  @$pb.TagNumber(3)
  $fixnum.Int64 get clientLogicalClock => $_getI64(2);
  @$pb.TagNumber(3)
  set clientLogicalClock($fixnum.Int64 value) => $_setInt64(2, value);
  @$pb.TagNumber(3)
  $core.bool hasClientLogicalClock() => $_has(2);
  @$pb.TagNumber(3)
  void clearClientLogicalClock() => $_clearField(3);

  @$pb.TagNumber(4)
  $fixnum.Int64 get clientCount => $_getI64(3);
  @$pb.TagNumber(4)
  set clientCount($fixnum.Int64 value) => $_setInt64(3, value);
  @$pb.TagNumber(4)
  $core.bool hasClientCount() => $_has(3);
  @$pb.TagNumber(4)
  void clearClientCount() => $_clearField(4);
}

class DiffBucket extends $pb.GeneratedMessage {
  factory DiffBucket({
    $core.int? bucketIndex,
    $fixnum.Int64? serverHash,
    $fixnum.Int64? clientHash,
  }) {
    final result = create();
    if (bucketIndex != null) result.bucketIndex = bucketIndex;
    if (serverHash != null) result.serverHash = serverHash;
    if (clientHash != null) result.clientHash = clientHash;
    return result;
  }

  DiffBucket._();

  factory DiffBucket.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory DiffBucket.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'DiffBucket',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'hyperspace'),
      createEmptyInstance: create)
    ..aI(1, _omitFieldNames ? '' : 'bucketIndex',
        fieldType: $pb.PbFieldType.OU3)
    ..a<$fixnum.Int64>(
        2, _omitFieldNames ? '' : 'serverHash', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..a<$fixnum.Int64>(
        3, _omitFieldNames ? '' : 'clientHash', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DiffBucket clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DiffBucket copyWith(void Function(DiffBucket) updates) =>
      super.copyWith((message) => updates(message as DiffBucket)) as DiffBucket;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static DiffBucket create() => DiffBucket._();
  @$core.override
  DiffBucket createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static DiffBucket getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<DiffBucket>(create);
  static DiffBucket? _defaultInstance;

  @$pb.TagNumber(1)
  $core.int get bucketIndex => $_getIZ(0);
  @$pb.TagNumber(1)
  set bucketIndex($core.int value) => $_setUnsignedInt32(0, value);
  @$pb.TagNumber(1)
  $core.bool hasBucketIndex() => $_has(0);
  @$pb.TagNumber(1)
  void clearBucketIndex() => $_clearField(1);

  @$pb.TagNumber(2)
  $fixnum.Int64 get serverHash => $_getI64(1);
  @$pb.TagNumber(2)
  set serverHash($fixnum.Int64 value) => $_setInt64(1, value);
  @$pb.TagNumber(2)
  $core.bool hasServerHash() => $_has(1);
  @$pb.TagNumber(2)
  void clearServerHash() => $_clearField(2);

  @$pb.TagNumber(3)
  $fixnum.Int64 get clientHash => $_getI64(2);
  @$pb.TagNumber(3)
  set clientHash($fixnum.Int64 value) => $_setInt64(2, value);
  @$pb.TagNumber(3)
  $core.bool hasClientHash() => $_has(2);
  @$pb.TagNumber(3)
  void clearClientHash() => $_clearField(3);
}

class SyncHandshakeResponse extends $pb.GeneratedMessage {
  factory SyncHandshakeResponse({
    $core.Iterable<DiffBucket>? diffBuckets,
    $fixnum.Int64? serverLogicalClock,
    $fixnum.Int64? serverCount,
    $core.bool? inSync,
  }) {
    final result = create();
    if (diffBuckets != null) result.diffBuckets.addAll(diffBuckets);
    if (serverLogicalClock != null)
      result.serverLogicalClock = serverLogicalClock;
    if (serverCount != null) result.serverCount = serverCount;
    if (inSync != null) result.inSync = inSync;
    return result;
  }

  SyncHandshakeResponse._();

  factory SyncHandshakeResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory SyncHandshakeResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'SyncHandshakeResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'hyperspace'),
      createEmptyInstance: create)
    ..pPM<DiffBucket>(1, _omitFieldNames ? '' : 'diffBuckets',
        subBuilder: DiffBucket.create)
    ..a<$fixnum.Int64>(
        2, _omitFieldNames ? '' : 'serverLogicalClock', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..a<$fixnum.Int64>(
        3, _omitFieldNames ? '' : 'serverCount', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..aOB(4, _omitFieldNames ? '' : 'inSync')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SyncHandshakeResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SyncHandshakeResponse copyWith(
          void Function(SyncHandshakeResponse) updates) =>
      super.copyWith((message) => updates(message as SyncHandshakeResponse))
          as SyncHandshakeResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static SyncHandshakeResponse create() => SyncHandshakeResponse._();
  @$core.override
  SyncHandshakeResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static SyncHandshakeResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<SyncHandshakeResponse>(create);
  static SyncHandshakeResponse? _defaultInstance;

  /// Which buckets differ between client and server
  @$pb.TagNumber(1)
  $pb.PbList<DiffBucket> get diffBuckets => $_getList(0);

  /// Server's current logical clock
  @$pb.TagNumber(2)
  $fixnum.Int64 get serverLogicalClock => $_getI64(1);
  @$pb.TagNumber(2)
  set serverLogicalClock($fixnum.Int64 value) => $_setInt64(1, value);
  @$pb.TagNumber(2)
  $core.bool hasServerLogicalClock() => $_has(1);
  @$pb.TagNumber(2)
  void clearServerLogicalClock() => $_clearField(2);

  /// Server's total vector count
  @$pb.TagNumber(3)
  $fixnum.Int64 get serverCount => $_getI64(2);
  @$pb.TagNumber(3)
  set serverCount($fixnum.Int64 value) => $_setInt64(2, value);
  @$pb.TagNumber(3)
  $core.bool hasServerCount() => $_has(2);
  @$pb.TagNumber(3)
  void clearServerCount() => $_clearField(3);

  /// True if digests match perfectly (no sync needed)
  @$pb.TagNumber(4)
  $core.bool get inSync => $_getBF(3);
  @$pb.TagNumber(4)
  set inSync($core.bool value) => $_setBool(3, value);
  @$pb.TagNumber(4)
  $core.bool hasInSync() => $_has(3);
  @$pb.TagNumber(4)
  void clearInSync() => $_clearField(4);
}

class SyncPullRequest extends $pb.GeneratedMessage {
  factory SyncPullRequest({
    $core.String? collection,
    $core.Iterable<$core.int>? bucketIndices,
  }) {
    final result = create();
    if (collection != null) result.collection = collection;
    if (bucketIndices != null) result.bucketIndices.addAll(bucketIndices);
    return result;
  }

  SyncPullRequest._();

  factory SyncPullRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory SyncPullRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'SyncPullRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'hyperspace'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'collection')
    ..p<$core.int>(
        2, _omitFieldNames ? '' : 'bucketIndices', $pb.PbFieldType.KU3)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SyncPullRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SyncPullRequest copyWith(void Function(SyncPullRequest) updates) =>
      super.copyWith((message) => updates(message as SyncPullRequest))
          as SyncPullRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static SyncPullRequest create() => SyncPullRequest._();
  @$core.override
  SyncPullRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static SyncPullRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<SyncPullRequest>(create);
  static SyncPullRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get collection => $_getSZ(0);
  @$pb.TagNumber(1)
  set collection($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasCollection() => $_has(0);
  @$pb.TagNumber(1)
  void clearCollection() => $_clearField(1);

  /// Which bucket indices to pull vectors from
  @$pb.TagNumber(2)
  $pb.PbList<$core.int> get bucketIndices => $_getList(1);
}

/// A single vector transferred during sync
class SyncVectorData extends $pb.GeneratedMessage {
  factory SyncVectorData({
    $core.String? collection,
    $core.int? id,
    $core.Iterable<$core.double>? vector,
    $core.Iterable<$core.MapEntry<$core.String, $core.String>>? metadata,
    $core.int? bucketIndex,
  }) {
    final result = create();
    if (collection != null) result.collection = collection;
    if (id != null) result.id = id;
    if (vector != null) result.vector.addAll(vector);
    if (metadata != null) result.metadata.addEntries(metadata);
    if (bucketIndex != null) result.bucketIndex = bucketIndex;
    return result;
  }

  SyncVectorData._();

  factory SyncVectorData.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory SyncVectorData.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'SyncVectorData',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'hyperspace'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'collection')
    ..aI(2, _omitFieldNames ? '' : 'id', fieldType: $pb.PbFieldType.OU3)
    ..p<$core.double>(3, _omitFieldNames ? '' : 'vector', $pb.PbFieldType.KD)
    ..m<$core.String, $core.String>(4, _omitFieldNames ? '' : 'metadata',
        entryClassName: 'SyncVectorData.MetadataEntry',
        keyFieldType: $pb.PbFieldType.OS,
        valueFieldType: $pb.PbFieldType.OS,
        packageName: const $pb.PackageName('hyperspace'))
    ..aI(5, _omitFieldNames ? '' : 'bucketIndex',
        fieldType: $pb.PbFieldType.OU3)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SyncVectorData clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SyncVectorData copyWith(void Function(SyncVectorData) updates) =>
      super.copyWith((message) => updates(message as SyncVectorData))
          as SyncVectorData;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static SyncVectorData create() => SyncVectorData._();
  @$core.override
  SyncVectorData createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static SyncVectorData getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<SyncVectorData>(create);
  static SyncVectorData? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get collection => $_getSZ(0);
  @$pb.TagNumber(1)
  set collection($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasCollection() => $_has(0);
  @$pb.TagNumber(1)
  void clearCollection() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.int get id => $_getIZ(1);
  @$pb.TagNumber(2)
  set id($core.int value) => $_setUnsignedInt32(1, value);
  @$pb.TagNumber(2)
  $core.bool hasId() => $_has(1);
  @$pb.TagNumber(2)
  void clearId() => $_clearField(2);

  @$pb.TagNumber(3)
  $pb.PbList<$core.double> get vector => $_getList(2);

  @$pb.TagNumber(4)
  $pb.PbMap<$core.String, $core.String> get metadata => $_getMap(3);

  @$pb.TagNumber(5)
  $core.int get bucketIndex => $_getIZ(4);
  @$pb.TagNumber(5)
  set bucketIndex($core.int value) => $_setUnsignedInt32(4, value);
  @$pb.TagNumber(5)
  $core.bool hasBucketIndex() => $_has(4);
  @$pb.TagNumber(5)
  void clearBucketIndex() => $_clearField(5);
}

class SyncPushResponse extends $pb.GeneratedMessage {
  factory SyncPushResponse({
    $core.int? accepted,
    $core.int? rejected,
    $core.int? duplicates,
  }) {
    final result = create();
    if (accepted != null) result.accepted = accepted;
    if (rejected != null) result.rejected = rejected;
    if (duplicates != null) result.duplicates = duplicates;
    return result;
  }

  SyncPushResponse._();

  factory SyncPushResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory SyncPushResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'SyncPushResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'hyperspace'),
      createEmptyInstance: create)
    ..aI(1, _omitFieldNames ? '' : 'accepted', fieldType: $pb.PbFieldType.OU3)
    ..aI(2, _omitFieldNames ? '' : 'rejected', fieldType: $pb.PbFieldType.OU3)
    ..aI(3, _omitFieldNames ? '' : 'duplicates', fieldType: $pb.PbFieldType.OU3)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SyncPushResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SyncPushResponse copyWith(void Function(SyncPushResponse) updates) =>
      super.copyWith((message) => updates(message as SyncPushResponse))
          as SyncPushResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static SyncPushResponse create() => SyncPushResponse._();
  @$core.override
  SyncPushResponse createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static SyncPushResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<SyncPushResponse>(create);
  static SyncPushResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $core.int get accepted => $_getIZ(0);
  @$pb.TagNumber(1)
  set accepted($core.int value) => $_setUnsignedInt32(0, value);
  @$pb.TagNumber(1)
  $core.bool hasAccepted() => $_has(0);
  @$pb.TagNumber(1)
  void clearAccepted() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.int get rejected => $_getIZ(1);
  @$pb.TagNumber(2)
  set rejected($core.int value) => $_setUnsignedInt32(1, value);
  @$pb.TagNumber(2)
  $core.bool hasRejected() => $_has(1);
  @$pb.TagNumber(2)
  void clearRejected() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.int get duplicates => $_getIZ(2);
  @$pb.TagNumber(3)
  set duplicates($core.int value) => $_setUnsignedInt32(2, value);
  @$pb.TagNumber(3)
  $core.bool hasDuplicates() => $_has(2);
  @$pb.TagNumber(3)
  void clearDuplicates() => $_clearField(3);
}

class FreezeCollectionRequest extends $pb.GeneratedMessage {
  factory FreezeCollectionRequest({
    $core.String? name,
  }) {
    final result = create();
    if (name != null) result.name = name;
    return result;
  }

  FreezeCollectionRequest._();

  factory FreezeCollectionRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory FreezeCollectionRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'FreezeCollectionRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'hyperspace'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'name')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  FreezeCollectionRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  FreezeCollectionRequest copyWith(
          void Function(FreezeCollectionRequest) updates) =>
      super.copyWith((message) => updates(message as FreezeCollectionRequest))
          as FreezeCollectionRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static FreezeCollectionRequest create() => FreezeCollectionRequest._();
  @$core.override
  FreezeCollectionRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static FreezeCollectionRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<FreezeCollectionRequest>(create);
  static FreezeCollectionRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get name => $_getSZ(0);
  @$pb.TagNumber(1)
  set name($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasName() => $_has(0);
  @$pb.TagNumber(1)
  void clearName() => $_clearField(1);
}

class UnfreezeCollectionRequest extends $pb.GeneratedMessage {
  factory UnfreezeCollectionRequest({
    $core.String? name,
  }) {
    final result = create();
    if (name != null) result.name = name;
    return result;
  }

  UnfreezeCollectionRequest._();

  factory UnfreezeCollectionRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromBuffer(data, registry);
  factory UnfreezeCollectionRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      create()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'UnfreezeCollectionRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'hyperspace'),
      createEmptyInstance: create)
    ..aOS(1, _omitFieldNames ? '' : 'name')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UnfreezeCollectionRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UnfreezeCollectionRequest copyWith(
          void Function(UnfreezeCollectionRequest) updates) =>
      super.copyWith((message) => updates(message as UnfreezeCollectionRequest))
          as UnfreezeCollectionRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  static UnfreezeCollectionRequest create() => UnfreezeCollectionRequest._();
  @$core.override
  UnfreezeCollectionRequest createEmptyInstance() => create();
  @$core.pragma('dart2js:noInline')
  static UnfreezeCollectionRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<UnfreezeCollectionRequest>(create);
  static UnfreezeCollectionRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get name => $_getSZ(0);
  @$pb.TagNumber(1)
  set name($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasName() => $_has(0);
  @$pb.TagNumber(1)
  void clearName() => $_clearField(1);
}

const $core.bool _omitFieldNames =
    $core.bool.fromEnvironment('protobuf.omit_field_names');
const $core.bool _omitMessageNames =
    $core.bool.fromEnvironment('protobuf.omit_message_names');
