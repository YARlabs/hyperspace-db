///
//  Generated code. Do not modify.
//  source: hyperspace.proto
//
// @dart = 2.12
// ignore_for_file: annotate_overrides,camel_case_types,constant_identifier_names,directives_ordering,library_prefixes,non_constant_identifier_names,prefer_final_fields,return_of_invalid_type,unnecessary_const,unnecessary_import,unnecessary_this,unused_import,unused_shown_name

// ignore_for_file: UNDEFINED_SHOWN_NAME
import 'dart:core' as $core;
import 'package:protobuf/protobuf.dart' as $pb;

class QuantizationMode extends $pb.ProtobufEnum {
  static const QuantizationMode NONE = QuantizationMode._(0, const $core.bool.fromEnvironment('protobuf.omit_enum_names') ? '' : 'NONE');
  static const QuantizationMode SCALAR_I8 = QuantizationMode._(1, const $core.bool.fromEnvironment('protobuf.omit_enum_names') ? '' : 'SCALAR_I8');

  static const $core.List<QuantizationMode> values = <QuantizationMode> [
    NONE,
    SCALAR_I8,
  ];

  static final $core.Map<$core.int, QuantizationMode> _byValue = $pb.ProtobufEnum.initByValue(values);
  static QuantizationMode? valueOf($core.int value) => _byValue[value];

  const QuantizationMode._($core.int v, $core.String n) : super(v, n);
}

class DurabilityLevel extends $pb.ProtobufEnum {
  static const DurabilityLevel DEFAULT_LEVEL = DurabilityLevel._(0, const $core.bool.fromEnvironment('protobuf.omit_enum_names') ? '' : 'DEFAULT_LEVEL');
  static const DurabilityLevel ASYNC = DurabilityLevel._(1, const $core.bool.fromEnvironment('protobuf.omit_enum_names') ? '' : 'ASYNC');
  static const DurabilityLevel BATCH = DurabilityLevel._(2, const $core.bool.fromEnvironment('protobuf.omit_enum_names') ? '' : 'BATCH');
  static const DurabilityLevel STRICT = DurabilityLevel._(3, const $core.bool.fromEnvironment('protobuf.omit_enum_names') ? '' : 'STRICT');

  static const $core.List<DurabilityLevel> values = <DurabilityLevel> [
    DEFAULT_LEVEL,
    ASYNC,
    BATCH,
    STRICT,
  ];

  static final $core.Map<$core.int, DurabilityLevel> _byValue = $pb.ProtobufEnum.initByValue(values);
  static DurabilityLevel? valueOf($core.int value) => _byValue[value];

  const DurabilityLevel._($core.int v, $core.String n) : super(v, n);
}

class EventType extends $pb.ProtobufEnum {
  static const EventType EVENT_UNKNOWN = EventType._(0, const $core.bool.fromEnvironment('protobuf.omit_enum_names') ? '' : 'EVENT_UNKNOWN');
  static const EventType VECTOR_INSERTED = EventType._(1, const $core.bool.fromEnvironment('protobuf.omit_enum_names') ? '' : 'VECTOR_INSERTED');
  static const EventType VECTOR_DELETED = EventType._(2, const $core.bool.fromEnvironment('protobuf.omit_enum_names') ? '' : 'VECTOR_DELETED');
  static const EventType TRAJECTORY_STEP = EventType._(3, const $core.bool.fromEnvironment('protobuf.omit_enum_names') ? '' : 'TRAJECTORY_STEP');

  static const $core.List<EventType> values = <EventType> [
    EVENT_UNKNOWN,
    VECTOR_INSERTED,
    VECTOR_DELETED,
    TRAJECTORY_STEP,
  ];

  static final $core.Map<$core.int, EventType> _byValue = $pb.ProtobufEnum.initByValue(values);
  static EventType? valueOf($core.int value) => _byValue[value];

  const EventType._($core.int v, $core.String n) : super(v, n);
}

