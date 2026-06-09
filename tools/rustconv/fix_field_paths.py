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

def anon_member_for(tag, field):
    """If `field` lives inside an anon member of struct `tag`, return that member name."""
    body = struct_body(tag)
    if not body:
        return None
    # members look like:  pub __bindgen_anon_1: <Type>,
    for am, aty in re.findall(r"pub (__bindgen_anon_\d+)\s*:\s*([A-Za-z0-9_]+)", body):
        ab = struct_body(aty)
        if ab and re.search(r"pub " + re.escape(field) + r"\s*:", ab):
            return am
    return None

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
            am = anon_member_for(tag, field)
            if am:
                edits[fname].append((sp["line_start"], sp["column_start"], "anon", (field, am)))
    applied = 0
    for fname, elist in edits.items():
        path = os.path.join(CRATE_SRC, fname)
        if not os.path.exists(path):
            continue
        lines = open(path).read().split("\n")
        # apply right-to-left within a line so columns stay valid
        for line, col, kind, payload in sorted(elist, key=lambda e: (-e[0], -e[1])):
            s = lines[line - 1]
            if kind == "anon":
                field, am = payload
                # find the `.field` (or `field` after `.`) at/around col; splice the anon member before it
                # primary span points at the field token; insert `__bindgen_anon_N.` before it
                idx = col - 1
                if s[idx:idx + len(field)] == field and idx > 0 and s[idx - 1] == ".":
                    lines[line - 1] = s[:idx] + f"{am}." + s[idx:]
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
