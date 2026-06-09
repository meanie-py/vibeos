#!/usr/bin/env python3
"""fix_misc_method_variadic.py — deterministic repair for three residual rustc families
that surface AFTER the import/field/cast/bitfield cascade has run:

  E0599 "no method named `cast` found for fn item ..."   -> a fn-NAME got wrapped in `.cast()`
        by fix_arg_casts (it can't tell a fn item from a pointer expr). Strip the `.cast()`.
  E0599 "no method named `X` found for struct `__BindgenUnionField<T>`"  -> a bindgen union
        member needs `.as_ref()` (read) / `.as_mut()` (write, incl. `set_` setters) before the
        accessor. Insert it. (Same shape fix_field_paths does for E0609, but the bitfield-method
        pass turned `.X` into `.X()` first, so it now reports as E0599 not E0609.)
  E0617 "can't pass `u8`/`u16`/`i8`/`i16` to variadic function"  -> C default argument promotion:
        an integer narrower than `int` is promoted to `int` at a `...` call. Cast the arg `as i32`.

All three are mechanical and span-exact. No LLM.

Usage: fix_misc_method_variadic.py <crate-src-dir> <target-triple>   (iterate until stable)
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


def main():
    # file -> list of (line, col0, kind, payload)
    edits = defaultdict(list)
    seen = set()
    for m in rustc_json():
        code = (m.get("code") or {}).get("code")
        txt = m["message"]
        sp = next((s for s in m["spans"] if s.get("is_primary")), None)
        if not sp:
            continue
        fname = os.path.basename(sp["file_name"])
        key = (fname, sp["line_start"], sp["column_start"], code)
        if key in seen:
            continue
        if code == "E0599":
            fm = re.search(r"no method named `([A-Za-z0-9_]+)` found for (fn item|struct `[^`]*?(__BindgenUnionField)[^`]*`)", txt)
            if not fm:
                continue
            method, kind_txt = fm.group(1), fm.group(2)
            if method == "cast" and kind_txt == "fn item":
                seen.add(key)
                edits[fname].append((sp["line_start"], sp["column_start"], "strip_cast", None))
            elif "__BindgenUnionField" in kind_txt:
                seen.add(key)
                want_mut = method.startswith("set_")
                edits[fname].append((sp["line_start"], sp["column_start"], "union", want_mut))
        elif code == "E0617":
            if re.search(r"can't pass `[iu](?:8|16)` to variadic function", txt):
                seen.add(key)
                edits[fname].append((sp["line_start"], sp["column_start"],
                                     "promote", (sp["column_end"])))

    applied = 0
    for fname, elist in edits.items():
        path = os.path.join(CRATE_SRC, fname)
        if not os.path.exists(path):
            continue
        lines = open(path).read().split("\n")
        for line, col, kind, payload in sorted(elist, key=lambda e: (-e[0], -e[1])):
            s = lines[line - 1]
            idx = col - 1
            if kind == "strip_cast":
                # span points at `cast` in `.cast()`; remove the leading `.` through `()`
                # find the `.` immediately before the method name
                dot = s.rfind(".", 0, idx)
                m = re.match(r"\.cast\s*\(\s*\)", s[dot:]) if dot != -1 else None
                if m:
                    lines[line - 1] = s[:dot] + s[dot + m.end():]
                    applied += 1
            elif kind == "union":
                # span points at the method name after `MEMBER.` ; insert as_ref()/as_mut()
                if idx > 0 and s[idx - 1] == ".":
                    ins = "as_mut()." if payload else "as_ref()."
                    lines[line - 1] = s[:idx] + ins + s[idx:]
                    applied += 1
            elif kind == "promote":
                # span covers the arg expression [col, col_end); wrap as (expr) as i32
                col_end = payload
                expr = s[idx:col_end - 1]
                lines[line - 1] = s[:idx] + "(" + expr + ") as i32" + s[col_end - 1:]
                applied += 1
        open(path, "w").write("\n".join(lines))
    print(f"applied {applied} misc method/variadic fixes across {len(edits)} files")


if __name__ == "__main__":
    main()
