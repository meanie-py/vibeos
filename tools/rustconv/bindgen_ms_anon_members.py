#!/usr/bin/env python3
"""Restore MS-extension anonymous struct members that bindgen turned into padding.

C under -fms-extensions lets a struct embed another struct TYPE without a member name:

    typedef struct _CHANNEL_DATA_PATA {
        CHANNEL_DATA_COMMON;            // "Common data, must be the first member."
        PCHANNEL_LOAD_TASK_FILE LoadTaskFile;
        ...

and every field of CHANNEL_DATA_COMMON is then reachable directly (`ChanData->ChanInfo`).
libclang lays this out correctly, but bindgen cannot name it and emits opaque padding
(`pub __bindgen_padding_0: __BindgenOpaqueArray<[u8; 176usize]>`), so all of those fields
vanish from the Rust struct. pciidex's channel structs and atapi's `_ATAPORT_IO_CONTEXT` lost
~150 field accesses to it.

This pass reads the C headers it is given, finds structs whose FIRST member is a bare typedef
name, and replaces the leading padding field of the matching Rust struct with
`pub __bindgen_anon_1: <TYPE>` -- the spelling crate-repair's anon_union_field rule already
routes `x.ChanInfo` through. It only handles the FIRST member (the base-struct idiom), where
the padding provably starts at offset 0; any other position is reported, not guessed. Each
rewrite is pinned by a compile-time size assertion, so a wrong substitution fails the build
instead of shifting every later field.

usage: bindgen_ms_anon_members.py <types_<arch>.rs> <header.h> [...]
"""

from __future__ import annotations

import re
import sys

_STRUCT_RE = re.compile(r"typedef\s+struct\s+(\w+)\s*\{", re.S)
_FIRST_MEMBER_RE = re.compile(r"\s*(?:/\*.*?\*/\s*|//[^\n]*\n\s*)*([A-Z_][A-Z0-9_]*)\s*;", re.S)
_PADDING_RE = (
    r"^(?P<indent>\s*)pub __bindgen_padding_0: "
    r"(?:__BindgenOpaqueArray<\[u8; (?P<a>\d+)usize\]>|\[u8; (?P<b>\d+)usize\]),\n"
)


def bare_first_members(header_text: str) -> dict[str, str]:
    """{struct tag: embedded TYPE} for structs whose first member is a bare type name."""
    out: dict[str, str] = {}
    for m in _STRUCT_RE.finditer(header_text):
        first = _FIRST_MEMBER_RE.match(header_text, m.end())
        if first:
            out[m.group(1)] = first.group(1)
    return out


def transform(rust: str, members: dict[str, str]) -> tuple[str, list[str], list[str]]:
    done: list[str] = []
    asserts: list[str] = []
    for tag, ty in sorted(members.items()):
        head = re.search(rf"^pub struct {re.escape(tag)} \{{\n", rust, re.M)
        if head is None:
            continue
        pad = re.compile(_PADDING_RE, re.M).match(rust, head.end())
        if pad is None:
            continue
        if not re.search(rf"^pub (?:type|struct) {re.escape(ty)}\b", rust, re.M):
            continue  # the embedded type is not in this island: nothing to name
        # Name the STRUCT TAG, not the typedef: crate-repair's anon-field map follows
        # `__bindgen_anon_N: <tag>` to the tag's fields and does not chase type aliases.
        alias = re.search(rf"^pub type {re.escape(ty)} = (\w+);", rust, re.M)
        if alias:
            ty = alias.group(1)
        size = pad.group("a") or pad.group("b")
        rust = (
            rust[: pad.start()]
            + f"{pad.group('indent')}pub __bindgen_anon_1: {ty},\n"
            + rust[pad.end() :]
        )
        asserts.append(
            f"const _: () = assert!(core::mem::size_of::<{ty}>() == {size}, "
            f'"{tag}: the embedded {ty} must fill the padding bindgen emitted for it");'
        )
        done.append(f"{tag}.{ty}")
    if asserts:
        rust = (
            rust.rstrip("\n")
            + "\n\n// MS anonymous struct members restored by bindgen_ms_anon_members.py:\n"
            + "\n".join(asserts)
            + "\n"
        )
    return rust, done, asserts


def main(argv: list[str]) -> int:
    if len(argv) < 3:
        print(__doc__.strip().splitlines()[-1], file=sys.stderr)
        return 2
    rust_path, headers = argv[1], argv[2:]
    members: dict[str, str] = {}
    for h in headers:
        try:
            with open(h, errors="replace") as f:
                members.update(bare_first_members(f.read()))
        except OSError:
            continue
    with open(rust_path) as f:
        rust = f.read()
    new, done, _ = transform(rust, members)
    if new != rust:
        with open(rust_path, "w") as f:
            f.write(new)
    if done:
        print(f"{rust_path}: restored {', '.join(done)}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
