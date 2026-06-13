#!/usr/bin/env python3
"""
verify_tables.py — Data-table differ for the ntoskrnl C->Rust translation.

Catches FABRICATED lookup-table contents: Rust `static`/`const` fixed-size integer
arrays whose element values diverge from the same-named C array initializer.

This is the array-shaped sibling of the named-constant fabrication class
(e.g. BV_COLOR palette, SCAN_IN_PROGRESS). Named-constant checks (verify_constants.py)
compare scalar `const FOO = N;` against `#define FOO N`. They are BLIND to values
hidden *inside* an array initializer list:

    Rust:  static KiTrapIoTable: [UCHAR; 12] = [ 0xE4, 0xE5, 0xEC, ... ];
    C:     UCHAR KiTrapIoTable[] = { 0xE4, 0xE5, 0xEC, ... };

If the translator drops, reorders, or invents an element, only an element-by-element
diff finds it. That is what this tool does.

DESIGN
------
1. Scan Rust sources for module-level (and fn-local `static`) items:
        (static|const) NAME : [ELEM ; N] = [ literal-list ];
   where ELEM is an integer/byte type (u8/u16/u32/u64/i32/usize/ULONG/UCHAR/...).
   Parse the initializer list into integer values. Element exprs supported:
     - decimal / hex literals (0x..), with optional u8/u16/.. suffix
     - char/byte-char literals: b'A', 'A', b'\\\\', b'\\0', b'\\n' ...
     - `EXPR as TYPE` casts (the `as u16` idiom) -> unwrap
     - simple `A | B | C` bit-ORs of resolvable terms
     - named constants resolvable from C #defines (same name on both sides)
   Any element we can't confidently resolve -> the whole table is "unverifiable".

2. Find the C counterpart: grep ReactOS C/H sources for
        TYPE NAME [ ... ] = { ... };
   matched by exact name first, then case-insensitive + underscore-stripped
   (Rust may rename MmSysPteTables -> MM_SYS_PTE_TABLES). Parse its initializer
   list with the same resolver.

3. Diff element-by-element. Report name, Rust file:line, C file:line, count of
   differing elements, first few (index: rust vs c), and length mismatches.

4. CONSERVATIVE: only report a MISMATCH when BOTH sides parsed cleanly. If either
   side has an element we can't resolve, classify as UNVERIFIABLE, never mismatch.
   No guessing.

Self-test / default run:
    python3 tools/verify_tables.py
        -> scans sdk/lib/rust/ntoskrnl-rust/src, greps ntoskrnl/ + sdk/ for C arrays.

    python3 tools/verify_tables.py path/to/file_or_dir [...]   # restrict Rust scan
    python3 tools/verify_tables.py --verbose                    # list every table
"""

import argparse
import os
import re
import sys

REPO_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

DEFAULT_RUST_ROOT = os.path.join(REPO_ROOT, "sdk", "lib", "rust", "ntoskrnl-rust", "src")
# Where to look for C arrays + #defines.
C_SOURCE_ROOTS = [
    os.path.join(REPO_ROOT, "ntoskrnl"),
    os.path.join(REPO_ROOT, "sdk", "include"),
    os.path.join(REPO_ROOT, "sdk", "lib"),
]

INT_ELEM_TYPES = {
    "u8", "u16", "u32", "u64", "u128", "usize",
    "i8", "i16", "i32", "i64", "i128", "isize",
    "UCHAR", "USHORT", "ULONG", "ULONG64", "ULONG_PTR", "ULONGLONG",
    "CHAR", "SHORT", "LONG", "LONG64", "LONGLONG", "BYTE", "WORD", "DWORD",
    "WCHAR", "wchar_t", "UINT8", "UINT16", "UINT32", "UINT64",
    "INT8", "INT16", "INT32", "INT64", "SIZE_T",
}
# C element types we accept (superset; same as above plus a few C spellings)
C_INT_ELEM_TYPES = INT_ELEM_TYPES | {"unsigned", "int", "char", "short", "long"}


