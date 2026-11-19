#!/usr/bin/env python3
"""Checks that every SDK's generated protobuf stubs contain every message field of
crates/hyperspace-proto/proto/hyperspace.proto.

The check is independent of generator versions (headers, formatting): it looks for the
per-language accessor / field-name token of each field. It exists because hand-patched or
stale stubs compile fine yet silently drop new fields (e.g. `vector_f32`, `ef_search`).

Usage: python3 scripts/check_sdk_protos.py   (exit code 1 on drift)
"""
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
PROTO = ROOT / "crates/hyperspace-proto/proto/hyperspace.proto"

SCALAR_TYPES = r"(?:double|float|int32|int64|uint32|uint64|sint32|sint64|fixed32|fixed64|sfixed32|sfixed64|bool|string|bytes)"
FIELD_RE = re.compile(
    r"^\s*(?:optional\s+|repeated\s+)?(?:map<[^>]+>|[A-Za-z_][\w.]*)\s+([a-z_][a-z0-9_]*)\s*=\s*\d+\s*[;\[]"
)


def parse_fields(text: str) -> dict[str, list[str]]:
    """Returns {message_name: [field_name, ...]} for top-level and nested messages."""
    text = re.sub(r"//[^\n]*", "", text)
    fields: dict[str, list[str]] = {}
    stack: list[tuple[str, str]] = []  # (kind, name)
    for raw in text.splitlines():
        line = raw.strip()
        m = re.match(r"^(message|enum|oneof|service)\s+(\w+)\s*\{", line)
        if m:
            stack.append((m.group(1), m.group(2)))
            if m.group(1) == "message":
                fields.setdefault(m.group(2), [])
            continue
        if stack and stack[-1][0] in ("message", "oneof"):
            fm = FIELD_RE.match(raw)
            if fm and not line.startswith(("reserved", "option", "rpc")):
                owner = next(n for k, n in reversed(stack) if k == "message")
                fields[owner].append(fm.group(1))
        for _ in range(line.count("}")):
            if stack:
                stack.pop()
    return fields


def camel(name: str) -> str:
    head, *rest = name.split("_")
    return head + "".join(p[:1].upper() + p[1:] for p in rest)


def pascal(name: str) -> str:
    c = camel(name)
    return c[:1].upper() + c[1:]


def read(rel: str) -> str | None:
    p = ROOT / rel
    if not p.exists():
        return None
    text = p.read_text(encoding="utf-8", errors="replace")
    if rel.endswith("_pb2.py"):
        # protoc C-escapes the serialized descriptor (e.g. `\x66ull_dimension`); decode it.
        m = re.search(r"AddSerializedFile\((b'(?:[^'\\]|\\.)*')\)", text)
        if m:
            import ast

            text = ast.literal_eval(m.group(1)).decode("latin-1")
    return text


# language -> (stub file(s), token builder)
CHECKS = {
    "python": (["sdks/python/hyperspace/proto/hyperspace_pb2.py"], lambda f: f),
    "go": (["sdks/go/proto/hyperspace.pb.go"], lambda f: f"name={f}"),
    "typescript": (["sdks/ts/src/proto/hyperspace_pb.js"], lambda f: f"get{pascal(f)}"),
    "dart": (["sdks/dart/lib/src/generated/hyperspace.pb.dart"], lambda f: f"'{camel(f)}'"),
    "cpp": (["sdks/cpp/proto/hyperspace.pb.h"], lambda f: f" {f}("),
}


def main() -> int:
    fields = parse_fields(PROTO.read_text(encoding="utf-8"))
    total = sum(len(v) for v in fields.values())
    if total < 100:
        print(f"parser found only {total} fields: proto layout changed?", file=sys.stderr)
        return 1
    failed = False
    for lang, (files, token) in CHECKS.items():
        blobs = [read(f) for f in files]
        if any(b is None for b in blobs):
            print(f"[{lang}] missing stub file(s): {files}")
            failed = True
            continue
        blob = "\n".join(blobs)  # type: ignore[arg-type]
        missing = [
            f"{msg}.{f}"
            for msg, fs in fields.items()
            for f in fs
            if token(f) not in blob
        ]
        if missing:
            failed = True
            shown = ", ".join(missing[:12]) + (" ..." if len(missing) > 12 else "")
            print(f"[{lang}] {len(missing)} field(s) missing from generated stubs: {shown}")
        else:
            print(f"[{lang}] ok ({total} fields)")
    if failed:
        print("\nRegenerate the stubs (see each SDK's generate_protos script / README).")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
