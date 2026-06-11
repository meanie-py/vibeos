#!/usr/bin/env python3
"""fix_field_paths.py — deterministic bindgen field-path repair for translated crates.

The LLM translates C `x->Field` / `x.Field` as a direct `.Field`, but bindgen nests fields
under anonymous members. The correct Rust path is fully determined by the bindgen struct:
  - field lives in an anon member      ->  insert `.__bindgen_anon_N` :  `.F` -> `.__bindgen_anon_1.F`
  - field is behind a __BindgenUnionField wrapper -> insert `.as_ref()`/`.as_mut()`
rustc reports the exact span + the owning type for every E0609/E0614; we look the field up
in that type's generated definition and splice the one correct path. No LLM, no guessing.

This handles the mechanical subset (anon nesting + union-wrapper). It does NOT touch
type-identity mismatches (e.g. an incomplete `ddk::_SCSI_REQUEST_BLOCK` vs the full scsi.h
one) — those are an import/substrate fix, not a path fix, and are left for review.

Usage: fix_field_paths.py <crate-src-dir> <target-triple>   (iterate until stable)
"""
import sys, re, os, json, subprocess
from collections import defaultdict

CRATE_SRC = sys.argv[1]
TARGET = sys.argv[2]
CRATE = os.path.basename(os.path.dirname(CRATE_SRC))
WS = os.path.abspath(os.path.join(CRATE_SRC, "../.."))
arch = {"i686-pc-windows-gnu": "i386", "x86_64-pc-windows-gnu": "amd64",
        "aarch64-pc-windows-gnu": "arm64"}[TARGET]
TYPES = open(os.path.join(CRATE_SRC, f"types_{arch}.rs")).read()

def struct_body(tag):
    """Return the `{ ... }` body text of `pub struct <tag>` (or _tag), else None."""
    for name in (tag, "_" + tag, tag.lstrip("_")):
        m = re.search(r"pub struct " + re.escape(name) + r"\b[^{]*\{", TYPES)
        if not m:
            continue
        i = m.end() - 1
        depth, j = 0, i
        while j < len(TYPES):
            if TYPES[j] == "{": depth += 1
            elif TYPES[j] == "}":
                depth -= 1
                if depth == 0:
                    return TYPES[i:j + 1]
            j += 1
    return None

def field_path(tag, field, want_mut=False, depth=0, seen=None):
    """Return the member-chain PREFIX to reach `field` from struct `tag` — e.g.
    '__bindgen_anon_1.__bindgen_anon_2.as_ref().' — by recursive BFS through nested anon
    members and `__BindgenUnionField<T>` wrappers (insert `.as_ref()`/`.as_mut()` after each
    union hop). Returns None if absent OR AMBIGUOUS (>1 nested member reaches `field`) — we
    never guess between same-named sibling fields; those go to review."""
    if seen is None:
        seen = set()
    body = struct_body(tag)
    if not body or depth > 6:
        return None
    if re.search(r"pub " + re.escape(field) + r"\s*:", body):
        return ""                                   # field is directly in this body
    results = []
    for m in re.finditer(r"pub (\w+)\s*:\s*(?:__BindgenUnionField<\s*([A-Za-z0-9_]+)\s*>|([A-Za-z0-9_]+))", body):
        member, union_inner, plain = m.group(1), m.group(2), m.group(3)
        is_union = union_inner is not None
        inner = union_inner or plain
        if not inner or inner in seen:
            continue
        if not (member.startswith("__bindgen_anon") or is_union or inner.startswith("_")):
            continue                                # only descend into anon/union/bindgen types
        sub = field_path(inner, field, want_mut, depth + 1, seen | {inner})
        if sub is not None:
            hop = member + "."
            if is_union:
                hop += "as_mut()." if want_mut else "as_ref()."
            results.append(hop + sub)
    return results[0] if len(results) == 1 else None

def rustc_json():
    r = subprocess.run(["cargo", "check", "-p", CRATE, "--target", TARGET,
                        "--message-format=json"], cwd=WS, capture_output=True, text=True)
    for ln in r.stdout.splitlines():
        try: msg = json.loads(ln)
        except ValueError: continue
        if msg.get("reason") != "compiler-message": continue
        yield msg["message"]

def main():
    # edits per file: list of (line, col0-based, kind, payload)
    edits = defaultdict(list)
    seen = set()
    for m in rustc_json():
        code = (m.get("code") or {}).get("code")
        if code not in ("E0609", "E0614"):
            continue
        txt = m["message"]
        fm = re.search(r"no field `([A-Za-z0-9_]+)` on type `[^`]*?([A-Za-z0-9_]+)`", txt)
        union = re.search(r"__BindgenUnionField<", txt)
        sp = next((s for s in m["spans"] if s.get("is_primary")), None)
        if not sp:
            continue
        fname = os.path.basename(sp["file_name"])
        key = (fname, sp["line_start"], sp["column_start"])
        if key in seen:
            continue
        seen.add(key)
        if union:
            edits[fname].append((sp["line_start"], sp["column_start"], "union", None))
        elif fm:
            field, tag = fm.group(1), fm.group(2)
            edits[fname].append((sp["line_start"], sp["column_start"], "path", (field, tag)))
    applied = 0
    for fname, elist in edits.items():
        path = os.path.join(CRATE_SRC, fname)
        if not os.path.exists(path):
            continue
        lines = open(path).read().split("\n")
        # apply right-to-left within a line so columns stay valid
        for line, col, kind, payload in sorted(elist, key=lambda e: (-e[0], -e[1])):
            s = lines[line - 1]
            if kind == "path":
                field, tag = payload
                # primary span points at the field token; splice the full nested-member chain
                # (anon hops + union as_ref/as_mut) before it.
                idx = col - 1
                if s[idx:idx + len(field)] == field and idx > 0 and s[idx - 1] == ".":
                    after = s[idx + len(field):].lstrip()
                    before = s[:idx].rstrip()
                    want_mut = (after.startswith("=") and not after.startswith("==")) \
                        or before.endswith(("&raw mut", "addr_of_mut!(", "&mut", "&mut ("))
                    prefix = field_path(tag, field, want_mut)
                    if prefix:                       # non-empty -> a real nested path
                        lines[line - 1] = s[:idx] + prefix + s[idx:]
                        applied += 1
            elif kind == "union":
                # span points at the field token after a __BindgenUnionField member access:
                # ...MEMBER.FIELD...  -> ...MEMBER.as_ref().FIELD...   (read; as_mut handled in a later pass)
                idx = col - 1
                if idx > 0 and s[idx - 1] == ".":
                    lines[line - 1] = s[:idx] + "as_ref()." + s[idx:]
                    applied += 1
        open(path, "w").write("\n".join(lines))
    print(f"applied {applied} field-path fixes across {len(edits)} files")

if __name__ == "__main__":
    main()
