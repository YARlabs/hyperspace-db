///
//  Generated code. Do not modify.
//  source: hyperspace.proto
//
// @dart = 2.12
// ignore_for_file: annotate_overrides,camel_case_types,constant_identifier_names,directives_ordering,library_prefixes,non_constant_identifier_names,prefer_final_fields,return_of_invalid_type,unnecessary_const,unnecessary_import,unnecessary_this,unused_import,unused_shown_name

import 'dart:core' as $core;

import 'package:fixnum/fixnum.dart' as $fixnum;
import 'package:protobuf/protobuf.dart' as $pb;

import 'hyperspace.pbenum.dart';

export 'hyperspace.pbenum.dart';

class ReplicationRequest extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'ReplicationRequest', package: const $pb.PackageName(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'hyperspace'), createEmptyInstance: create)
    ..a<$fixnum.Int64>(1, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'lastLogicalClock', $pb.PbFieldType.OU6, defaultOrMaker: $fixnum.Int64.ZERO)
    ..hasRequiredFields = false
  ;

  ReplicationRequest._() : super();
  factory ReplicationRequest({
    $fixnum.Int64? lastLogicalClock,
  }) {
    final _result = create();
    if (lastLogicalClock != null) {
      _result.lastLogicalClock = lastLogicalClock;
    }
    return _result;
  }
  factory ReplicationRequest.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory ReplicationRequest.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  ReplicationRequest clone() => ReplicationRequest()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  ReplicationRequest copyWith(void Function(ReplicationRequest) updates) => super.copyWith((message) => updates(message as ReplicationRequest)) as ReplicationRequest; // ignore: deprecated_member_use
  $pb.BuilderInfo get info_ => _i;
  @$core.pragma('dart2js:noInline')
  static ReplicationRequest create() => ReplicationRequest._();
  ReplicationRequest createEmptyInstance() => create();
  static $pb.PbList<ReplicationRequest> createRepeated() => $pb.PbList<ReplicationRequest>();
  @$core.pragma('dart2js:noInline')
  static ReplicationRequest getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<ReplicationRequest>(create);
  static ReplicationRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $fixnum.Int64 get lastLogicalClock => $_getI64(0);
  @$pb.TagNumber(1)
  set lastLogicalClock($fixnum.Int64 v) { $_setInt64(0, v); }
  @$pb.TagNumber(1)
  $core.bool hasLastLogicalClock() => $_has(0);
  @$pb.TagNumber(1)
  void clearLastLogicalClock() => clearField(1);
}

enum ReplicationLog_Operation {
  insert, 
  createCollection, 
  deleteCollection, 
  delete, 
  notSet
}

class ReplicationLog extends $pb.GeneratedMessage {
  static const $core.Map<$core.int, ReplicationLog_Operation> _ReplicationLog_OperationByTag = {
    4 : ReplicationLog_Operation.insert,
    5 : ReplicationLog_Operation.createCollection,
    6 : ReplicationLog_Operation.deleteCollection,
    7 : ReplicationLog_Operation.delete,
    0 : ReplicationLog_Operation.notSet
  };
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'ReplicationLog', package: const $pb.PackageName(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'hyperspace'), createEmptyInstance: create)
    ..oo(0, [4, 5, 6, 7])
    ..a<$fixnum.Int64>(1, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'logicalClock', $pb.PbFieldType.OU6, defaultOrMaker: $fixnum.Int64.ZERO)
    ..aOS(2, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'originNodeId')
    ..aOS(3, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'collection')
    ..aOM<InsertOp>(4, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'insert', subBuilder: InsertOp.create)
    ..aOM<CreateCollectionOp>(5, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'createCollection', subBuilder: CreateCollectionOp.create)
    ..aOM<DeleteCollectionOp>(6, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'deleteCollection', subBuilder: DeleteCollectionOp.create)
    ..aOM<DeleteOp>(7, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'delete', subBuilder: DeleteOp.create)
    ..hasRequiredFields = false
  ;

  ReplicationLog._() : super();
  factory ReplicationLog({
    $fixnum.Int64? logicalClock,
    $core.String? originNodeId,
    $core.String? collection,
    InsertOp? insert,
    CreateCollectionOp? createCollection,
    DeleteCollectionOp? deleteCollection,
    DeleteOp? delete,
  }) {
    final _result = create();
    if (logicalClock != null) {
      _result.logicalClock = logicalClock;
    }
    if (originNodeId != null) {
      _result.originNodeId = originNodeId;
    }
    if (collection != null) {
      _result.collection = collection;
    }
    if (insert != null) {
      _result.insert = insert;
    }
    if (createCollection != null) {
      _result.createCollection = createCollection;
    }
    if (deleteCollection != null) {
      _result.deleteCollection = deleteCollection;
    }
    if (delete != null) {
      _result.delete = delete;
    }
    return _result;
  }
  factory ReplicationLog.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory ReplicationLog.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  ReplicationLog clone() => ReplicationLog()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  ReplicationLog copyWith(void Function(ReplicationLog) updates) => super.copyWith((message) => updates(message as ReplicationLog)) as ReplicationLog; // ignore: deprecated_member_use
  $pb.BuilderInfo get info_ => _i;
  @$core.pragma('dart2js:noInline')
  static ReplicationLog create() => ReplicationLog._();
  ReplicationLog createEmptyInstance() => create();
  static $pb.PbList<ReplicationLog> createRepeated() => $pb.PbList<ReplicationLog>();
  @$core.pragma('dart2js:noInline')
  static ReplicationLog getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<ReplicationLog>(create);
  static ReplicationLog? _defaultInstance;

  ReplicationLog_Operation whichOperation() => _ReplicationLog_OperationByTag[$_whichOneof(0)]!;
  void clearOperation() => clearField($_whichOneof(0));

  @$pb.TagNumber(1)
  $fixnum.Int64 get logicalClock => $_getI64(0);
  @$pb.TagNumber(1)
  set logicalClock($fixnum.Int64 v) { $_setInt64(0, v); }
  @$pb.TagNumber(1)
  $core.bool hasLogicalClock() => $_has(0);
  @$pb.TagNumber(1)
  void clearLogicalClock() => clearField(1);

  @$pb.TagNumber(2)
  $core.String get originNodeId => $_getSZ(1);
  @$pb.TagNumber(2)
  set originNodeId($core.String v) { $_setString(1, v); }
  @$pb.TagNumber(2)
  $core.bool hasOriginNodeId() => $_has(1);
  @$pb.TagNumber(2)
  void clearOriginNodeId() => clearField(2);

  @$pb.TagNumber(3)
  $core.String get collection => $_getSZ(2);
  @$pb.TagNumber(3)
  set collection($core.String v) { $_setString(2, v); }
  @$pb.TagNumber(3)
  $core.bool hasCollection() => $_has(2);
  @$pb.TagNumber(3)
  void clearCollection() => clearField(3);

  @$pb.TagNumber(4)
  InsertOp get insert => $_getN(3);
  @$pb.TagNumber(4)
  set insert(InsertOp v) { setField(4, v); }
  @$pb.TagNumber(4)
  $core.bool hasInsert() => $_has(3);
  @$pb.TagNumber(4)
  void clearInsert() => clearField(4);
  @$pb.TagNumber(4)
  InsertOp ensureInsert() => $_ensure(3);

  @$pb.TagNumber(5)
  CreateCollectionOp get createCollection => $_getN(4);
  @$pb.TagNumber(5)
  set createCollection(CreateCollectionOp v) { setField(5, v); }
  @$pb.TagNumber(5)
  $core.bool hasCreateCollection() => $_has(4);
  @$pb.TagNumber(5)
  void clearCreateCollection() => clearField(5);
  @$pb.TagNumber(5)
  CreateCollectionOp ensureCreateCollection() => $_ensure(4);

  @$pb.TagNumber(6)
  DeleteCollectionOp get deleteCollection => $_getN(5);
  @$pb.TagNumber(6)
  set deleteCollection(DeleteCollectionOp v) { setField(6, v); }
  @$pb.TagNumber(6)
  $core.bool hasDeleteCollection() => $_has(5);
  @$pb.TagNumber(6)
  void clearDeleteCollection() => clearField(6);
  @$pb.TagNumber(6)
  DeleteCollectionOp ensureDeleteCollection() => $_ensure(5);

  @$pb.TagNumber(7)
  DeleteOp get delete => $_getN(6);
  @$pb.TagNumber(7)
  set delete(DeleteOp v) { setField(7, v); }
  @$pb.TagNumber(7)
  $core.bool hasDelete() => $_has(6);
  @$pb.TagNumber(7)
  void clearDelete() => clearField(7);
  @$pb.TagNumber(7)
  DeleteOp ensureDelete() => $_ensure(6);
}

class InsertOp extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'InsertOp', package: const $pb.PackageName(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'hyperspace'), createEmptyInstance: create)
    ..a<$core.int>(1, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'id', $pb.PbFieldType.OU3)
    ..p<$core.double>(2, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'vector', $pb.PbFieldType.KD)
    ..m<$core.String, $core.String>(3, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'metadata', entryClassName: 'InsertOp.MetadataEntry', keyFieldType: $pb.PbFieldType.OS, valueFieldType: $pb.PbFieldType.OS, packageName: const $pb.PackageName('hyperspace'))
    ..m<$core.String, MetadataValue>(4, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'typedMetadata', entryClassName: 'InsertOp.TypedMetadataEntry', keyFieldType: $pb.PbFieldType.OS, valueFieldType: $pb.PbFieldType.OM, valueCreator: MetadataValue.create, packageName: const $pb.PackageName('hyperspace'))
    ..hasRequiredFields = false
  ;

  InsertOp._() : super();
  factory InsertOp({
    $core.int? id,
    $core.Iterable<$core.double>? vector,
    $core.Map<$core.String, $core.String>? metadata,
    $core.Map<$core.String, MetadataValue>? typedMetadata,
  }) {
    final _result = create();
    if (id != null) {
      _result.id = id;
    }
    if (vector != null) {
      _result.vector.addAll(vector);
    }
    if (metadata != null) {
      _result.metadata.addAll(metadata);
    }
    if (typedMetadata != null) {
      _result.typedMetadata.addAll(typedMetadata);
    }
    return _result;
  }
  factory InsertOp.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory InsertOp.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  InsertOp clone() => InsertOp()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  InsertOp copyWith(void Function(InsertOp) updates) => super.copyWith((message) => updates(message as InsertOp)) as InsertOp; // ignore: deprecated_member_use
  $pb.BuilderInfo get info_ => _i;
  @$core.pragma('dart2js:noInline')
  static InsertOp create() => InsertOp._();
  InsertOp createEmptyInstance() => create();
  static $pb.PbList<InsertOp> createRepeated() => $pb.PbList<InsertOp>();
  @$core.pragma('dart2js:noInline')
  static InsertOp getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<InsertOp>(create);
  static InsertOp? _defaultInstance;

  @$pb.TagNumber(1)
  $core.int get id => $_getIZ(0);
  @$pb.TagNumber(1)
  set id($core.int v) { $_setUnsignedInt32(0, v); }
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => clearField(1);

  @$pb.TagNumber(2)
  $core.List<$core.double> get vector => $_getList(1);

  @$pb.TagNumber(3)
  $core.Map<$core.String, $core.String> get metadata => $_getMap(2);

  @$pb.TagNumber(4)
  $core.Map<$core.String, MetadataValue> get typedMetadata => $_getMap(3);
}

class CreateCollectionOp extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'CreateCollectionOp', package: const $pb.PackageName(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'hyperspace'), createEmptyInstance: create)
    ..aOM<CollectionSchema>(3, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'schema', subBuilder: CollectionSchema.create)
    ..hasRequiredFields = false
  ;

  CreateCollectionOp._() : super();
  factory CreateCollectionOp({
    CollectionSchema? schema,
  }) {
    final _result = create();
    if (schema != null) {
      _result.schema = schema;
    }
    return _result;
  }
  factory CreateCollectionOp.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory CreateCollectionOp.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  CreateCollectionOp clone() => CreateCollectionOp()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  CreateCollectionOp copyWith(void Function(CreateCollectionOp) updates) => super.copyWith((message) => updates(message as CreateCollectionOp)) as CreateCollectionOp; // ignore: deprecated_member_use
  $pb.BuilderInfo get info_ => _i;
  @$core.pragma('dart2js:noInline')
  static CreateCollectionOp create() => CreateCollectionOp._();
  CreateCollectionOp createEmptyInstance() => create();
  static $pb.PbList<CreateCollectionOp> createRepeated() => $pb.PbList<CreateCollectionOp>();
  @$core.pragma('dart2js:noInline')
  static CreateCollectionOp getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<CreateCollectionOp>(create);
  static CreateCollectionOp? _defaultInstance;

  @$pb.TagNumber(3)
  CollectionSchema get schema => $_getN(0);
  @$pb.TagNumber(3)
  set schema(CollectionSchema v) { setField(3, v); }
  @$pb.TagNumber(3)
  $core.bool hasSchema() => $_has(0);
  @$pb.TagNumber(3)
  void clearSchema() => clearField(3);
  @$pb.TagNumber(3)
  CollectionSchema ensureSchema() => $_ensure(0);
}

class DeleteCollectionOp extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'DeleteCollectionOp', package: const $pb.PackageName(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'hyperspace'), createEmptyInstance: create)
    ..hasRequiredFields = false
  ;

  DeleteCollectionOp._() : super();
  factory DeleteCollectionOp() => create();
  factory DeleteCollectionOp.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory DeleteCollectionOp.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  DeleteCollectionOp clone() => DeleteCollectionOp()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  DeleteCollectionOp copyWith(void Function(DeleteCollectionOp) updates) => super.copyWith((message) => updates(message as DeleteCollectionOp)) as DeleteCollectionOp; // ignore: deprecated_member_use
  $pb.BuilderInfo get info_ => _i;
  @$core.pragma('dart2js:noInline')
  static DeleteCollectionOp create() => DeleteCollectionOp._();
  DeleteCollectionOp createEmptyInstance() => create();
  static $pb.PbList<DeleteCollectionOp> createRepeated() => $pb.PbList<DeleteCollectionOp>();
  @$core.pragma('dart2js:noInline')
  static DeleteCollectionOp getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<DeleteCollectionOp>(create);
  static DeleteCollectionOp? _defaultInstance;
}

class DeleteOp extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'DeleteOp', package: const $pb.PackageName(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'hyperspace'), createEmptyInstance: create)
    ..a<$core.int>(1, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'id', $pb.PbFieldType.OU3)
    ..hasRequiredFields = false
  ;

  DeleteOp._() : super();
  factory DeleteOp({
    $core.int? id,
  }) {
    final _result = create();
    if (id != null) {
      _result.id = id;
    }
    return _result;
  }
  factory DeleteOp.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory DeleteOp.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  DeleteOp clone() => DeleteOp()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  DeleteOp copyWith(void Function(DeleteOp) updates) => super.copyWith((message) => updates(message as DeleteOp)) as DeleteOp; // ignore: deprecated_member_use
  $pb.BuilderInfo get info_ => _i;
  @$core.pragma('dart2js:noInline')
  static DeleteOp create() => DeleteOp._();
  DeleteOp createEmptyInstance() => create();
  static $pb.PbList<DeleteOp> createRepeated() => $pb.PbList<DeleteOp>();
  @$core.pragma('dart2js:noInline')
  static DeleteOp getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<DeleteOp>(create);
  static DeleteOp? _defaultInstance;

  @$pb.TagNumber(1)
  $core.int get id => $_getIZ(0);
  @$pb.TagNumber(1)
  set id($core.int v) { $_setUnsignedInt32(0, v); }
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => clearField(1);
}

class QuantizationConfig extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'QuantizationConfig', package: const $pb.PackageName(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'hyperspace'), createEmptyInstance: create)
    ..e<QuantizationMode>(1, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'mode', $pb.PbFieldType.OE, defaultOrMaker: QuantizationMode.NONE, valueOf: QuantizationMode.valueOf, enumValues: QuantizationMode.values)
    ..hasRequiredFields = false
  ;

  QuantizationConfig._() : super();
  factory QuantizationConfig({
    QuantizationMode? mode,
  }) {
    final _result = create();
    if (mode != null) {
      _result.mode = mode;
    }
    return _result;
  }
  factory QuantizationConfig.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory QuantizationConfig.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  QuantizationConfig clone() => QuantizationConfig()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  QuantizationConfig copyWith(void Function(QuantizationConfig) updates) => super.copyWith((message) => updates(message as QuantizationConfig)) as QuantizationConfig; // ignore: deprecated_member_use
  $pb.BuilderInfo get info_ => _i;
  @$core.pragma('dart2js:noInline')
  static QuantizationConfig create() => QuantizationConfig._();
  QuantizationConfig createEmptyInstance() => create();
  static $pb.PbList<QuantizationConfig> createRepeated() => $pb.PbList<QuantizationConfig>();
  @$core.pragma('dart2js:noInline')
  static QuantizationConfig getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<QuantizationConfig>(create);
  static QuantizationConfig? _defaultInstance;

  @$pb.TagNumber(1)
  QuantizationMode get mode => $_getN(0);
  @$pb.TagNumber(1)
  set mode(QuantizationMode v) { setField(1, v); }
  @$pb.TagNumber(1)
  $core.bool hasMode() => $_has(0);
  @$pb.TagNumber(1)
  void clearMode() => clearField(1);
}

class CreateCollectionRequest extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'CreateCollectionRequest', package: const $pb.PackageName(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'hyperspace'), createEmptyInstance: create)
    ..aOS(1, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'name')
    ..aOM<CollectionSchema>(5, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'schema', subBuilder: CollectionSchema.create)
    ..hasRequiredFields = false
  ;

  CreateCollectionRequest._() : super();
  factory CreateCollectionRequest({
    $core.String? name,
    CollectionSchema? schema,
  }) {
    final _result = create();
    if (name != null) {
      _result.name = name;
    }
    if (schema != null) {
      _result.schema = schema;
    }
    return _result;
  }
  factory CreateCollectionRequest.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory CreateCollectionRequest.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  CreateCollectionRequest clone() => CreateCollectionRequest()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  CreateCollectionRequest copyWith(void Function(CreateCollectionRequest) updates) => super.copyWith((message) => updates(message as CreateCollectionRequest)) as CreateCollectionRequest; // ignore: deprecated_member_use
  $pb.BuilderInfo get info_ => _i;
  @$core.pragma('dart2js:noInline')
  static CreateCollectionRequest create() => CreateCollectionRequest._();
  CreateCollectionRequest createEmptyInstance() => create();
  static $pb.PbList<CreateCollectionRequest> createRepeated() => $pb.PbList<CreateCollectionRequest>();
  @$core.pragma('dart2js:noInline')
  static CreateCollectionRequest getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<CreateCollectionRequest>(create);
  static CreateCollectionRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get name => $_getSZ(0);
  @$pb.TagNumber(1)
  set name($core.String v) { $_setString(0, v); }
  @$pb.TagNumber(1)
  $core.bool hasName() => $_has(0);
  @$pb.TagNumber(1)
  void clearName() => clearField(1);

  @$pb.TagNumber(5)
  CollectionSchema get schema => $_getN(1);
  @$pb.TagNumber(5)
  set schema(CollectionSchema v) { setField(5, v); }
  @$pb.TagNumber(5)
  $core.bool hasSchema() => $_has(1);
  @$pb.TagNumber(5)
  void clearSchema() => clearField(5);
  @$pb.TagNumber(5)
  CollectionSchema ensureSchema() => $_ensure(1);
}

class VectorComponent extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'VectorComponent', package: const $pb.PackageName(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'hyperspace'), createEmptyInstance: create)
    ..aOS(1, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'name')
    ..aOS(2, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'metric')
    ..a<$core.int>(3, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'fullDimension', $pb.PbFieldType.OU3)
    ..a<$core.double>(4, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'weight', $pb.PbFieldType.OF)
    ..hasRequiredFields = false
  ;

  VectorComponent._() : super();
  factory VectorComponent({
    $core.String? name,
    $core.String? metric,
    $core.int? fullDimension,
    $core.double? weight,
  }) {
    final _result = create();
    if (name != null) {
      _result.name = name;
    }
    if (metric != null) {
      _result.metric = metric;
    }
    if (fullDimension != null) {
      _result.fullDimension = fullDimension;
    }
    if (weight != null) {
      _result.weight = weight;
    }
    return _result;
  }
  factory VectorComponent.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory VectorComponent.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  VectorComponent clone() => VectorComponent()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  VectorComponent copyWith(void Function(VectorComponent) updates) => super.copyWith((message) => updates(message as VectorComponent)) as VectorComponent; // ignore: deprecated_member_use
  $pb.BuilderInfo get info_ => _i;
  @$core.pragma('dart2js:noInline')
  static VectorComponent create() => VectorComponent._();
  VectorComponent createEmptyInstance() => create();
  static $pb.PbList<VectorComponent> createRepeated() => $pb.PbList<VectorComponent>();
  @$core.pragma('dart2js:noInline')
  static VectorComponent getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<VectorComponent>(create);
  static VectorComponent? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get name => $_getSZ(0);
  @$pb.TagNumber(1)
  set name($core.String v) { $_setString(0, v); }
  @$pb.TagNumber(1)
  $core.bool hasName() => $_has(0);
  @$pb.TagNumber(1)
  void clearName() => clearField(1);

  @$pb.TagNumber(2)
  $core.String get metric => $_getSZ(1);
  @$pb.TagNumber(2)
  set metric($core.String v) { $_setString(1, v); }
  @$pb.TagNumber(2)
  $core.bool hasMetric() => $_has(1);
  @$pb.TagNumber(2)
  void clearMetric() => clearField(2);

  @$pb.TagNumber(3)
  $core.int get fullDimension => $_getIZ(2);
  @$pb.TagNumber(3)
  set fullDimension($core.int v) { $_setUnsignedInt32(2, v); }
  @$pb.TagNumber(3)
  $core.bool hasFullDimension() => $_has(2);
  @$pb.TagNumber(3)
  void clearFullDimension() => clearField(3);

  @$pb.TagNumber(4)
  $core.double get weight => $_getN(3);
  @$pb.TagNumber(4)
  set weight($core.double v) { $_setFloat(3, v); }
  @$pb.TagNumber(4)
  $core.bool hasWeight() => $_has(3);
  @$pb.TagNumber(4)
  void clearWeight() => clearField(4);
}

class MrlLayer extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'MrlLayer', package: const $pb.PackageName(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'hyperspace'), createEmptyInstance: create)
    ..aOS(1, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'componentName')
    ..a<$core.int>(2, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'cutoffDimension', $pb.PbFieldType.OU3)
    ..aOB(3, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'storeInRam')
    ..a<$core.int>(4, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'rerankTopK', $pb.PbFieldType.OU3)
    ..hasRequiredFields = false
  ;

  MrlLayer._() : super();
  factory MrlLayer({
    $core.String? componentName,
    $core.int? cutoffDimension,
    $core.bool? storeInRam,
    $core.int? rerankTopK,
  }) {
    final _result = create();
    if (componentName != null) {
      _result.componentName = componentName;
    }
    if (cutoffDimension != null) {
      _result.cutoffDimension = cutoffDimension;
    }
    if (storeInRam != null) {
      _result.storeInRam = storeInRam;
    }
    if (rerankTopK != null) {
      _result.rerankTopK = rerankTopK;
    }
    return _result;
  }
  factory MrlLayer.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory MrlLayer.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  MrlLayer clone() => MrlLayer()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  MrlLayer copyWith(void Function(MrlLayer) updates) => super.copyWith((message) => updates(message as MrlLayer)) as MrlLayer; // ignore: deprecated_member_use
  $pb.BuilderInfo get info_ => _i;
  @$core.pragma('dart2js:noInline')
  static MrlLayer create() => MrlLayer._();
  MrlLayer createEmptyInstance() => create();
  static $pb.PbList<MrlLayer> createRepeated() => $pb.PbList<MrlLayer>();
  @$core.pragma('dart2js:noInline')
  static MrlLayer getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<MrlLayer>(create);
  static MrlLayer? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get componentName => $_getSZ(0);
  @$pb.TagNumber(1)
  set componentName($core.String v) { $_setString(0, v); }
  @$pb.TagNumber(1)
  $core.bool hasComponentName() => $_has(0);
  @$pb.TagNumber(1)
  void clearComponentName() => clearField(1);

  @$pb.TagNumber(2)
  $core.int get cutoffDimension => $_getIZ(1);
  @$pb.TagNumber(2)
  set cutoffDimension($core.int v) { $_setUnsignedInt32(1, v); }
  @$pb.TagNumber(2)
  $core.bool hasCutoffDimension() => $_has(1);
  @$pb.TagNumber(2)
  void clearCutoffDimension() => clearField(2);

  @$pb.TagNumber(3)
  $core.bool get storeInRam => $_getBF(2);
  @$pb.TagNumber(3)
  set storeInRam($core.bool v) { $_setBool(2, v); }
  @$pb.TagNumber(3)
  $core.bool hasStoreInRam() => $_has(2);
  @$pb.TagNumber(3)
  void clearStoreInRam() => clearField(3);

  @$pb.TagNumber(4)
  $core.int get rerankTopK => $_getIZ(3);
  @$pb.TagNumber(4)
  set rerankTopK($core.int v) { $_setUnsignedInt32(3, v); }
  @$pb.TagNumber(4)
  $core.bool hasRerankTopK() => $_has(3);
  @$pb.TagNumber(4)
  void clearRerankTopK() => clearField(4);
}

class CollectionSchema extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'CollectionSchema', package: const $pb.PackageName(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'hyperspace'), createEmptyInstance: create)
    ..pc<VectorComponent>(1, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'components', $pb.PbFieldType.PM, subBuilder: VectorComponent.create)
    ..pc<MrlLayer>(2, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'cascadePipeline', $pb.PbFieldType.PM, subBuilder: MrlLayer.create)
    ..hasRequiredFields = false
  ;

  CollectionSchema._() : super();
  factory CollectionSchema({
    $core.Iterable<VectorComponent>? components,
    $core.Iterable<MrlLayer>? cascadePipeline,
  }) {
    final _result = create();
    if (components != null) {
      _result.components.addAll(components);
    }
    if (cascadePipeline != null) {
      _result.cascadePipeline.addAll(cascadePipeline);
    }
    return _result;
  }
  factory CollectionSchema.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory CollectionSchema.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  CollectionSchema clone() => CollectionSchema()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  CollectionSchema copyWith(void Function(CollectionSchema) updates) => super.copyWith((message) => updates(message as CollectionSchema)) as CollectionSchema; // ignore: deprecated_member_use
  $pb.BuilderInfo get info_ => _i;
  @$core.pragma('dart2js:noInline')
  static CollectionSchema create() => CollectionSchema._();
  CollectionSchema createEmptyInstance() => create();
  static $pb.PbList<CollectionSchema> createRepeated() => $pb.PbList<CollectionSchema>();
  @$core.pragma('dart2js:noInline')
  static CollectionSchema getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<CollectionSchema>(create);
  static CollectionSchema? _defaultInstance;

  @$pb.TagNumber(1)
  $core.List<VectorComponent> get components => $_getList(0);

  @$pb.TagNumber(2)
  $core.List<MrlLayer> get cascadePipeline => $_getList(1);
}

