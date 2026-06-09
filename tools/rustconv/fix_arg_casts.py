#!/usr/bin/env python3
"""fix_arg_casts.py — deterministic raw-pointer cast repair for translated driver crates.

The LLM translation passes a typed pointer where the callee wants a different pointer type
(`SrbGetDataTransferLength((*pkt).Srb)` — callee wants `*mut c_void`, gets
`*mut _STORAGE_REQUEST_BLOCK_HEADER`). Rust does NOT coerce raw-pointer types at a call arg,
so rustc emits E0308 with the EXACT expected/found types and the EXACT byte span of the
offending expression. The fix is fully determined: wrap the expression in `(...).cast()`.

`<*mut T>::cast::<U>()` infers `U` from the surrounding expected type, so `(expr).cast()`
becomes whatever pointer the call site demands — we never have to NAME (or correctly
path-qualify) the target type, which is what makes this deterministic and safe. We only touch
mismatches where BOTH sides are raw pointers of the SAME mutability (`*mut`->`*mut`,
`*const`->`*const`); anything else (mut/const flips, non-pointer mismatches) is reported, never
guessed — those are left for the (LLM) repair pass.

Unlike the repair pass, this CANNOT drop a call, reorder logic, or invent syntax: it only ever
inserts `(` ... `).cast()` around an existing sub-expression at a rustc-reported span.

Usage: fix_arg_casts.py <crate-src-dir> <target-triple>   (re-run until stable)
"""
import sys, os, json, subprocess, re

CRATE_SRC = os.path.abspath(sys.argv[1])
TARGET = sys.argv[2]
CRATE = os.path.basename(os.path.dirname(CRATE_SRC))
WS = os.path.abspath(os.path.join(CRATE_SRC, "../.."))

PTR = re.compile(r"^\*(mut|const)\s")


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
    """Pull (expected, found) raw-pointer type strings from an E0308 message + children."""
    exp = found = None
    for c in [msg] + msg.get("children", []):
        t = c["message"]
        m = re.search(r"expected raw pointer `([^`]+)`", t)
        if m:
            exp = m.group(1)
        m = re.search(r"found raw pointer `([^`]+)`", t)
        if m:
            found = m.group(1)
    return exp, found


def main():
    # edits per file: (byte_start, byte_end). De-dup by span.
    edits = {}
    skipped = []
    for msg in rustc_json():
        if (msg.get("code") or {}).get("code") != "E0308":
            continue
        exp, found = exp_found(msg)
        if not exp or not found:
            continue
        em, fm = PTR.match(exp), PTR.match(found)
        if not (em and fm):
            continue
        if em.group(1) != fm.group(1):       # mut/const flip — let the repair pass judge it
            skipped.append((exp, found))
            continue
        if exp == found:
            continue
        sp = next((s for s in msg["spans"] if s.get("is_primary")), None)
        if not sp:
            continue
        fname = os.path.basename(sp["file_name"])
        if fname.startswith("types_") or fname in ("glue.rs", "lib.rs"):
            continue
        edits.setdefault(fname, {})[(sp["byte_start"], sp["byte_end"])] = True

    applied = 0
    for fname, spans in edits.items():
        path = os.path.join(CRATE_SRC, fname)
        if not os.path.exists(path):
            continue
        b = open(path, "rb").read()
        for (bs, be) in sorted(spans, key=lambda s: -s[0]):   # right-to-left keeps offsets valid
            expr = b[bs:be]
            # don't double-wrap something already cast
            if expr.rstrip().endswith(b".cast()"):
                continue
            b = b[:bs] + b"(" + expr + b").cast()" + b[be:]
            applied += 1
        open(path, "wb").write(b)
    print(f"applied {applied} pointer-cast fixes across {len(edits)} files")
    if skipped:
        u = sorted(set(skipped))
        print(f"SKIPPED {len(skipped)} mut/const-flip mismatches (repair-pass territory):")
        for exp, found in u[:12]:
            print(f"  expected {exp}  <-  found {found}")


if __name__ == "__main__":
    main()
