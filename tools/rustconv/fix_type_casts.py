#!/usr/bin/env python3
"""fix_type_casts.py — deterministic E0308 repair for translated crates (non-pointer-pointee
companion to fix_arg_casts.py).

C transliteration leaves three systemic E0308 shapes that rustc reports with the exact expected
type in the PRIMARY span's `label` (NOT the children — that's why fix_arg_casts misses them):

  1. int width/sign  : expected `u8`, found `u32`   -> wrap the span `(expr) as u8`
  2. ptr mut/const   : expected `*const i8`, found `*mut i8`  -> `(expr).cast_const()` / `.cast_mut()`
  3. fn ABI / item   : expected `"stdcall" fn(...)`, found `"system" fn`  -> `(expr) as <fnptr>`
                       (the fn-ptr type comes from the `= note: expected fn pointer ...` child)

Int `as` is faithful to C (which does implicit narrowing/sign conversion); it can silence a
genuine width bug the same way C does — acceptable for a transliteration pass, the smoke harness
is the runtime gate. We touch ONLY these shapes and skip anything else (struct/tuple/Option/ref
mismatches) for the LLM repair pass. Byte-span splices, so it can't reorder or drop logic.

Usage: fix_type_casts.py <crate-src-dir> <target-triple>   (re-run until stable)
"""
import sys, os, json, subprocess, re

CRATE_SRC = os.path.abspath(sys.argv[1])
TARGET = sys.argv[2]
CRATE = os.path.basename(os.path.dirname(CRATE_SRC))
WS = os.path.abspath(os.path.join(CRATE_SRC, "../.."))

INT = {"u8", "u16", "u32", "u64", "u128", "usize", "i8", "i16", "i32", "i64", "i128", "isize"}
PTR = re.compile(r"^\*(mut|const)\s+(.+)$")
EXP_FOUND = re.compile(r"expected `([^`]+)`,?\s+found `([^`]+)`")


def rustc_json():
    r = subprocess.run(["cargo", "check", "-p", CRATE, "--target", TARGET,
                        "--message-format=json"], cwd=WS, capture_output=True, text=True)
    for ln in r.stdout.splitlines():
        try:
            m = json.loads(ln)
        except ValueError:
            continue
        if m.get("reason") == "compiler-message":
            yield m["message"]


def exp_found(msg):
    """Return (expected, found) from the primary span label, the message, or a child note."""
    sp = next((s for s in msg["spans"] if s.get("is_primary")), None)
    texts = []
    if sp and sp.get("label"):
        texts.append(sp["label"])
    texts.append(msg["message"])
    for c in msg.get("children", []):
        texts.append(c["message"])
    for t in texts:
        m = EXP_FOUND.search(t)
        if m:
            return m.group(1), m.group(2), sp
    return None, None, sp


def fnptr_type(msg):
    """Pull the expected fn-pointer type string from a `= note: expected fn pointer ...` child."""
    for c in msg.get("children", []):
        m = re.search(r"expected fn pointer `([^`]+)`", c["message"])
        if m:
            return m.group(1)
    return None


def main():
    edits = {}   # file -> {(bs,be): replacement}
    counts = {"int": 0, "ptrflip": 0, "fnabi": 0}
    for msg in rustc_json():
        if (msg.get("code") or {}).get("code") != "E0308":
            continue
        exp, found, sp = exp_found(msg)
        if not exp or not sp or sp.get("file_name", "").endswith(("types_i386.rs", "types_amd64.rs", "types_arm64.rs")):
            continue
        fname = os.path.basename(sp["file_name"])
        if fname.startswith("types_") or fname in ("glue.rs", "lib.rs"):
            continue
        bs, be = sp["byte_start"], sp["byte_end"]
        rep = None
        if exp in INT and found in INT:                       # 1. int width/sign
            rep = ("INT", f"(@) as {exp}")
            counts["int"] += 1
        else:
            pe, pf = PTR.match(exp), PTR.match(found)
            if pe and pf and pe.group(2).strip() == pf.group(2).strip() \
                    and pe.group(1) != pf.group(1):           # 2. same pointee, mut/const flip
                meth = "cast_const" if pe.group(1) == "const" else "cast_mut"
                rep = ("PTR", f"(@).{meth}()")
                counts["ptrflip"] += 1
            elif ('"' in exp and "fn" in exp) or "fn item" in found:  # 3. fn ABI / item
                fp = fnptr_type(msg)
                if fp:
                    rep = ("FN", f"(@) as {fp}")
                    counts["fnabi"] += 1
        if rep:
            edits.setdefault(fname, {})[(bs, be)] = rep[1]

    applied = 0
    for fname, spans in edits.items():
        p = os.path.join(CRATE_SRC, fname)
        if not os.path.exists(p):
            continue
        b = open(p, "rb").read()
        for (bs, be) in sorted(spans, key=lambda s: -s[0]):
            expr = b[bs:be]
            if expr.rstrip().endswith(b")") and b").cast" in expr[-12:]:
                continue
            rep = spans[(bs, be)].replace("@", expr.decode("utf-8", "replace")).encode()
            b = b[:bs] + rep + b[be:]
            applied += 1
        open(p, "wb").write(b)
    print(f"applied {applied} type-cast fixes "
          f"(int={counts['int']}, ptr-flip={counts['ptrflip']}, fn-abi={counts['fnabi']}) "
          f"across {len(edits)} files")


if __name__ == "__main__":
    main()