class CollectionComponent extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'CollectionComponent', package: const $pb.PackageName(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'hyperspace'), createEmptyInstance: create)
    ..aOS(1, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'space')
    ..a<$core.int>(2, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'dimension', $pb.PbFieldType.OU3)
    ..aOS(3, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'metric')
    ..a<$core.double>(4, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'weight', $pb.PbFieldType.OF)
    ..hasRequiredFields = false
  ;

  CollectionComponent._() : super();
  factory CollectionComponent({
    $core.String? space,
    $core.int? dimension,
    $core.String? metric,
    $core.double? weight,
  }) {
    final _result = create();
    if (space != null) {
      _result.space = space;
    }
    if (dimension != null) {
      _result.dimension = dimension;
    }
    if (metric != null) {
      _result.metric = metric;
    }
    if (weight != null) {
      _result.weight = weight;
    }
    return _result;
  }
  factory CollectionComponent.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory CollectionComponent.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  CollectionComponent clone() => CollectionComponent()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  CollectionComponent copyWith(void Function(CollectionComponent) updates) => super.copyWith((message) => updates(message as CollectionComponent)) as CollectionComponent; // ignore: deprecated_member_use
  $pb.BuilderInfo get info_ => _i;
  @$core.pragma('dart2js:noInline')
  static CollectionComponent create() => CollectionComponent._();
  CollectionComponent createEmptyInstance() => create();
  static $pb.PbList<CollectionComponent> createRepeated() => $pb.PbList<CollectionComponent>();
  @$core.pragma('dart2js:noInline')
  static CollectionComponent getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<CollectionComponent>(create);
  static CollectionComponent? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get space => $_getSZ(0);
  @$pb.TagNumber(1)
  set space($core.String v) { $_setString(0, v); }
  @$pb.TagNumber(1)
  $core.bool hasSpace() => $_has(0);
  @$pb.TagNumber(1)
  void clearSpace() => clearField(1);

  @$pb.TagNumber(2)
  $core.int get dimension => $_getIZ(1);
  @$pb.TagNumber(2)
  set dimension($core.int v) { $_setUnsignedInt32(1, v); }
  @$pb.TagNumber(2)
  $core.bool hasDimension() => $_has(1);
  @$pb.TagNumber(2)
  void clearDimension() => clearField(2);

  @$pb.TagNumber(3)
  $core.String get metric => $_getSZ(2);
  @$pb.TagNumber(3)
  set metric($core.String v) { $_setString(2, v); }
  @$pb.TagNumber(3)
  $core.bool hasMetric() => $_has(2);
  @$pb.TagNumber(3)
  void clearMetric() => clearField(3);

  @$pb.TagNumber(4)
  $core.double get weight => $_getN(3);
  @$pb.TagNumber(4)
  set weight($core.double v) { $_setFloat(3, v); }
  @$pb.TagNumber(4)
  $core.bool hasWeight() => $_has(3);
  @$pb.TagNumber(4)
  void clearWeight() => clearField(4);
}

class DeleteCollectionRequest extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'DeleteCollectionRequest', package: const $pb.PackageName(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'hyperspace'), createEmptyInstance: create)
    ..aOS(1, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'name')
    ..hasRequiredFields = false
  ;

  DeleteCollectionRequest._() : super();
  factory DeleteCollectionRequest({
    $core.String? name,
  }) {
    final _result = create();
    if (name != null) {
      _result.name = name;
    }
    return _result;
  }
  factory DeleteCollectionRequest.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory DeleteCollectionRequest.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  DeleteCollectionRequest clone() => DeleteCollectionRequest()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  DeleteCollectionRequest copyWith(void Function(DeleteCollectionRequest) updates) => super.copyWith((message) => updates(message as DeleteCollectionRequest)) as DeleteCollectionRequest; // ignore: deprecated_member_use
  $pb.BuilderInfo get info_ => _i;
  @$core.pragma('dart2js:noInline')
  static DeleteCollectionRequest create() => DeleteCollectionRequest._();
  DeleteCollectionRequest createEmptyInstance() => create();
  static $pb.PbList<DeleteCollectionRequest> createRepeated() => $pb.PbList<DeleteCollectionRequest>();
  @$core.pragma('dart2js:noInline')
  static DeleteCollectionRequest getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<DeleteCollectionRequest>(create);
  static DeleteCollectionRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get name => $_getSZ(0);
  @$pb.TagNumber(1)
  set name($core.String v) { $_setString(0, v); }
  @$pb.TagNumber(1)
  $core.bool hasName() => $_has(0);
  @$pb.TagNumber(1)
  void clearName() => clearField(1);
}

class CollectionSummary extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'CollectionSummary', package: const $pb.PackageName(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'hyperspace'), createEmptyInstance: create)
    ..aOS(1, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'name')
    ..a<$fixnum.Int64>(2, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'count', $pb.PbFieldType.OU6, defaultOrMaker: $fixnum.Int64.ZERO)
    ..aOM<CollectionSchema>(5, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'schema', subBuilder: CollectionSchema.create)
    ..hasRequiredFields = false
  ;

  CollectionSummary._() : super();
  factory CollectionSummary({
    $core.String? name,
    $fixnum.Int64? count,
    CollectionSchema? schema,
  }) {
    final _result = create();
    if (name != null) {
      _result.name = name;
    }
    if (count != null) {
      _result.count = count;
    }
    if (schema != null) {
      _result.schema = schema;
    }
    return _result;
  }
  factory CollectionSummary.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory CollectionSummary.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  CollectionSummary clone() => CollectionSummary()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  CollectionSummary copyWith(void Function(CollectionSummary) updates) => super.copyWith((message) => updates(message as CollectionSummary)) as CollectionSummary; // ignore: deprecated_member_use
  $pb.BuilderInfo get info_ => _i;
  @$core.pragma('dart2js:noInline')
  static CollectionSummary create() => CollectionSummary._();
  CollectionSummary createEmptyInstance() => create();
  static $pb.PbList<CollectionSummary> createRepeated() => $pb.PbList<CollectionSummary>();
  @$core.pragma('dart2js:noInline')
  static CollectionSummary getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<CollectionSummary>(create);
  static CollectionSummary? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get name => $_getSZ(0);
  @$pb.TagNumber(1)
  set name($core.String v) { $_setString(0, v); }
  @$pb.TagNumber(1)
  $core.bool hasName() => $_has(0);
  @$pb.TagNumber(1)
  void clearName() => clearField(1);

  @$pb.TagNumber(2)
  $fixnum.Int64 get count => $_getI64(1);
  @$pb.TagNumber(2)
  set count($fixnum.Int64 v) { $_setInt64(1, v); }
  @$pb.TagNumber(2)
  $core.bool hasCount() => $_has(1);
  @$pb.TagNumber(2)
  void clearCount() => clearField(2);

  @$pb.TagNumber(5)
  CollectionSchema get schema => $_getN(2);
  @$pb.TagNumber(5)
  set schema(CollectionSchema v) { setField(5, v); }
  @$pb.TagNumber(5)
  $core.bool hasSchema() => $_has(2);
  @$pb.TagNumber(5)
  void clearSchema() => clearField(5);
  @$pb.TagNumber(5)
  CollectionSchema ensureSchema() => $_ensure(2);
}

class ListCollectionsResponse extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'ListCollectionsResponse', package: const $pb.PackageName(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'hyperspace'), createEmptyInstance: create)
    ..pc<CollectionSummary>(1, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'collections', $pb.PbFieldType.PM, subBuilder: CollectionSummary.create)
    ..hasRequiredFields = false
  ;

  ListCollectionsResponse._() : super();
  factory ListCollectionsResponse({
    $core.Iterable<CollectionSummary>? collections,
  }) {
    final _result = create();
    if (collections != null) {
      _result.collections.addAll(collections);
    }
    return _result;
  }
  factory ListCollectionsResponse.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory ListCollectionsResponse.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  ListCollectionsResponse clone() => ListCollectionsResponse()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  ListCollectionsResponse copyWith(void Function(ListCollectionsResponse) updates) => super.copyWith((message) => updates(message as ListCollectionsResponse)) as ListCollectionsResponse; // ignore: deprecated_member_use
  $pb.BuilderInfo get info_ => _i;
  @$core.pragma('dart2js:noInline')
  static ListCollectionsResponse create() => ListCollectionsResponse._();
  ListCollectionsResponse createEmptyInstance() => create();
  static $pb.PbList<ListCollectionsResponse> createRepeated() => $pb.PbList<ListCollectionsResponse>();
  @$core.pragma('dart2js:noInline')
  static ListCollectionsResponse getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<ListCollectionsResponse>(create);
  static ListCollectionsResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $core.List<CollectionSummary> get collections => $_getList(0);
}

class CollectionStatsRequest extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'CollectionStatsRequest', package: const $pb.PackageName(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'hyperspace'), createEmptyInstance: create)
    ..aOS(1, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'name')
    ..hasRequiredFields = false
  ;

  CollectionStatsRequest._() : super();
  factory CollectionStatsRequest({
    $core.String? name,
  }) {
    final _result = create();
    if (name != null) {
      _result.name = name;
    }
    return _result;
  }
  factory CollectionStatsRequest.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory CollectionStatsRequest.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  CollectionStatsRequest clone() => CollectionStatsRequest()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  CollectionStatsRequest copyWith(void Function(CollectionStatsRequest) updates) => super.copyWith((message) => updates(message as CollectionStatsRequest)) as CollectionStatsRequest; // ignore: deprecated_member_use
  $pb.BuilderInfo get info_ => _i;
  @$core.pragma('dart2js:noInline')
  static CollectionStatsRequest create() => CollectionStatsRequest._();
  CollectionStatsRequest createEmptyInstance() => create();
  static $pb.PbList<CollectionStatsRequest> createRepeated() => $pb.PbList<CollectionStatsRequest>();
  @$core.pragma('dart2js:noInline')
  static CollectionStatsRequest getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<CollectionStatsRequest>(create);
  static CollectionStatsRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get name => $_getSZ(0);
  @$pb.TagNumber(1)
  set name($core.String v) { $_setString(0, v); }
  @$pb.TagNumber(1)
  $core.bool hasName() => $_has(0);
  @$pb.TagNumber(1)
  void clearName() => clearField(1);
}

class CollectionStatsResponse extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'CollectionStatsResponse', package: const $pb.PackageName(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'hyperspace'), createEmptyInstance: create)
    ..a<$fixnum.Int64>(1, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'count', $pb.PbFieldType.OU6, defaultOrMaker: $fixnum.Int64.ZERO)
    ..a<$fixnum.Int64>(4, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'indexingQueue', $pb.PbFieldType.OU6, defaultOrMaker: $fixnum.Int64.ZERO)
    ..a<$fixnum.Int64>(5, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'diskUsageBytes', $pb.PbFieldType.OU6, defaultOrMaker: $fixnum.Int64.ZERO)
    ..a<$fixnum.Int64>(6, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'ramUsageBytes', $pb.PbFieldType.OU6, defaultOrMaker: $fixnum.Int64.ZERO)
    ..a<$fixnum.Int64>(7, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'activeTasks', $pb.PbFieldType.OU6, defaultOrMaker: $fixnum.Int64.ZERO)
    ..aOM<CollectionSchema>(8, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'schema', subBuilder: CollectionSchema.create)
    ..hasRequiredFields = false
  ;

  CollectionStatsResponse._() : super();
  factory CollectionStatsResponse({
    $fixnum.Int64? count,
    $fixnum.Int64? indexingQueue,
    $fixnum.Int64? diskUsageBytes,
    $fixnum.Int64? ramUsageBytes,
    $fixnum.Int64? activeTasks,
    CollectionSchema? schema,
  }) {
    final _result = create();
    if (count != null) {
      _result.count = count;
    }
    if (indexingQueue != null) {
      _result.indexingQueue = indexingQueue;
    }
    if (diskUsageBytes != null) {
      _result.diskUsageBytes = diskUsageBytes;
    }
    if (ramUsageBytes != null) {
      _result.ramUsageBytes = ramUsageBytes;
    }
    if (activeTasks != null) {
      _result.activeTasks = activeTasks;
    }
    if (schema != null) {
      _result.schema = schema;
    }
    return _result;
  }
  factory CollectionStatsResponse.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory CollectionStatsResponse.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  CollectionStatsResponse clone() => CollectionStatsResponse()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  CollectionStatsResponse copyWith(void Function(CollectionStatsResponse) updates) => super.copyWith((message) => updates(message as CollectionStatsResponse)) as CollectionStatsResponse; // ignore: deprecated_member_use
  $pb.BuilderInfo get info_ => _i;
  @$core.pragma('dart2js:noInline')
  static CollectionStatsResponse create() => CollectionStatsResponse._();
  CollectionStatsResponse createEmptyInstance() => create();
  static $pb.PbList<CollectionStatsResponse> createRepeated() => $pb.PbList<CollectionStatsResponse>();
  @$core.pragma('dart2js:noInline')
  static CollectionStatsResponse getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<CollectionStatsResponse>(create);
  static CollectionStatsResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $fixnum.Int64 get count => $_getI64(0);
  @$pb.TagNumber(1)
  set count($fixnum.Int64 v) { $_setInt64(0, v); }
  @$pb.TagNumber(1)
  $core.bool hasCount() => $_has(0);
  @$pb.TagNumber(1)
  void clearCount() => clearField(1);

  @$pb.TagNumber(4)
  $fixnum.Int64 get indexingQueue => $_getI64(1);
  @$pb.TagNumber(4)
  set indexingQueue($fixnum.Int64 v) { $_setInt64(1, v); }
  @$pb.TagNumber(4)
  $core.bool hasIndexingQueue() => $_has(1);
  @$pb.TagNumber(4)
  void clearIndexingQueue() => clearField(4);

  @$pb.TagNumber(5)
  $fixnum.Int64 get diskUsageBytes => $_getI64(2);
  @$pb.TagNumber(5)
  set diskUsageBytes($fixnum.Int64 v) { $_setInt64(2, v); }
  @$pb.TagNumber(5)
  $core.bool hasDiskUsageBytes() => $_has(2);
  @$pb.TagNumber(5)
  void clearDiskUsageBytes() => clearField(5);

  @$pb.TagNumber(6)
  $fixnum.Int64 get ramUsageBytes => $_getI64(3);
  @$pb.TagNumber(6)
  set ramUsageBytes($fixnum.Int64 v) { $_setInt64(3, v); }
  @$pb.TagNumber(6)
  $core.bool hasRamUsageBytes() => $_has(3);
  @$pb.TagNumber(6)
  void clearRamUsageBytes() => clearField(6);

  @$pb.TagNumber(7)
  $fixnum.Int64 get activeTasks => $_getI64(4);
  @$pb.TagNumber(7)
  set activeTasks($fixnum.Int64 v) { $_setInt64(4, v); }
  @$pb.TagNumber(7)
  $core.bool hasActiveTasks() => $_has(4);
  @$pb.TagNumber(7)
  void clearActiveTasks() => clearField(7);

  @$pb.TagNumber(8)
  CollectionSchema get schema => $_getN(5);
  @$pb.TagNumber(8)
  set schema(CollectionSchema v) { setField(8, v); }
  @$pb.TagNumber(8)
  $core.bool hasSchema() => $_has(5);
  @$pb.TagNumber(8)
  void clearSchema() => clearField(8);
  @$pb.TagNumber(8)
  CollectionSchema ensureSchema() => $_ensure(5);
}

class RebuildIndexRequest extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'RebuildIndexRequest', package: const $pb.PackageName(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'hyperspace'), createEmptyInstance: create)
    ..aOS(1, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'name')
    ..aOM<VacuumFilterQuery>(2, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'filterQuery', subBuilder: VacuumFilterQuery.create)
    ..hasRequiredFields = false
  ;

  RebuildIndexRequest._() : super();
  factory RebuildIndexRequest({
    $core.String? name,
    VacuumFilterQuery? filterQuery,
  }) {
    final _result = create();
    if (name != null) {
      _result.name = name;
    }
    if (filterQuery != null) {
      _result.filterQuery = filterQuery;
    }
    return _result;
  }
  factory RebuildIndexRequest.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory RebuildIndexRequest.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  RebuildIndexRequest clone() => RebuildIndexRequest()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  RebuildIndexRequest copyWith(void Function(RebuildIndexRequest) updates) => super.copyWith((message) => updates(message as RebuildIndexRequest)) as RebuildIndexRequest; // ignore: deprecated_member_use
  $pb.BuilderInfo get info_ => _i;
  @$core.pragma('dart2js:noInline')
  static RebuildIndexRequest create() => RebuildIndexRequest._();
  RebuildIndexRequest createEmptyInstance() => create();
  static $pb.PbList<RebuildIndexRequest> createRepeated() => $pb.PbList<RebuildIndexRequest>();
  @$core.pragma('dart2js:noInline')
  static RebuildIndexRequest getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<RebuildIndexRequest>(create);
  static RebuildIndexRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get name => $_getSZ(0);
  @$pb.TagNumber(1)
  set name($core.String v) { $_setString(0, v); }
  @$pb.TagNumber(1)
  $core.bool hasName() => $_has(0);
  @$pb.TagNumber(1)
  void clearName() => clearField(1);

  @$pb.TagNumber(2)
  VacuumFilterQuery get filterQuery => $_getN(1);
  @$pb.TagNumber(2)
  set filterQuery(VacuumFilterQuery v) { setField(2, v); }
  @$pb.TagNumber(2)
  $core.bool hasFilterQuery() => $_has(1);
  @$pb.TagNumber(2)
  void clearFilterQuery() => clearField(2);
  @$pb.TagNumber(2)
  VacuumFilterQuery ensureFilterQuery() => $_ensure(1);
}

class ConfigUpdate extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'ConfigUpdate', package: const $pb.PackageName(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'hyperspace'), createEmptyInstance: create)
    ..aOS(1, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'collection')
    ..a<$core.int>(2, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'efSearch', $pb.PbFieldType.OU3)
    ..a<$core.int>(3, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'efConstruction', $pb.PbFieldType.OU3)
    ..a<$core.int>(4, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'm', $pb.PbFieldType.OU3)
    ..hasRequiredFields = false
  ;

  ConfigUpdate._() : super();
  factory ConfigUpdate({
    $core.String? collection,
    $core.int? efSearch,
    $core.int? efConstruction,
    $core.int? m,
  }) {
    final _result = create();
    if (collection != null) {
      _result.collection = collection;
    }
    if (efSearch != null) {
      _result.efSearch = efSearch;
    }
    if (efConstruction != null) {
      _result.efConstruction = efConstruction;
    }
    if (m != null) {
      _result.m = m;
    }
    return _result;
  }
  factory ConfigUpdate.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory ConfigUpdate.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  ConfigUpdate clone() => ConfigUpdate()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  ConfigUpdate copyWith(void Function(ConfigUpdate) updates) => super.copyWith((message) => updates(message as ConfigUpdate)) as ConfigUpdate; // ignore: deprecated_member_use
  $pb.BuilderInfo get info_ => _i;
  @$core.pragma('dart2js:noInline')
  static ConfigUpdate create() => ConfigUpdate._();
  ConfigUpdate createEmptyInstance() => create();
  static $pb.PbList<ConfigUpdate> createRepeated() => $pb.PbList<ConfigUpdate>();
  @$core.pragma('dart2js:noInline')
  static ConfigUpdate getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<ConfigUpdate>(create);
  static ConfigUpdate? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get collection => $_getSZ(0);
  @$pb.TagNumber(1)
  set collection($core.String v) { $_setString(0, v); }
  @$pb.TagNumber(1)
  $core.bool hasCollection() => $_has(0);
  @$pb.TagNumber(1)
  void clearCollection() => clearField(1);

  @$pb.TagNumber(2)
  $core.int get efSearch => $_getIZ(1);
  @$pb.TagNumber(2)
  set efSearch($core.int v) { $_setUnsignedInt32(1, v); }
  @$pb.TagNumber(2)
  $core.bool hasEfSearch() => $_has(1);
  @$pb.TagNumber(2)
  void clearEfSearch() => clearField(2);

  @$pb.TagNumber(3)
  $core.int get efConstruction => $_getIZ(2);
  @$pb.TagNumber(3)
  set efConstruction($core.int v) { $_setUnsignedInt32(2, v); }
  @$pb.TagNumber(3)
  $core.bool hasEfConstruction() => $_has(2);
  @$pb.TagNumber(3)
  void clearEfConstruction() => clearField(3);

  @$pb.TagNumber(4)
  $core.int get m => $_getIZ(3);
  @$pb.TagNumber(4)
  set m($core.int v) { $_setUnsignedInt32(3, v); }
  @$pb.TagNumber(4)
  $core.bool hasM() => $_has(3);
  @$pb.TagNumber(4)
  void clearM() => clearField(4);
}

class VacuumFilterQuery extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'VacuumFilterQuery', package: const $pb.PackageName(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'hyperspace'), createEmptyInstance: create)
    ..aOS(1, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'key')
    ..aOS(2, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'op')
    ..a<$core.double>(3, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'value', $pb.PbFieldType.OD)
    ..hasRequiredFields = false
  ;

  VacuumFilterQuery._() : super();
  factory VacuumFilterQuery({
    $core.String? key,
    $core.String? op,
    $core.double? value,
  }) {
    final _result = create();
    if (key != null) {
      _result.key = key;
    }
    if (op != null) {
      _result.op = op;
    }
    if (value != null) {
      _result.value = value;
    }
    return _result;
  }
  factory VacuumFilterQuery.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory VacuumFilterQuery.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  VacuumFilterQuery clone() => VacuumFilterQuery()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  VacuumFilterQuery copyWith(void Function(VacuumFilterQuery) updates) => super.copyWith((message) => updates(message as VacuumFilterQuery)) as VacuumFilterQuery; // ignore: deprecated_member_use
  $pb.BuilderInfo get info_ => _i;
  @$core.pragma('dart2js:noInline')
  static VacuumFilterQuery create() => VacuumFilterQuery._();
  VacuumFilterQuery createEmptyInstance() => create();
  static $pb.PbList<VacuumFilterQuery> createRepeated() => $pb.PbList<VacuumFilterQuery>();
  @$core.pragma('dart2js:noInline')
  static VacuumFilterQuery getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<VacuumFilterQuery>(create);
  static VacuumFilterQuery? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get key => $_getSZ(0);
  @$pb.TagNumber(1)
  set key($core.String v) { $_setString(0, v); }
  @$pb.TagNumber(1)
  $core.bool hasKey() => $_has(0);
  @$pb.TagNumber(1)
  void clearKey() => clearField(1);

  @$pb.TagNumber(2)
  $core.String get op => $_getSZ(1);
  @$pb.TagNumber(2)
  set op($core.String v) { $_setString(1, v); }
  @$pb.TagNumber(2)
  $core.bool hasOp() => $_has(1);
  @$pb.TagNumber(2)
  void clearOp() => clearField(2);

  @$pb.TagNumber(3)
  $core.double get value => $_getN(2);
  @$pb.TagNumber(3)
  set value($core.double v) { $_setDouble(2, v); }
  @$pb.TagNumber(3)
  $core.bool hasValue() => $_has(2);
  @$pb.TagNumber(3)
  void clearValue() => clearField(3);
}

class ReconsolidationRequest extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'ReconsolidationRequest', package: const $pb.PackageName(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'hyperspace'), createEmptyInstance: create)
    ..aOS(1, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'collection')
    ..p<$core.double>(2, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'targetVector', $pb.PbFieldType.KD)
    ..a<$core.double>(3, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'learningRate', $pb.PbFieldType.OD)
    ..hasRequiredFields = false
  ;

  ReconsolidationRequest._() : super();
  factory ReconsolidationRequest({
    $core.String? collection,
    $core.Iterable<$core.double>? targetVector,
    $core.double? learningRate,
  }) {
    final _result = create();
    if (collection != null) {
      _result.collection = collection;
    }
    if (targetVector != null) {
      _result.targetVector.addAll(targetVector);
    }
    if (learningRate != null) {
      _result.learningRate = learningRate;
    }
    return _result;
  }
  factory ReconsolidationRequest.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory ReconsolidationRequest.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  ReconsolidationRequest clone() => ReconsolidationRequest()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  ReconsolidationRequest copyWith(void Function(ReconsolidationRequest) updates) => super.copyWith((message) => updates(message as ReconsolidationRequest)) as ReconsolidationRequest; // ignore: deprecated_member_use
  $pb.BuilderInfo get info_ => _i;
  @$core.pragma('dart2js:noInline')
  static ReconsolidationRequest create() => ReconsolidationRequest._();
  ReconsolidationRequest createEmptyInstance() => create();
  static $pb.PbList<ReconsolidationRequest> createRepeated() => $pb.PbList<ReconsolidationRequest>();
  @$core.pragma('dart2js:noInline')
  static ReconsolidationRequest getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<ReconsolidationRequest>(create);
  static ReconsolidationRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get collection => $_getSZ(0);
  @$pb.TagNumber(1)
  set collection($core.String v) { $_setString(0, v); }
  @$pb.TagNumber(1)
  $core.bool hasCollection() => $_has(0);
  @$pb.TagNumber(1)
  void clearCollection() => clearField(1);

  @$pb.TagNumber(2)
  $core.List<$core.double> get targetVector => $_getList(1);

  @$pb.TagNumber(3)
  $core.double get learningRate => $_getN(2);
  @$pb.TagNumber(3)
  set learningRate($core.double v) { $_setDouble(2, v); }
  @$pb.TagNumber(3)
  $core.bool hasLearningRate() => $_has(2);
  @$pb.TagNumber(3)
  void clearLearningRate() => clearField(3);
}

