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

import 'package:protobuf/protobuf.dart' as $pb;

class QuantizationMode extends $pb.ProtobufEnum {
  static const QuantizationMode NONE =
      QuantizationMode._(0, _omitEnumNames ? '' : 'NONE');
  static const QuantizationMode SCALAR_I8 =
      QuantizationMode._(1, _omitEnumNames ? '' : 'SCALAR_I8');

  static const $core.List<QuantizationMode> values = <QuantizationMode>[
    NONE,
    SCALAR_I8,
  ];

  static final $core.List<QuantizationMode?> _byValue =
      $pb.ProtobufEnum.$_initByValueList(values, 1);
  static QuantizationMode? valueOf($core.int value) =>
      value < 0 || value >= _byValue.length ? null : _byValue[value];

  const QuantizationMode._(super.value, super.name);
}

class DurabilityLevel extends $pb.ProtobufEnum {
  static const DurabilityLevel DEFAULT_LEVEL =
      DurabilityLevel._(0, _omitEnumNames ? '' : 'DEFAULT_LEVEL');
  static const DurabilityLevel ASYNC =
      DurabilityLevel._(1, _omitEnumNames ? '' : 'ASYNC');
  static const DurabilityLevel BATCH =
      DurabilityLevel._(2, _omitEnumNames ? '' : 'BATCH');
  static const DurabilityLevel STRICT =
      DurabilityLevel._(3, _omitEnumNames ? '' : 'STRICT');

  static const $core.List<DurabilityLevel> values = <DurabilityLevel>[
    DEFAULT_LEVEL,
    ASYNC,
    BATCH,
    STRICT,
  ];

  static final $core.List<DurabilityLevel?> _byValue =
      $pb.ProtobufEnum.$_initByValueList(values, 3);
  static DurabilityLevel? valueOf($core.int value) =>
      value < 0 || value >= _byValue.length ? null : _byValue[value];

  const DurabilityLevel._(super.value, super.name);
}

class EventType extends $pb.ProtobufEnum {
  static const EventType EVENT_UNKNOWN =
      EventType._(0, _omitEnumNames ? '' : 'EVENT_UNKNOWN');
  static const EventType VECTOR_INSERTED =
      EventType._(1, _omitEnumNames ? '' : 'VECTOR_INSERTED');
  static const EventType VECTOR_DELETED =
      EventType._(2, _omitEnumNames ? '' : 'VECTOR_DELETED');
  static const EventType TRAJECTORY_STEP =
      EventType._(3, _omitEnumNames ? '' : 'TRAJECTORY_STEP');

  static const $core.List<EventType> values = <EventType>[
    EVENT_UNKNOWN,
    VECTOR_INSERTED,
    VECTOR_DELETED,
    TRAJECTORY_STEP,
  ];

  static final $core.List<EventType?> _byValue =
      $pb.ProtobufEnum.$_initByValueList(values, 3);
  static EventType? valueOf($core.int value) =>
      value < 0 || value >= _byValue.length ? null : _byValue[value];

  const EventType._(super.value, super.name);
}

const $core.bool _omitEnumNames =
    $core.bool.fromEnvironment('protobuf.omit_enum_names');