# ----------------------------------------------------------------------------
# #define harvesting (for resolving named constants like PAGE_NOACCESS)
# ----------------------------------------------------------------------------

_DEFINE_RE = re.compile(
    r'^\s*#\s*define\s+([A-Za-z_][A-Za-z0-9_]*)\s+(.+?)\s*(?:/\*.*)?$'
)


def harvest_defines(roots):
    """Return {NAME: raw_value_str} of all object-like #defines under roots.

    First definition wins (mimics include order roughly; conflicts are rare for
    the constants we resolve). Function-like macros (NAME(...)) are skipped.
    """
    defines = {}
    for root in roots:
        if not os.path.isdir(root):
            continue
        for dirpath, _dirs, files in os.walk(root):
            for fn in files:
                if not fn.endswith((".h", ".c", ".hpp")):
                    continue
                path = os.path.join(dirpath, fn)
                try:
                    with open(path, "r", errors="replace") as f:
                        for line in f:
                            m = _DEFINE_RE.match(line)
                            if not m:
                                continue
                            name, val = m.group(1), m.group(2)
                            # skip function-like macros
                            if name.endswith("(") or re.match(r'^[A-Za-z_]\w*\(', line.strip()[len("#define"):].strip()):
                                pass
                            # better function-like detection: char right after name in the raw line
                            after = line.split(name, 1)[1] if name in line else ""
                            if after.startswith("("):
                                continue
                            if name not in defines:
                                defines[name] = val.strip()
                except OSError:
                    continue
    return defines


# ----------------------------------------------------------------------------
# Integer-expression evaluation (shared by both sides)
# ----------------------------------------------------------------------------

class Unresolvable(Exception):
    pass


_CHAR_ESCAPES = {
    "\\0": 0, "\\n": 10, "\\r": 13, "\\t": 9, "\\\\": 92,
    "\\'": 39, "\\\"": 34, "\\0": 0,
}


def _parse_char_literal(s):
    """Parse a Rust/C char or byte-char literal body (without quotes resolved).

    Accepts forms:  b'A'  'A'  b'\\\\'  '\\0'  '\\n'  b'\\x41'
    Returns int or raises Unresolvable.
    """
    s = s.strip()
    m = re.match(r"^b?'(.*)'$", s, re.DOTALL)
    if not m:
        raise Unresolvable(s)
    body = m.group(1)
    if len(body) == 1:
        return ord(body)
    # escape sequences
    if body.startswith("\\x"):
        try:
            return int(body[2:], 16)
        except ValueError:
            raise Unresolvable(s)
    if body in _CHAR_ESCAPES:
        return _CHAR_ESCAPES[body]
    # generic single backslash escape
    if body == "\\\\":
        return 92
    if len(body) == 2 and body[0] == "\\":
        simple = {"0": 0, "n": 10, "r": 13, "t": 9, "\\": 92, "'": 39, '"': 34}
        if body[1] in simple:
            return simple[body[1]]
    raise Unresolvable(s)


def _strip_int_suffix(tok):
    # 0x10u8, 123ULONG-no... handle rust suffixes & C UL/L suffixes
    tok = tok.strip()
    # rust suffixes
    tok = re.sub(r'(?<=[0-9a-fA-F])(_?(u|i)(8|16|32|64|128|size))$', '', tok)
    # underscores in rust int literals
    tok2 = tok.replace("_", "")
    # C suffixes
    tok2 = re.sub(r'(?i)(u|l)+$', '', tok2)
    return tok2