class InsertRequest extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'InsertRequest', package: const $pb.PackageName(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'hyperspace'), createEmptyInstance: create)
    ..aOS(1, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'collection')
    ..p<$core.double>(2, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'vector', $pb.PbFieldType.KD)
    ..a<$core.int>(3, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'id', $pb.PbFieldType.OU3)
    ..m<$core.String, $core.String>(4, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'metadata', entryClassName: 'InsertRequest.MetadataEntry', keyFieldType: $pb.PbFieldType.OS, valueFieldType: $pb.PbFieldType.OS, packageName: const $pb.PackageName('hyperspace'))
    ..aOS(5, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'originNodeId')
    ..a<$fixnum.Int64>(6, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'logicalClock', $pb.PbFieldType.OU6, defaultOrMaker: $fixnum.Int64.ZERO)
    ..e<DurabilityLevel>(7, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'durability', $pb.PbFieldType.OE, defaultOrMaker: DurabilityLevel.DEFAULT_LEVEL, valueOf: DurabilityLevel.valueOf, enumValues: DurabilityLevel.values)
    ..m<$core.String, MetadataValue>(8, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'typedMetadata', entryClassName: 'InsertRequest.TypedMetadataEntry', keyFieldType: $pb.PbFieldType.OS, valueFieldType: $pb.PbFieldType.OM, valueCreator: MetadataValue.create, packageName: const $pb.PackageName('hyperspace'))
    ..a<$core.List<$core.int>>(9, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'payload', $pb.PbFieldType.OY)
    ..hasRequiredFields = false
  ;

  InsertRequest._() : super();
  factory InsertRequest({
    $core.String? collection,
    $core.Iterable<$core.double>? vector,
    $core.int? id,
    $core.Map<$core.String, $core.String>? metadata,
    $core.String? originNodeId,
    $fixnum.Int64? logicalClock,
    DurabilityLevel? durability,
    $core.Map<$core.String, MetadataValue>? typedMetadata,
    $core.List<$core.int>? payload,
  }) {
    final _result = create();
    if (collection != null) {
      _result.collection = collection;
    }
    if (vector != null) {
      _result.vector.addAll(vector);
    }
    if (id != null) {
      _result.id = id;
    }
    if (metadata != null) {
      _result.metadata.addAll(metadata);
    }
    if (originNodeId != null) {
      _result.originNodeId = originNodeId;
    }
    if (logicalClock != null) {
      _result.logicalClock = logicalClock;
    }
    if (durability != null) {
      _result.durability = durability;
    }
    if (typedMetadata != null) {
      _result.typedMetadata.addAll(typedMetadata);
    }
    if (payload != null) {
      _result.payload = payload;
    }
    return _result;
  }
  factory InsertRequest.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory InsertRequest.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  InsertRequest clone() => InsertRequest()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  InsertRequest copyWith(void Function(InsertRequest) updates) => super.copyWith((message) => updates(message as InsertRequest)) as InsertRequest; // ignore: deprecated_member_use
  $pb.BuilderInfo get info_ => _i;
  @$core.pragma('dart2js:noInline')
  static InsertRequest create() => InsertRequest._();
  InsertRequest createEmptyInstance() => create();
  static $pb.PbList<InsertRequest> createRepeated() => $pb.PbList<InsertRequest>();
  @$core.pragma('dart2js:noInline')
  static InsertRequest getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<InsertRequest>(create);
  static InsertRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get collection => $_getSZ(0);
  @$pb.TagNumber(1)
  set collection($core.String v) { $_setString(0, v); }
  @$pb.TagNumber(1)
  $core.bool hasCollection() => $_has(0);
  @$pb.TagNumber(1)
  void clearCollection() => clearField(1);

  @$pb.TagNumber(2)
  $core.List<$core.double> get vector => $_getList(1);

  @$pb.TagNumber(3)
  $core.int get id => $_getIZ(2);
  @$pb.TagNumber(3)
  set id($core.int v) { $_setUnsignedInt32(2, v); }
  @$pb.TagNumber(3)
  $core.bool hasId() => $_has(2);
  @$pb.TagNumber(3)
  void clearId() => clearField(3);

  @$pb.TagNumber(4)
  $core.Map<$core.String, $core.String> get metadata => $_getMap(3);

  @$pb.TagNumber(5)
  $core.String get originNodeId => $_getSZ(4);
  @$pb.TagNumber(5)
  set originNodeId($core.String v) { $_setString(4, v); }
  @$pb.TagNumber(5)
  $core.bool hasOriginNodeId() => $_has(4);
  @$pb.TagNumber(5)
  void clearOriginNodeId() => clearField(5);

  @$pb.TagNumber(6)
  $fixnum.Int64 get logicalClock => $_getI64(5);
  @$pb.TagNumber(6)
  set logicalClock($fixnum.Int64 v) { $_setInt64(5, v); }
  @$pb.TagNumber(6)
  $core.bool hasLogicalClock() => $_has(5);
  @$pb.TagNumber(6)
  void clearLogicalClock() => clearField(6);

  @$pb.TagNumber(7)
  DurabilityLevel get durability => $_getN(6);
  @$pb.TagNumber(7)
  set durability(DurabilityLevel v) { setField(7, v); }
  @$pb.TagNumber(7)
  $core.bool hasDurability() => $_has(6);
  @$pb.TagNumber(7)
  void clearDurability() => clearField(7);

  @$pb.TagNumber(8)
  $core.Map<$core.String, MetadataValue> get typedMetadata => $_getMap(7);

  @$pb.TagNumber(9)
  $core.List<$core.int> get payload => $_getN(8);
  @$pb.TagNumber(9)
  set payload($core.List<$core.int> v) { $_setBytes(8, v); }
  @$pb.TagNumber(9)
  $core.bool hasPayload() => $_has(8);
  @$pb.TagNumber(9)
  void clearPayload() => clearField(9);
}

class VectorData extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'VectorData', package: const $pb.PackageName(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'hyperspace'), createEmptyInstance: create)
    ..p<$core.double>(1, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'vector', $pb.PbFieldType.KD)
    ..a<$core.int>(2, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'id', $pb.PbFieldType.OU3)
    ..m<$core.String, $core.String>(3, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'metadata', entryClassName: 'VectorData.MetadataEntry', keyFieldType: $pb.PbFieldType.OS, valueFieldType: $pb.PbFieldType.OS, packageName: const $pb.PackageName('hyperspace'))
    ..m<$core.String, MetadataValue>(4, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'typedMetadata', entryClassName: 'VectorData.TypedMetadataEntry', keyFieldType: $pb.PbFieldType.OS, valueFieldType: $pb.PbFieldType.OM, valueCreator: MetadataValue.create, packageName: const $pb.PackageName('hyperspace'))
    ..hasRequiredFields = false
  ;

  VectorData._() : super();
  factory VectorData({
    $core.Iterable<$core.double>? vector,
    $core.int? id,
    $core.Map<$core.String, $core.String>? metadata,
    $core.Map<$core.String, MetadataValue>? typedMetadata,
  }) {
    final _result = create();
    if (vector != null) {
      _result.vector.addAll(vector);
    }
    if (id != null) {
      _result.id = id;
    }
    if (metadata != null) {
      _result.metadata.addAll(metadata);
    }
    if (typedMetadata != null) {
      _result.typedMetadata.addAll(typedMetadata);
    }
    return _result;
  }
  factory VectorData.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory VectorData.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  VectorData clone() => VectorData()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  VectorData copyWith(void Function(VectorData) updates) => super.copyWith((message) => updates(message as VectorData)) as VectorData; // ignore: deprecated_member_use
  $pb.BuilderInfo get info_ => _i;
  @$core.pragma('dart2js:noInline')
  static VectorData create() => VectorData._();
  VectorData createEmptyInstance() => create();
  static $pb.PbList<VectorData> createRepeated() => $pb.PbList<VectorData>();
  @$core.pragma('dart2js:noInline')
  static VectorData getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<VectorData>(create);
  static VectorData? _defaultInstance;

  @$pb.TagNumber(1)
  $core.List<$core.double> get vector => $_getList(0);

  @$pb.TagNumber(2)
  $core.int get id => $_getIZ(1);
  @$pb.TagNumber(2)
  set id($core.int v) { $_setUnsignedInt32(1, v); }
  @$pb.TagNumber(2)
  $core.bool hasId() => $_has(1);
  @$pb.TagNumber(2)
  void clearId() => clearField(2);

  @$pb.TagNumber(3)
  $core.Map<$core.String, $core.String> get metadata => $_getMap(2);

  @$pb.TagNumber(4)
  $core.Map<$core.String, MetadataValue> get typedMetadata => $_getMap(3);
}

class BatchInsertRequest extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'BatchInsertRequest', package: const $pb.PackageName(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'hyperspace'), createEmptyInstance: create)
    ..aOS(1, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'collection')
    ..pc<VectorData>(2, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'vectors', $pb.PbFieldType.PM, subBuilder: VectorData.create)
    ..aOS(3, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'originNodeId')
    ..a<$fixnum.Int64>(4, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'logicalClock', $pb.PbFieldType.OU6, defaultOrMaker: $fixnum.Int64.ZERO)
    ..e<DurabilityLevel>(5, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'durability', $pb.PbFieldType.OE, defaultOrMaker: DurabilityLevel.DEFAULT_LEVEL, valueOf: DurabilityLevel.valueOf, enumValues: DurabilityLevel.values)
    ..hasRequiredFields = false
  ;

  BatchInsertRequest._() : super();
  factory BatchInsertRequest({
    $core.String? collection,
    $core.Iterable<VectorData>? vectors,
    $core.String? originNodeId,
    $fixnum.Int64? logicalClock,
    DurabilityLevel? durability,
  }) {
    final _result = create();
    if (collection != null) {
      _result.collection = collection;
    }
    if (vectors != null) {
      _result.vectors.addAll(vectors);
    }
    if (originNodeId != null) {
      _result.originNodeId = originNodeId;
    }
    if (logicalClock != null) {
      _result.logicalClock = logicalClock;
    }
    if (durability != null) {
      _result.durability = durability;
    }
    return _result;
  }
  factory BatchInsertRequest.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory BatchInsertRequest.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  BatchInsertRequest clone() => BatchInsertRequest()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  BatchInsertRequest copyWith(void Function(BatchInsertRequest) updates) => super.copyWith((message) => updates(message as BatchInsertRequest)) as BatchInsertRequest; // ignore: deprecated_member_use
  $pb.BuilderInfo get info_ => _i;
  @$core.pragma('dart2js:noInline')
  static BatchInsertRequest create() => BatchInsertRequest._();
  BatchInsertRequest createEmptyInstance() => create();
  static $pb.PbList<BatchInsertRequest> createRepeated() => $pb.PbList<BatchInsertRequest>();
  @$core.pragma('dart2js:noInline')
  static BatchInsertRequest getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<BatchInsertRequest>(create);
  static BatchInsertRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get collection => $_getSZ(0);
  @$pb.TagNumber(1)
  set collection($core.String v) { $_setString(0, v); }
  @$pb.TagNumber(1)
  $core.bool hasCollection() => $_has(0);
  @$pb.TagNumber(1)
  void clearCollection() => clearField(1);

  @$pb.TagNumber(2)
  $core.List<VectorData> get vectors => $_getList(1);

  @$pb.TagNumber(3)
  $core.String get originNodeId => $_getSZ(2);
  @$pb.TagNumber(3)
  set originNodeId($core.String v) { $_setString(2, v); }
  @$pb.TagNumber(3)
  $core.bool hasOriginNodeId() => $_has(2);
  @$pb.TagNumber(3)
  void clearOriginNodeId() => clearField(3);

  @$pb.TagNumber(4)
  $fixnum.Int64 get logicalClock => $_getI64(3);
  @$pb.TagNumber(4)
  set logicalClock($fixnum.Int64 v) { $_setInt64(3, v); }
  @$pb.TagNumber(4)
  $core.bool hasLogicalClock() => $_has(3);
  @$pb.TagNumber(4)
  void clearLogicalClock() => clearField(4);

  @$pb.TagNumber(5)
  DurabilityLevel get durability => $_getN(4);
  @$pb.TagNumber(5)
  set durability(DurabilityLevel v) { setField(5, v); }
  @$pb.TagNumber(5)
  $core.bool hasDurability() => $_has(4);
  @$pb.TagNumber(5)
  void clearDurability() => clearField(5);
}

class InsertTextRequest extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'InsertTextRequest', package: const $pb.PackageName(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'hyperspace'), createEmptyInstance: create)
    ..aOS(1, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'collection')
    ..a<$core.int>(2, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'id', $pb.PbFieldType.OU3)
    ..aOS(3, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'text')
    ..m<$core.String, $core.String>(4, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'metadata', entryClassName: 'InsertTextRequest.MetadataEntry', keyFieldType: $pb.PbFieldType.OS, valueFieldType: $pb.PbFieldType.OS, packageName: const $pb.PackageName('hyperspace'))
    ..e<DurabilityLevel>(5, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'durability', $pb.PbFieldType.OE, defaultOrMaker: DurabilityLevel.DEFAULT_LEVEL, valueOf: DurabilityLevel.valueOf, enumValues: DurabilityLevel.values)
    ..hasRequiredFields = false
  ;

  InsertTextRequest._() : super();
  factory InsertTextRequest({
    $core.String? collection,
    $core.int? id,
    $core.String? text,
    $core.Map<$core.String, $core.String>? metadata,
    DurabilityLevel? durability,
  }) {
    final _result = create();
    if (collection != null) {
      _result.collection = collection;
    }
    if (id != null) {
      _result.id = id;
    }
    if (text != null) {
      _result.text = text;
    }
    if (metadata != null) {
      _result.metadata.addAll(metadata);
    }
    if (durability != null) {
      _result.durability = durability;
    }
    return _result;
  }
  factory InsertTextRequest.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory InsertTextRequest.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  InsertTextRequest clone() => InsertTextRequest()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  InsertTextRequest copyWith(void Function(InsertTextRequest) updates) => super.copyWith((message) => updates(message as InsertTextRequest)) as InsertTextRequest; // ignore: deprecated_member_use
  $pb.BuilderInfo get info_ => _i;
  @$core.pragma('dart2js:noInline')
  static InsertTextRequest create() => InsertTextRequest._();
  InsertTextRequest createEmptyInstance() => create();
  static $pb.PbList<InsertTextRequest> createRepeated() => $pb.PbList<InsertTextRequest>();
  @$core.pragma('dart2js:noInline')
  static InsertTextRequest getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<InsertTextRequest>(create);
  static InsertTextRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get collection => $_getSZ(0);
  @$pb.TagNumber(1)
  set collection($core.String v) { $_setString(0, v); }
  @$pb.TagNumber(1)
  $core.bool hasCollection() => $_has(0);
  @$pb.TagNumber(1)
  void clearCollection() => clearField(1);

  @$pb.TagNumber(2)
  $core.int get id => $_getIZ(1);
  @$pb.TagNumber(2)
  set id($core.int v) { $_setUnsignedInt32(1, v); }
  @$pb.TagNumber(2)
  $core.bool hasId() => $_has(1);
  @$pb.TagNumber(2)
  void clearId() => clearField(2);

  @$pb.TagNumber(3)
  $core.String get text => $_getSZ(2);
  @$pb.TagNumber(3)
  set text($core.String v) { $_setString(2, v); }
  @$pb.TagNumber(3)
  $core.bool hasText() => $_has(2);
  @$pb.TagNumber(3)
  void clearText() => clearField(3);

  @$pb.TagNumber(4)
  $core.Map<$core.String, $core.String> get metadata => $_getMap(3);

  @$pb.TagNumber(5)
  DurabilityLevel get durability => $_getN(4);
  @$pb.TagNumber(5)
  set durability(DurabilityLevel v) { setField(5, v); }
  @$pb.TagNumber(5)
  $core.bool hasDurability() => $_has(4);
  @$pb.TagNumber(5)
  void clearDurability() => clearField(5);
}

class VectorizeRequest extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'VectorizeRequest', package: const $pb.PackageName(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'hyperspace'), createEmptyInstance: create)
    ..aOS(1, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'text')
    ..aOS(2, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'metric')
    ..hasRequiredFields = false
  ;

  VectorizeRequest._() : super();
  factory VectorizeRequest({
    $core.String? text,
    $core.String? metric,
  }) {
    final _result = create();
    if (text != null) {
      _result.text = text;
    }
    if (metric != null) {
      _result.metric = metric;
    }
    return _result;
  }
  factory VectorizeRequest.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory VectorizeRequest.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  VectorizeRequest clone() => VectorizeRequest()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  VectorizeRequest copyWith(void Function(VectorizeRequest) updates) => super.copyWith((message) => updates(message as VectorizeRequest)) as VectorizeRequest; // ignore: deprecated_member_use
  $pb.BuilderInfo get info_ => _i;
  @$core.pragma('dart2js:noInline')
  static VectorizeRequest create() => VectorizeRequest._();
  VectorizeRequest createEmptyInstance() => create();
  static $pb.PbList<VectorizeRequest> createRepeated() => $pb.PbList<VectorizeRequest>();
  @$core.pragma('dart2js:noInline')
  static VectorizeRequest getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<VectorizeRequest>(create);
  static VectorizeRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get text => $_getSZ(0);
  @$pb.TagNumber(1)
  set text($core.String v) { $_setString(0, v); }
  @$pb.TagNumber(1)
  $core.bool hasText() => $_has(0);
  @$pb.TagNumber(1)
  void clearText() => clearField(1);

  @$pb.TagNumber(2)
  $core.String get metric => $_getSZ(1);
  @$pb.TagNumber(2)
  set metric($core.String v) { $_setString(1, v); }
  @$pb.TagNumber(2)
  $core.bool hasMetric() => $_has(1);
  @$pb.TagNumber(2)
  void clearMetric() => clearField(2);
}

class VectorizeResponse extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'VectorizeResponse', package: const $pb.PackageName(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'hyperspace'), createEmptyInstance: create)
    ..p<$core.double>(1, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'vector', $pb.PbFieldType.KD)
    ..hasRequiredFields = false
  ;

  VectorizeResponse._() : super();
  factory VectorizeResponse({
    $core.Iterable<$core.double>? vector,
  }) {
    final _result = create();
    if (vector != null) {
      _result.vector.addAll(vector);
    }
    return _result;
  }
  factory VectorizeResponse.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory VectorizeResponse.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  VectorizeResponse clone() => VectorizeResponse()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  VectorizeResponse copyWith(void Function(VectorizeResponse) updates) => super.copyWith((message) => updates(message as VectorizeResponse)) as VectorizeResponse; // ignore: deprecated_member_use
  $pb.BuilderInfo get info_ => _i;
  @$core.pragma('dart2js:noInline')
  static VectorizeResponse create() => VectorizeResponse._();
  VectorizeResponse createEmptyInstance() => create();
  static $pb.PbList<VectorizeResponse> createRepeated() => $pb.PbList<VectorizeResponse>();
  @$core.pragma('dart2js:noInline')
  static VectorizeResponse getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<VectorizeResponse>(create);
  static VectorizeResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $core.List<$core.double> get vector => $_getList(0);
}

class SearchTextRequest extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'SearchTextRequest', package: const $pb.PackageName(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'hyperspace'), createEmptyInstance: create)
    ..aOS(1, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'collection')
    ..aOS(2, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'text')
    ..a<$core.int>(3, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'topK', $pb.PbFieldType.OU3)
    ..m<$core.String, $core.String>(4, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'filter', entryClassName: 'SearchTextRequest.FilterEntry', keyFieldType: $pb.PbFieldType.OS, valueFieldType: $pb.PbFieldType.OS, packageName: const $pb.PackageName('hyperspace'))
    ..pc<Filter>(5, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'filters', $pb.PbFieldType.PM, subBuilder: Filter.create)
    ..aOM<Bm25Options>(6, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'bm25Options', subBuilder: Bm25Options.create)
    ..a<$core.double>(7, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'hybridAlpha', $pb.PbFieldType.OF)
    ..aOB(8, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'includePayload')
    ..m<$core.String, $core.double>(9, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'componentWeights', entryClassName: 'SearchTextRequest.ComponentWeightsEntry', keyFieldType: $pb.PbFieldType.OS, valueFieldType: $pb.PbFieldType.OF, packageName: const $pb.PackageName('hyperspace'))
    ..hasRequiredFields = false
  ;

  SearchTextRequest._() : super();
  factory SearchTextRequest({
    $core.String? collection,
    $core.String? text,
    $core.int? topK,
    $core.Map<$core.String, $core.String>? filter,
    $core.Iterable<Filter>? filters,
    Bm25Options? bm25Options,
    $core.double? hybridAlpha,
    $core.bool? includePayload,
    $core.Map<$core.String, $core.double>? componentWeights,
  }) {
    final _result = create();
    if (collection != null) {
      _result.collection = collection;
    }
    if (text != null) {
      _result.text = text;
    }
    if (topK != null) {
      _result.topK = topK;
    }
    if (filter != null) {
      _result.filter.addAll(filter);
    }
    if (filters != null) {
      _result.filters.addAll(filters);
    }
    if (bm25Options != null) {
      _result.bm25Options = bm25Options;
    }
    if (hybridAlpha != null) {
      _result.hybridAlpha = hybridAlpha;
    }
    if (includePayload != null) {
      _result.includePayload = includePayload;
    }
    if (componentWeights != null) {
      _result.componentWeights.addAll(componentWeights);
    }
    return _result;
  }
  factory SearchTextRequest.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory SearchTextRequest.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  SearchTextRequest clone() => SearchTextRequest()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  SearchTextRequest copyWith(void Function(SearchTextRequest) updates) => super.copyWith((message) => updates(message as SearchTextRequest)) as SearchTextRequest; // ignore: deprecated_member_use
  $pb.BuilderInfo get info_ => _i;
  @$core.pragma('dart2js:noInline')
  static SearchTextRequest create() => SearchTextRequest._();
  SearchTextRequest createEmptyInstance() => create();
  static $pb.PbList<SearchTextRequest> createRepeated() => $pb.PbList<SearchTextRequest>();
  @$core.pragma('dart2js:noInline')
  static SearchTextRequest getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<SearchTextRequest>(create);
  static SearchTextRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get collection => $_getSZ(0);
  @$pb.TagNumber(1)
  set collection($core.String v) { $_setString(0, v); }
  @$pb.TagNumber(1)
  $core.bool hasCollection() => $_has(0);
  @$pb.TagNumber(1)
  void clearCollection() => clearField(1);

  @$pb.TagNumber(2)
  $core.String get text => $_getSZ(1);
  @$pb.TagNumber(2)
  set text($core.String v) { $_setString(1, v); }
  @$pb.TagNumber(2)
  $core.bool hasText() => $_has(1);
  @$pb.TagNumber(2)
  void clearText() => clearField(2);

  @$pb.TagNumber(3)
  $core.int get topK => $_getIZ(2);
  @$pb.TagNumber(3)
  set topK($core.int v) { $_setUnsignedInt32(2, v); }
  @$pb.TagNumber(3)
  $core.bool hasTopK() => $_has(2);
  @$pb.TagNumber(3)
  void clearTopK() => clearField(3);

  @$pb.TagNumber(4)
  $core.Map<$core.String, $core.String> get filter => $_getMap(3);

  @$pb.TagNumber(5)
  $core.List<Filter> get filters => $_getList(4);

  @$pb.TagNumber(6)
  Bm25Options get bm25Options => $_getN(5);
  @$pb.TagNumber(6)
  set bm25Options(Bm25Options v) { setField(6, v); }
  @$pb.TagNumber(6)
  $core.bool hasBm25Options() => $_has(5);
  @$pb.TagNumber(6)
  void clearBm25Options() => clearField(6);
  @$pb.TagNumber(6)
  Bm25Options ensureBm25Options() => $_ensure(5);

  @$pb.TagNumber(7)
  $core.double get hybridAlpha => $_getN(6);
  @$pb.TagNumber(7)
  set hybridAlpha($core.double v) { $_setFloat(6, v); }
  @$pb.TagNumber(7)
  $core.bool hasHybridAlpha() => $_has(6);
  @$pb.TagNumber(7)
  void clearHybridAlpha() => clearField(7);

  @$pb.TagNumber(8)
  $core.bool get includePayload => $_getBF(7);
  @$pb.TagNumber(8)
  set includePayload($core.bool v) { $_setBool(7, v); }
  @$pb.TagNumber(8)
  $core.bool hasIncludePayload() => $_has(7);
  @$pb.TagNumber(8)
  void clearIncludePayload() => clearField(8);

  @$pb.TagNumber(9)
  $core.Map<$core.String, $core.double> get componentWeights => $_getMap(8);
}

class Bm25Options extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'Bm25Options', package: const $pb.PackageName(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'hyperspace'), createEmptyInstance: create)
    ..aOS(1, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'method')
    ..a<$core.double>(2, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'k1', $pb.PbFieldType.OF)
    ..a<$core.double>(3, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'b', $pb.PbFieldType.OF)
    ..a<$core.double>(4, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'delta', $pb.PbFieldType.OF)
    ..aOS(5, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'language')
    ..a<$core.int>(6, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'ngrams', $pb.PbFieldType.OU3)
    ..aOS(7, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'fusionMethod')
    ..hasRequiredFields = false
  ;

  Bm25Options._() : super();
  factory Bm25Options({
    $core.String? method,
    $core.double? k1,
    $core.double? b,
    $core.double? delta,
    $core.String? language,
    $core.int? ngrams,
    $core.String? fusionMethod,
  }) {
    final _result = create();
    if (method != null) {
      _result.method = method;
    }
    if (k1 != null) {
      _result.k1 = k1;
    }
    if (b != null) {
      _result.b = b;
    }
    if (delta != null) {
      _result.delta = delta;
    }
    if (language != null) {
      _result.language = language;
    }
    if (ngrams != null) {
      _result.ngrams = ngrams;
    }
    if (fusionMethod != null) {
      _result.fusionMethod = fusionMethod;
    }
    return _result;
  }
  factory Bm25Options.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory Bm25Options.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  Bm25Options clone() => Bm25Options()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  Bm25Options copyWith(void Function(Bm25Options) updates) => super.copyWith((message) => updates(message as Bm25Options)) as Bm25Options; // ignore: deprecated_member_use
  $pb.BuilderInfo get info_ => _i;
  @$core.pragma('dart2js:noInline')
  static Bm25Options create() => Bm25Options._();
  Bm25Options createEmptyInstance() => create();
  static $pb.PbList<Bm25Options> createRepeated() => $pb.PbList<Bm25Options>();
  @$core.pragma('dart2js:noInline')
  static Bm25Options getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<Bm25Options>(create);
  static Bm25Options? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get method => $_getSZ(0);
  @$pb.TagNumber(1)
  set method($core.String v) { $_setString(0, v); }
  @$pb.TagNumber(1)
  $core.bool hasMethod() => $_has(0);
  @$pb.TagNumber(1)
  void clearMethod() => clearField(1);

  @$pb.TagNumber(2)
  $core.double get k1 => $_getN(1);
  @$pb.TagNumber(2)
  set k1($core.double v) { $_setFloat(1, v); }
  @$pb.TagNumber(2)
  $core.bool hasK1() => $_has(1);
  @$pb.TagNumber(2)
  void clearK1() => clearField(2);

  @$pb.TagNumber(3)
  $core.double get b => $_getN(2);
  @$pb.TagNumber(3)
  set b($core.double v) { $_setFloat(2, v); }
  @$pb.TagNumber(3)
  $core.bool hasB() => $_has(2);
  @$pb.TagNumber(3)
  void clearB() => clearField(3);

  @$pb.TagNumber(4)
  $core.double get delta => $_getN(3);
  @$pb.TagNumber(4)
  set delta($core.double v) { $_setFloat(3, v); }
  @$pb.TagNumber(4)
  $core.bool hasDelta() => $_has(3);
  @$pb.TagNumber(4)
  void clearDelta() => clearField(4);

  @$pb.TagNumber(5)
  $core.String get language => $_getSZ(4);
  @$pb.TagNumber(5)
  set language($core.String v) { $_setString(4, v); }
  @$pb.TagNumber(5)
  $core.bool hasLanguage() => $_has(4);
  @$pb.TagNumber(5)
  void clearLanguage() => clearField(5);

  @$pb.TagNumber(6)
  $core.int get ngrams => $_getIZ(5);
  @$pb.TagNumber(6)
  set ngrams($core.int v) { $_setUnsignedInt32(5, v); }
  @$pb.TagNumber(6)
  $core.bool hasNgrams() => $_has(5);
  @$pb.TagNumber(6)
  void clearNgrams() => clearField(6);

  @$pb.TagNumber(7)
  $core.String get fusionMethod => $_getSZ(6);
  @$pb.TagNumber(7)
  set fusionMethod($core.String v) { $_setString(6, v); }
  @$pb.TagNumber(7)
  $core.bool hasFusionMethod() => $_has(6);
  @$pb.TagNumber(7)
  void clearFusionMethod() => clearField(7);
}

