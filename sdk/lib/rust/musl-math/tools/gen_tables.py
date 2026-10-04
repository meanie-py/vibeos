#!/usr/bin/env python3
"""gen_tables.py -- musl's ARM-routine coefficient tables (src/math/*_data.c) as Rust.

Rust has no hex float literal, so every `0x1.71547652b82fep0` becomes
`f64::from_bits(0x...)` (or f32), exact by construction -- float.fromhex is exact and the
bits are what the C compiler would emit. Integer table entries (`tab[]` of uint64) stay
integers. Configuration matches how we port the code: EXP_USE_TOINT_NARROW = 0 and no
__FP_FAST_FMA, so the `#if` arms taken are the non-FMA ones on every arch.

Usage: gen_tables.py <musl src/math dir> > src/tables.rs
"""
import re
import struct
import sys
from pathlib import Path

# struct name -> (data file, [(field, kind, count-or-None)]) ; kind f64/f32/u64/pair64/pair32
SPEC = {
    "EXP": ("exp_data.c", "f64", [
        ("invln2N", "f64", None), ("shift", "f64", None), ("negln2hiN", "f64", None),
        ("negln2loN", "f64", None), ("poly", "f64", 4), ("exp2_shift", "f64", None),
        ("exp2_poly", "f64", 5), ("tab", "u64", 256)]),
    "LOG": ("log_data.c", "f64", [
        ("ln2hi", "f64", None), ("ln2lo", "f64", None), ("poly", "f64", 5), ("poly1", "f64", 11),
        ("tab", "f64", 256), ("tab2", "f64", 256)]),
    "LOG2": ("log2_data.c", "f64", [
        ("invln2hi", "f64", None), ("invln2lo", "f64", None), ("poly", "f64", 6), ("poly1", "f64", 10),
        ("tab", "f64", 128), ("tab2", "f64", 128)]),
    "POW_LOG": ("pow_data.c", "f64", [
        ("ln2hi", "f64", None), ("ln2lo", "f64", None), ("poly", "f64", 7), ("tab", "f64", 512)]),
    "EXP2F": ("exp2f_data.c", "f32", [
        ("tab", "u64", 32), ("shift_scaled", "f64", None), ("poly", "f64", 3), ("shift", "f64", None),
        ("invln2_scaled", "f64", None), ("poly_scaled", "f64", 3)]),
    "LOGF": ("logf_data.c", "f32", [("tab", "f64", 32), ("ln2", "f64", None), ("poly", "f64", 3)]),
    "LOG2F": ("log2f_data.c", "f32", [("tab", "f64", 32), ("poly", "f64", 4)]),
    "POWF_LOG2": ("powf_data.c", "f32", [("tab", "f64", 32), ("poly", "f64", 5)]),
}


def strip(text: str) -> str:
    text = re.sub(r"/\*.*?\*/", "", text, flags=re.S)
    text = re.sub(r"//[^\n]*", "", text)
    # pow_data.c's `#define A(a, b, c) {a, 0, b, c},` -- the zero is the struct's pad.
    text = re.sub(r"^\s*A\(([^,]+),([^,]+),([^)]+)\)", r"{\1, 0,\2,\3},", text, flags=re.M)
    # Resolve the only conditionals the data files carry, to the arms we port.
    out, stack = [], []
    for line in text.splitlines():
        s = line.strip()
        if s.startswith("#if"):
            cond = s
            take = not ("__FP_FAST_FMA" in cond and "!" not in cond) and "EXP_USE_TOINT_NARROW" not in cond
            if "!__FP_FAST_FMA" in cond or "!(__FP_FAST_FMA" in cond:
                take = True
            stack.append(take)
            continue
        if s.startswith("#else"):
            stack[-1] = not stack[-1]
            continue
        if s.startswith("#endif"):
            stack.pop()
            continue
        if s.startswith("#"):
            continue
        if all(stack):
            out.append(line)
    return "\n".join(out)


NUM = re.compile(r"[-+]?0x[0-9a-fA-F]*\.?[0-9a-fA-F]*p[-+]?\d+f?|[-+]?0x[0-9a-fA-F]+[uUlL]*|[-+]?\d+\.\d*(?:e[-+]?\d+)?")