def eval_int_expr(expr, defines, depth=0):
    """Evaluate an integer expression to an int, or raise Unresolvable.

    Supported: int/hex literals, char literals, `EXPR as TYPE` (Rust),
    C casts `(TYPE)EXPR`, `A | B`, `A + B` (rare), parentheses, and named
    constants resolved from `defines`. Intentionally conservative.
    """
    if depth > 40:
        raise Unresolvable(expr)
    e = expr.strip()
    if not e:
        raise Unresolvable(expr)

    # strip trailing line comments
    e = re.sub(r'/\*.*?\*/', ' ', e).strip()
    e = re.sub(r'//.*$', '', e).strip()

    # Rust cast: `EXPR as TYPE`  -> evaluate EXPR
    m = re.match(r'^(.*)\bas\s+[A-Za-z_]\w*\s*$', e, re.DOTALL)
    if m:
        return eval_int_expr(m.group(1), defines, depth + 1)

    # C cast: `(TYPE)EXPR`
    m = re.match(r'^\(\s*(?:unsigned\s+|signed\s+|const\s+)*[A-Za-z_]\w*\s*\*?\s*\)\s*(.+)$', e)
    if m and m.group(1).strip():
        # only treat as cast if the inner thing isn't itself a paren group we need
        try:
            return eval_int_expr(m.group(1), defines, depth + 1)
        except Unresolvable:
            pass

    # Bitwise OR (lowest precedence we support)
    # split on top-level '|' (not '||')
    parts = _split_top_level(e, "|")
    if len(parts) > 1:
        acc = 0
        for p in parts:
            acc |= eval_int_expr(p, defines, depth + 1)
        return acc

    # Addition (some tables use A + B)
    parts = _split_top_level(e, "+")
    if len(parts) > 1:
        acc = 0
        for p in parts:
            acc += eval_int_expr(p, defines, depth + 1)
        return acc

    # Left shift A << B
    parts = _split_top_level(e, "<<")
    if len(parts) == 2:
        return eval_int_expr(parts[0], defines, depth + 1) << eval_int_expr(parts[1], defines, depth + 1)

    # Multiplication A * B (e.g. quantum tables: `1 * 6`). Skip '*'-as-deref:
    # only treat as multiply when both sides parse as integer exprs.
    parts = _split_top_level(e, "*")
    if len(parts) > 1 and all(p.strip() for p in parts):
        acc = 1
        try:
            for p in parts:
                acc *= eval_int_expr(p, defines, depth + 1)
            return acc
        except Unresolvable:
            pass

    # parenthesized
    if e.startswith("(") and e.endswith(")"):
        inner = e[1:-1].strip()
        if _balanced(inner):
            return eval_int_expr(inner, defines, depth + 1)

    # char/byte-char literal
    if re.match(r"^b?'", e):
        return _parse_char_literal(e)

    # numeric literal
    cleaned = _strip_int_suffix(e)
    if re.match(r'^0[xX][0-9a-fA-F]+$', cleaned):
        return int(cleaned, 16)
    if re.match(r'^0[bB][01]+$', cleaned):
        return int(cleaned, 2)
    if re.match(r'^[+-]?[0-9]+$', cleaned):
        return int(cleaned, 10)

    # named constant -> resolve via defines
    m = re.match(r'^[A-Za-z_]\w*$', e)
    if m:
        if e in defines:
            return eval_int_expr(defines[e], defines, depth + 1)
        raise Unresolvable(e)

    raise Unresolvable(expr)


def _split_top_level(s, op):
    """Split s on `op` only at paren-depth 0. Avoid splitting '||' / '&&' tokens
    when op is a single char by checking neighbours minimally."""
    out = []
    depth = 0
    i = 0
    cur = []
    L = len(op)
    while i < len(s):
        c = s[i]
        if c in "([":
            depth += 1
            cur.append(c)
            i += 1
            continue
        if c in ")]":
            depth -= 1
            cur.append(c)
            i += 1
            continue
        if depth == 0 and s[i:i + L] == op:
            # guard against '||' when op is '|'
            if op == "|" and (s[i:i + 2] == "||"):
                cur.append(s[i:i + 2])
                i += 2
                continue
            out.append("".join(cur))
            cur = []
            i += L
            continue
        cur.append(c)
        i += 1
    out.append("".join(cur))
    return out


def _balanced(s):
    depth = 0
    for c in s:
        if c in "([":
            depth += 1
        elif c in ")]":
            depth -= 1
            if depth < 0:
                return False
    return depth == 0