class InsertResponse extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'InsertResponse', package: const $pb.PackageName(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'hyperspace'), createEmptyInstance: create)
    ..aOB(1, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'success')
    ..hasRequiredFields = false
  ;

  InsertResponse._() : super();
  factory InsertResponse({
    $core.bool? success,
  }) {
    final _result = create();
    if (success != null) {
      _result.success = success;
    }
    return _result;
  }
  factory InsertResponse.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory InsertResponse.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  InsertResponse clone() => InsertResponse()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  InsertResponse copyWith(void Function(InsertResponse) updates) => super.copyWith((message) => updates(message as InsertResponse)) as InsertResponse; // ignore: deprecated_member_use
  $pb.BuilderInfo get info_ => _i;
  @$core.pragma('dart2js:noInline')
  static InsertResponse create() => InsertResponse._();
  InsertResponse createEmptyInstance() => create();
  static $pb.PbList<InsertResponse> createRepeated() => $pb.PbList<InsertResponse>();
  @$core.pragma('dart2js:noInline')
  static InsertResponse getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<InsertResponse>(create);
  static InsertResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $core.bool get success => $_getBF(0);
  @$pb.TagNumber(1)
  set success($core.bool v) { $_setBool(0, v); }
  @$pb.TagNumber(1)
  $core.bool hasSuccess() => $_has(0);
  @$pb.TagNumber(1)
  void clearSuccess() => clearField(1);
}

class DeleteRequest extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'DeleteRequest', package: const $pb.PackageName(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'hyperspace'), createEmptyInstance: create)
    ..aOS(1, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'collection')
    ..a<$core.int>(2, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'id', $pb.PbFieldType.OU3)
    ..hasRequiredFields = false
  ;

  DeleteRequest._() : super();
  factory DeleteRequest({
    $core.String? collection,
    $core.int? id,
  }) {
    final _result = create();
    if (collection != null) {
      _result.collection = collection;
    }
    if (id != null) {
      _result.id = id;
    }
    return _result;
  }
  factory DeleteRequest.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory DeleteRequest.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  DeleteRequest clone() => DeleteRequest()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  DeleteRequest copyWith(void Function(DeleteRequest) updates) => super.copyWith((message) => updates(message as DeleteRequest)) as DeleteRequest; // ignore: deprecated_member_use
  $pb.BuilderInfo get info_ => _i;
  @$core.pragma('dart2js:noInline')
  static DeleteRequest create() => DeleteRequest._();
  DeleteRequest createEmptyInstance() => create();
  static $pb.PbList<DeleteRequest> createRepeated() => $pb.PbList<DeleteRequest>();
  @$core.pragma('dart2js:noInline')
  static DeleteRequest getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<DeleteRequest>(create);
  static DeleteRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get collection => $_getSZ(0);
  @$pb.TagNumber(1)
  set collection($core.String v) { $_setString(0, v); }
  @$pb.TagNumber(1)
  $core.bool hasCollection() => $_has(0);
  @$pb.TagNumber(1)
  void clearCollection() => clearField(1);

  @$pb.TagNumber(2)
  $core.int get id => $_getIZ(1);
  @$pb.TagNumber(2)
  set id($core.int v) { $_setUnsignedInt32(1, v); }
  @$pb.TagNumber(2)
  $core.bool hasId() => $_has(1);
  @$pb.TagNumber(2)
  void clearId() => clearField(2);
}

class DeleteResponse extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'DeleteResponse', package: const $pb.PackageName(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'hyperspace'), createEmptyInstance: create)
    ..aOB(1, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'success')
    ..hasRequiredFields = false
  ;

  DeleteResponse._() : super();
  factory DeleteResponse({
    $core.bool? success,
  }) {
    final _result = create();
    if (success != null) {
      _result.success = success;
    }
    return _result;
  }
  factory DeleteResponse.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory DeleteResponse.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  DeleteResponse clone() => DeleteResponse()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  DeleteResponse copyWith(void Function(DeleteResponse) updates) => super.copyWith((message) => updates(message as DeleteResponse)) as DeleteResponse; // ignore: deprecated_member_use
  $pb.BuilderInfo get info_ => _i;
  @$core.pragma('dart2js:noInline')
  static DeleteResponse create() => DeleteResponse._();
  DeleteResponse createEmptyInstance() => create();
  static $pb.PbList<DeleteResponse> createRepeated() => $pb.PbList<DeleteResponse>();
  @$core.pragma('dart2js:noInline')
  static DeleteResponse getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<DeleteResponse>(create);
  static DeleteResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $core.bool get success => $_getBF(0);
  @$pb.TagNumber(1)
  set success($core.bool v) { $_setBool(0, v); }
  @$pb.TagNumber(1)
  $core.bool hasSuccess() => $_has(0);
  @$pb.TagNumber(1)
  void clearSuccess() => clearField(1);
}

class GetPointsRequest extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'GetPointsRequest', package: const $pb.PackageName(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'hyperspace'), createEmptyInstance: create)
    ..aOS(1, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'collection')
    ..p<$core.int>(2, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'ids', $pb.PbFieldType.KU3)
    ..hasRequiredFields = false
  ;

  GetPointsRequest._() : super();
  factory GetPointsRequest({
    $core.String? collection,
    $core.Iterable<$core.int>? ids,
  }) {
    final _result = create();
    if (collection != null) {
      _result.collection = collection;
    }
    if (ids != null) {
      _result.ids.addAll(ids);
    }
    return _result;
  }
  factory GetPointsRequest.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory GetPointsRequest.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  GetPointsRequest clone() => GetPointsRequest()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  GetPointsRequest copyWith(void Function(GetPointsRequest) updates) => super.copyWith((message) => updates(message as GetPointsRequest)) as GetPointsRequest; // ignore: deprecated_member_use
  $pb.BuilderInfo get info_ => _i;
  @$core.pragma('dart2js:noInline')
  static GetPointsRequest create() => GetPointsRequest._();
  GetPointsRequest createEmptyInstance() => create();
  static $pb.PbList<GetPointsRequest> createRepeated() => $pb.PbList<GetPointsRequest>();
  @$core.pragma('dart2js:noInline')
  static GetPointsRequest getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<GetPointsRequest>(create);
  static GetPointsRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get collection => $_getSZ(0);
  @$pb.TagNumber(1)
  set collection($core.String v) { $_setString(0, v); }
  @$pb.TagNumber(1)
  $core.bool hasCollection() => $_has(0);
  @$pb.TagNumber(1)
  void clearCollection() => clearField(1);

  @$pb.TagNumber(2)
  $core.List<$core.int> get ids => $_getList(1);
}

class GetPointsResponse extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'GetPointsResponse', package: const $pb.PackageName(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'hyperspace'), createEmptyInstance: create)
    ..pc<VectorData>(1, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'points', $pb.PbFieldType.PM, subBuilder: VectorData.create)
    ..hasRequiredFields = false
  ;

  GetPointsResponse._() : super();
  factory GetPointsResponse({
    $core.Iterable<VectorData>? points,
  }) {
    final _result = create();
    if (points != null) {
      _result.points.addAll(points);
    }
    return _result;
  }
  factory GetPointsResponse.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory GetPointsResponse.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  GetPointsResponse clone() => GetPointsResponse()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  GetPointsResponse copyWith(void Function(GetPointsResponse) updates) => super.copyWith((message) => updates(message as GetPointsResponse)) as GetPointsResponse; // ignore: deprecated_member_use
  $pb.BuilderInfo get info_ => _i;
  @$core.pragma('dart2js:noInline')
  static GetPointsResponse create() => GetPointsResponse._();
  GetPointsResponse createEmptyInstance() => create();
  static $pb.PbList<GetPointsResponse> createRepeated() => $pb.PbList<GetPointsResponse>();
  @$core.pragma('dart2js:noInline')
  static GetPointsResponse getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<GetPointsResponse>(create);
  static GetPointsResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $core.List<VectorData> get points => $_getList(0);
}

class UpdatePayloadRequest extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'UpdatePayloadRequest', package: const $pb.PackageName(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'hyperspace'), createEmptyInstance: create)
    ..aOS(1, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'collection')
    ..a<$core.int>(2, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'id', $pb.PbFieldType.OU3)
    ..m<$core.String, $core.String>(3, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'metadata', entryClassName: 'UpdatePayloadRequest.MetadataEntry', keyFieldType: $pb.PbFieldType.OS, valueFieldType: $pb.PbFieldType.OS, packageName: const $pb.PackageName('hyperspace'))
    ..m<$core.String, MetadataValue>(4, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'typedMetadata', entryClassName: 'UpdatePayloadRequest.TypedMetadataEntry', keyFieldType: $pb.PbFieldType.OS, valueFieldType: $pb.PbFieldType.OM, valueCreator: MetadataValue.create, packageName: const $pb.PackageName('hyperspace'))
    ..hasRequiredFields = false
  ;

  UpdatePayloadRequest._() : super();
  factory UpdatePayloadRequest({
    $core.String? collection,
    $core.int? id,
    $core.Map<$core.String, $core.String>? metadata,
    $core.Map<$core.String, MetadataValue>? typedMetadata,
  }) {
    final _result = create();
    if (collection != null) {
      _result.collection = collection;
    }
    if (id != null) {
      _result.id = id;
    }
    if (metadata != null) {
      _result.metadata.addAll(metadata);
    }
    if (typedMetadata != null) {
      _result.typedMetadata.addAll(typedMetadata);
    }
    return _result;
  }
  factory UpdatePayloadRequest.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory UpdatePayloadRequest.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  UpdatePayloadRequest clone() => UpdatePayloadRequest()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  UpdatePayloadRequest copyWith(void Function(UpdatePayloadRequest) updates) => super.copyWith((message) => updates(message as UpdatePayloadRequest)) as UpdatePayloadRequest; // ignore: deprecated_member_use
  $pb.BuilderInfo get info_ => _i;
  @$core.pragma('dart2js:noInline')
  static UpdatePayloadRequest create() => UpdatePayloadRequest._();
  UpdatePayloadRequest createEmptyInstance() => create();
  static $pb.PbList<UpdatePayloadRequest> createRepeated() => $pb.PbList<UpdatePayloadRequest>();
  @$core.pragma('dart2js:noInline')
  static UpdatePayloadRequest getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<UpdatePayloadRequest>(create);
  static UpdatePayloadRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get collection => $_getSZ(0);
  @$pb.TagNumber(1)
  set collection($core.String v) { $_setString(0, v); }
  @$pb.TagNumber(1)
  $core.bool hasCollection() => $_has(0);
  @$pb.TagNumber(1)
  void clearCollection() => clearField(1);

  @$pb.TagNumber(2)
  $core.int get id => $_getIZ(1);
  @$pb.TagNumber(2)
  set id($core.int v) { $_setUnsignedInt32(1, v); }
  @$pb.TagNumber(2)
  $core.bool hasId() => $_has(1);
  @$pb.TagNumber(2)
  void clearId() => clearField(2);

  @$pb.TagNumber(3)
  $core.Map<$core.String, $core.String> get metadata => $_getMap(2);

  @$pb.TagNumber(4)
  $core.Map<$core.String, MetadataValue> get typedMetadata => $_getMap(3);
}

class ScrollRequest extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'ScrollRequest', package: const $pb.PackageName(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'hyperspace'), createEmptyInstance: create)
    ..aOS(1, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'collection')
    ..a<$core.int>(2, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'limit', $pb.PbFieldType.OU3)
    ..a<$core.int>(3, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'offset', $pb.PbFieldType.OU3)
    ..pc<Filter>(4, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'filters', $pb.PbFieldType.PM, subBuilder: Filter.create)
    ..hasRequiredFields = false
  ;

  ScrollRequest._() : super();
  factory ScrollRequest({
    $core.String? collection,
    $core.int? limit,
    $core.int? offset,
    $core.Iterable<Filter>? filters,
  }) {
    final _result = create();
    if (collection != null) {
      _result.collection = collection;
    }
    if (limit != null) {
      _result.limit = limit;
    }
    if (offset != null) {
      _result.offset = offset;
    }
    if (filters != null) {
      _result.filters.addAll(filters);
    }
    return _result;
  }
  factory ScrollRequest.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory ScrollRequest.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  ScrollRequest clone() => ScrollRequest()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  ScrollRequest copyWith(void Function(ScrollRequest) updates) => super.copyWith((message) => updates(message as ScrollRequest)) as ScrollRequest; // ignore: deprecated_member_use
  $pb.BuilderInfo get info_ => _i;
  @$core.pragma('dart2js:noInline')
  static ScrollRequest create() => ScrollRequest._();
  ScrollRequest createEmptyInstance() => create();
  static $pb.PbList<ScrollRequest> createRepeated() => $pb.PbList<ScrollRequest>();
  @$core.pragma('dart2js:noInline')
  static ScrollRequest getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<ScrollRequest>(create);
  static ScrollRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get collection => $_getSZ(0);
  @$pb.TagNumber(1)
  set collection($core.String v) { $_setString(0, v); }
  @$pb.TagNumber(1)
  $core.bool hasCollection() => $_has(0);
  @$pb.TagNumber(1)
  void clearCollection() => clearField(1);

  @$pb.TagNumber(2)
  $core.int get limit => $_getIZ(1);
  @$pb.TagNumber(2)
  set limit($core.int v) { $_setUnsignedInt32(1, v); }
  @$pb.TagNumber(2)
  $core.bool hasLimit() => $_has(1);
  @$pb.TagNumber(2)
  void clearLimit() => clearField(2);

  @$pb.TagNumber(3)
  $core.int get offset => $_getIZ(2);
  @$pb.TagNumber(3)
  set offset($core.int v) { $_setUnsignedInt32(2, v); }
  @$pb.TagNumber(3)
  $core.bool hasOffset() => $_has(2);
  @$pb.TagNumber(3)
  void clearOffset() => clearField(3);

  @$pb.TagNumber(4)
  $core.List<Filter> get filters => $_getList(3);
}

class ScrollResponse extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'ScrollResponse', package: const $pb.PackageName(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'hyperspace'), createEmptyInstance: create)
    ..pc<VectorData>(1, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'points', $pb.PbFieldType.PM, subBuilder: VectorData.create)
    ..hasRequiredFields = false
  ;

  ScrollResponse._() : super();
  factory ScrollResponse({
    $core.Iterable<VectorData>? points,
  }) {
    final _result = create();
    if (points != null) {
      _result.points.addAll(points);
    }
    return _result;
  }
  factory ScrollResponse.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory ScrollResponse.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  ScrollResponse clone() => ScrollResponse()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  ScrollResponse copyWith(void Function(ScrollResponse) updates) => super.copyWith((message) => updates(message as ScrollResponse)) as ScrollResponse; // ignore: deprecated_member_use
  $pb.BuilderInfo get info_ => _i;
  @$core.pragma('dart2js:noInline')
  static ScrollResponse create() => ScrollResponse._();
  ScrollResponse createEmptyInstance() => create();
  static $pb.PbList<ScrollResponse> createRepeated() => $pb.PbList<ScrollResponse>();
  @$core.pragma('dart2js:noInline')
  static ScrollResponse getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<ScrollResponse>(create);
  static ScrollResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $core.List<VectorData> get points => $_getList(0);
}

class CountRequest extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'CountRequest', package: const $pb.PackageName(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'hyperspace'), createEmptyInstance: create)
    ..aOS(1, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'collection')
    ..pc<Filter>(2, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'filters', $pb.PbFieldType.PM, subBuilder: Filter.create)
    ..hasRequiredFields = false
  ;

  CountRequest._() : super();
  factory CountRequest({
    $core.String? collection,
    $core.Iterable<Filter>? filters,
  }) {
    final _result = create();
    if (collection != null) {
      _result.collection = collection;
    }
    if (filters != null) {
      _result.filters.addAll(filters);
    }
    return _result;
  }
  factory CountRequest.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory CountRequest.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  CountRequest clone() => CountRequest()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  CountRequest copyWith(void Function(CountRequest) updates) => super.copyWith((message) => updates(message as CountRequest)) as CountRequest; // ignore: deprecated_member_use
  $pb.BuilderInfo get info_ => _i;
  @$core.pragma('dart2js:noInline')
  static CountRequest create() => CountRequest._();
  CountRequest createEmptyInstance() => create();
  static $pb.PbList<CountRequest> createRepeated() => $pb.PbList<CountRequest>();
  @$core.pragma('dart2js:noInline')
  static CountRequest getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<CountRequest>(create);
  static CountRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get collection => $_getSZ(0);
  @$pb.TagNumber(1)
  set collection($core.String v) { $_setString(0, v); }
  @$pb.TagNumber(1)
  $core.bool hasCollection() => $_has(0);
  @$pb.TagNumber(1)
  void clearCollection() => clearField(1);

  @$pb.TagNumber(2)
  $core.List<Filter> get filters => $_getList(1);
}

class CountResponse extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'CountResponse', package: const $pb.PackageName(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'hyperspace'), createEmptyInstance: create)
    ..a<$fixnum.Int64>(1, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'count', $pb.PbFieldType.OU6, defaultOrMaker: $fixnum.Int64.ZERO)
    ..hasRequiredFields = false
  ;

  CountResponse._() : super();
  factory CountResponse({
    $fixnum.Int64? count,
  }) {
    final _result = create();
    if (count != null) {
      _result.count = count;
    }
    return _result;
  }
  factory CountResponse.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory CountResponse.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  CountResponse clone() => CountResponse()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  CountResponse copyWith(void Function(CountResponse) updates) => super.copyWith((message) => updates(message as CountResponse)) as CountResponse; // ignore: deprecated_member_use
  $pb.BuilderInfo get info_ => _i;
  @$core.pragma('dart2js:noInline')
  static CountResponse create() => CountResponse._();
  CountResponse createEmptyInstance() => create();
  static $pb.PbList<CountResponse> createRepeated() => $pb.PbList<CountResponse>();
  @$core.pragma('dart2js:noInline')
  static CountResponse getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<CountResponse>(create);
  static CountResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $fixnum.Int64 get count => $_getI64(0);
  @$pb.TagNumber(1)
  set count($fixnum.Int64 v) { $_setInt64(0, v); }
  @$pb.TagNumber(1)
  $core.bool hasCount() => $_has(0);
  @$pb.TagNumber(1)
  void clearCount() => clearField(1);
}

class HealthCheckResponse extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'HealthCheckResponse', package: const $pb.PackageName(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'hyperspace'), createEmptyInstance: create)
    ..aOS(1, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'status')
    ..hasRequiredFields = false
  ;

  HealthCheckResponse._() : super();
  factory HealthCheckResponse({
    $core.String? status,
  }) {
    final _result = create();
    if (status != null) {
      _result.status = status;
    }
    return _result;
  }
  factory HealthCheckResponse.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory HealthCheckResponse.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  HealthCheckResponse clone() => HealthCheckResponse()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  HealthCheckResponse copyWith(void Function(HealthCheckResponse) updates) => super.copyWith((message) => updates(message as HealthCheckResponse)) as HealthCheckResponse; // ignore: deprecated_member_use
  $pb.BuilderInfo get info_ => _i;
  @$core.pragma('dart2js:noInline')
  static HealthCheckResponse create() => HealthCheckResponse._();
  HealthCheckResponse createEmptyInstance() => create();
  static $pb.PbList<HealthCheckResponse> createRepeated() => $pb.PbList<HealthCheckResponse>();
  @$core.pragma('dart2js:noInline')
  static HealthCheckResponse getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<HealthCheckResponse>(create);
  static HealthCheckResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get status => $_getSZ(0);
  @$pb.TagNumber(1)
  set status($core.String v) { $_setString(0, v); }
  @$pb.TagNumber(1)
  $core.bool hasStatus() => $_has(0);
  @$pb.TagNumber(1)
  void clearStatus() => clearField(1);
}

class SearchRequest extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'SearchRequest', package: const $pb.PackageName(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'hyperspace'), createEmptyInstance: create)
    ..aOS(1, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'collection')
    ..p<$core.double>(2, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'vector', $pb.PbFieldType.KD)
    ..a<$core.int>(3, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'topK', $pb.PbFieldType.OU3)
    ..m<$core.String, $core.String>(4, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'filter', entryClassName: 'SearchRequest.FilterEntry', keyFieldType: $pb.PbFieldType.OS, valueFieldType: $pb.PbFieldType.OS, packageName: const $pb.PackageName('hyperspace'))
    ..pc<Filter>(5, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'filters', $pb.PbFieldType.PM, subBuilder: Filter.create)
    ..aOS(6, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'hybridQuery')
    ..a<$core.double>(7, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'hybridAlpha', $pb.PbFieldType.OF)
    ..aOB(8, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'useWasserstein')
    ..aOM<Bm25Options>(9, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'bm25Options', subBuilder: Bm25Options.create)
    ..a<$core.int>(10, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'mrlDimension', $pb.PbFieldType.OU3)
    ..aOB(11, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'includePayload')
    ..m<$core.String, $core.double>(12, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'componentWeights', entryClassName: 'SearchRequest.ComponentWeightsEntry', keyFieldType: $pb.PbFieldType.OS, valueFieldType: $pb.PbFieldType.OF, packageName: const $pb.PackageName('hyperspace'))
    ..hasRequiredFields = false
  ;

  SearchRequest._() : super();
  factory SearchRequest({
    $core.String? collection,
    $core.Iterable<$core.double>? vector,
    $core.int? topK,
    $core.Map<$core.String, $core.String>? filter,
    $core.Iterable<Filter>? filters,
    $core.String? hybridQuery,
    $core.double? hybridAlpha,
    $core.bool? useWasserstein,
    Bm25Options? bm25Options,
    $core.int? mrlDimension,
    $core.bool? includePayload,
    $core.Map<$core.String, $core.double>? componentWeights,
  }) {
    final _result = create();
    if (collection != null) {
      _result.collection = collection;
    }
    if (vector != null) {
      _result.vector.addAll(vector);
    }
    if (topK != null) {
      _result.topK = topK;
    }
    if (filter != null) {
      _result.filter.addAll(filter);
    }
    if (filters != null) {
      _result.filters.addAll(filters);
    }
    if (hybridQuery != null) {
      _result.hybridQuery = hybridQuery;
    }
    if (hybridAlpha != null) {
      _result.hybridAlpha = hybridAlpha;
    }
    if (useWasserstein != null) {
      _result.useWasserstein = useWasserstein;
    }
    if (bm25Options != null) {
      _result.bm25Options = bm25Options;
    }
    if (mrlDimension != null) {
      _result.mrlDimension = mrlDimension;
    }
    if (includePayload != null) {
      _result.includePayload = includePayload;
    }
    if (componentWeights != null) {
      _result.componentWeights.addAll(componentWeights);
    }
    return _result;
  }
  factory SearchRequest.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory SearchRequest.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  SearchRequest clone() => SearchRequest()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  SearchRequest copyWith(void Function(SearchRequest) updates) => super.copyWith((message) => updates(message as SearchRequest)) as SearchRequest; // ignore: deprecated_member_use
  $pb.BuilderInfo get info_ => _i;
  @$core.pragma('dart2js:noInline')
  static SearchRequest create() => SearchRequest._();
  SearchRequest createEmptyInstance() => create();
  static $pb.PbList<SearchRequest> createRepeated() => $pb.PbList<SearchRequest>();
  @$core.pragma('dart2js:noInline')
  static SearchRequest getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<SearchRequest>(create);
  static SearchRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get collection => $_getSZ(0);
  @$pb.TagNumber(1)
  set collection($core.String v) { $_setString(0, v); }
  @$pb.TagNumber(1)
  $core.bool hasCollection() => $_has(0);
  @$pb.TagNumber(1)
  void clearCollection() => clearField(1);

  @$pb.TagNumber(2)
  $core.List<$core.double> get vector => $_getList(1);

  @$pb.TagNumber(3)
  $core.int get topK => $_getIZ(2);
  @$pb.TagNumber(3)
  set topK($core.int v) { $_setUnsignedInt32(2, v); }
  @$pb.TagNumber(3)
  $core.bool hasTopK() => $_has(2);
  @$pb.TagNumber(3)
  void clearTopK() => clearField(3);

  @$pb.TagNumber(4)
  $core.Map<$core.String, $core.String> get filter => $_getMap(3);

  @$pb.TagNumber(5)
  $core.List<Filter> get filters => $_getList(4);

  @$pb.TagNumber(6)
  $core.String get hybridQuery => $_getSZ(5);
  @$pb.TagNumber(6)
  set hybridQuery($core.String v) { $_setString(5, v); }
  @$pb.TagNumber(6)
  $core.bool hasHybridQuery() => $_has(5);
  @$pb.TagNumber(6)
  void clearHybridQuery() => clearField(6);

  @$pb.TagNumber(7)
  $core.double get hybridAlpha => $_getN(6);
  @$pb.TagNumber(7)
  set hybridAlpha($core.double v) { $_setFloat(6, v); }
  @$pb.TagNumber(7)
  $core.bool hasHybridAlpha() => $_has(6);
  @$pb.TagNumber(7)
  void clearHybridAlpha() => clearField(7);

  @$pb.TagNumber(8)
  $core.bool get useWasserstein => $_getBF(7);
  @$pb.TagNumber(8)
  set useWasserstein($core.bool v) { $_setBool(7, v); }
  @$pb.TagNumber(8)
  $core.bool hasUseWasserstein() => $_has(7);
  @$pb.TagNumber(8)
  void clearUseWasserstein() => clearField(8);

  @$pb.TagNumber(9)
  Bm25Options get bm25Options => $_getN(8);
  @$pb.TagNumber(9)
  set bm25Options(Bm25Options v) { setField(9, v); }
  @$pb.TagNumber(9)
  $core.bool hasBm25Options() => $_has(8);
  @$pb.TagNumber(9)
  void clearBm25Options() => clearField(9);
  @$pb.TagNumber(9)
  Bm25Options ensureBm25Options() => $_ensure(8);

  @$pb.TagNumber(10)
  $core.int get mrlDimension => $_getIZ(9);
  @$pb.TagNumber(10)
  set mrlDimension($core.int v) { $_setUnsignedInt32(9, v); }
  @$pb.TagNumber(10)
  $core.bool hasMrlDimension() => $_has(9);
  @$pb.TagNumber(10)
  void clearMrlDimension() => clearField(10);

  @$pb.TagNumber(11)
  $core.bool get includePayload => $_getBF(10);
  @$pb.TagNumber(11)
  set includePayload($core.bool v) { $_setBool(10, v); }
  @$pb.TagNumber(11)
  $core.bool hasIncludePayload() => $_has(10);
  @$pb.TagNumber(11)
  void clearIncludePayload() => clearField(11);

  @$pb.TagNumber(12)
  $core.Map<$core.String, $core.double> get componentWeights => $_getMap(11);
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
  notSet
}

