#!/usr/bin/env python3
"""fix_bitfield_methods.py — deterministic bindgen bitfield-accessor repair.

bindgen lowers a C bitfield `T Foo : N;` into a packed `_bitfield_1` storage field plus a
getter/setter PAIR on the struct:  `pub fn Foo(&self) -> T`  and  `pub fn set_Foo(&mut self, val: T)`.
The LLM translates the C field access `x->Foo` / `x->Foo = v` as a plain `.Foo`, which rustc
rejects with E0615 ("attempted to take value of method `Foo`") for a read or E0615/E0070 for a
write. The correct Rust is fully determined:
    read   `expr.Foo`        ->  `expr.Foo()`
    write  `expr.Foo = val`  ->  `expr.set_Foo(val)`
rustc gives us the exact span of the offending `.Foo` token; we rewrite in place. No LLM.

Usage: fix_bitfield_methods.py <crate-src-dir> <target-triple>   (iterate until stable)
"""
import sys, re, os, json, subprocess
from collections import defaultdict

CRATE_SRC = sys.argv[1]
TARGET = sys.argv[2]
CRATE = os.path.basename(os.path.dirname(CRATE_SRC))
WS = os.path.abspath(os.path.join(CRATE_SRC, "../.."))


def rustc_json():
    r = subprocess.run(["cargo", "check", "-p", CRATE, "--target", TARGET,
                        "--message-format=json"], cwd=WS, capture_output=True, text=True)
    for ln in r.stdout.splitlines():
        try:
            msg = json.loads(ln)
        except ValueError:
            continue
        if msg.get("reason") == "compiler-message":
            yield msg["message"]


def find_write_value_end(s, eq_idx):
    """Given `... = <value>;` with `=` at eq_idx, return (value_start, value_end) where
    value is the RHS expression up to the statement-terminating `;` or `,` at paren-depth 0."""
    i = eq_idx + 1
    while i < len(s) and s[i] == " ":
        i += 1
    start = i
    depth = 0
    while i < len(s):
        c = s[i]
        if c in "([{":
            depth += 1
        elif c in ")]}":
            if depth == 0:
                break
            depth -= 1
        elif c in ";," and depth == 0:
            break
        i += 1
    return start, i


def main():
    # file -> list of (line, col0, method-name)
    hits = defaultdict(list)
    seen = set()
    for m in rustc_json():
        if (m.get("code") or {}).get("code") != "E0615":
            continue
        fm = re.search(r"value of method `([A-Za-z0-9_]+)`", m["message"])
        if not fm:
            continue
        sp = next((s for s in m["spans"] if s.get("is_primary")), None)
        if not sp:
            continue
        fname = os.path.basename(sp["file_name"])
        key = (fname, sp["line_start"], sp["column_start"])
        if key in seen:
            continue
        seen.add(key)
        hits[fname].append((sp["line_start"], sp["column_start"], fm.group(1)))

    applied = 0
    for fname, elist in hits.items():
        path = os.path.join(CRATE_SRC, fname)
        if not os.path.exists(path):
            continue
        lines = open(path).read().split("\n")
        # right-to-left within a line so columns stay valid
        for line, col, name in sorted(elist, key=lambda e: (-e[0], -e[1])):
            s = lines[line - 1]
            idx = col - 1
            if s[idx:idx + len(name)] != name or idx == 0 or s[idx - 1] != ".":
                continue
            after_pos = idx + len(name)
            after = s[after_pos:]
            # write context: `.Foo = <val>` (single `=`, not `==`/`!=`/`<=`/`>=`)
            wm = re.match(r"\s*=\s*[^=]", after)
            if wm:
                eq_rel = after.index("=")
                vs, ve = find_write_value_end(s, after_pos + eq_rel)
                value = s[vs:ve].rstrip()
                # `expr.Foo = value` -> `expr.set_Foo(value)`
                newline = s[:idx] + "set_" + name + "(" + value + ")" + s[ve:]
                lines[line - 1] = newline
                applied += 1
            else:
                # read context: `.Foo` -> `.Foo()`  (insert `()` after the name)
                lines[line - 1] = s[:after_pos] + "()" + s[after_pos:]
                applied += 1
        open(path, "w").write("\n".join(lines))
    print(f"applied {applied} bitfield-accessor fixes across {len(hits)} files")


if __name__ == "__main__":
    main()
