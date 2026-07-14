#!/usr/bin/env python3
"""Apply rustc's own E0308 spans as explicit casts, en masse.

WHY THIS EXISTS
---------------
Translating a C DLL to Rust produces a large, boring, and completely mechanical class of
error: C's implicit integer conversions become explicit ones. The single biggest cluster in
ws2_32 was 267 x `expected i32, found u32` — every one of them the same shape:

    INT ErrorCode;                       // i32
    if (ErrorCode != ERROR_SUCCESS)      // bindgen gives ERROR_SUCCESS: u32

bindgen types a non-negative `#define` as u32; the C code stores it in an `INT`. Neither
side is wrong, and there is no substrate-level fix: making the constants signed just trades
the errors for their mirror image (msafd measured E0308 325 -> 458 doing exactly that, see
tools/rustconv/gen_driver_types.sh :: msafd MACRO_SIGNED). The fix that actually shipped
there was a per-site cast — and per-site casts are what rustc is already telling us to make,
with byte-exact spans, in machine-readable JSON.

So: read the spans, wrap them, iterate. This does not GUESS anything — every edit is placed
where the compiler said the type is wrong, and cast to the type the compiler said it wanted.

WHAT IT DELIBERATELY WILL NOT DO
--------------------------------
* It only casts between types on an allowlist (integer primitives + raw pointers). A
  mismatch involving a struct, a reference, or a lifetime is a REAL bug in the translation
  and must not be papered over with a cast — those are left for a human to read.
* It never emits `.try_into().unwrap()` (which is what `cargo fix` would do): an unwrap in
  kernel/driver code is a panic path, and `scan_silent_failures` gates on exactly that.
* It refuses a span that is already a cast, so re-running is idempotent.

USAGE
    fix_type_mismatches.py -p ws2_32-rust --target x86_64-pc-windows-gnu [--apply] [--max-rounds N]

Default is a DRY RUN that prints what it would change, grouped by (expected, found).
"""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from collections import Counter
from pathlib import Path

RUST_DIR = Path(__file__).resolve().parent.parent / "sdk" / "lib" / "rust"

# Casting between these is meaning-preserving in the C sense (it is what the C compiler was
# doing implicitly). Anything NOT here is a real type error and is left alone.
_INT = {
    "i8", "i16", "i32", "i64", "isize",
    "u8", "u16", "u32", "u64", "usize",
    "c_int", "c_uint", "c_long", "c_ulong", "c_char", "c_uchar", "c_short", "c_ushort",
}  # fmt: skip

_PTR_RE = re.compile(r"^\*(?:mut|const)\s+\S+$")


def _castable(expected: str, found: str) -> bool:
    if expected == found:
        return False
    if expected in _INT and found in _INT:
        return True
    # Pointer-to-pointer: C casts these freely (HANDLE/PVOID/LPCRITICAL_SECTION all alias).
    return bool(_PTR_RE.match(expected) and _PTR_RE.match(found))


_MISMATCH_RE = re.compile(r"^mismatched types$")
# rustc states the mismatch in TWO different shapes and you need both:
#   inline  — "expected `i32`, found `u32`"
#   stacked — "expected raw pointer `*mut u16`\n      found raw pointer `*const u16`"
# The stacked form is what you get for constness/mutability differences, which is the single
# biggest cluster after the integer ones. Matching only the inline form silently ignores them.
_EXP_FOUND_RE = re.compile(r"expected `([^`]+)`, found `([^`]+)`")
_EXP_FOUND_STACKED_RE = re.compile(
    r"expected (?:raw pointer|reference) `([^`]+)`\s*\n?\s*"
    r"found (?:raw pointer|reference) `([^`]+)`"
)


def _expected_found(d: dict) -> tuple[str, str] | None:
    blob = d.get("message", "") + " " + " ".join((s.get("label") or "") for s in d.get("spans", []))
    for child in d.get("children", []):
        blob += " " + (child.get("message") or "")
    m = _EXP_FOUND_RE.search(blob) or _EXP_FOUND_STACKED_RE.search(blob)
    return (m.group(1), m.group(2)) if m else None


# A match-arm PATTERN position: `    AF_INET => {`  /  `    AF_INET | AF_INET6 =>`.
# rustc reports E0308 on these (a u32 const pattern under an i32 scrutinee), and the span it
# hands back points at the pattern — but `(AF_INET as i32)` is not a pattern in any Rust, so
# wrapping it produces a syntax error. It fails loudly rather than silently, but it is still
# the tool being wrong, and the REAL fix is to cast the scrutinee (`match x as u32 { … }`),
# which is a whole-match edit this per-span tool cannot make. Skip and report them.
_PATTERN_POS_RE = re.compile(r"^\s*[^=;{}]+=>")


def _text_after(span: dict) -> str:
    """The source immediately following the span, on its last line (for callee detection)."""
    texts = span.get("text", [])
    if not texts:
        return ""
    last = texts[-1]
    line = last.get("text", "")
    end_col = last.get("highlight_end", 1) - 1
    return line[end_col : end_col + 4]