class Filter extends $pb.GeneratedMessage {
  static const $core.Map<$core.int, Filter_Condition> _Filter_ConditionByTag = {
    1 : Filter_Condition.match,
    2 : Filter_Condition.range,
    3 : Filter_Condition.inCone,
    4 : Filter_Condition.inBox,
    5 : Filter_Condition.inBall,
    6 : Filter_Condition.andOp,
    7 : Filter_Condition.orOp,
    8 : Filter_Condition.notOp,
    0 : Filter_Condition.notSet
  };
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'Filter', package: const $pb.PackageName(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'hyperspace'), createEmptyInstance: create)
    ..oo(0, [1, 2, 3, 4, 5, 6, 7, 8])
    ..aOM<Match>(1, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'match', subBuilder: Match.create)
    ..aOM<Range>(2, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'range', subBuilder: Range.create)
    ..aOM<InCone>(3, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'inCone', subBuilder: InCone.create)
    ..aOM<InBox>(4, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'inBox', subBuilder: InBox.create)
    ..aOM<InBall>(5, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'inBall', subBuilder: InBall.create)
    ..aOM<FilterAnd>(6, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'andOp', subBuilder: FilterAnd.create)
    ..aOM<FilterOr>(7, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'orOp', subBuilder: FilterOr.create)
    ..aOM<FilterNot>(8, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'notOp', subBuilder: FilterNot.create)
    ..hasRequiredFields = false
  ;

  Filter._() : super();
  factory Filter({
    Match? match,
    Range? range,
    InCone? inCone,
    InBox? inBox,
    InBall? inBall,
    FilterAnd? andOp,
    FilterOr? orOp,
    FilterNot? notOp,
  }) {
    final _result = create();
    if (match != null) {
      _result.match = match;
    }
    if (range != null) {
      _result.range = range;
    }
    if (inCone != null) {
      _result.inCone = inCone;
    }
    if (inBox != null) {
      _result.inBox = inBox;
    }
    if (inBall != null) {
      _result.inBall = inBall;
    }
    if (andOp != null) {
      _result.andOp = andOp;
    }
    if (orOp != null) {
      _result.orOp = orOp;
    }
    if (notOp != null) {
      _result.notOp = notOp;
    }
    return _result;
  }
  factory Filter.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory Filter.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  Filter clone() => Filter()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  Filter copyWith(void Function(Filter) updates) => super.copyWith((message) => updates(message as Filter)) as Filter; // ignore: deprecated_member_use
  $pb.BuilderInfo get info_ => _i;
  @$core.pragma('dart2js:noInline')
  static Filter create() => Filter._();
  Filter createEmptyInstance() => create();
  static $pb.PbList<Filter> createRepeated() => $pb.PbList<Filter>();
  @$core.pragma('dart2js:noInline')
  static Filter getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<Filter>(create);
  static Filter? _defaultInstance;

  Filter_Condition whichCondition() => _Filter_ConditionByTag[$_whichOneof(0)]!;
  void clearCondition() => clearField($_whichOneof(0));

  @$pb.TagNumber(1)
  Match get match => $_getN(0);
  @$pb.TagNumber(1)
  set match(Match v) { setField(1, v); }
  @$pb.TagNumber(1)
  $core.bool hasMatch() => $_has(0);
  @$pb.TagNumber(1)
  void clearMatch() => clearField(1);
  @$pb.TagNumber(1)
  Match ensureMatch() => $_ensure(0);

  @$pb.TagNumber(2)
  Range get range => $_getN(1);
  @$pb.TagNumber(2)
  set range(Range v) { setField(2, v); }
  @$pb.TagNumber(2)
  $core.bool hasRange() => $_has(1);
  @$pb.TagNumber(2)
  void clearRange() => clearField(2);
  @$pb.TagNumber(2)
  Range ensureRange() => $_ensure(1);

  @$pb.TagNumber(3)
  InCone get inCone => $_getN(2);
  @$pb.TagNumber(3)
  set inCone(InCone v) { setField(3, v); }
  @$pb.TagNumber(3)
  $core.bool hasInCone() => $_has(2);
  @$pb.TagNumber(3)
  void clearInCone() => clearField(3);
  @$pb.TagNumber(3)
  InCone ensureInCone() => $_ensure(2);

  @$pb.TagNumber(4)
  InBox get inBox => $_getN(3);
  @$pb.TagNumber(4)
  set inBox(InBox v) { setField(4, v); }
  @$pb.TagNumber(4)
  $core.bool hasInBox() => $_has(3);
  @$pb.TagNumber(4)
  void clearInBox() => clearField(4);
  @$pb.TagNumber(4)
  InBox ensureInBox() => $_ensure(3);

  @$pb.TagNumber(5)
  InBall get inBall => $_getN(4);
  @$pb.TagNumber(5)
  set inBall(InBall v) { setField(5, v); }
  @$pb.TagNumber(5)
  $core.bool hasInBall() => $_has(4);
  @$pb.TagNumber(5)
  void clearInBall() => clearField(5);
  @$pb.TagNumber(5)
  InBall ensureInBall() => $_ensure(4);

  @$pb.TagNumber(6)
  FilterAnd get andOp => $_getN(5);
  @$pb.TagNumber(6)
  set andOp(FilterAnd v) { setField(6, v); }
  @$pb.TagNumber(6)
  $core.bool hasAndOp() => $_has(5);
  @$pb.TagNumber(6)
  void clearAndOp() => clearField(6);
  @$pb.TagNumber(6)
  FilterAnd ensureAndOp() => $_ensure(5);

  @$pb.TagNumber(7)
  FilterOr get orOp => $_getN(6);
  @$pb.TagNumber(7)
  set orOp(FilterOr v) { setField(7, v); }
  @$pb.TagNumber(7)
  $core.bool hasOrOp() => $_has(6);
  @$pb.TagNumber(7)
  void clearOrOp() => clearField(7);
  @$pb.TagNumber(7)
  FilterOr ensureOrOp() => $_ensure(6);

  @$pb.TagNumber(8)
  FilterNot get notOp => $_getN(7);
  @$pb.TagNumber(8)
  set notOp(FilterNot v) { setField(8, v); }
  @$pb.TagNumber(8)
  $core.bool hasNotOp() => $_has(7);
  @$pb.TagNumber(8)
  void clearNotOp() => clearField(8);
  @$pb.TagNumber(8)
  FilterNot ensureNotOp() => $_ensure(7);
}

class FilterAnd extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'FilterAnd', package: const $pb.PackageName(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'hyperspace'), createEmptyInstance: create)
    ..pc<Filter>(1, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'conditions', $pb.PbFieldType.PM, subBuilder: Filter.create)
    ..hasRequiredFields = false
  ;

  FilterAnd._() : super();
  factory FilterAnd({
    $core.Iterable<Filter>? conditions,
  }) {
    final _result = create();
    if (conditions != null) {
      _result.conditions.addAll(conditions);
    }
    return _result;
  }
  factory FilterAnd.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory FilterAnd.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  FilterAnd clone() => FilterAnd()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  FilterAnd copyWith(void Function(FilterAnd) updates) => super.copyWith((message) => updates(message as FilterAnd)) as FilterAnd; // ignore: deprecated_member_use
  $pb.BuilderInfo get info_ => _i;
  @$core.pragma('dart2js:noInline')
  static FilterAnd create() => FilterAnd._();
  FilterAnd createEmptyInstance() => create();
  static $pb.PbList<FilterAnd> createRepeated() => $pb.PbList<FilterAnd>();
  @$core.pragma('dart2js:noInline')
  static FilterAnd getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<FilterAnd>(create);
  static FilterAnd? _defaultInstance;

  @$pb.TagNumber(1)
  $core.List<Filter> get conditions => $_getList(0);
}

class FilterOr extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'FilterOr', package: const $pb.PackageName(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'hyperspace'), createEmptyInstance: create)
    ..pc<Filter>(1, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'conditions', $pb.PbFieldType.PM, subBuilder: Filter.create)
    ..hasRequiredFields = false
  ;

  FilterOr._() : super();
  factory FilterOr({
    $core.Iterable<Filter>? conditions,
  }) {
    final _result = create();
    if (conditions != null) {
      _result.conditions.addAll(conditions);
    }
    return _result;
  }
  factory FilterOr.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory FilterOr.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  FilterOr clone() => FilterOr()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  FilterOr copyWith(void Function(FilterOr) updates) => super.copyWith((message) => updates(message as FilterOr)) as FilterOr; // ignore: deprecated_member_use
  $pb.BuilderInfo get info_ => _i;
  @$core.pragma('dart2js:noInline')
  static FilterOr create() => FilterOr._();
  FilterOr createEmptyInstance() => create();
  static $pb.PbList<FilterOr> createRepeated() => $pb.PbList<FilterOr>();
  @$core.pragma('dart2js:noInline')
  static FilterOr getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<FilterOr>(create);
  static FilterOr? _defaultInstance;

  @$pb.TagNumber(1)
  $core.List<Filter> get conditions => $_getList(0);
}

class FilterNot extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'FilterNot', package: const $pb.PackageName(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'hyperspace'), createEmptyInstance: create)
    ..aOM<Filter>(1, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'condition', subBuilder: Filter.create)
    ..hasRequiredFields = false
  ;

  FilterNot._() : super();
  factory FilterNot({
    Filter? condition,
  }) {
    final _result = create();
    if (condition != null) {
      _result.condition = condition;
    }
    return _result;
  }
  factory FilterNot.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory FilterNot.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  FilterNot clone() => FilterNot()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  FilterNot copyWith(void Function(FilterNot) updates) => super.copyWith((message) => updates(message as FilterNot)) as FilterNot; // ignore: deprecated_member_use
  $pb.BuilderInfo get info_ => _i;
  @$core.pragma('dart2js:noInline')
  static FilterNot create() => FilterNot._();
  FilterNot createEmptyInstance() => create();
  static $pb.PbList<FilterNot> createRepeated() => $pb.PbList<FilterNot>();
  @$core.pragma('dart2js:noInline')
  static FilterNot getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<FilterNot>(create);
  static FilterNot? _defaultInstance;

  @$pb.TagNumber(1)
  Filter get condition => $_getN(0);
  @$pb.TagNumber(1)
  set condition(Filter v) { setField(1, v); }
  @$pb.TagNumber(1)
  $core.bool hasCondition() => $_has(0);
  @$pb.TagNumber(1)
  void clearCondition() => clearField(1);
  @$pb.TagNumber(1)
  Filter ensureCondition() => $_ensure(0);
}

class Match extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'Match', package: const $pb.PackageName(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'hyperspace'), createEmptyInstance: create)
    ..aOS(1, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'key')
    ..aOS(2, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'value')
    ..hasRequiredFields = false
  ;

  Match._() : super();
  factory Match({
    $core.String? key,
    $core.String? value,
  }) {
    final _result = create();
    if (key != null) {
      _result.key = key;
    }
    if (value != null) {
      _result.value = value;
    }
    return _result;
  }
  factory Match.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory Match.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  Match clone() => Match()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  Match copyWith(void Function(Match) updates) => super.copyWith((message) => updates(message as Match)) as Match; // ignore: deprecated_member_use
  $pb.BuilderInfo get info_ => _i;
  @$core.pragma('dart2js:noInline')
  static Match create() => Match._();
  Match createEmptyInstance() => create();
  static $pb.PbList<Match> createRepeated() => $pb.PbList<Match>();
  @$core.pragma('dart2js:noInline')
  static Match getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<Match>(create);
  static Match? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get key => $_getSZ(0);
  @$pb.TagNumber(1)
  set key($core.String v) { $_setString(0, v); }
  @$pb.TagNumber(1)
  $core.bool hasKey() => $_has(0);
  @$pb.TagNumber(1)
  void clearKey() => clearField(1);

  @$pb.TagNumber(2)
  $core.String get value => $_getSZ(1);
  @$pb.TagNumber(2)
  set value($core.String v) { $_setString(1, v); }
  @$pb.TagNumber(2)
  $core.bool hasValue() => $_has(1);
  @$pb.TagNumber(2)
  void clearValue() => clearField(2);
}

class Range extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'Range', package: const $pb.PackageName(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'hyperspace'), createEmptyInstance: create)
    ..aOS(1, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'key')
    ..aInt64(2, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'gte')
    ..aInt64(3, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'lte')
    ..a<$core.double>(4, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'gteF64', $pb.PbFieldType.OD)
    ..a<$core.double>(5, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'lteF64', $pb.PbFieldType.OD)
    ..hasRequiredFields = false
  ;

  Range._() : super();
  factory Range({
    $core.String? key,
    $fixnum.Int64? gte,
    $fixnum.Int64? lte,
    $core.double? gteF64,
    $core.double? lteF64,
  }) {
    final _result = create();
    if (key != null) {
      _result.key = key;
    }
    if (gte != null) {
      _result.gte = gte;
    }
    if (lte != null) {
      _result.lte = lte;
    }
    if (gteF64 != null) {
      _result.gteF64 = gteF64;
    }
    if (lteF64 != null) {
      _result.lteF64 = lteF64;
    }
    return _result;
  }
  factory Range.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory Range.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  Range clone() => Range()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  Range copyWith(void Function(Range) updates) => super.copyWith((message) => updates(message as Range)) as Range; // ignore: deprecated_member_use
  $pb.BuilderInfo get info_ => _i;
  @$core.pragma('dart2js:noInline')
  static Range create() => Range._();
  Range createEmptyInstance() => create();
  static $pb.PbList<Range> createRepeated() => $pb.PbList<Range>();
  @$core.pragma('dart2js:noInline')
  static Range getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<Range>(create);
  static Range? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get key => $_getSZ(0);
  @$pb.TagNumber(1)
  set key($core.String v) { $_setString(0, v); }
  @$pb.TagNumber(1)
  $core.bool hasKey() => $_has(0);
  @$pb.TagNumber(1)
  void clearKey() => clearField(1);

  @$pb.TagNumber(2)
  $fixnum.Int64 get gte => $_getI64(1);
  @$pb.TagNumber(2)
  set gte($fixnum.Int64 v) { $_setInt64(1, v); }
  @$pb.TagNumber(2)
  $core.bool hasGte() => $_has(1);
  @$pb.TagNumber(2)
  void clearGte() => clearField(2);

  @$pb.TagNumber(3)
  $fixnum.Int64 get lte => $_getI64(2);
  @$pb.TagNumber(3)
  set lte($fixnum.Int64 v) { $_setInt64(2, v); }
  @$pb.TagNumber(3)
  $core.bool hasLte() => $_has(2);
  @$pb.TagNumber(3)
  void clearLte() => clearField(3);

  @$pb.TagNumber(4)
  $core.double get gteF64 => $_getN(3);
  @$pb.TagNumber(4)
  set gteF64($core.double v) { $_setDouble(3, v); }
  @$pb.TagNumber(4)
  $core.bool hasGteF64() => $_has(3);
  @$pb.TagNumber(4)
  void clearGteF64() => clearField(4);

  @$pb.TagNumber(5)
  $core.double get lteF64 => $_getN(4);
  @$pb.TagNumber(5)
  set lteF64($core.double v) { $_setDouble(4, v); }
  @$pb.TagNumber(5)
  $core.bool hasLteF64() => $_has(4);
  @$pb.TagNumber(5)
  void clearLteF64() => clearField(5);
}

class InCone extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'InCone', package: const $pb.PackageName(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'hyperspace'), createEmptyInstance: create)
    ..p<$core.double>(1, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'axes', $pb.PbFieldType.KD)
    ..p<$core.double>(2, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'apertures', $pb.PbFieldType.KD)
    ..a<$core.double>(3, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'cen', $pb.PbFieldType.OD)
    ..hasRequiredFields = false
  ;

  InCone._() : super();
  factory InCone({
    $core.Iterable<$core.double>? axes,
    $core.Iterable<$core.double>? apertures,
    $core.double? cen,
  }) {
    final _result = create();
    if (axes != null) {
      _result.axes.addAll(axes);
    }
    if (apertures != null) {
      _result.apertures.addAll(apertures);
    }
    if (cen != null) {
      _result.cen = cen;
    }
    return _result;
  }
  factory InCone.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory InCone.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  InCone clone() => InCone()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  InCone copyWith(void Function(InCone) updates) => super.copyWith((message) => updates(message as InCone)) as InCone; // ignore: deprecated_member_use
  $pb.BuilderInfo get info_ => _i;
  @$core.pragma('dart2js:noInline')
  static InCone create() => InCone._();
  InCone createEmptyInstance() => create();
  static $pb.PbList<InCone> createRepeated() => $pb.PbList<InCone>();
  @$core.pragma('dart2js:noInline')
  static InCone getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<InCone>(create);
  static InCone? _defaultInstance;

  @$pb.TagNumber(1)
  $core.List<$core.double> get axes => $_getList(0);

  @$pb.TagNumber(2)
  $core.List<$core.double> get apertures => $_getList(1);

  @$pb.TagNumber(3)
  $core.double get cen => $_getN(2);
  @$pb.TagNumber(3)
  set cen($core.double v) { $_setDouble(2, v); }
  @$pb.TagNumber(3)
  $core.bool hasCen() => $_has(2);
  @$pb.TagNumber(3)
  void clearCen() => clearField(3);
}

class InBox extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'InBox', package: const $pb.PackageName(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'hyperspace'), createEmptyInstance: create)
    ..p<$core.double>(1, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'minBounds', $pb.PbFieldType.KD)
    ..p<$core.double>(2, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'maxBounds', $pb.PbFieldType.KD)
    ..hasRequiredFields = false
  ;

  InBox._() : super();
  factory InBox({
    $core.Iterable<$core.double>? minBounds,
    $core.Iterable<$core.double>? maxBounds,
  }) {
    final _result = create();
    if (minBounds != null) {
      _result.minBounds.addAll(minBounds);
    }
    if (maxBounds != null) {
      _result.maxBounds.addAll(maxBounds);
    }
    return _result;
  }
  factory InBox.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory InBox.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  InBox clone() => InBox()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  InBox copyWith(void Function(InBox) updates) => super.copyWith((message) => updates(message as InBox)) as InBox; // ignore: deprecated_member_use
  $pb.BuilderInfo get info_ => _i;
  @$core.pragma('dart2js:noInline')
  static InBox create() => InBox._();
  InBox createEmptyInstance() => create();
  static $pb.PbList<InBox> createRepeated() => $pb.PbList<InBox>();
  @$core.pragma('dart2js:noInline')
  static InBox getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<InBox>(create);
  static InBox? _defaultInstance;

  @$pb.TagNumber(1)
  $core.List<$core.double> get minBounds => $_getList(0);

  @$pb.TagNumber(2)
  $core.List<$core.double> get maxBounds => $_getList(1);
}

class InBall extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'InBall', package: const $pb.PackageName(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'hyperspace'), createEmptyInstance: create)
    ..p<$core.double>(1, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'center', $pb.PbFieldType.KD)
    ..a<$core.double>(2, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'radius', $pb.PbFieldType.OD)
    ..hasRequiredFields = false
  ;

  InBall._() : super();
  factory InBall({
    $core.Iterable<$core.double>? center,
    $core.double? radius,
  }) {
    final _result = create();
    if (center != null) {
      _result.center.addAll(center);
    }
    if (radius != null) {
      _result.radius = radius;
    }
    return _result;
  }
  factory InBall.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory InBall.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  InBall clone() => InBall()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  InBall copyWith(void Function(InBall) updates) => super.copyWith((message) => updates(message as InBall)) as InBall; // ignore: deprecated_member_use
  $pb.BuilderInfo get info_ => _i;
  @$core.pragma('dart2js:noInline')
  static InBall create() => InBall._();
  InBall createEmptyInstance() => create();
  static $pb.PbList<InBall> createRepeated() => $pb.PbList<InBall>();
  @$core.pragma('dart2js:noInline')
  static InBall getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<InBall>(create);
  static InBall? _defaultInstance;

  @$pb.TagNumber(1)
  $core.List<$core.double> get center => $_getList(0);

  @$pb.TagNumber(2)
  $core.double get radius => $_getN(1);
  @$pb.TagNumber(2)
  set radius($core.double v) { $_setDouble(1, v); }
  @$pb.TagNumber(2)
  $core.bool hasRadius() => $_has(1);
  @$pb.TagNumber(2)
  void clearRadius() => clearField(2);
}

class SearchResponse extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'SearchResponse', package: const $pb.PackageName(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'hyperspace'), createEmptyInstance: create)
    ..pc<SearchResult>(1, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'results', $pb.PbFieldType.PM, subBuilder: SearchResult.create)
    ..hasRequiredFields = false
  ;

  SearchResponse._() : super();
  factory SearchResponse({
    $core.Iterable<SearchResult>? results,
  }) {
    final _result = create();
    if (results != null) {
      _result.results.addAll(results);
    }
    return _result;
  }
  factory SearchResponse.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory SearchResponse.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  SearchResponse clone() => SearchResponse()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  SearchResponse copyWith(void Function(SearchResponse) updates) => super.copyWith((message) => updates(message as SearchResponse)) as SearchResponse; // ignore: deprecated_member_use
  $pb.BuilderInfo get info_ => _i;
  @$core.pragma('dart2js:noInline')
  static SearchResponse create() => SearchResponse._();
  SearchResponse createEmptyInstance() => create();
  static $pb.PbList<SearchResponse> createRepeated() => $pb.PbList<SearchResponse>();
  @$core.pragma('dart2js:noInline')
  static SearchResponse getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<SearchResponse>(create);
  static SearchResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $core.List<SearchResult> get results => $_getList(0);
}

class BatchSearchRequest extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'BatchSearchRequest', package: const $pb.PackageName(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'hyperspace'), createEmptyInstance: create)
    ..pc<SearchRequest>(1, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'searches', $pb.PbFieldType.PM, subBuilder: SearchRequest.create)
    ..hasRequiredFields = false
  ;

  BatchSearchRequest._() : super();
  factory BatchSearchRequest({
    $core.Iterable<SearchRequest>? searches,
  }) {
    final _result = create();
    if (searches != null) {
      _result.searches.addAll(searches);
    }
    return _result;
  }
  factory BatchSearchRequest.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory BatchSearchRequest.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  BatchSearchRequest clone() => BatchSearchRequest()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  BatchSearchRequest copyWith(void Function(BatchSearchRequest) updates) => super.copyWith((message) => updates(message as BatchSearchRequest)) as BatchSearchRequest; // ignore: deprecated_member_use
  $pb.BuilderInfo get info_ => _i;
  @$core.pragma('dart2js:noInline')
  static BatchSearchRequest create() => BatchSearchRequest._();
  BatchSearchRequest createEmptyInstance() => create();
  static $pb.PbList<BatchSearchRequest> createRepeated() => $pb.PbList<BatchSearchRequest>();
  @$core.pragma('dart2js:noInline')
  static BatchSearchRequest getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<BatchSearchRequest>(create);
  static BatchSearchRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.List<SearchRequest> get searches => $_getList(0);
}

class BatchSearchResponse extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'BatchSearchResponse', package: const $pb.PackageName(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'hyperspace'), createEmptyInstance: create)
    ..pc<SearchResponse>(1, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'responses', $pb.PbFieldType.PM, subBuilder: SearchResponse.create)
    ..hasRequiredFields = false
  ;

  BatchSearchResponse._() : super();
  factory BatchSearchResponse({
    $core.Iterable<SearchResponse>? responses,
  }) {
    final _result = create();
    if (responses != null) {
      _result.responses.addAll(responses);
    }
    return _result;
  }
  factory BatchSearchResponse.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory BatchSearchResponse.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  BatchSearchResponse clone() => BatchSearchResponse()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  BatchSearchResponse copyWith(void Function(BatchSearchResponse) updates) => super.copyWith((message) => updates(message as BatchSearchResponse)) as BatchSearchResponse; // ignore: deprecated_member_use
  $pb.BuilderInfo get info_ => _i;
  @$core.pragma('dart2js:noInline')
  static BatchSearchResponse create() => BatchSearchResponse._();
  BatchSearchResponse createEmptyInstance() => create();
  static $pb.PbList<BatchSearchResponse> createRepeated() => $pb.PbList<BatchSearchResponse>();
  @$core.pragma('dart2js:noInline')
  static BatchSearchResponse getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<BatchSearchResponse>(create);
  static BatchSearchResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $core.List<SearchResponse> get responses => $_getList(0);
}

class SearchMultiCollectionRequest extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'SearchMultiCollectionRequest', package: const $pb.PackageName(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'hyperspace'), createEmptyInstance: create)
    ..pPS(1, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'collections')
    ..p<$core.double>(2, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'vector', $pb.PbFieldType.KD)
    ..a<$core.int>(3, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'topK', $pb.PbFieldType.OU3)
    ..hasRequiredFields = false
  ;

  SearchMultiCollectionRequest._() : super();
  factory SearchMultiCollectionRequest({
    $core.Iterable<$core.String>? collections,
    $core.Iterable<$core.double>? vector,
    $core.int? topK,
  }) {
    final _result = create();
    if (collections != null) {
      _result.collections.addAll(collections);
    }
    if (vector != null) {
      _result.vector.addAll(vector);
    }
    if (topK != null) {
      _result.topK = topK;
    }
    return _result;
  }
  factory SearchMultiCollectionRequest.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory SearchMultiCollectionRequest.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  SearchMultiCollectionRequest clone() => SearchMultiCollectionRequest()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  SearchMultiCollectionRequest copyWith(void Function(SearchMultiCollectionRequest) updates) => super.copyWith((message) => updates(message as SearchMultiCollectionRequest)) as SearchMultiCollectionRequest; // ignore: deprecated_member_use
  $pb.BuilderInfo get info_ => _i;
  @$core.pragma('dart2js:noInline')
  static SearchMultiCollectionRequest create() => SearchMultiCollectionRequest._();
  SearchMultiCollectionRequest createEmptyInstance() => create();
  static $pb.PbList<SearchMultiCollectionRequest> createRepeated() => $pb.PbList<SearchMultiCollectionRequest>();
  @$core.pragma('dart2js:noInline')
  static SearchMultiCollectionRequest getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<SearchMultiCollectionRequest>(create);
  static SearchMultiCollectionRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.List<$core.String> get collections => $_getList(0);

  @$pb.TagNumber(2)
  $core.List<$core.double> get vector => $_getList(1);

  @$pb.TagNumber(3)
  $core.int get topK => $_getIZ(2);
  @$pb.TagNumber(3)
  set topK($core.int v) { $_setUnsignedInt32(2, v); }
  @$pb.TagNumber(3)
  $core.bool hasTopK() => $_has(2);
  @$pb.TagNumber(3)
  void clearTopK() => clearField(3);
}

class SearchMultiCollectionResponse extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'SearchMultiCollectionResponse', package: const $pb.PackageName(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'hyperspace'), createEmptyInstance: create)
    ..m<$core.String, SearchResponse>(1, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'responses', entryClassName: 'SearchMultiCollectionResponse.ResponsesEntry', keyFieldType: $pb.PbFieldType.OS, valueFieldType: $pb.PbFieldType.OM, valueCreator: SearchResponse.create, packageName: const $pb.PackageName('hyperspace'))
    ..hasRequiredFields = false
  ;

  SearchMultiCollectionResponse._() : super();
  factory SearchMultiCollectionResponse({
    $core.Map<$core.String, SearchResponse>? responses,
  }) {
    final _result = create();
    if (responses != null) {
      _result.responses.addAll(responses);
    }
    return _result;
  }
  factory SearchMultiCollectionResponse.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory SearchMultiCollectionResponse.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  SearchMultiCollectionResponse clone() => SearchMultiCollectionResponse()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  SearchMultiCollectionResponse copyWith(void Function(SearchMultiCollectionResponse) updates) => super.copyWith((message) => updates(message as SearchMultiCollectionResponse)) as SearchMultiCollectionResponse; // ignore: deprecated_member_use
  $pb.BuilderInfo get info_ => _i;
  @$core.pragma('dart2js:noInline')
  static SearchMultiCollectionResponse create() => SearchMultiCollectionResponse._();
  SearchMultiCollectionResponse createEmptyInstance() => create();
  static $pb.PbList<SearchMultiCollectionResponse> createRepeated() => $pb.PbList<SearchMultiCollectionResponse>();
  @$core.pragma('dart2js:noInline')
  static SearchMultiCollectionResponse getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<SearchMultiCollectionResponse>(create);
  static SearchMultiCollectionResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $core.Map<$core.String, SearchResponse> get responses => $_getMap(0);
}