# ----------------------------------------------------------------------------
# Initializer list splitting (comma-separated, paren/bracket aware)
# ----------------------------------------------------------------------------

def split_initializer(body):
    """Split a `{...}` / `[...]` body (without the brackets) into element exprs.

    Strips comments first, then splits on top-level commas. Trailing empty
    element (from a trailing comma) is dropped.
    """
    # remove block + line comments
    body = re.sub(r'/\*.*?\*/', ' ', body, flags=re.DOTALL)
    body = re.sub(r'//[^\n]*', ' ', body)
    elems = []
    depth = 0
    cur = []
    for c in body:
        if c in "([{":
            depth += 1
            cur.append(c)
        elif c in ")]}":
            depth -= 1
            cur.append(c)
        elif c == "," and depth == 0:
            elems.append("".join(cur).strip())
            cur = []
        else:
            cur.append(c)
    last = "".join(cur).strip()
    if last:
        elems.append(last)
    # drop empties
    return [e for e in elems if e.strip()]


# ----------------------------------------------------------------------------
# Rust table extraction
# ----------------------------------------------------------------------------

# matches:  [pub] static|const NAME : [ ELEM ; N ] = [
_RUST_DECL_RE = re.compile(
    r'\b(?:pub\s+)?(?:static|const)\s+(?:mut\s+)?'
    r'([A-Za-z_]\w*)\s*:\s*'
    r'\[\s*([A-Za-z_]\w*)\s*;\s*([0-9A-Za-z_]+)\s*\]\s*=\s*\['
)


def extract_rust_tables(path):
    """Yield dicts: {name, elem_type, decl_len(str), values|None, unresolved,
    file, line} for each module/fn-local fixed integer array with a literal list."""
    try:
        with open(path, "r", errors="replace") as f:
            text = f.read()
    except OSError:
        return
    results = []
    for m in _RUST_DECL_RE.finditer(text):
        name, elem, declN = m.group(1), m.group(2), m.group(3)
        if elem not in INT_ELEM_TYPES:
            continue
        # find matching closing ']' for the initializer
        start = m.end() - 1  # position of '['
        body, end = _extract_bracket_body(text, start, "[", "]")
        if body is None:
            continue
        line = text.count("\n", 0, m.start()) + 1
        elems = split_initializer(body)
        values, unresolved = _resolve_elems(elems)
        results.append({
            "name": name,
            "elem_type": elem,
            "decl_len": declN,
            "raw_count": len(elems),
            "values": values,
            "unresolved": unresolved,
            "file": path,
            "line": line,
        })
    return results


def _extract_bracket_body(text, open_idx, open_ch, close_ch):
    """text[open_idx]==open_ch. Return (body_without_brackets, idx_after_close)."""
    assert text[open_idx] == open_ch
    depth = 0
    i = open_idx
    while i < len(text):
        c = text[i]
        if c == open_ch:
            depth += 1
        elif c == close_ch:
            depth -= 1
            if depth == 0:
                return text[open_idx + 1:i], i + 1
        i += 1
    return None, None


def _resolve_elems(elems, defines=None):
    """Resolve element exprs to ints. Returns (values_or_None, unresolved_list).

    `defines` may be supplied to resolve named constants; pass None to resolve
    only literals/chars (used as a first cheap pass)."""
    defines = defines or {}
    values = []
    unresolved = []
    for e in elems:
        try:
            values.append(eval_int_expr(e, defines))
        except Unresolvable:
            unresolved.append(e)
            values.append(None)
    if unresolved:
        return None, unresolved
    return values, []


# ----------------------------------------------------------------------------
# C array extraction
# ----------------------------------------------------------------------------