def collect(pkg: str, target: str) -> list[dict]:
    """Run cargo check and return the E0308 primary spans we can mechanically fix."""
    cmd = [
        "cargo", "check", "-p", pkg, "--target", target,
        "--message-format=json", "--quiet",
    ]  # fmt: skip
    proc = subprocess.run(cmd, cwd=RUST_DIR, capture_output=True, text=True)
    out: list[dict] = []
    skipped: list[str] = []
    for line in proc.stdout.splitlines():
        try:
            msg = json.loads(line)
        except ValueError:
            continue
        if msg.get("reason") != "compiler-message":
            continue
        d = msg["message"]
        if (d.get("code") or {}).get("code") != "E0308":
            continue
        ef = _expected_found(d)
        if not ef:
            continue
        expected, found = ef
        if not _castable(expected, found):
            continue
        for s in d.get("spans", []):
            if not s.get("is_primary"):
                continue
            line = s.get("text", [{}])[0].get("text", "")
            if _PATTERN_POS_RE.match(line):
                skipped.append(f"{s['file_name']}:{s['line_start']}  [pattern] {line.strip()[:50]}")
                continue
            # CALLEE position: the span is the thing being CALLED, so wrapping it produces
            # `(f as i32)(args)` — a cast of a fn pointer to an integer, then a call on the
            # integer. rustc's E0308 span can land here when the mismatch is really about the
            # call's RESULT. Casting the callee is never what we mean.
            src_after = _text_after(s)
            if src_after.lstrip().startswith("("):
                skipped.append(f"{s['file_name']}:{s['line_start']}  [callee] {line.strip()[:50]}")
                continue
            out.append(
                {
                    "file": s["file_name"],
                    "start": s["byte_start"],
                    "end": s["byte_end"],
                    "text": line,
                    "expected": expected,
                    "found": found,
                }
            )
    if skipped:
        print(
            f"  SKIPPED {len(skipped)} match-arm pattern(s) — cast the SCRUTINEE, not the pattern:"
        )
        for s in skipped[:8]:
            print(f"      {s}")
    return out


def apply(edits: list[dict]) -> int:
    """Rewrite each span as `(<expr> as <expected>)`. Applied back-to-front per file so the
    byte offsets of the not-yet-applied edits stay valid."""
    by_file: dict[str, list[dict]] = {}
    for e in edits:
        by_file.setdefault(e["file"], []).append(e)

    changed = 0
    for fname, es in by_file.items():
        path = RUST_DIR / fname
        if not path.is_file():
            path = Path(fname)
        if not path.is_file():
            print(f"  ?? cannot find {fname}", file=sys.stderr)
            continue
        raw = path.read_bytes()
        # De-dup identical spans; sort descending so earlier offsets stay valid.
        #
        # NESTED SPANS ARE THE TRAP. rustc happily reports E0308 on an argument AND on the
        # enclosing call in the same run:  f(x)  ->  span[x] and span[f(x)].  Applying them
        # back-to-front rewrites the inner span first, which SHIFTS the outer span's end
        # offset — so the outer edit's `)` lands inside the text the inner edit just grew,
        # clobbering the call's closing paren. (It cost a mangled socklife.rs to notice.)
        # Keep only the innermost of any overlapping group: it is the more precise fix, and
        # the next round re-reports the outer one against clean offsets if it is still wrong.
        seen = set()
        uniq = []
        for e in sorted(es, key=lambda x: -x["start"]):
            key = (e["start"], e["end"])
            if key in seen:
                continue
            seen.add(key)
            uniq.append(e)

        kept: list[dict] = []
        for e in sorted(uniq, key=lambda x: x["end"] - x["start"]):  # innermost first
            if any(k["start"] >= e["start"] and k["end"] <= e["end"] for k in kept):
                continue  # this span CONTAINS one we already took — defer it to a later round
            kept.append(e)

        for e in sorted(kept, key=lambda x: -x["start"]):
            snippet = raw[e["start"] : e["end"]].decode("utf-8", "replace")
            # Idempotence: never double-cast a span that already ends in the cast we want.
            if re.search(rf"\bas\s+{re.escape(e['expected'])}\s*$", snippet):
                continue
            new = f"({snippet} as {e['expected']})".encode()
            raw = raw[: e["start"]] + new + raw[e["end"] :]
            changed += 1
        path.write_bytes(raw)
    return changed


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("-p", "--package", required=True)
    ap.add_argument("--target", required=True)
    ap.add_argument("--apply", action="store_true", help="write the edits (default: dry run)")
    ap.add_argument("--max-rounds", type=int, default=6)
    args = ap.parse_args()

    total = 0
    for rnd in range(1, args.max_rounds + 1):
        edits = collect(args.package, args.target)
        if not edits:
            print(f"round {rnd}: no mechanically-castable E0308 left.")
            break
        kinds = Counter((e["expected"], e["found"]) for e in edits)
        print(f"round {rnd}: {len(edits)} castable span(s)")
        for (exp, fnd), n in kinds.most_common(8):
            print(f"    {n:4d}  expected `{exp}`  found `{fnd}`")
        if not args.apply:
            print("\n(dry run — pass --apply to write)")
            return 0
        n = apply(edits)
        total += n
        print(f"    applied {n}")
        if n == 0:
            break

    if args.apply:
        print(f"\nTOTAL: {total} cast(s) applied.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