class SearchResult extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'SearchResult', package: const $pb.PackageName(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'hyperspace'), createEmptyInstance: create)
    ..a<$core.int>(1, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'id', $pb.PbFieldType.OU3)
    ..a<$core.double>(2, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'distance', $pb.PbFieldType.OD)
    ..m<$core.String, $core.String>(3, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'metadata', entryClassName: 'SearchResult.MetadataEntry', keyFieldType: $pb.PbFieldType.OS, valueFieldType: $pb.PbFieldType.OS, packageName: const $pb.PackageName('hyperspace'))
    ..m<$core.String, MetadataValue>(4, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'typedMetadata', entryClassName: 'SearchResult.TypedMetadataEntry', keyFieldType: $pb.PbFieldType.OS, valueFieldType: $pb.PbFieldType.OM, valueCreator: MetadataValue.create, packageName: const $pb.PackageName('hyperspace'))
    ..a<$core.List<$core.int>>(5, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'payload', $pb.PbFieldType.OY)
    ..hasRequiredFields = false
  ;

  SearchResult._() : super();
  factory SearchResult({
    $core.int? id,
    $core.double? distance,
    $core.Map<$core.String, $core.String>? metadata,
    $core.Map<$core.String, MetadataValue>? typedMetadata,
    $core.List<$core.int>? payload,
  }) {
    final _result = create();
    if (id != null) {
      _result.id = id;
    }
    if (distance != null) {
      _result.distance = distance;
    }
    if (metadata != null) {
      _result.metadata.addAll(metadata);
    }
    if (typedMetadata != null) {
      _result.typedMetadata.addAll(typedMetadata);
    }
    if (payload != null) {
      _result.payload = payload;
    }
    return _result;
  }
  factory SearchResult.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory SearchResult.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  SearchResult clone() => SearchResult()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  SearchResult copyWith(void Function(SearchResult) updates) => super.copyWith((message) => updates(message as SearchResult)) as SearchResult; // ignore: deprecated_member_use
  $pb.BuilderInfo get info_ => _i;
  @$core.pragma('dart2js:noInline')
  static SearchResult create() => SearchResult._();
  SearchResult createEmptyInstance() => create();
  static $pb.PbList<SearchResult> createRepeated() => $pb.PbList<SearchResult>();
  @$core.pragma('dart2js:noInline')
  static SearchResult getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<SearchResult>(create);
  static SearchResult? _defaultInstance;

  @$pb.TagNumber(1)
  $core.int get id => $_getIZ(0);
  @$pb.TagNumber(1)
  set id($core.int v) { $_setUnsignedInt32(0, v); }
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => clearField(1);

  @$pb.TagNumber(2)
  $core.double get distance => $_getN(1);
  @$pb.TagNumber(2)
  set distance($core.double v) { $_setDouble(1, v); }
  @$pb.TagNumber(2)
  $core.bool hasDistance() => $_has(1);
  @$pb.TagNumber(2)
  void clearDistance() => clearField(2);

  @$pb.TagNumber(3)
  $core.Map<$core.String, $core.String> get metadata => $_getMap(2);

  @$pb.TagNumber(4)
  $core.Map<$core.String, MetadataValue> get typedMetadata => $_getMap(3);

  @$pb.TagNumber(5)
  $core.List<$core.int> get payload => $_getN(4);
  @$pb.TagNumber(5)
  set payload($core.List<$core.int> v) { $_setBytes(4, v); }
  @$pb.TagNumber(5)
  $core.bool hasPayload() => $_has(4);
  @$pb.TagNumber(5)
  void clearPayload() => clearField(5);
}

class GetNodeRequest extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'GetNodeRequest', package: const $pb.PackageName(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'hyperspace'), createEmptyInstance: create)
    ..aOS(1, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'collection')
    ..a<$core.int>(2, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'id', $pb.PbFieldType.OU3)
    ..a<$core.int>(3, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'layer', $pb.PbFieldType.OU3)
    ..hasRequiredFields = false
  ;

  GetNodeRequest._() : super();
  factory GetNodeRequest({
    $core.String? collection,
    $core.int? id,
    $core.int? layer,
  }) {
    final _result = create();
    if (collection != null) {
      _result.collection = collection;
    }
    if (id != null) {
      _result.id = id;
    }
    if (layer != null) {
      _result.layer = layer;
    }
    return _result;
  }
  factory GetNodeRequest.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory GetNodeRequest.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  GetNodeRequest clone() => GetNodeRequest()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  GetNodeRequest copyWith(void Function(GetNodeRequest) updates) => super.copyWith((message) => updates(message as GetNodeRequest)) as GetNodeRequest; // ignore: deprecated_member_use
  $pb.BuilderInfo get info_ => _i;
  @$core.pragma('dart2js:noInline')
  static GetNodeRequest create() => GetNodeRequest._();
  GetNodeRequest createEmptyInstance() => create();
  static $pb.PbList<GetNodeRequest> createRepeated() => $pb.PbList<GetNodeRequest>();
  @$core.pragma('dart2js:noInline')
  static GetNodeRequest getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<GetNodeRequest>(create);
  static GetNodeRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get collection => $_getSZ(0);
  @$pb.TagNumber(1)
  set collection($core.String v) { $_setString(0, v); }
  @$pb.TagNumber(1)
  $core.bool hasCollection() => $_has(0);
  @$pb.TagNumber(1)
  void clearCollection() => clearField(1);

  @$pb.TagNumber(2)
  $core.int get id => $_getIZ(1);
  @$pb.TagNumber(2)
  set id($core.int v) { $_setUnsignedInt32(1, v); }
  @$pb.TagNumber(2)
  $core.bool hasId() => $_has(1);
  @$pb.TagNumber(2)
  void clearId() => clearField(2);

  @$pb.TagNumber(3)
  $core.int get layer => $_getIZ(2);
  @$pb.TagNumber(3)
  set layer($core.int v) { $_setUnsignedInt32(2, v); }
  @$pb.TagNumber(3)
  $core.bool hasLayer() => $_has(2);
  @$pb.TagNumber(3)
  void clearLayer() => clearField(3);
}

class GraphNode extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'GraphNode', package: const $pb.PackageName(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'hyperspace'), createEmptyInstance: create)
    ..a<$core.int>(1, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'id', $pb.PbFieldType.OU3)
    ..a<$core.int>(2, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'layer', $pb.PbFieldType.OU3)
    ..p<$core.int>(3, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'neighbors', $pb.PbFieldType.KU3)
    ..m<$core.String, $core.String>(4, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'metadata', entryClassName: 'GraphNode.MetadataEntry', keyFieldType: $pb.PbFieldType.OS, valueFieldType: $pb.PbFieldType.OS, packageName: const $pb.PackageName('hyperspace'))
    ..m<$core.String, MetadataValue>(5, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'typedMetadata', entryClassName: 'GraphNode.TypedMetadataEntry', keyFieldType: $pb.PbFieldType.OS, valueFieldType: $pb.PbFieldType.OM, valueCreator: MetadataValue.create, packageName: const $pb.PackageName('hyperspace'))
    ..hasRequiredFields = false
  ;

  GraphNode._() : super();
  factory GraphNode({
    $core.int? id,
    $core.int? layer,
    $core.Iterable<$core.int>? neighbors,
    $core.Map<$core.String, $core.String>? metadata,
    $core.Map<$core.String, MetadataValue>? typedMetadata,
  }) {
    final _result = create();
    if (id != null) {
      _result.id = id;
    }
    if (layer != null) {
      _result.layer = layer;
    }
    if (neighbors != null) {
      _result.neighbors.addAll(neighbors);
    }
    if (metadata != null) {
      _result.metadata.addAll(metadata);
    }
    if (typedMetadata != null) {
      _result.typedMetadata.addAll(typedMetadata);
    }
    return _result;
  }
  factory GraphNode.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory GraphNode.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  GraphNode clone() => GraphNode()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  GraphNode copyWith(void Function(GraphNode) updates) => super.copyWith((message) => updates(message as GraphNode)) as GraphNode; // ignore: deprecated_member_use
  $pb.BuilderInfo get info_ => _i;
  @$core.pragma('dart2js:noInline')
  static GraphNode create() => GraphNode._();
  GraphNode createEmptyInstance() => create();
  static $pb.PbList<GraphNode> createRepeated() => $pb.PbList<GraphNode>();
  @$core.pragma('dart2js:noInline')
  static GraphNode getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<GraphNode>(create);
  static GraphNode? _defaultInstance;

  @$pb.TagNumber(1)
  $core.int get id => $_getIZ(0);
  @$pb.TagNumber(1)
  set id($core.int v) { $_setUnsignedInt32(0, v); }
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => clearField(1);

  @$pb.TagNumber(2)
  $core.int get layer => $_getIZ(1);
  @$pb.TagNumber(2)
  set layer($core.int v) { $_setUnsignedInt32(1, v); }
  @$pb.TagNumber(2)
  $core.bool hasLayer() => $_has(1);
  @$pb.TagNumber(2)
  void clearLayer() => clearField(2);

  @$pb.TagNumber(3)
  $core.List<$core.int> get neighbors => $_getList(2);

  @$pb.TagNumber(4)
  $core.Map<$core.String, $core.String> get metadata => $_getMap(3);

  @$pb.TagNumber(5)
  $core.Map<$core.String, MetadataValue> get typedMetadata => $_getMap(4);
}

class GetNeighborsRequest extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'GetNeighborsRequest', package: const $pb.PackageName(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'hyperspace'), createEmptyInstance: create)
    ..aOS(1, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'collection')
    ..a<$core.int>(2, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'id', $pb.PbFieldType.OU3)
    ..a<$core.int>(3, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'layer', $pb.PbFieldType.OU3)
    ..a<$core.int>(4, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'limit', $pb.PbFieldType.OU3)
    ..a<$core.int>(5, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'offset', $pb.PbFieldType.OU3)
    ..hasRequiredFields = false
  ;

  GetNeighborsRequest._() : super();
  factory GetNeighborsRequest({
    $core.String? collection,
    $core.int? id,
    $core.int? layer,
    $core.int? limit,
    $core.int? offset,
  }) {
    final _result = create();
    if (collection != null) {
      _result.collection = collection;
    }
    if (id != null) {
      _result.id = id;
    }
    if (layer != null) {
      _result.layer = layer;
    }
    if (limit != null) {
      _result.limit = limit;
    }
    if (offset != null) {
      _result.offset = offset;
    }
    return _result;
  }
  factory GetNeighborsRequest.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory GetNeighborsRequest.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  GetNeighborsRequest clone() => GetNeighborsRequest()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  GetNeighborsRequest copyWith(void Function(GetNeighborsRequest) updates) => super.copyWith((message) => updates(message as GetNeighborsRequest)) as GetNeighborsRequest; // ignore: deprecated_member_use
  $pb.BuilderInfo get info_ => _i;
  @$core.pragma('dart2js:noInline')
  static GetNeighborsRequest create() => GetNeighborsRequest._();
  GetNeighborsRequest createEmptyInstance() => create();
  static $pb.PbList<GetNeighborsRequest> createRepeated() => $pb.PbList<GetNeighborsRequest>();
  @$core.pragma('dart2js:noInline')
  static GetNeighborsRequest getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<GetNeighborsRequest>(create);
  static GetNeighborsRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get collection => $_getSZ(0);
  @$pb.TagNumber(1)
  set collection($core.String v) { $_setString(0, v); }
  @$pb.TagNumber(1)
  $core.bool hasCollection() => $_has(0);
  @$pb.TagNumber(1)
  void clearCollection() => clearField(1);

  @$pb.TagNumber(2)
  $core.int get id => $_getIZ(1);
  @$pb.TagNumber(2)
  set id($core.int v) { $_setUnsignedInt32(1, v); }
  @$pb.TagNumber(2)
  $core.bool hasId() => $_has(1);
  @$pb.TagNumber(2)
  void clearId() => clearField(2);

  @$pb.TagNumber(3)
  $core.int get layer => $_getIZ(2);
  @$pb.TagNumber(3)
  set layer($core.int v) { $_setUnsignedInt32(2, v); }
  @$pb.TagNumber(3)
  $core.bool hasLayer() => $_has(2);
  @$pb.TagNumber(3)
  void clearLayer() => clearField(3);

  @$pb.TagNumber(4)
  $core.int get limit => $_getIZ(3);
  @$pb.TagNumber(4)
  set limit($core.int v) { $_setUnsignedInt32(3, v); }
  @$pb.TagNumber(4)
  $core.bool hasLimit() => $_has(3);
  @$pb.TagNumber(4)
  void clearLimit() => clearField(4);

  @$pb.TagNumber(5)
  $core.int get offset => $_getIZ(4);
  @$pb.TagNumber(5)
  set offset($core.int v) { $_setUnsignedInt32(4, v); }
  @$pb.TagNumber(5)
  $core.bool hasOffset() => $_has(4);
  @$pb.TagNumber(5)
  void clearOffset() => clearField(5);
}

class GetNeighborsResponse extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'GetNeighborsResponse', package: const $pb.PackageName(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'hyperspace'), createEmptyInstance: create)
    ..pc<GraphNode>(1, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'neighbors', $pb.PbFieldType.PM, subBuilder: GraphNode.create)
    ..p<$core.double>(2, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'edgeWeights', $pb.PbFieldType.KD)
    ..hasRequiredFields = false
  ;

  GetNeighborsResponse._() : super();
  factory GetNeighborsResponse({
    $core.Iterable<GraphNode>? neighbors,
    $core.Iterable<$core.double>? edgeWeights,
  }) {
    final _result = create();
    if (neighbors != null) {
      _result.neighbors.addAll(neighbors);
    }
    if (edgeWeights != null) {
      _result.edgeWeights.addAll(edgeWeights);
    }
    return _result;
  }
  factory GetNeighborsResponse.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory GetNeighborsResponse.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  GetNeighborsResponse clone() => GetNeighborsResponse()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  GetNeighborsResponse copyWith(void Function(GetNeighborsResponse) updates) => super.copyWith((message) => updates(message as GetNeighborsResponse)) as GetNeighborsResponse; // ignore: deprecated_member_use
  $pb.BuilderInfo get info_ => _i;
  @$core.pragma('dart2js:noInline')
  static GetNeighborsResponse create() => GetNeighborsResponse._();
  GetNeighborsResponse createEmptyInstance() => create();
  static $pb.PbList<GetNeighborsResponse> createRepeated() => $pb.PbList<GetNeighborsResponse>();
  @$core.pragma('dart2js:noInline')
  static GetNeighborsResponse getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<GetNeighborsResponse>(create);
  static GetNeighborsResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $core.List<GraphNode> get neighbors => $_getList(0);

  @$pb.TagNumber(2)
  $core.List<$core.double> get edgeWeights => $_getList(1);
}

class TraverseRequest extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'TraverseRequest', package: const $pb.PackageName(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'hyperspace'), createEmptyInstance: create)
    ..aOS(1, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'collection')
    ..a<$core.int>(2, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'startId', $pb.PbFieldType.OU3)
    ..a<$core.int>(3, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'maxDepth', $pb.PbFieldType.OU3)
    ..a<$core.int>(4, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'maxNodes', $pb.PbFieldType.OU3)
    ..a<$core.int>(5, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'layer', $pb.PbFieldType.OU3)
    ..m<$core.String, $core.String>(6, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'filter', entryClassName: 'TraverseRequest.FilterEntry', keyFieldType: $pb.PbFieldType.OS, valueFieldType: $pb.PbFieldType.OS, packageName: const $pb.PackageName('hyperspace'))
    ..pc<Filter>(7, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'filters', $pb.PbFieldType.PM, subBuilder: Filter.create)
    ..hasRequiredFields = false
  ;

  TraverseRequest._() : super();
  factory TraverseRequest({
    $core.String? collection,
    $core.int? startId,
    $core.int? maxDepth,
    $core.int? maxNodes,
    $core.int? layer,
    $core.Map<$core.String, $core.String>? filter,
    $core.Iterable<Filter>? filters,
  }) {
    final _result = create();
    if (collection != null) {
      _result.collection = collection;
    }
    if (startId != null) {
      _result.startId = startId;
    }
    if (maxDepth != null) {
      _result.maxDepth = maxDepth;
    }
    if (maxNodes != null) {
      _result.maxNodes = maxNodes;
    }
    if (layer != null) {
      _result.layer = layer;
    }
    if (filter != null) {
      _result.filter.addAll(filter);
    }
    if (filters != null) {
      _result.filters.addAll(filters);
    }
    return _result;
  }
  factory TraverseRequest.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory TraverseRequest.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  TraverseRequest clone() => TraverseRequest()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  TraverseRequest copyWith(void Function(TraverseRequest) updates) => super.copyWith((message) => updates(message as TraverseRequest)) as TraverseRequest; // ignore: deprecated_member_use
  $pb.BuilderInfo get info_ => _i;
  @$core.pragma('dart2js:noInline')
  static TraverseRequest create() => TraverseRequest._();
  TraverseRequest createEmptyInstance() => create();
  static $pb.PbList<TraverseRequest> createRepeated() => $pb.PbList<TraverseRequest>();
  @$core.pragma('dart2js:noInline')
  static TraverseRequest getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<TraverseRequest>(create);
  static TraverseRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get collection => $_getSZ(0);
  @$pb.TagNumber(1)
  set collection($core.String v) { $_setString(0, v); }
  @$pb.TagNumber(1)
  $core.bool hasCollection() => $_has(0);
  @$pb.TagNumber(1)
  void clearCollection() => clearField(1);

  @$pb.TagNumber(2)
  $core.int get startId => $_getIZ(1);
  @$pb.TagNumber(2)
  set startId($core.int v) { $_setUnsignedInt32(1, v); }
  @$pb.TagNumber(2)
  $core.bool hasStartId() => $_has(1);
  @$pb.TagNumber(2)
  void clearStartId() => clearField(2);

  @$pb.TagNumber(3)
  $core.int get maxDepth => $_getIZ(2);
  @$pb.TagNumber(3)
  set maxDepth($core.int v) { $_setUnsignedInt32(2, v); }
  @$pb.TagNumber(3)
  $core.bool hasMaxDepth() => $_has(2);
  @$pb.TagNumber(3)
  void clearMaxDepth() => clearField(3);

  @$pb.TagNumber(4)
  $core.int get maxNodes => $_getIZ(3);
  @$pb.TagNumber(4)
  set maxNodes($core.int v) { $_setUnsignedInt32(3, v); }
  @$pb.TagNumber(4)
  $core.bool hasMaxNodes() => $_has(3);
  @$pb.TagNumber(4)
  void clearMaxNodes() => clearField(4);

  @$pb.TagNumber(5)
  $core.int get layer => $_getIZ(4);
  @$pb.TagNumber(5)
  set layer($core.int v) { $_setUnsignedInt32(4, v); }
  @$pb.TagNumber(5)
  $core.bool hasLayer() => $_has(4);
  @$pb.TagNumber(5)
  void clearLayer() => clearField(5);

  @$pb.TagNumber(6)
  $core.Map<$core.String, $core.String> get filter => $_getMap(5);

  @$pb.TagNumber(7)
  $core.List<Filter> get filters => $_getList(6);
}

class TraverseResponse extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'TraverseResponse', package: const $pb.PackageName(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'hyperspace'), createEmptyInstance: create)
    ..pc<GraphNode>(1, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'nodes', $pb.PbFieldType.PM, subBuilder: GraphNode.create)
    ..hasRequiredFields = false
  ;

  TraverseResponse._() : super();
  factory TraverseResponse({
    $core.Iterable<GraphNode>? nodes,
  }) {
    final _result = create();
    if (nodes != null) {
      _result.nodes.addAll(nodes);
    }
    return _result;
  }
  factory TraverseResponse.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory TraverseResponse.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  TraverseResponse clone() => TraverseResponse()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  TraverseResponse copyWith(void Function(TraverseResponse) updates) => super.copyWith((message) => updates(message as TraverseResponse)) as TraverseResponse; // ignore: deprecated_member_use
  $pb.BuilderInfo get info_ => _i;
  @$core.pragma('dart2js:noInline')
  static TraverseResponse create() => TraverseResponse._();
  TraverseResponse createEmptyInstance() => create();
  static $pb.PbList<TraverseResponse> createRepeated() => $pb.PbList<TraverseResponse>();
  @$core.pragma('dart2js:noInline')
  static TraverseResponse getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<TraverseResponse>(create);
  static TraverseResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $core.List<GraphNode> get nodes => $_getList(0);
}

class FindSemanticClustersRequest extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'FindSemanticClustersRequest', package: const $pb.PackageName(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'hyperspace'), createEmptyInstance: create)
    ..aOS(1, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'collection')
    ..a<$core.int>(2, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'layer', $pb.PbFieldType.OU3)
    ..a<$core.int>(3, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'minClusterSize', $pb.PbFieldType.OU3)
    ..a<$core.int>(4, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'maxClusters', $pb.PbFieldType.OU3)
    ..a<$core.int>(5, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'maxNodes', $pb.PbFieldType.OU3)
    ..hasRequiredFields = false
  ;

  FindSemanticClustersRequest._() : super();
  factory FindSemanticClustersRequest({
    $core.String? collection,
    $core.int? layer,
    $core.int? minClusterSize,
    $core.int? maxClusters,
    $core.int? maxNodes,
  }) {
    final _result = create();
    if (collection != null) {
      _result.collection = collection;
    }
    if (layer != null) {
      _result.layer = layer;
    }
    if (minClusterSize != null) {
      _result.minClusterSize = minClusterSize;
    }
    if (maxClusters != null) {
      _result.maxClusters = maxClusters;
    }
    if (maxNodes != null) {
      _result.maxNodes = maxNodes;
    }
    return _result;
  }
  factory FindSemanticClustersRequest.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory FindSemanticClustersRequest.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  FindSemanticClustersRequest clone() => FindSemanticClustersRequest()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  FindSemanticClustersRequest copyWith(void Function(FindSemanticClustersRequest) updates) => super.copyWith((message) => updates(message as FindSemanticClustersRequest)) as FindSemanticClustersRequest; // ignore: deprecated_member_use
  $pb.BuilderInfo get info_ => _i;
  @$core.pragma('dart2js:noInline')
  static FindSemanticClustersRequest create() => FindSemanticClustersRequest._();
  FindSemanticClustersRequest createEmptyInstance() => create();
  static $pb.PbList<FindSemanticClustersRequest> createRepeated() => $pb.PbList<FindSemanticClustersRequest>();
  @$core.pragma('dart2js:noInline')
  static FindSemanticClustersRequest getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<FindSemanticClustersRequest>(create);
  static FindSemanticClustersRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get collection => $_getSZ(0);
  @$pb.TagNumber(1)
  set collection($core.String v) { $_setString(0, v); }
  @$pb.TagNumber(1)
  $core.bool hasCollection() => $_has(0);
  @$pb.TagNumber(1)
  void clearCollection() => clearField(1);

  @$pb.TagNumber(2)
  $core.int get layer => $_getIZ(1);
  @$pb.TagNumber(2)
  set layer($core.int v) { $_setUnsignedInt32(1, v); }
  @$pb.TagNumber(2)
  $core.bool hasLayer() => $_has(1);
  @$pb.TagNumber(2)
  void clearLayer() => clearField(2);

  @$pb.TagNumber(3)
  $core.int get minClusterSize => $_getIZ(2);
  @$pb.TagNumber(3)
  set minClusterSize($core.int v) { $_setUnsignedInt32(2, v); }
  @$pb.TagNumber(3)
  $core.bool hasMinClusterSize() => $_has(2);
  @$pb.TagNumber(3)
  void clearMinClusterSize() => clearField(3);

  @$pb.TagNumber(4)
  $core.int get maxClusters => $_getIZ(3);
  @$pb.TagNumber(4)
  set maxClusters($core.int v) { $_setUnsignedInt32(3, v); }
  @$pb.TagNumber(4)
  $core.bool hasMaxClusters() => $_has(3);
  @$pb.TagNumber(4)
  void clearMaxClusters() => clearField(4);

  @$pb.TagNumber(5)
  $core.int get maxNodes => $_getIZ(4);
  @$pb.TagNumber(5)
  set maxNodes($core.int v) { $_setUnsignedInt32(4, v); }
  @$pb.TagNumber(5)
  $core.bool hasMaxNodes() => $_has(4);
  @$pb.TagNumber(5)
  void clearMaxNodes() => clearField(5);
}

class GetConceptParentsRequest extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'GetConceptParentsRequest', package: const $pb.PackageName(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'hyperspace'), createEmptyInstance: create)
    ..aOS(1, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'collection')
    ..a<$core.int>(2, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'id', $pb.PbFieldType.OU3)
    ..a<$core.int>(3, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'layer', $pb.PbFieldType.OU3)
    ..a<$core.int>(4, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'limit', $pb.PbFieldType.OU3)
    ..hasRequiredFields = false
  ;

  GetConceptParentsRequest._() : super();
  factory GetConceptParentsRequest({
    $core.String? collection,
    $core.int? id,
    $core.int? layer,
    $core.int? limit,
  }) {
    final _result = create();
    if (collection != null) {
      _result.collection = collection;
    }
    if (id != null) {
      _result.id = id;
    }
    if (layer != null) {
      _result.layer = layer;
    }
    if (limit != null) {
      _result.limit = limit;
    }
    return _result;
  }
  factory GetConceptParentsRequest.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory GetConceptParentsRequest.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  GetConceptParentsRequest clone() => GetConceptParentsRequest()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  GetConceptParentsRequest copyWith(void Function(GetConceptParentsRequest) updates) => super.copyWith((message) => updates(message as GetConceptParentsRequest)) as GetConceptParentsRequest; // ignore: deprecated_member_use
  $pb.BuilderInfo get info_ => _i;
  @$core.pragma('dart2js:noInline')
  static GetConceptParentsRequest create() => GetConceptParentsRequest._();
  GetConceptParentsRequest createEmptyInstance() => create();
  static $pb.PbList<GetConceptParentsRequest> createRepeated() => $pb.PbList<GetConceptParentsRequest>();
  @$core.pragma('dart2js:noInline')
  static GetConceptParentsRequest getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<GetConceptParentsRequest>(create);
  static GetConceptParentsRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get collection => $_getSZ(0);
  @$pb.TagNumber(1)
  set collection($core.String v) { $_setString(0, v); }
  @$pb.TagNumber(1)
  $core.bool hasCollection() => $_has(0);
  @$pb.TagNumber(1)
  void clearCollection() => clearField(1);

  @$pb.TagNumber(2)
  $core.int get id => $_getIZ(1);
  @$pb.TagNumber(2)
  set id($core.int v) { $_setUnsignedInt32(1, v); }
  @$pb.TagNumber(2)
  $core.bool hasId() => $_has(1);
  @$pb.TagNumber(2)
  void clearId() => clearField(2);

  @$pb.TagNumber(3)
  $core.int get layer => $_getIZ(2);
  @$pb.TagNumber(3)
  set layer($core.int v) { $_setUnsignedInt32(2, v); }
  @$pb.TagNumber(3)
  $core.bool hasLayer() => $_has(2);
  @$pb.TagNumber(3)
  void clearLayer() => clearField(3);

  @$pb.TagNumber(4)
  $core.int get limit => $_getIZ(3);
  @$pb.TagNumber(4)
  set limit($core.int v) { $_setUnsignedInt32(3, v); }
  @$pb.TagNumber(4)
  $core.bool hasLimit() => $_has(3);
  @$pb.TagNumber(4)
  void clearLimit() => clearField(4);
}