# matches a C array definition with an initializer:  TYPE NAME[...] = {
# allows leading qualifiers (const/static/extern/volatile/unsigned/signed)
_C_DECL_RE = re.compile(
    r'(?:^|[\n;}])\s*'
    r'(?:(?:const|static|extern|volatile|unsigned|signed)\s+)*'
    r'([A-Za-z_]\w*)\s+'          # base type
    r'([A-Za-z_]\w*)\s*'          # name
    r'\[[^\];=]*\]\s*'            # [N] or []
    r'=\s*\{',
    re.MULTILINE,
)


def build_c_array_index(roots):
    """Scan C sources once; return {name: [entry,...]} where entry has
    {name, base_type, body, file, line}. Only integer-typed arrays kept."""
    index = {}
    for root in roots:
        if not os.path.isdir(root):
            continue
        for dirpath, _dirs, files in os.walk(root):
            for fn in files:
                if not fn.endswith((".c", ".h", ".cpp", ".hpp")):
                    continue
                path = os.path.join(dirpath, fn)
                try:
                    with open(path, "r", errors="replace") as f:
                        text = f.read()
                except OSError:
                    continue
                for m in _C_DECL_RE.finditer(text):
                    base_type, name = m.group(1), m.group(2)
                    if base_type not in C_INT_ELEM_TYPES:
                        continue
                    brace_idx = m.end() - 1
                    body, _end = _extract_bracket_body(text, brace_idx, "{", "}")
                    if body is None:
                        continue
                    line = text.count("\n", 0, m.start()) + 1
                    index.setdefault(name, []).append({
                        "name": name,
                        "base_type": base_type,
                        "body": body,
                        "file": path,
                        "line": line,
                    })
    return index


def _norm(name):
    return name.replace("_", "").lower()


ARCH_TOKENS = ("arm64", "amd64", "i386", "arm", "x86", "x64", "aarch64")


def _arch_of(path):
    """Return the arch token embedded in a path (as a path component), or None.

    The SAME table name often has per-arch C definitions (e.g.
    ntoskrnl/ke/arm64/stubs.c AND ntoskrnl/mm/i386/page.c both define
    MmProtectToValue with intentionally different values). To avoid a
    cross-arch false positive we must diff the arm64 Rust file against the
    arm64 C file, not the i386 one.
    """
    parts = re.split(r'[\\/]', path)
    for p in parts:
        if p in ARCH_TOKENS:
            return p
    return None


def _rank_c_entry(rust_file, ce):
    """Lower is better. Prefer same-arch, then same-directory affinity."""
    rust_arch = _arch_of(rust_file)
    c_arch = _arch_of(ce["file"])
    score = 0
    if rust_arch is not None and c_arch is not None:
        if rust_arch != c_arch:
            score += 100          # cross-arch: strongly deprioritize
    elif rust_arch is not None and c_arch is None:
        score += 10               # rust is arch-specific, C is generic
    elif rust_arch is None and c_arch is not None:
        score += 10
    # directory basename affinity (ke/, mm/, fsrtl/ ...)
    rdir = os.path.basename(os.path.dirname(rust_file))
    cdir = os.path.basename(os.path.dirname(ce["file"]))
    if rdir != cdir:
        score += 1
    return score


def find_c_match(rust_name, c_index, c_norm_index):
    """Return (entries, exact: bool) or (None, False)."""
    # exact name
    if rust_name in c_index:
        return c_index[rust_name], True
    # case-insensitive / underscore-stripped
    key = _norm(rust_name)
    if key in c_norm_index:
        return c_norm_index[key], False
    return None, False


def _is_case_flip(a, b):
    """True if a,b are the same ASCII letter differing only in case."""
    if a is None or b is None:
        return False
    if a == b:
        return True
    if 0 <= a < 128 and 0 <= b < 128:
        ca, cb = chr(a), chr(b)
        return ca.isalpha() and cb.isalpha() and ca.lower() == cb.lower()
    return False


# ----------------------------------------------------------------------------
# Main diff driver
# ----------------------------------------------------------------------------