def fields_of(body: str) -> dict[str, list[str]]:
    """`.name = expr` / `.name = { ... }` designated initializers, by NAME -- the .c files
    initialize in a different order from the struct declaration (exp_data.c puts
    negln2hiN before shift), so position is not a key."""
    out: dict[str, list[str]] = {}
    for m in re.finditer(r"\.([A-Za-z0-9_]+)\s*=\s*", body):
        name, i = m.group(1), m.end()
        if body[i] == "{":
            depth, j = 0, i
            while True:
                if body[j] == "{":
                    depth += 1
                elif body[j] == "}":
                    depth -= 1
                    if depth == 0:
                        break
                j += 1
            inner = body[i + 1:j]
        else:
            j = i
            while j < len(body) and body[j] not in ",}":
                j += 1
            inner = body[i:j]
        vals = [t.strip().strip("{}").strip() for t in re.split(r",", inner.replace("{", ",").replace("}", ","))]
        out[name] = [v for v in vals if v]
    return out


def evaluate(expr: str, n: int) -> float | int:
    # POWF_SCALE is 1 << POWF_SCALE_BITS, and POWF_SCALE_BITS is 0 without TOINT_INTRINSICS.
    expr = expr.replace("POWF_SCALE", "1.0")
    expr = re.sub(r"\bN\b", str(n), expr)
    # A float suffix on a DECIMAL literal only (hex floats take theirs below). A blanket
    # rstrip("f") ate the last digit of every table integer ending in f: 0x...890f.
    expr = re.sub(r"(\d\.\d*(?:[eE][-+]?\d+)?)[fF]\b", r"\1", expr)
    expr = re.sub(r"\b(0x[0-9a-fA-F]+)(?:ULL|ull|ul|UL|u|U)\b", r"\1", expr)

    def hexf(m):
        return repr(float.fromhex(m.group(0).rstrip("f")))
    expr = re.sub(r"0x[0-9a-fA-F]*\.[0-9a-fA-F]*p[-+]?\d+f?|0x[0-9a-fA-F]+p[-+]?\d+f?", hexf, expr)
    return eval(expr, {"__builtins__": {}})


def main() -> int:
    src = Path(sys.argv[1])
    out = ["// GENERATED by tools/gen_tables.py from musl src/math/*_data.c -- do not edit.",
           "// Copyright (c) 2017-2018, Arm Limited. SPDX-License-Identifier: MIT (see ref/COPYRIGHT).",
           "#![allow(clippy::unreadable_literal, clippy::excessive_precision)]", ""]
    nbits = {"exp_data.c": 128, "log_data.c": 128, "log2_data.c": 64, "pow_data.c": 128,
             "exp2f_data.c": 32, "logf_data.c": 16, "log2f_data.c": 16, "powf_data.c": 16}
    for name, (fname, _width, fields) in SPEC.items():
        text = strip((src / fname).read_text())
        body = text[text.index("{") + 1: text.rindex("}")]
        byname = fields_of(body)
        n = nbits[fname]
        if set(byname) != {f for f, _, _ in fields}:
            sys.stderr.write(f"{fname}: fields {sorted(byname)} != spec {sorted(f for f, _, _ in fields)}\n")
            return 1
        for field, kind, count in fields:
            items = byname[field]
            if len(items) != (count or 1):
                sys.stderr.write(f"{fname}.{field}: {len(items)} values, spec wants {count or 1}\n")
                return 1
            const = f"{name}_{field.upper()}"
            if kind == "u64":
                rs = [f"0x{evaluate(v, n):016x}" for v in items]
                ty = "u64"
            else:
                rs = [f"f64::from_bits(0x{struct.unpack('<Q', struct.pack('<d', float(evaluate(v, n))))[0]:016x})" for v in items]
                ty = "f64"
            if count is None:
                out.append(f"pub(crate) const {const}: {ty} = {rs[0]};")
            else:
                out.append(f"pub(crate) static {const}: [{ty}; {count}] = [")
                out += [f"    {r}," for r in rs]
                out.append("];")
        out.append("")
    print("\n".join(out).rstrip("\n"))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