class GetConceptParentsResponse extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'GetConceptParentsResponse', package: const $pb.PackageName(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'hyperspace'), createEmptyInstance: create)
    ..pc<GraphNode>(1, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'parents', $pb.PbFieldType.PM, subBuilder: GraphNode.create)
    ..hasRequiredFields = false
  ;

  GetConceptParentsResponse._() : super();
  factory GetConceptParentsResponse({
    $core.Iterable<GraphNode>? parents,
  }) {
    final _result = create();
    if (parents != null) {
      _result.parents.addAll(parents);
    }
    return _result;
  }
  factory GetConceptParentsResponse.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory GetConceptParentsResponse.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  GetConceptParentsResponse clone() => GetConceptParentsResponse()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  GetConceptParentsResponse copyWith(void Function(GetConceptParentsResponse) updates) => super.copyWith((message) => updates(message as GetConceptParentsResponse)) as GetConceptParentsResponse; // ignore: deprecated_member_use
  $pb.BuilderInfo get info_ => _i;
  @$core.pragma('dart2js:noInline')
  static GetConceptParentsResponse create() => GetConceptParentsResponse._();
  GetConceptParentsResponse createEmptyInstance() => create();
  static $pb.PbList<GetConceptParentsResponse> createRepeated() => $pb.PbList<GetConceptParentsResponse>();
  @$core.pragma('dart2js:noInline')
  static GetConceptParentsResponse getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<GetConceptParentsResponse>(create);
  static GetConceptParentsResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $core.List<GraphNode> get parents => $_getList(0);
}

class GraphCluster extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'GraphCluster', package: const $pb.PackageName(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'hyperspace'), createEmptyInstance: create)
    ..p<$core.int>(1, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'nodeIds', $pb.PbFieldType.KU3)
    ..hasRequiredFields = false
  ;

  GraphCluster._() : super();
  factory GraphCluster({
    $core.Iterable<$core.int>? nodeIds,
  }) {
    final _result = create();
    if (nodeIds != null) {
      _result.nodeIds.addAll(nodeIds);
    }
    return _result;
  }
  factory GraphCluster.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory GraphCluster.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  GraphCluster clone() => GraphCluster()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  GraphCluster copyWith(void Function(GraphCluster) updates) => super.copyWith((message) => updates(message as GraphCluster)) as GraphCluster; // ignore: deprecated_member_use
  $pb.BuilderInfo get info_ => _i;
  @$core.pragma('dart2js:noInline')
  static GraphCluster create() => GraphCluster._();
  GraphCluster createEmptyInstance() => create();
  static $pb.PbList<GraphCluster> createRepeated() => $pb.PbList<GraphCluster>();
  @$core.pragma('dart2js:noInline')
  static GraphCluster getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<GraphCluster>(create);
  static GraphCluster? _defaultInstance;

  @$pb.TagNumber(1)
  $core.List<$core.int> get nodeIds => $_getList(0);
}

class FindSemanticClustersResponse extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'FindSemanticClustersResponse', package: const $pb.PackageName(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'hyperspace'), createEmptyInstance: create)
    ..pc<GraphCluster>(1, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'clusters', $pb.PbFieldType.PM, subBuilder: GraphCluster.create)
    ..hasRequiredFields = false
  ;

  FindSemanticClustersResponse._() : super();
  factory FindSemanticClustersResponse({
    $core.Iterable<GraphCluster>? clusters,
  }) {
    final _result = create();
    if (clusters != null) {
      _result.clusters.addAll(clusters);
    }
    return _result;
  }
  factory FindSemanticClustersResponse.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory FindSemanticClustersResponse.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  FindSemanticClustersResponse clone() => FindSemanticClustersResponse()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  FindSemanticClustersResponse copyWith(void Function(FindSemanticClustersResponse) updates) => super.copyWith((message) => updates(message as FindSemanticClustersResponse)) as FindSemanticClustersResponse; // ignore: deprecated_member_use
  $pb.BuilderInfo get info_ => _i;
  @$core.pragma('dart2js:noInline')
  static FindSemanticClustersResponse create() => FindSemanticClustersResponse._();
  FindSemanticClustersResponse createEmptyInstance() => create();
  static $pb.PbList<FindSemanticClustersResponse> createRepeated() => $pb.PbList<FindSemanticClustersResponse>();
  @$core.pragma('dart2js:noInline')
  static FindSemanticClustersResponse getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<FindSemanticClustersResponse>(create);
  static FindSemanticClustersResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $core.List<GraphCluster> get clusters => $_getList(0);
}

enum MetadataValue_Kind {
  stringValue, 
  intValue, 
  doubleValue, 
  boolValue, 
  notSet
}

class MetadataValue extends $pb.GeneratedMessage {
  static const $core.Map<$core.int, MetadataValue_Kind> _MetadataValue_KindByTag = {
    1 : MetadataValue_Kind.stringValue,
    2 : MetadataValue_Kind.intValue,
    3 : MetadataValue_Kind.doubleValue,
    4 : MetadataValue_Kind.boolValue,
    0 : MetadataValue_Kind.notSet
  };
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'MetadataValue', package: const $pb.PackageName(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'hyperspace'), createEmptyInstance: create)
    ..oo(0, [1, 2, 3, 4])
    ..aOS(1, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'stringValue')
    ..aInt64(2, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'intValue')
    ..a<$core.double>(3, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'doubleValue', $pb.PbFieldType.OD)
    ..aOB(4, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'boolValue')
    ..hasRequiredFields = false
  ;

  MetadataValue._() : super();
  factory MetadataValue({
    $core.String? stringValue,
    $fixnum.Int64? intValue,
    $core.double? doubleValue,
    $core.bool? boolValue,
  }) {
    final _result = create();
    if (stringValue != null) {
      _result.stringValue = stringValue;
    }
    if (intValue != null) {
      _result.intValue = intValue;
    }
    if (doubleValue != null) {
      _result.doubleValue = doubleValue;
    }
    if (boolValue != null) {
      _result.boolValue = boolValue;
    }
    return _result;
  }
  factory MetadataValue.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory MetadataValue.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  MetadataValue clone() => MetadataValue()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  MetadataValue copyWith(void Function(MetadataValue) updates) => super.copyWith((message) => updates(message as MetadataValue)) as MetadataValue; // ignore: deprecated_member_use
  $pb.BuilderInfo get info_ => _i;
  @$core.pragma('dart2js:noInline')
  static MetadataValue create() => MetadataValue._();
  MetadataValue createEmptyInstance() => create();
  static $pb.PbList<MetadataValue> createRepeated() => $pb.PbList<MetadataValue>();
  @$core.pragma('dart2js:noInline')
  static MetadataValue getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<MetadataValue>(create);
  static MetadataValue? _defaultInstance;

  MetadataValue_Kind whichKind() => _MetadataValue_KindByTag[$_whichOneof(0)]!;
  void clearKind() => clearField($_whichOneof(0));

  @$pb.TagNumber(1)
  $core.String get stringValue => $_getSZ(0);
  @$pb.TagNumber(1)
  set stringValue($core.String v) { $_setString(0, v); }
  @$pb.TagNumber(1)
  $core.bool hasStringValue() => $_has(0);
  @$pb.TagNumber(1)
  void clearStringValue() => clearField(1);

  @$pb.TagNumber(2)
  $fixnum.Int64 get intValue => $_getI64(1);
  @$pb.TagNumber(2)
  set intValue($fixnum.Int64 v) { $_setInt64(1, v); }
  @$pb.TagNumber(2)
  $core.bool hasIntValue() => $_has(1);
  @$pb.TagNumber(2)
  void clearIntValue() => clearField(2);

  @$pb.TagNumber(3)
  $core.double get doubleValue => $_getN(2);
  @$pb.TagNumber(3)
  set doubleValue($core.double v) { $_setDouble(2, v); }
  @$pb.TagNumber(3)
  $core.bool hasDoubleValue() => $_has(2);
  @$pb.TagNumber(3)
  void clearDoubleValue() => clearField(3);

  @$pb.TagNumber(4)
  $core.bool get boolValue => $_getBF(3);
  @$pb.TagNumber(4)
  set boolValue($core.bool v) { $_setBool(3, v); }
  @$pb.TagNumber(4)
  $core.bool hasBoolValue() => $_has(3);
  @$pb.TagNumber(4)
  void clearBoolValue() => clearField(4);
}

class EventSubscriptionRequest extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'EventSubscriptionRequest', package: const $pb.PackageName(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'hyperspace'), createEmptyInstance: create)
    ..pc<EventType>(1, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'types', $pb.PbFieldType.KE, valueOf: EventType.valueOf, enumValues: EventType.values, defaultEnumValue: EventType.EVENT_UNKNOWN)
    ..aOS(2, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'collection')
    ..hasRequiredFields = false
  ;

  EventSubscriptionRequest._() : super();
  factory EventSubscriptionRequest({
    $core.Iterable<EventType>? types,
    $core.String? collection,
  }) {
    final _result = create();
    if (types != null) {
      _result.types.addAll(types);
    }
    if (collection != null) {
      _result.collection = collection;
    }
    return _result;
  }
  factory EventSubscriptionRequest.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory EventSubscriptionRequest.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  EventSubscriptionRequest clone() => EventSubscriptionRequest()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  EventSubscriptionRequest copyWith(void Function(EventSubscriptionRequest) updates) => super.copyWith((message) => updates(message as EventSubscriptionRequest)) as EventSubscriptionRequest; // ignore: deprecated_member_use
  $pb.BuilderInfo get info_ => _i;
  @$core.pragma('dart2js:noInline')
  static EventSubscriptionRequest create() => EventSubscriptionRequest._();
  EventSubscriptionRequest createEmptyInstance() => create();
  static $pb.PbList<EventSubscriptionRequest> createRepeated() => $pb.PbList<EventSubscriptionRequest>();
  @$core.pragma('dart2js:noInline')
  static EventSubscriptionRequest getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<EventSubscriptionRequest>(create);
  static EventSubscriptionRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.List<EventType> get types => $_getList(0);

  @$pb.TagNumber(2)
  $core.String get collection => $_getSZ(1);
  @$pb.TagNumber(2)
  set collection($core.String v) { $_setString(1, v); }
  @$pb.TagNumber(2)
  $core.bool hasCollection() => $_has(1);
  @$pb.TagNumber(2)
  void clearCollection() => clearField(2);
}

class VectorInsertedEvent extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'VectorInsertedEvent', package: const $pb.PackageName(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'hyperspace'), createEmptyInstance: create)
    ..a<$core.int>(1, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'id', $pb.PbFieldType.OU3)
    ..aOS(2, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'collection')
    ..a<$fixnum.Int64>(3, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'logicalClock', $pb.PbFieldType.OU6, defaultOrMaker: $fixnum.Int64.ZERO)
    ..aOS(4, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'originNodeId')
    ..m<$core.String, $core.String>(5, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'metadata', entryClassName: 'VectorInsertedEvent.MetadataEntry', keyFieldType: $pb.PbFieldType.OS, valueFieldType: $pb.PbFieldType.OS, packageName: const $pb.PackageName('hyperspace'))
    ..m<$core.String, MetadataValue>(6, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'typedMetadata', entryClassName: 'VectorInsertedEvent.TypedMetadataEntry', keyFieldType: $pb.PbFieldType.OS, valueFieldType: $pb.PbFieldType.OM, valueCreator: MetadataValue.create, packageName: const $pb.PackageName('hyperspace'))
    ..hasRequiredFields = false
  ;

  VectorInsertedEvent._() : super();
  factory VectorInsertedEvent({
    $core.int? id,
    $core.String? collection,
    $fixnum.Int64? logicalClock,
    $core.String? originNodeId,
    $core.Map<$core.String, $core.String>? metadata,
    $core.Map<$core.String, MetadataValue>? typedMetadata,
  }) {
    final _result = create();
    if (id != null) {
      _result.id = id;
    }
    if (collection != null) {
      _result.collection = collection;
    }
    if (logicalClock != null) {
      _result.logicalClock = logicalClock;
    }
    if (originNodeId != null) {
      _result.originNodeId = originNodeId;
    }
    if (metadata != null) {
      _result.metadata.addAll(metadata);
    }
    if (typedMetadata != null) {
      _result.typedMetadata.addAll(typedMetadata);
    }
    return _result;
  }
  factory VectorInsertedEvent.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory VectorInsertedEvent.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  VectorInsertedEvent clone() => VectorInsertedEvent()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  VectorInsertedEvent copyWith(void Function(VectorInsertedEvent) updates) => super.copyWith((message) => updates(message as VectorInsertedEvent)) as VectorInsertedEvent; // ignore: deprecated_member_use
  $pb.BuilderInfo get info_ => _i;
  @$core.pragma('dart2js:noInline')
  static VectorInsertedEvent create() => VectorInsertedEvent._();
  VectorInsertedEvent createEmptyInstance() => create();
  static $pb.PbList<VectorInsertedEvent> createRepeated() => $pb.PbList<VectorInsertedEvent>();
  @$core.pragma('dart2js:noInline')
  static VectorInsertedEvent getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<VectorInsertedEvent>(create);
  static VectorInsertedEvent? _defaultInstance;

  @$pb.TagNumber(1)
  $core.int get id => $_getIZ(0);
  @$pb.TagNumber(1)
  set id($core.int v) { $_setUnsignedInt32(0, v); }
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => clearField(1);

  @$pb.TagNumber(2)
  $core.String get collection => $_getSZ(1);
  @$pb.TagNumber(2)
  set collection($core.String v) { $_setString(1, v); }
  @$pb.TagNumber(2)
  $core.bool hasCollection() => $_has(1);
  @$pb.TagNumber(2)
  void clearCollection() => clearField(2);

  @$pb.TagNumber(3)
  $fixnum.Int64 get logicalClock => $_getI64(2);
  @$pb.TagNumber(3)
  set logicalClock($fixnum.Int64 v) { $_setInt64(2, v); }
  @$pb.TagNumber(3)
  $core.bool hasLogicalClock() => $_has(2);
  @$pb.TagNumber(3)
  void clearLogicalClock() => clearField(3);

  @$pb.TagNumber(4)
  $core.String get originNodeId => $_getSZ(3);
  @$pb.TagNumber(4)
  set originNodeId($core.String v) { $_setString(3, v); }
  @$pb.TagNumber(4)
  $core.bool hasOriginNodeId() => $_has(3);
  @$pb.TagNumber(4)
  void clearOriginNodeId() => clearField(4);

  @$pb.TagNumber(5)
  $core.Map<$core.String, $core.String> get metadata => $_getMap(4);

  @$pb.TagNumber(6)
  $core.Map<$core.String, MetadataValue> get typedMetadata => $_getMap(5);
}

class TrajectoryStepEvent extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'TrajectoryStepEvent', package: const $pb.PackageName(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'hyperspace'), createEmptyInstance: create)
    ..a<$core.int>(1, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'id', $pb.PbFieldType.OU3)
    ..aOS(2, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'collection')
    ..a<$core.double>(3, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'x', $pb.PbFieldType.OF)
    ..a<$core.double>(4, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'y', $pb.PbFieldType.OF)
    ..m<$core.String, $core.String>(5, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'metadata', entryClassName: 'TrajectoryStepEvent.MetadataEntry', keyFieldType: $pb.PbFieldType.OS, valueFieldType: $pb.PbFieldType.OS, packageName: const $pb.PackageName('hyperspace'))
    ..hasRequiredFields = false
  ;

  TrajectoryStepEvent._() : super();
  factory TrajectoryStepEvent({
    $core.int? id,
    $core.String? collection,
    $core.double? x,
    $core.double? y,
    $core.Map<$core.String, $core.String>? metadata,
  }) {
    final _result = create();
    if (id != null) {
      _result.id = id;
    }
    if (collection != null) {
      _result.collection = collection;
    }
    if (x != null) {
      _result.x = x;
    }
    if (y != null) {
      _result.y = y;
    }
    if (metadata != null) {
      _result.metadata.addAll(metadata);
    }
    return _result;
  }
  factory TrajectoryStepEvent.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory TrajectoryStepEvent.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  TrajectoryStepEvent clone() => TrajectoryStepEvent()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  TrajectoryStepEvent copyWith(void Function(TrajectoryStepEvent) updates) => super.copyWith((message) => updates(message as TrajectoryStepEvent)) as TrajectoryStepEvent; // ignore: deprecated_member_use
  $pb.BuilderInfo get info_ => _i;
  @$core.pragma('dart2js:noInline')
  static TrajectoryStepEvent create() => TrajectoryStepEvent._();
  TrajectoryStepEvent createEmptyInstance() => create();
  static $pb.PbList<TrajectoryStepEvent> createRepeated() => $pb.PbList<TrajectoryStepEvent>();
  @$core.pragma('dart2js:noInline')
  static TrajectoryStepEvent getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<TrajectoryStepEvent>(create);
  static TrajectoryStepEvent? _defaultInstance;

  @$pb.TagNumber(1)
  $core.int get id => $_getIZ(0);
  @$pb.TagNumber(1)
  set id($core.int v) { $_setUnsignedInt32(0, v); }
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => clearField(1);

  @$pb.TagNumber(2)
  $core.String get collection => $_getSZ(1);
  @$pb.TagNumber(2)
  set collection($core.String v) { $_setString(1, v); }
  @$pb.TagNumber(2)
  $core.bool hasCollection() => $_has(1);
  @$pb.TagNumber(2)
  void clearCollection() => clearField(2);

  @$pb.TagNumber(3)
  $core.double get x => $_getN(2);
  @$pb.TagNumber(3)
  set x($core.double v) { $_setFloat(2, v); }
  @$pb.TagNumber(3)
  $core.bool hasX() => $_has(2);
  @$pb.TagNumber(3)
  void clearX() => clearField(3);

  @$pb.TagNumber(4)
  $core.double get y => $_getN(3);
  @$pb.TagNumber(4)
  set y($core.double v) { $_setFloat(3, v); }
  @$pb.TagNumber(4)
  $core.bool hasY() => $_has(3);
  @$pb.TagNumber(4)
  void clearY() => clearField(4);

  @$pb.TagNumber(5)
  $core.Map<$core.String, $core.String> get metadata => $_getMap(4);
}

class VectorDeletedEvent extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'VectorDeletedEvent', package: const $pb.PackageName(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'hyperspace'), createEmptyInstance: create)
    ..a<$core.int>(1, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'id', $pb.PbFieldType.OU3)
    ..aOS(2, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'collection')
    ..a<$fixnum.Int64>(3, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'logicalClock', $pb.PbFieldType.OU6, defaultOrMaker: $fixnum.Int64.ZERO)
    ..aOS(4, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'originNodeId')
    ..hasRequiredFields = false
  ;

  VectorDeletedEvent._() : super();
  factory VectorDeletedEvent({
    $core.int? id,
    $core.String? collection,
    $fixnum.Int64? logicalClock,
    $core.String? originNodeId,
  }) {
    final _result = create();
    if (id != null) {
      _result.id = id;
    }
    if (collection != null) {
      _result.collection = collection;
    }
    if (logicalClock != null) {
      _result.logicalClock = logicalClock;
    }
    if (originNodeId != null) {
      _result.originNodeId = originNodeId;
    }
    return _result;
  }
  factory VectorDeletedEvent.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory VectorDeletedEvent.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  VectorDeletedEvent clone() => VectorDeletedEvent()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  VectorDeletedEvent copyWith(void Function(VectorDeletedEvent) updates) => super.copyWith((message) => updates(message as VectorDeletedEvent)) as VectorDeletedEvent; // ignore: deprecated_member_use
  $pb.BuilderInfo get info_ => _i;
  @$core.pragma('dart2js:noInline')
  static VectorDeletedEvent create() => VectorDeletedEvent._();
  VectorDeletedEvent createEmptyInstance() => create();
  static $pb.PbList<VectorDeletedEvent> createRepeated() => $pb.PbList<VectorDeletedEvent>();
  @$core.pragma('dart2js:noInline')
  static VectorDeletedEvent getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<VectorDeletedEvent>(create);
  static VectorDeletedEvent? _defaultInstance;

  @$pb.TagNumber(1)
  $core.int get id => $_getIZ(0);
  @$pb.TagNumber(1)
  set id($core.int v) { $_setUnsignedInt32(0, v); }
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => clearField(1);

  @$pb.TagNumber(2)
  $core.String get collection => $_getSZ(1);
  @$pb.TagNumber(2)
  set collection($core.String v) { $_setString(1, v); }
  @$pb.TagNumber(2)
  $core.bool hasCollection() => $_has(1);
  @$pb.TagNumber(2)
  void clearCollection() => clearField(2);

  @$pb.TagNumber(3)
  $fixnum.Int64 get logicalClock => $_getI64(2);
  @$pb.TagNumber(3)
  set logicalClock($fixnum.Int64 v) { $_setInt64(2, v); }
  @$pb.TagNumber(3)
  $core.bool hasLogicalClock() => $_has(2);
  @$pb.TagNumber(3)
  void clearLogicalClock() => clearField(3);

  @$pb.TagNumber(4)
  $core.String get originNodeId => $_getSZ(3);
  @$pb.TagNumber(4)
  set originNodeId($core.String v) { $_setString(3, v); }
  @$pb.TagNumber(4)
  $core.bool hasOriginNodeId() => $_has(3);
  @$pb.TagNumber(4)
  void clearOriginNodeId() => clearField(4);
}

enum EventMessage_Payload {
  vectorInserted, 
  vectorDeleted, 
  trajectoryStep, 
  notSet
}

class EventMessage extends $pb.GeneratedMessage {
  static const $core.Map<$core.int, EventMessage_Payload> _EventMessage_PayloadByTag = {
    2 : EventMessage_Payload.vectorInserted,
    3 : EventMessage_Payload.vectorDeleted,
    4 : EventMessage_Payload.trajectoryStep,
    0 : EventMessage_Payload.notSet
  };
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'EventMessage', package: const $pb.PackageName(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'hyperspace'), createEmptyInstance: create)
    ..oo(0, [2, 3, 4])
    ..e<EventType>(1, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'type', $pb.PbFieldType.OE, defaultOrMaker: EventType.EVENT_UNKNOWN, valueOf: EventType.valueOf, enumValues: EventType.values)
    ..aOM<VectorInsertedEvent>(2, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'vectorInserted', subBuilder: VectorInsertedEvent.create)
    ..aOM<VectorDeletedEvent>(3, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'vectorDeleted', subBuilder: VectorDeletedEvent.create)
    ..aOM<TrajectoryStepEvent>(4, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'trajectoryStep', subBuilder: TrajectoryStepEvent.create)
    ..hasRequiredFields = false
  ;

  EventMessage._() : super();
  factory EventMessage({
    EventType? type,
    VectorInsertedEvent? vectorInserted,
    VectorDeletedEvent? vectorDeleted,
    TrajectoryStepEvent? trajectoryStep,
  }) {
    final _result = create();
    if (type != null) {
      _result.type = type;
    }
    if (vectorInserted != null) {
      _result.vectorInserted = vectorInserted;
    }
    if (vectorDeleted != null) {
      _result.vectorDeleted = vectorDeleted;
    }
    if (trajectoryStep != null) {
      _result.trajectoryStep = trajectoryStep;
    }
    return _result;
  }
  factory EventMessage.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory EventMessage.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  EventMessage clone() => EventMessage()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  EventMessage copyWith(void Function(EventMessage) updates) => super.copyWith((message) => updates(message as EventMessage)) as EventMessage; // ignore: deprecated_member_use
  $pb.BuilderInfo get info_ => _i;
  @$core.pragma('dart2js:noInline')
  static EventMessage create() => EventMessage._();
  EventMessage createEmptyInstance() => create();
  static $pb.PbList<EventMessage> createRepeated() => $pb.PbList<EventMessage>();
  @$core.pragma('dart2js:noInline')
  static EventMessage getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<EventMessage>(create);
  static EventMessage? _defaultInstance;

  EventMessage_Payload whichPayload() => _EventMessage_PayloadByTag[$_whichOneof(0)]!;
  void clearPayload() => clearField($_whichOneof(0));

  @$pb.TagNumber(1)
  EventType get type => $_getN(0);
  @$pb.TagNumber(1)
  set type(EventType v) { setField(1, v); }
  @$pb.TagNumber(1)
  $core.bool hasType() => $_has(0);
  @$pb.TagNumber(1)
  void clearType() => clearField(1);

  @$pb.TagNumber(2)
  VectorInsertedEvent get vectorInserted => $_getN(1);
  @$pb.TagNumber(2)
  set vectorInserted(VectorInsertedEvent v) { setField(2, v); }
  @$pb.TagNumber(2)
  $core.bool hasVectorInserted() => $_has(1);
  @$pb.TagNumber(2)
  void clearVectorInserted() => clearField(2);
  @$pb.TagNumber(2)
  VectorInsertedEvent ensureVectorInserted() => $_ensure(1);

  @$pb.TagNumber(3)
  VectorDeletedEvent get vectorDeleted => $_getN(2);
  @$pb.TagNumber(3)
  set vectorDeleted(VectorDeletedEvent v) { setField(3, v); }
  @$pb.TagNumber(3)
  $core.bool hasVectorDeleted() => $_has(2);
  @$pb.TagNumber(3)
  void clearVectorDeleted() => clearField(3);
  @$pb.TagNumber(3)
  VectorDeletedEvent ensureVectorDeleted() => $_ensure(2);

  @$pb.TagNumber(4)
  TrajectoryStepEvent get trajectoryStep => $_getN(3);
  @$pb.TagNumber(4)
  set trajectoryStep(TrajectoryStepEvent v) { setField(4, v); }
  @$pb.TagNumber(4)
  $core.bool hasTrajectoryStep() => $_has(3);
  @$pb.TagNumber(4)
  void clearTrajectoryStep() => clearField(4);
  @$pb.TagNumber(4)
  TrajectoryStepEvent ensureTrajectoryStep() => $_ensure(3);
}

class Empty extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'Empty', package: const $pb.PackageName(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'hyperspace'), createEmptyInstance: create)
    ..hasRequiredFields = false
  ;

  Empty._() : super();
  factory Empty() => create();
  factory Empty.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory Empty.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  Empty clone() => Empty()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  Empty copyWith(void Function(Empty) updates) => super.copyWith((message) => updates(message as Empty)) as Empty; // ignore: deprecated_member_use
  $pb.BuilderInfo get info_ => _i;
  @$core.pragma('dart2js:noInline')
  static Empty create() => Empty._();
  Empty createEmptyInstance() => create();
  static $pb.PbList<Empty> createRepeated() => $pb.PbList<Empty>();
  @$core.pragma('dart2js:noInline')
  static Empty getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<Empty>(create);
  static Empty? _defaultInstance;
}

class StatusResponse extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'StatusResponse', package: const $pb.PackageName(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'hyperspace'), createEmptyInstance: create)
    ..aOS(1, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'status')
    ..hasRequiredFields = false
  ;

  StatusResponse._() : super();
  factory StatusResponse({
    $core.String? status,
  }) {
    final _result = create();
    if (status != null) {
      _result.status = status;
    }
    return _result;
  }
  factory StatusResponse.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory StatusResponse.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  StatusResponse clone() => StatusResponse()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  StatusResponse copyWith(void Function(StatusResponse) updates) => super.copyWith((message) => updates(message as StatusResponse)) as StatusResponse; // ignore: deprecated_member_use
  $pb.BuilderInfo get info_ => _i;
  @$core.pragma('dart2js:noInline')
  static StatusResponse create() => StatusResponse._();
  StatusResponse createEmptyInstance() => create();
  static $pb.PbList<StatusResponse> createRepeated() => $pb.PbList<StatusResponse>();
  @$core.pragma('dart2js:noInline')
  static StatusResponse getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<StatusResponse>(create);
  static StatusResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get status => $_getSZ(0);
  @$pb.TagNumber(1)
  set status($core.String v) { $_setString(0, v); }
  @$pb.TagNumber(1)
  $core.bool hasStatus() => $_has(0);
  @$pb.TagNumber(1)
  void clearStatus() => clearField(1);
}