def gather_rust_files(paths):
    files = []
    for p in paths:
        if os.path.isfile(p) and p.endswith(".rs"):
            files.append(p)
        elif os.path.isdir(p):
            for dp, _d, fs in os.walk(p):
                for fn in fs:
                    if fn.endswith(".rs"):
                        files.append(os.path.join(dp, fn))
    return sorted(set(files))


def rel(path):
    try:
        return os.path.relpath(path, REPO_ROOT)
    except ValueError:
        return path


def main():
    ap = argparse.ArgumentParser(description="Diff Rust integer data tables vs C arrays.")
    ap.add_argument("paths", nargs="*", help="Rust files/dirs to scan (default: ntoskrnl-rust/src)")
    ap.add_argument("--verbose", action="store_true", help="List every table + classification")
    ap.add_argument("--max-diffs", type=int, default=8, help="Max per-table diffs to print")
    args = ap.parse_args()

    rust_paths = args.paths or [DEFAULT_RUST_ROOT]
    rust_files = gather_rust_files(rust_paths)

    print(f"[*] Scanning {len(rust_files)} Rust files under: "
          + ", ".join(rel(p) for p in rust_paths), file=sys.stderr)

    # 1. Rust tables
    rust_tables = []
    for rf in rust_files:
        if "old_translations" in rf or rf.endswith(".bak"):
            continue
        rust_tables.extend(extract_rust_tables(rf))

    print(f"[*] Building C array index + #define table (this scans ntoskrnl/ + sdk/) ...",
          file=sys.stderr)
    c_index = build_c_array_index(C_SOURCE_ROOTS)
    c_norm_index = {}
    for nm, entries in c_index.items():
        c_norm_index.setdefault(_norm(nm), []).extend(entries)
    defines = harvest_defines(C_SOURCE_ROOTS)
    print(f"[*] C arrays indexed: {len(c_index)} names; #defines: {len(defines)}",
          file=sys.stderr)

    matched = 0
    mismatched = []
    unverifiable = []
    clean = []
    unmatched = 0

    for rt in rust_tables:
        c_entries, exact_match = find_c_match(rt["name"], c_index, c_norm_index)
        if not c_entries:
            unmatched += 1
            if args.verbose:
                print(f"    [no-C-match] {rt['name']}  {rel(rt['file'])}:{rt['line']}")
            continue

        # Re-resolve Rust elems WITH defines (named constants like PAGE_NOACCESS)
        rust_elems_raw = _reparse_rust_elems(rt)
        rust_vals, rust_unres = _resolve_elems(rust_elems_raw, defines)

        # Pick the best C entry. Rank by arch/dir affinity FIRST (so an arm64
        # Rust file diffs against the arm64 C array, not the i386 one), then
        # among equally-ranked entries prefer one that parses cleanly.
        ranked = sorted(c_entries, key=lambda ce: _rank_c_entry(rt["file"], ce))
        best_rank = _rank_c_entry(rt["file"], ranked[0])
        top = [ce for ce in ranked if _rank_c_entry(rt["file"], ce) == best_rank]
        best = None
        best_vals = None
        best_unres = None
        for ce in top:
            c_elems = split_initializer(ce["body"])
            cv, cu = _resolve_elems(c_elems, defines)
            if cv is not None:
                best, best_vals, best_unres = ce, cv, cu
                break
            if best is None:
                best, best_vals, best_unres = ce, cv, cu
        ce = best
        matched += 1

        if rust_vals is None or best_vals is None:
            reason = []
            if rust_vals is None:
                reason.append(f"rust unresolved: {rust_unres[:3]}")
            if best_vals is None:
                reason.append(f"c unresolved: {best_unres[:3]}")
            unverifiable.append((rt, ce, "; ".join(reason)))
            continue

        # Both resolved. Diff.
        diffs = []
        n = max(len(rust_vals), len(best_vals))
        for i in range(n):
            rv = rust_vals[i] if i < len(rust_vals) else None
            cv = best_vals[i] if i < len(best_vals) else None
            if rv != cv:
                diffs.append((i, rv, cv))
        if len(rust_vals) != len(best_vals) or diffs:
            # Wrong-match guard: a FUZZY (non-exact) name match whose every
            # differing element is merely an ASCII letter case-flip is almost
            # certainly two DIFFERENT strings that normalize to the same name
            # (e.g. Rust "Control" matched to an unrelated C "CONTROL"), not a
            # fabrication. Reclassify as unverifiable rather than cry wolf.
            if (not exact_match
                    and len(rust_vals) == len(best_vals)
                    and diffs
                    and all(_is_case_flip(a, b) for _i, a, b in diffs)):
                unverifiable.append(
                    (rt, ce, f"fuzzy name-match + diffs are pure case-flips "
                             f"(likely UNRELATED C array '{ce['name']}')"))
                continue
            mismatched.append((rt, ce, rust_vals, best_vals, diffs, exact_match))
        else:
            clean.append((rt, ce))

    # ---- Report ----
    print()
    print("=" * 78)
    print("DATA-TABLE DIFFER REPORT")
    print("=" * 78)
    print(f"Rust integer tables found : {len(rust_tables)}")
    print(f"  matched to a C array    : {matched}")
    print(f"  no C counterpart        : {unmatched}")
    print(f"  -> clean (identical)    : {len(clean)}")
    print(f"  -> MISMATCH             : {len(mismatched)}")
    print(f"  -> unverifiable         : {len(unverifiable)}")
    print()

    if mismatched:
        print("#" * 78)
        print(f"# {len(mismatched)} MISMATCHED TABLE(S) — possible fabrication")
        print("#" * 78)
        for rt, ce, rv, cv, diffs, exact in mismatched:
            print()
            match_kind = "exact-name" if exact else "fuzzy-name"
            print(f"  TABLE: {rt['name']}   (elem {rt['elem_type']})   [{match_kind} match]")
            print(f"    Rust: {rel(rt['file'])}:{rt['line']}   len={len(rv)}")
            print(f"    C   : {rel(ce['file'])}:{ce['line']}   len={len(cv)}  (C name: {ce['name']})")
            if len(rv) != len(cv):
                print(f"    !! LENGTH MISMATCH: rust={len(rv)} c={len(cv)}")
            print(f"    differing elements: {len(diffs)}")
            for i, a, b in diffs[:args.max_diffs]:
                sa = "MISSING" if a is None else f"{a}(0x{a:x})"
                sb = "MISSING" if b is None else f"{b}(0x{b:x})"
                print(f"      [{i}] rust={sa}  c={sb}")
            if len(diffs) > args.max_diffs:
                print(f"      ... +{len(diffs) - args.max_diffs} more")
        print()

    if args.verbose:
        if clean:
            print("-" * 60)
            print(f"CLEAN ({len(clean)}):")
            for rt, ce in clean:
                print(f"  OK {rt['name']:<32} {rel(rt['file'])}:{rt['line']}"
                      f"  == {rel(ce['file'])}:{ce['line']}")
        if unverifiable:
            print("-" * 60)
            print(f"UNVERIFIABLE ({len(unverifiable)}):")
            for rt, ce, reason in unverifiable:
                print(f"  ?? {rt['name']:<32} {rel(rt['file'])}:{rt['line']}  [{reason}]")
    else:
        if unverifiable:
            print(f"({len(unverifiable)} unverifiable tables not shown; use --verbose)")

    return 1 if mismatched else 0


def _reparse_rust_elems(rt):
    """Re-extract the raw element exprs for a rust table dict."""
    try:
        with open(rt["file"], "r", errors="replace") as f:
            text = f.read()
    except OSError:
        return []
    # locate the decl again at the recorded line
    for m in _RUST_DECL_RE.finditer(text):
        if m.group(1) != rt["name"]:
            continue
        line = text.count("\n", 0, m.start()) + 1
        if line != rt["line"]:
            continue
        body, _ = _extract_bracket_body(text, m.end() - 1, "[", "]")
        if body is None:
            return []
        return split_initializer(body)
    return []


if __name__ == "__main__":
    sys.exit(main())