class MonitorRequest extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'MonitorRequest', package: const $pb.PackageName(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'hyperspace'), createEmptyInstance: create)
    ..hasRequiredFields = false
  ;

  MonitorRequest._() : super();
  factory MonitorRequest() => create();
  factory MonitorRequest.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory MonitorRequest.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  MonitorRequest clone() => MonitorRequest()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  MonitorRequest copyWith(void Function(MonitorRequest) updates) => super.copyWith((message) => updates(message as MonitorRequest)) as MonitorRequest; // ignore: deprecated_member_use
  $pb.BuilderInfo get info_ => _i;
  @$core.pragma('dart2js:noInline')
  static MonitorRequest create() => MonitorRequest._();
  MonitorRequest createEmptyInstance() => create();
  static $pb.PbList<MonitorRequest> createRepeated() => $pb.PbList<MonitorRequest>();
  @$core.pragma('dart2js:noInline')
  static MonitorRequest getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<MonitorRequest>(create);
  static MonitorRequest? _defaultInstance;
}

class SystemStats extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'SystemStats', package: const $pb.PackageName(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'hyperspace'), createEmptyInstance: create)
    ..a<$fixnum.Int64>(1, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'totalCollections', $pb.PbFieldType.OU6, defaultOrMaker: $fixnum.Int64.ZERO)
    ..a<$fixnum.Int64>(2, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'totalVectors', $pb.PbFieldType.OU6, defaultOrMaker: $fixnum.Int64.ZERO)
    ..a<$core.double>(3, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'totalMemoryMb', $pb.PbFieldType.OD)
    ..a<$core.double>(4, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'qps', $pb.PbFieldType.OD)
    ..hasRequiredFields = false
  ;

  SystemStats._() : super();
  factory SystemStats({
    $fixnum.Int64? totalCollections,
    $fixnum.Int64? totalVectors,
    $core.double? totalMemoryMb,
    $core.double? qps,
  }) {
    final _result = create();
    if (totalCollections != null) {
      _result.totalCollections = totalCollections;
    }
    if (totalVectors != null) {
      _result.totalVectors = totalVectors;
    }
    if (totalMemoryMb != null) {
      _result.totalMemoryMb = totalMemoryMb;
    }
    if (qps != null) {
      _result.qps = qps;
    }
    return _result;
  }
  factory SystemStats.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory SystemStats.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  SystemStats clone() => SystemStats()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  SystemStats copyWith(void Function(SystemStats) updates) => super.copyWith((message) => updates(message as SystemStats)) as SystemStats; // ignore: deprecated_member_use
  $pb.BuilderInfo get info_ => _i;
  @$core.pragma('dart2js:noInline')
  static SystemStats create() => SystemStats._();
  SystemStats createEmptyInstance() => create();
  static $pb.PbList<SystemStats> createRepeated() => $pb.PbList<SystemStats>();
  @$core.pragma('dart2js:noInline')
  static SystemStats getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<SystemStats>(create);
  static SystemStats? _defaultInstance;

  @$pb.TagNumber(1)
  $fixnum.Int64 get totalCollections => $_getI64(0);
  @$pb.TagNumber(1)
  set totalCollections($fixnum.Int64 v) { $_setInt64(0, v); }
  @$pb.TagNumber(1)
  $core.bool hasTotalCollections() => $_has(0);
  @$pb.TagNumber(1)
  void clearTotalCollections() => clearField(1);

  @$pb.TagNumber(2)
  $fixnum.Int64 get totalVectors => $_getI64(1);
  @$pb.TagNumber(2)
  set totalVectors($fixnum.Int64 v) { $_setInt64(1, v); }
  @$pb.TagNumber(2)
  $core.bool hasTotalVectors() => $_has(1);
  @$pb.TagNumber(2)
  void clearTotalVectors() => clearField(2);

  @$pb.TagNumber(3)
  $core.double get totalMemoryMb => $_getN(2);
  @$pb.TagNumber(3)
  set totalMemoryMb($core.double v) { $_setDouble(2, v); }
  @$pb.TagNumber(3)
  $core.bool hasTotalMemoryMb() => $_has(2);
  @$pb.TagNumber(3)
  void clearTotalMemoryMb() => clearField(3);

  @$pb.TagNumber(4)
  $core.double get qps => $_getN(3);
  @$pb.TagNumber(4)
  set qps($core.double v) { $_setDouble(3, v); }
  @$pb.TagNumber(4)
  $core.bool hasQps() => $_has(3);
  @$pb.TagNumber(4)
  void clearQps() => clearField(4);
}

class DigestRequest extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'DigestRequest', package: const $pb.PackageName(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'hyperspace'), createEmptyInstance: create)
    ..aOS(1, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'collection')
    ..hasRequiredFields = false
  ;

  DigestRequest._() : super();
  factory DigestRequest({
    $core.String? collection,
  }) {
    final _result = create();
    if (collection != null) {
      _result.collection = collection;
    }
    return _result;
  }
  factory DigestRequest.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory DigestRequest.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  DigestRequest clone() => DigestRequest()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  DigestRequest copyWith(void Function(DigestRequest) updates) => super.copyWith((message) => updates(message as DigestRequest)) as DigestRequest; // ignore: deprecated_member_use
  $pb.BuilderInfo get info_ => _i;
  @$core.pragma('dart2js:noInline')
  static DigestRequest create() => DigestRequest._();
  DigestRequest createEmptyInstance() => create();
  static $pb.PbList<DigestRequest> createRepeated() => $pb.PbList<DigestRequest>();
  @$core.pragma('dart2js:noInline')
  static DigestRequest getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<DigestRequest>(create);
  static DigestRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get collection => $_getSZ(0);
  @$pb.TagNumber(1)
  set collection($core.String v) { $_setString(0, v); }
  @$pb.TagNumber(1)
  $core.bool hasCollection() => $_has(0);
  @$pb.TagNumber(1)
  void clearCollection() => clearField(1);
}

class DigestResponse extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'DigestResponse', package: const $pb.PackageName(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'hyperspace'), createEmptyInstance: create)
    ..a<$fixnum.Int64>(1, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'logicalClock', $pb.PbFieldType.OU6, defaultOrMaker: $fixnum.Int64.ZERO)
    ..a<$fixnum.Int64>(2, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'stateHash', $pb.PbFieldType.OU6, defaultOrMaker: $fixnum.Int64.ZERO)
    ..p<$fixnum.Int64>(3, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'buckets', $pb.PbFieldType.KU6)
    ..a<$fixnum.Int64>(4, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'count', $pb.PbFieldType.OU6, defaultOrMaker: $fixnum.Int64.ZERO)
    ..hasRequiredFields = false
  ;

  DigestResponse._() : super();
  factory DigestResponse({
    $fixnum.Int64? logicalClock,
    $fixnum.Int64? stateHash,
    $core.Iterable<$fixnum.Int64>? buckets,
    $fixnum.Int64? count,
  }) {
    final _result = create();
    if (logicalClock != null) {
      _result.logicalClock = logicalClock;
    }
    if (stateHash != null) {
      _result.stateHash = stateHash;
    }
    if (buckets != null) {
      _result.buckets.addAll(buckets);
    }
    if (count != null) {
      _result.count = count;
    }
    return _result;
  }
  factory DigestResponse.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory DigestResponse.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  DigestResponse clone() => DigestResponse()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  DigestResponse copyWith(void Function(DigestResponse) updates) => super.copyWith((message) => updates(message as DigestResponse)) as DigestResponse; // ignore: deprecated_member_use
  $pb.BuilderInfo get info_ => _i;
  @$core.pragma('dart2js:noInline')
  static DigestResponse create() => DigestResponse._();
  DigestResponse createEmptyInstance() => create();
  static $pb.PbList<DigestResponse> createRepeated() => $pb.PbList<DigestResponse>();
  @$core.pragma('dart2js:noInline')
  static DigestResponse getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<DigestResponse>(create);
  static DigestResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $fixnum.Int64 get logicalClock => $_getI64(0);
  @$pb.TagNumber(1)
  set logicalClock($fixnum.Int64 v) { $_setInt64(0, v); }
  @$pb.TagNumber(1)
  $core.bool hasLogicalClock() => $_has(0);
  @$pb.TagNumber(1)
  void clearLogicalClock() => clearField(1);

  @$pb.TagNumber(2)
  $fixnum.Int64 get stateHash => $_getI64(1);
  @$pb.TagNumber(2)
  set stateHash($fixnum.Int64 v) { $_setInt64(1, v); }
  @$pb.TagNumber(2)
  $core.bool hasStateHash() => $_has(1);
  @$pb.TagNumber(2)
  void clearStateHash() => clearField(2);

  @$pb.TagNumber(3)
  $core.List<$fixnum.Int64> get buckets => $_getList(2);

  @$pb.TagNumber(4)
  $fixnum.Int64 get count => $_getI64(3);
  @$pb.TagNumber(4)
  set count($fixnum.Int64 v) { $_setInt64(3, v); }
  @$pb.TagNumber(4)
  $core.bool hasCount() => $_has(3);
  @$pb.TagNumber(4)
  void clearCount() => clearField(4);
}

class SyncHandshakeRequest extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'SyncHandshakeRequest', package: const $pb.PackageName(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'hyperspace'), createEmptyInstance: create)
    ..aOS(1, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'collection')
    ..p<$fixnum.Int64>(2, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'clientBuckets', $pb.PbFieldType.KU6)
    ..a<$fixnum.Int64>(3, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'clientLogicalClock', $pb.PbFieldType.OU6, defaultOrMaker: $fixnum.Int64.ZERO)
    ..a<$fixnum.Int64>(4, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'clientCount', $pb.PbFieldType.OU6, defaultOrMaker: $fixnum.Int64.ZERO)
    ..hasRequiredFields = false
  ;

  SyncHandshakeRequest._() : super();
  factory SyncHandshakeRequest({
    $core.String? collection,
    $core.Iterable<$fixnum.Int64>? clientBuckets,
    $fixnum.Int64? clientLogicalClock,
    $fixnum.Int64? clientCount,
  }) {
    final _result = create();
    if (collection != null) {
      _result.collection = collection;
    }
    if (clientBuckets != null) {
      _result.clientBuckets.addAll(clientBuckets);
    }
    if (clientLogicalClock != null) {
      _result.clientLogicalClock = clientLogicalClock;
    }
    if (clientCount != null) {
      _result.clientCount = clientCount;
    }
    return _result;
  }
  factory SyncHandshakeRequest.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory SyncHandshakeRequest.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  SyncHandshakeRequest clone() => SyncHandshakeRequest()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  SyncHandshakeRequest copyWith(void Function(SyncHandshakeRequest) updates) => super.copyWith((message) => updates(message as SyncHandshakeRequest)) as SyncHandshakeRequest; // ignore: deprecated_member_use
  $pb.BuilderInfo get info_ => _i;
  @$core.pragma('dart2js:noInline')
  static SyncHandshakeRequest create() => SyncHandshakeRequest._();
  SyncHandshakeRequest createEmptyInstance() => create();
  static $pb.PbList<SyncHandshakeRequest> createRepeated() => $pb.PbList<SyncHandshakeRequest>();
  @$core.pragma('dart2js:noInline')
  static SyncHandshakeRequest getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<SyncHandshakeRequest>(create);
  static SyncHandshakeRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get collection => $_getSZ(0);
  @$pb.TagNumber(1)
  set collection($core.String v) { $_setString(0, v); }
  @$pb.TagNumber(1)
  $core.bool hasCollection() => $_has(0);
  @$pb.TagNumber(1)
  void clearCollection() => clearField(1);

  @$pb.TagNumber(2)
  $core.List<$fixnum.Int64> get clientBuckets => $_getList(1);

  @$pb.TagNumber(3)
  $fixnum.Int64 get clientLogicalClock => $_getI64(2);
  @$pb.TagNumber(3)
  set clientLogicalClock($fixnum.Int64 v) { $_setInt64(2, v); }
  @$pb.TagNumber(3)
  $core.bool hasClientLogicalClock() => $_has(2);
  @$pb.TagNumber(3)
  void clearClientLogicalClock() => clearField(3);

  @$pb.TagNumber(4)
  $fixnum.Int64 get clientCount => $_getI64(3);
  @$pb.TagNumber(4)
  set clientCount($fixnum.Int64 v) { $_setInt64(3, v); }
  @$pb.TagNumber(4)
  $core.bool hasClientCount() => $_has(3);
  @$pb.TagNumber(4)
  void clearClientCount() => clearField(4);
}

class DiffBucket extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'DiffBucket', package: const $pb.PackageName(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'hyperspace'), createEmptyInstance: create)
    ..a<$core.int>(1, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'bucketIndex', $pb.PbFieldType.OU3)
    ..a<$fixnum.Int64>(2, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'serverHash', $pb.PbFieldType.OU6, defaultOrMaker: $fixnum.Int64.ZERO)
    ..a<$fixnum.Int64>(3, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'clientHash', $pb.PbFieldType.OU6, defaultOrMaker: $fixnum.Int64.ZERO)
    ..hasRequiredFields = false
  ;

  DiffBucket._() : super();
  factory DiffBucket({
    $core.int? bucketIndex,
    $fixnum.Int64? serverHash,
    $fixnum.Int64? clientHash,
  }) {
    final _result = create();
    if (bucketIndex != null) {
      _result.bucketIndex = bucketIndex;
    }
    if (serverHash != null) {
      _result.serverHash = serverHash;
    }
    if (clientHash != null) {
      _result.clientHash = clientHash;
    }
    return _result;
  }
  factory DiffBucket.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory DiffBucket.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  DiffBucket clone() => DiffBucket()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  DiffBucket copyWith(void Function(DiffBucket) updates) => super.copyWith((message) => updates(message as DiffBucket)) as DiffBucket; // ignore: deprecated_member_use
  $pb.BuilderInfo get info_ => _i;
  @$core.pragma('dart2js:noInline')
  static DiffBucket create() => DiffBucket._();
  DiffBucket createEmptyInstance() => create();
  static $pb.PbList<DiffBucket> createRepeated() => $pb.PbList<DiffBucket>();
  @$core.pragma('dart2js:noInline')
  static DiffBucket getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<DiffBucket>(create);
  static DiffBucket? _defaultInstance;

  @$pb.TagNumber(1)
  $core.int get bucketIndex => $_getIZ(0);
  @$pb.TagNumber(1)
  set bucketIndex($core.int v) { $_setUnsignedInt32(0, v); }
  @$pb.TagNumber(1)
  $core.bool hasBucketIndex() => $_has(0);
  @$pb.TagNumber(1)
  void clearBucketIndex() => clearField(1);

  @$pb.TagNumber(2)
  $fixnum.Int64 get serverHash => $_getI64(1);
  @$pb.TagNumber(2)
  set serverHash($fixnum.Int64 v) { $_setInt64(1, v); }
  @$pb.TagNumber(2)
  $core.bool hasServerHash() => $_has(1);
  @$pb.TagNumber(2)
  void clearServerHash() => clearField(2);

  @$pb.TagNumber(3)
  $fixnum.Int64 get clientHash => $_getI64(2);
  @$pb.TagNumber(3)
  set clientHash($fixnum.Int64 v) { $_setInt64(2, v); }
  @$pb.TagNumber(3)
  $core.bool hasClientHash() => $_has(2);
  @$pb.TagNumber(3)
  void clearClientHash() => clearField(3);
}

class SyncHandshakeResponse extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'SyncHandshakeResponse', package: const $pb.PackageName(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'hyperspace'), createEmptyInstance: create)
    ..pc<DiffBucket>(1, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'diffBuckets', $pb.PbFieldType.PM, subBuilder: DiffBucket.create)
    ..a<$fixnum.Int64>(2, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'serverLogicalClock', $pb.PbFieldType.OU6, defaultOrMaker: $fixnum.Int64.ZERO)
    ..a<$fixnum.Int64>(3, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'serverCount', $pb.PbFieldType.OU6, defaultOrMaker: $fixnum.Int64.ZERO)
    ..aOB(4, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'inSync')
    ..hasRequiredFields = false
  ;

  SyncHandshakeResponse._() : super();
  factory SyncHandshakeResponse({
    $core.Iterable<DiffBucket>? diffBuckets,
    $fixnum.Int64? serverLogicalClock,
    $fixnum.Int64? serverCount,
    $core.bool? inSync,
  }) {
    final _result = create();
    if (diffBuckets != null) {
      _result.diffBuckets.addAll(diffBuckets);
    }
    if (serverLogicalClock != null) {
      _result.serverLogicalClock = serverLogicalClock;
    }
    if (serverCount != null) {
      _result.serverCount = serverCount;
    }
    if (inSync != null) {
      _result.inSync = inSync;
    }
    return _result;
  }
  factory SyncHandshakeResponse.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory SyncHandshakeResponse.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  SyncHandshakeResponse clone() => SyncHandshakeResponse()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  SyncHandshakeResponse copyWith(void Function(SyncHandshakeResponse) updates) => super.copyWith((message) => updates(message as SyncHandshakeResponse)) as SyncHandshakeResponse; // ignore: deprecated_member_use
  $pb.BuilderInfo get info_ => _i;
  @$core.pragma('dart2js:noInline')
  static SyncHandshakeResponse create() => SyncHandshakeResponse._();
  SyncHandshakeResponse createEmptyInstance() => create();
  static $pb.PbList<SyncHandshakeResponse> createRepeated() => $pb.PbList<SyncHandshakeResponse>();
  @$core.pragma('dart2js:noInline')
  static SyncHandshakeResponse getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<SyncHandshakeResponse>(create);
  static SyncHandshakeResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $core.List<DiffBucket> get diffBuckets => $_getList(0);

  @$pb.TagNumber(2)
  $fixnum.Int64 get serverLogicalClock => $_getI64(1);
  @$pb.TagNumber(2)
  set serverLogicalClock($fixnum.Int64 v) { $_setInt64(1, v); }
  @$pb.TagNumber(2)
  $core.bool hasServerLogicalClock() => $_has(1);
  @$pb.TagNumber(2)
  void clearServerLogicalClock() => clearField(2);

  @$pb.TagNumber(3)
  $fixnum.Int64 get serverCount => $_getI64(2);
  @$pb.TagNumber(3)
  set serverCount($fixnum.Int64 v) { $_setInt64(2, v); }
  @$pb.TagNumber(3)
  $core.bool hasServerCount() => $_has(2);
  @$pb.TagNumber(3)
  void clearServerCount() => clearField(3);

  @$pb.TagNumber(4)
  $core.bool get inSync => $_getBF(3);
  @$pb.TagNumber(4)
  set inSync($core.bool v) { $_setBool(3, v); }
  @$pb.TagNumber(4)
  $core.bool hasInSync() => $_has(3);
  @$pb.TagNumber(4)
  void clearInSync() => clearField(4);
}

class SyncPullRequest extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'SyncPullRequest', package: const $pb.PackageName(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'hyperspace'), createEmptyInstance: create)
    ..aOS(1, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'collection')
    ..p<$core.int>(2, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'bucketIndices', $pb.PbFieldType.KU3)
    ..hasRequiredFields = false
  ;

  SyncPullRequest._() : super();
  factory SyncPullRequest({
    $core.String? collection,
    $core.Iterable<$core.int>? bucketIndices,
  }) {
    final _result = create();
    if (collection != null) {
      _result.collection = collection;
    }
    if (bucketIndices != null) {
      _result.bucketIndices.addAll(bucketIndices);
    }
    return _result;
  }
  factory SyncPullRequest.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory SyncPullRequest.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  SyncPullRequest clone() => SyncPullRequest()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  SyncPullRequest copyWith(void Function(SyncPullRequest) updates) => super.copyWith((message) => updates(message as SyncPullRequest)) as SyncPullRequest; // ignore: deprecated_member_use
  $pb.BuilderInfo get info_ => _i;
  @$core.pragma('dart2js:noInline')
  static SyncPullRequest create() => SyncPullRequest._();
  SyncPullRequest createEmptyInstance() => create();
  static $pb.PbList<SyncPullRequest> createRepeated() => $pb.PbList<SyncPullRequest>();
  @$core.pragma('dart2js:noInline')
  static SyncPullRequest getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<SyncPullRequest>(create);
  static SyncPullRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get collection => $_getSZ(0);
  @$pb.TagNumber(1)
  set collection($core.String v) { $_setString(0, v); }
  @$pb.TagNumber(1)
  $core.bool hasCollection() => $_has(0);
  @$pb.TagNumber(1)
  void clearCollection() => clearField(1);

  @$pb.TagNumber(2)
  $core.List<$core.int> get bucketIndices => $_getList(1);
}

class SyncVectorData extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'SyncVectorData', package: const $pb.PackageName(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'hyperspace'), createEmptyInstance: create)
    ..aOS(1, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'collection')
    ..a<$core.int>(2, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'id', $pb.PbFieldType.OU3)
    ..p<$core.double>(3, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'vector', $pb.PbFieldType.KD)
    ..m<$core.String, $core.String>(4, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'metadata', entryClassName: 'SyncVectorData.MetadataEntry', keyFieldType: $pb.PbFieldType.OS, valueFieldType: $pb.PbFieldType.OS, packageName: const $pb.PackageName('hyperspace'))
    ..a<$core.int>(5, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'bucketIndex', $pb.PbFieldType.OU3)
    ..hasRequiredFields = false
  ;

  SyncVectorData._() : super();
  factory SyncVectorData({
    $core.String? collection,
    $core.int? id,
    $core.Iterable<$core.double>? vector,
    $core.Map<$core.String, $core.String>? metadata,
    $core.int? bucketIndex,
  }) {
    final _result = create();
    if (collection != null) {
      _result.collection = collection;
    }
    if (id != null) {
      _result.id = id;
    }
    if (vector != null) {
      _result.vector.addAll(vector);
    }
    if (metadata != null) {
      _result.metadata.addAll(metadata);
    }
    if (bucketIndex != null) {
      _result.bucketIndex = bucketIndex;
    }
    return _result;
  }
  factory SyncVectorData.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory SyncVectorData.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  SyncVectorData clone() => SyncVectorData()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  SyncVectorData copyWith(void Function(SyncVectorData) updates) => super.copyWith((message) => updates(message as SyncVectorData)) as SyncVectorData; // ignore: deprecated_member_use
  $pb.BuilderInfo get info_ => _i;
  @$core.pragma('dart2js:noInline')
  static SyncVectorData create() => SyncVectorData._();
  SyncVectorData createEmptyInstance() => create();
  static $pb.PbList<SyncVectorData> createRepeated() => $pb.PbList<SyncVectorData>();
  @$core.pragma('dart2js:noInline')
  static SyncVectorData getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<SyncVectorData>(create);
  static SyncVectorData? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get collection => $_getSZ(0);
  @$pb.TagNumber(1)
  set collection($core.String v) { $_setString(0, v); }
  @$pb.TagNumber(1)
  $core.bool hasCollection() => $_has(0);
  @$pb.TagNumber(1)
  void clearCollection() => clearField(1);

  @$pb.TagNumber(2)
  $core.int get id => $_getIZ(1);
  @$pb.TagNumber(2)
  set id($core.int v) { $_setUnsignedInt32(1, v); }
  @$pb.TagNumber(2)
  $core.bool hasId() => $_has(1);
  @$pb.TagNumber(2)
  void clearId() => clearField(2);

  @$pb.TagNumber(3)
  $core.List<$core.double> get vector => $_getList(2);

  @$pb.TagNumber(4)
  $core.Map<$core.String, $core.String> get metadata => $_getMap(3);

  @$pb.TagNumber(5)
  $core.int get bucketIndex => $_getIZ(4);
  @$pb.TagNumber(5)
  set bucketIndex($core.int v) { $_setUnsignedInt32(4, v); }
  @$pb.TagNumber(5)
  $core.bool hasBucketIndex() => $_has(4);
  @$pb.TagNumber(5)
  void clearBucketIndex() => clearField(5);
}

class SyncPushResponse extends $pb.GeneratedMessage {
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'SyncPushResponse', package: const $pb.PackageName(const $core.bool.fromEnvironment('protobuf.omit_message_names') ? '' : 'hyperspace'), createEmptyInstance: create)
    ..a<$core.int>(1, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'accepted', $pb.PbFieldType.OU3)
    ..a<$core.int>(2, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'rejected', $pb.PbFieldType.OU3)
    ..a<$core.int>(3, const $core.bool.fromEnvironment('protobuf.omit_field_names') ? '' : 'duplicates', $pb.PbFieldType.OU3)
    ..hasRequiredFields = false
  ;

  SyncPushResponse._() : super();
  factory SyncPushResponse({
    $core.int? accepted,
    $core.int? rejected,
    $core.int? duplicates,
  }) {
    final _result = create();
    if (accepted != null) {
      _result.accepted = accepted;
    }
    if (rejected != null) {
      _result.rejected = rejected;
    }
    if (duplicates != null) {
      _result.duplicates = duplicates;
    }
    return _result;
  }
  factory SyncPushResponse.fromBuffer($core.List<$core.int> i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromBuffer(i, r);
  factory SyncPushResponse.fromJson($core.String i, [$pb.ExtensionRegistry r = $pb.ExtensionRegistry.EMPTY]) => create()..mergeFromJson(i, r);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.deepCopy] instead. '
  'Will be removed in next major version')
  SyncPushResponse clone() => SyncPushResponse()..mergeFromMessage(this);
  @$core.Deprecated(
  'Using this can add significant overhead to your binary. '
  'Use [GeneratedMessageGenericExtensions.rebuild] instead. '
  'Will be removed in next major version')
  SyncPushResponse copyWith(void Function(SyncPushResponse) updates) => super.copyWith((message) => updates(message as SyncPushResponse)) as SyncPushResponse; // ignore: deprecated_member_use
  $pb.BuilderInfo get info_ => _i;
  @$core.pragma('dart2js:noInline')
  static SyncPushResponse create() => SyncPushResponse._();
  SyncPushResponse createEmptyInstance() => create();
  static $pb.PbList<SyncPushResponse> createRepeated() => $pb.PbList<SyncPushResponse>();
  @$core.pragma('dart2js:noInline')
  static SyncPushResponse getDefault() => _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<SyncPushResponse>(create);
  static SyncPushResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $core.int get accepted => $_getIZ(0);
  @$pb.TagNumber(1)
  set accepted($core.int v) { $_setUnsignedInt32(0, v); }
  @$pb.TagNumber(1)
  $core.bool hasAccepted() => $_has(0);
  @$pb.TagNumber(1)
  void clearAccepted() => clearField(1);

  @$pb.TagNumber(2)
  $core.int get rejected => $_getIZ(1);
  @$pb.TagNumber(2)
  set rejected($core.int v) { $_setUnsignedInt32(1, v); }
  @$pb.TagNumber(2)
  $core.bool hasRejected() => $_has(1);
  @$pb.TagNumber(2)
  void clearRejected() => clearField(2);

  @$pb.TagNumber(3)
  $core.int get duplicates => $_getIZ(2);
  @$pb.TagNumber(3)
  set duplicates($core.int v) { $_setUnsignedInt32(2, v); }
  @$pb.TagNumber(3)
  $core.bool hasDuplicates() => $_has(2);
  @$pb.TagNumber(3)
  void clearDuplicates() => clearField(3);
}

