#!/usr/bin/env python3
"""
scan_cfg_vs_c.py — a per-arch `#[cfg]` whose arch set the C twin never draws.

The C decides which architectures share a code path: `#ifdef _WIN64 … #else …` puts
amd64 and arm64 on one side, i386 on the other. A translation that re-draws that line —
`#[cfg(target_arch = "x86_64")]` for `_WIN64` — hands one architecture the other path.
It builds and boots everywhere and is correct on the arch everyone tests, so nothing
notices: July's audit found the PXE 4-level cluster this way by reading 2,453 sites, and
user32's GetClassLongA/W (arm64 got the 32-bit index handling) was found by reading again.
This makes the comparison mechanical wherever the C has a conditional to compare against.

HOW. Arch universe U = {x86, x86_64, aarch64}.
  1. For each Rust fn body carrying `#[cfg(...)]` sites, evaluate each predicate to a
     subset of U (`target_arch`, `target_pointer_width`, `not`/`any`/`all`). A predicate
     that mentions anything else (`feature`, `target_os`, `debug_assertions`) is UNKNOWN
     and skipped: this tool never guesses.
  2. The C function of the same name (tools/c_twin.py, same module): evaluate every
     `#if`/`#ifdef`/`#ifndef`/`#elif`/`#else` branch in its body the same way
     (`_M_IX86`, `_M_AMD64`, `_M_ARM64`, `_WIN64`, `_X86_`, …). A branch naming a
     non-arch macro is UNKNOWN and skipped.
  3. The known C branch sets generate a partition of U into ATOMS (arches the C never
     separates). A Rust set that is not a union of atoms SPLITS an atom: it gives two
     arches the C keeps together different code. That is the finding.
     Unless the C names an arm64 macro somewhere in those conditionals, only the
     x86/x86_64 line is judged: a C `#ifdef _M_AMD64 … #else` lumps arm64 with i386
     because the code predates upstream's arm64 port, and a translation that groups
     arm64 with amd64 instead (4-level paging) exceeds the C rather than drifting.
  4. Exempt: a cfg attached to `asm!`/`global_asm!`/`use`/`extern`/`type`/`mod`, or to a
     statement reaching a `__bindgen_anon_N` member — per-arch spellings Rust needs even
     where the C has one portable intrinsic or one field name. So is a function the crate
     defines twice (per-arch FILE twins: i386/page.rs vs arm64/page.rs).
  Known blind spot: the comparison is per FUNCTION, not per block. A Rust-only block (a
  diagnostic, a bring-up allowance) in a function that also has an unrelated C `#if` is
  flagged; read it and mark it `nolint` with the reason.
Second check, no C needed: STACKED cfg attributes whose conjunction is empty — an item
compiled out on every arch, usually by an orphaned cfg left when its twin was deleted.

A C twin with no arch conditional at all is not evidence either way (the Rust cfg is
Rust-only: a bring-up shortcut, a diagnostic, an ABI twin) — counted in `--stats`, never
flagged. Those still need a human; this tool shrinks that pile, it does not empty it.

Usage:
    python3 tools/scan_cfg_vs_c.py            # report
    python3 tools/scan_cfg_vs_c.py --lint     # terse (CLI baseline gate)
    python3 tools/scan_cfg_vs_c.py --stats    # how the cfg population splits

Suppress a reviewed site with `// nolint: cfg-vs-c` on the cfg line. Exit 1 if any finding.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
from tools.c_twin import (
    CIndex,
    Func,
    blank_rust,
    crate_of,
    prefer,
    rust_functions,
    same_module,
)
from tools.source_walk import source_files

REPO = Path(__file__).resolve().parent.parent
RUST_ROOT = REPO / "sdk" / "lib" / "rust"
U = frozenset({"x86", "x86_64", "aarch64"})
X86ONLY = frozenset({"x86", "x86_64"})
_C_ARM64_AWARE = re.compile(
    r"_M_ARM64|_ARM64_|__aarch64__|_WIN64|_M_ARM\b|_ARM_\b|__arm__|_MI_PAGING_LEVELS"
)
NOLINT = "nolint: cfg-vs-c"

# --- Rust predicates --------------------------------------------------------------------

_RS_ARCH = {"x86": {"x86"}, "x86_64": {"x86_64"}, "aarch64": {"aarch64"}, "arm": set()}


def _split_args(s: str) -> list[str]:
    out, depth, cur = [], 0, ""
    for c in s:
        if c == "(":
            depth += 1
        elif c == ")":
            depth -= 1
        if c == "," and depth == 0:
            out.append(cur.strip())
            cur = ""
        else:
            cur += c
    if cur.strip():
        out.append(cur.strip())
    return out


def rust_set(pred: str) -> frozenset[str] | None:
    """Arch subset a cfg predicate selects, or None if it depends on anything else."""
    p = pred.strip()
    m = re.fullmatch(r'target_arch\s*=\s*"(\w+)"', p)
    if m:
        a = _RS_ARCH.get(m.group(1))
        return frozenset(a) if a is not None else None
    m = re.fullmatch(r'target_pointer_width\s*=\s*"(\d+)"', p)
    if m:
        return frozenset({"x86_64", "aarch64"} if m.group(1) == "64" else {"x86"})
    m = re.fullmatch(r"(not|any|all)\s*\((.*)\)", p, re.S)
    if m:
        parts = [rust_set(a) for a in _split_args(m.group(2))]
        if any(x is None for x in parts):
            return None
        sets = [x for x in parts if x is not None]
        if m.group(1) == "not":
            return U - sets[0] if len(sets) == 1 else None
        if m.group(1) == "any":
            return frozenset().union(*sets)
        return frozenset(U).intersection(*sets)
    return None


# --- C conditionals ---------------------------------------------------------------------

_C_ARCH = {
    **dict.fromkeys(("_M_IX86", "_X86_", "__i386__", "i386", "_M_I386"), frozenset({"x86"})),
    **dict.fromkeys(
        ("_M_AMD64", "_AMD64_", "__x86_64__", "_M_X64", "__amd64__"), frozenset({"x86_64"})
    ),
    **dict.fromkeys(("_M_ARM64", "_ARM64_", "__aarch64__"), frozenset({"aarch64"})),
    **dict.fromkeys(("_M_ARM", "_ARM_", "__arm__"), frozenset()),
    "_WIN64": {"x86_64", "aarch64"},
    "_WIN32": set(U),
}
# Per-arch NUMERIC macros a condition may compare: `#if (_MI_PAGING_LEVELS >= 4)` is an arch
# test in disguise (internal/{i386,amd64,arm64}/mm.h; i386 is 3 only under _X86PAE_, which
# no build of ours sets).
_C_ARCH_VALUE = {"_MI_PAGING_LEVELS": {"x86": 2, "x86_64": 4, "aarch64": 4}}
_C_TOK = re.compile(r"\s*(defined|\|\||&&|==|!=|>=|<=|>|<|!|\(|\)|[A-Za-z_]\w*|\d+)")
_CMP = {
    "==": lambda a, b: a == b,
    "!=": lambda a, b: a != b,
    ">=": lambda a, b: a >= b,
    "<=": lambda a, b: a <= b,
    ">": lambda a, b: a > b,
    "<": lambda a, b: a < b,
}


def c_set(expr: str) -> frozenset[str] | None:
    """Arch subset a preprocessor expression selects, or None if it is not pure-arch."""
    toks = _C_TOK.findall(expr.strip())
    if "".join(t.strip() for t in toks) != re.sub(r"\s+", "", expr.strip()):
        return None
    pos = 0

    def peek() -> str | None:
        return toks[pos].strip() if pos < len(toks) else None

    def take() -> str:
        nonlocal pos
        pos += 1
        return toks[pos - 1].strip()

    def primary() -> frozenset[str] | None:
        t = take()
        if t == "!":
            v = primary()
            return None if v is None else U - v
        if t == "(":
            v = orx()
            if take() != ")":
                raise ValueError
            return v
        if t == "defined":
            paren = peek() == "("
            if paren:
                take()
            name = take()
            if paren and take() != ")":
                raise ValueError
            a = _C_ARCH.get(name)
            return frozenset(a) if a is not None else None
        if t in _C_ARCH_VALUE and peek() in _CMP:
            op = take()
            n = take()
            if not n.isdigit():
                raise ValueError
            return frozenset(a for a, v in _C_ARCH_VALUE[t].items() if _CMP[op](v, int(n)))
        a = _C_ARCH.get(t)
        return frozenset(a) if a is not None else None

    def andx() -> frozenset[str] | None:
        v = primary()
        while peek() == "&&":
            take()
            w = primary()
            v = None if v is None or w is None else v & w
        return v

    def orx() -> frozenset[str] | None:
        v = andx()
        while peek() == "||":
            take()
            w = andx()
            v = None if v is None or w is None else v | w
        return v

    try:
        v = orx()
        return v if pos == len(toks) else None
    except (ValueError, IndexError):
        return None


_PP = re.compile(r"^[ \t]*#[ \t]*(if|ifdef|ifndef|elif|else|endif)\b(.*)$", re.M)


def c_branch_sets(body: str) -> list[tuple[frozenset[str], str]]:
    """Every pure-arch branch of every conditional in `body`, with its source text.
    `#else` is the complement of its chain's earlier branches, when all of them are known."""
    out: list[tuple[frozenset[str], str]] = []
    stack: list[list[frozenset[str] | None]] = []
    for m in _PP.finditer(body):
        kind, rest = m.group(1), m.group(2).split("//")[0].split("/*")[0].strip()
        if kind in ("if", "ifdef", "ifndef", "elif"):
            if kind == "ifdef":
                s = c_set(f"defined({rest})")
            elif kind == "ifndef":
                s = c_set(f"!defined({rest})")
            else:
                s = c_set(rest)
            if kind == "elif":
                if not stack:
                    continue
                stack[-1].append(s)
            else:
                stack.append([s])
            if s is not None:
                out.append((s, f"#{kind} {rest}"))
        elif kind == "else" and stack:
            prior = stack[-1]
            if all(p is not None for p in prior):
                s = U - frozenset().union(*[p for p in prior if p is not None])
                out.append((s, "#else"))
            prior.append(None)
        elif kind == "endif" and stack:
            stack.pop()
    return out


def atoms(sets: list[frozenset[str]], universe: frozenset[str] = U) -> list[frozenset[str]]:
    """The coarsest partition of `universe` that every set in `sets` is a union of."""
    parts = [universe]
    for s in sets:
        nxt = []
        for p in parts:
            for piece in (p & s, p - s):
                if piece:
                    nxt.append(piece)
        parts = nxt
    return parts


def splits(s: frozenset[str], parts: list[frozenset[str]]) -> frozenset[str] | None:
    """The atom `s` cuts through, if any."""
    for p in parts:
        if (p & s) and (p - s):
            return p
    return None


# --- Rust sites -------------------------------------------------------------------------

_CFG = re.compile(r"#\[cfg\(")
_FN_DEF = re.compile(r"\bfn\s+([A-Za-z_]\w*)\s*(?:<[^>{;]*>)?\s*\([^;{]*\)[^;{]*\{")
_EXEMPT_NEXT = re.compile(
    r"^\s*(?:#\[[^\]]*\]\s*)*(?:pub(?:\([^)]*\))?\s+)?(?:unsafe\s+)?"
    r"(?:extern\b|use\b|type\b|mod\b|global_asm!|(?:core::arch::)?asm!|"
    r"\{\s*(?:core::arch::)?asm!)"
)


def rust_cfgs(body: str) -> list[tuple[int, str, str]]:
    """(offset, predicate, attached text) for each `#[cfg(...)]` in a raw fn body."""
    out = []
    for m in _CFG.finditer(body):
        depth, i = 1, m.end()
        while i < len(body) and depth:
            if body[i] == "(":
                depth += 1
            elif body[i] == ")":
                depth -= 1
            i += 1
        pred = body[m.end() : i - 1]
        close = body.find("]", i - 1)
        out.append((m.start(), pred, body[close + 1 : close + 200]))
    return out


def _rel(p: Path) -> Path:
    try:
        return p.relative_to(REPO)
    except ValueError:
        return p


_CFG_LINE = re.compile(r"^\s*#\[cfg\((.*)\)\]\s*(?://.*)?$")


def _os_or_arch(pred: str) -> frozenset[str] | None:
    # Every target this workspace builds is `*-windows-*` (i686/x86_64-pc-windows-gnu,
    # aarch64-pc-windows-gnullvm), so `target_os = "windows"` is always true.
    p = pred.strip()
    if re.fullmatch(r'target_os\s*=\s*"windows"', p):
        return U
    if re.fullmatch(r'not\(\s*target_os\s*=\s*"windows"\s*\)', p):
        return frozenset()
    return rust_set(p)


def stacked_dead(f: Path, raw: str) -> list[tuple[Path, int, list[str]]]:
    """Runs of consecutive `#[cfg]` lines (blank lines allowed between) on one item whose
    conjunction selects NO arch. Stacked attributes AND together, so an orphaned cfg left
    behind by a deleted twin silently compiles its neighbour out everywhere: win32k's
    TEXT_NextLineW lost its text measurement on all three arches to four stacked cfgs
    (32-bit AND 64-bit AND windows AND not-windows)."""
    out = []
    lines = raw.split("\n")
    i = 0
    while i < len(lines):
        run: list[tuple[int, str]] = []
        j = i
        while j < len(lines) and (_CFG_LINE.match(lines[j]) or (run and not lines[j].strip())):
            m = _CFG_LINE.match(lines[j])
            if m:
                run.append((j + 1, m.group(1)))
            j += 1
        if len(run) >= 2 and NOLINT not in lines[run[0][0] - 1]:
            sets = [_os_or_arch(p) for _, p in run]
            known = [x for x in sets if x is not None]
            if len(known) == len(sets) and not frozenset(U).intersection(*known):
                out.append((_rel(f), run[0][0], [" ".join(p.split()) for _, p in run]))
        i = max(j, i + 1)
    return out


def _wl(f: Path, ln: int, cat: str, name: str, pred: str, twin: Func | None, nxt: str) -> str:
    code = " ".join(nxt.split())[:120]
    c = f"{_rel(twin.path)}:{twin.line}" if twin else "-"
    return f"{_rel(f)}:{ln}\t{cat}\t{name}\tcfg({' '.join(pred.split())})\t{c}\t{code}"


def main() -> int:
    args = [a for a in sys.argv[1:] if not a.startswith("-")]
    lint = "--lint" in sys.argv
    stats = "--stats" in sys.argv
    # --worklist FILE: the sites this tool CANNOT decide, one per line, for human/agent
    # review: `path:line<TAB>category<TAB>fn<TAB>cfg(pred)<TAB>C twin or -<TAB>attached code`.
    wl_path = next(
        (sys.argv[i + 1] for i, a in enumerate(sys.argv[:-1]) if a == "--worklist"), None
    )
    if wl_path in args:
        args.remove(wl_path)
    worklist: list[str] = []
    roots = [Path(a) for a in args] if args else [RUST_ROOT]
    sites: list[tuple[Path, Func, int, str, frozenset[str], str]] = []
    n = {"cfg": 0, "unknown": 0, "exempt": 0, "file_twin": 0}
    files = [f for root in roots for f in ([root] if root.is_file() else source_files(root))]
    # A name defined more than once in a crate is a per-arch FILE twin (i386/page.rs and
    # arm64/page.rs each have MmGetPageProtect): each copy compiles on only some arches, so
    # its in-body cfgs cannot be judged against the C's whole-arch partition.
    defs: dict[tuple[str, str], int] = {}
    parsed = []
    for f in files:
        raw = f.read_text(errors="replace")
        # A cheap count (a full parse of every bindgen file costs minutes): `fn name(` whose
        # signature reaches a `{` before any `;` is a definition, not an extern decl.
        for m in _FN_DEF.finditer(raw):
            key = (crate_of(f), m.group(1))
            defs[key] = defs.get(key, 0) + 1
        if "#[cfg(" in raw and "target_" in raw:
            parsed.append((f, raw))
    for f, raw in parsed:
        lines = raw.split("\n")
        for fn in rust_functions(blank_rust(raw), f):
            body = raw[fn.body_start : fn.body_start + len(fn.body)]
            for off, pred, nxt in rust_cfgs(body):
                s = rust_set(pred)
                if "target_arch" not in pred and "target_pointer_width" not in pred:
                    continue
                n["cfg"] += 1
                if s is None or s in (U, frozenset()):
                    n["unknown"] += 1
                    ln = raw.count("\n", 0, fn.body_start + off) + 1
                    if s is None and NOLINT not in lines[ln - 1]:
                        worklist.append(_wl(f, ln, "mixed-predicate", fn.name, pred, None, nxt))
                    continue
                # A statement whose arch twin differs only in bindgen's anonymous-member
                # numbering (`Tcb.__bindgen_anon_7` on i386, `_5` elsewhere) is LAYOUT.
                if _EXEMPT_NEXT.match(nxt) or "__bindgen_anon_" in nxt.split(";")[0]:
                    n["exempt"] += 1
                    continue
                if defs.get((crate_of(f), fn.name), 0) > 1:
                    n["file_twin"] += 1
                    continue
                ln = raw.count("\n", 0, fn.body_start + off) + 1
                if NOLINT in lines[ln - 1]:
                    continue
                sites.append((f, fn, ln, pred, s, nxt))
    index = CIndex()
    index.load({fn.name for _, fn, _, _, _, _ in sites})
    n.update(no_twin=0, c_no_arch=0, agrees=0, flagged=0)
    findings = []
    for f, fn, ln, pred, s, nxt in sites:
        twins = [t for t in prefer(f, index.get(fn.name)) if same_module(f, t)]
        if not twins:
            n["no_twin"] += 1
            worklist.append(_wl(f, ln, "no-c-twin", fn.name, pred, None, nxt))
            continue
        c = twins[0].path.read_text(errors="replace")
        cbody = c[twins[0].body_start : twins[0].body_start + len(twins[0].body)]
        branches = c_branch_sets(cbody)
        if not branches:
            n["c_no_arch"] += 1
            worklist.append(_wl(f, ln, "c-has-no-arch-if", fn.name, pred, twins[0], nxt))
            continue
        # A C that never names an arm64 macro never decided where arm64 goes: its `#else`
        # lumps aarch64 with i386 because upstream's arm64 port predates the code, and the
        # translation putting aarch64 with amd64 (4-level paging, 64-bit) is exceeding the
        # C, not drifting from it. Judge only the x86/x86_64 line there.
        universe = U if any(_C_ARM64_AWARE.search(t) for _, t in branches) else X86ONLY
        cut = splits(s & universe, atoms([b & universe for b, _ in branches], universe))
        if cut is None:
            n["agrees"] += 1
            continue
        n["flagged"] += 1
        evidence = next((t for b, t in branches if b == cut or (cut <= b)), branches[0][1])
        findings.append((_rel(f), ln, fn.name, pred, sorted(s), sorted(cut), evidence))
    dead = [d for f, raw in parsed for d in stacked_dead(f, raw)]
    for rel, ln, preds in dead:
        if lint:
            print(f"{rel}:{ln}: dead-cfg")
        else:
            print(f"{rel}:{ln}: dead-cfg: stacked cfgs are never all true: {' & '.join(preds)}")
    for rel, ln, name, pred, s, cut, ev in findings:
        if lint:
            print(f"{rel}:{ln}: cfg-vs-c {name}")
        else:
            print(
                f"{rel}:{ln}: cfg-vs-c: {name}: `cfg({' '.join(pred.split())})` = {s} "
                f"splits {cut}, which the C keeps together (`{ev}`)"
            )
    if wl_path:
        Path(wl_path).write_text("".join(w + "\n" for w in sorted(worklist)))
    if stats:
        print(
            f"\nin-body arch cfgs {n['cfg']}: {n['unknown']} not pure-arch, {n['exempt']} "
            f"asm/use/extern, {n['file_twin']} in per-arch file twins; of the rest "
            f"{n['no_twin']} have no C twin, {n['c_no_arch']} "
            f"a C twin with no arch #if (Rust-only cfg), {n['agrees']} agree with the C, "
            f"{n['flagged']} FLAGGED"
        )
    elif not lint:
        print(f"\n{len(findings)} site(s)")
        if findings:
            print("fix: draw the arch line where the C does — `_WIN64` is")
            print('     `target_pointer_width = "64"`; mark a reviewed divergence')
            print("     `// nolint: cfg-vs-c` with its reason.")
    return 1 if findings or dead else 0


if __name__ == "__main__":
    sys.exit(main())
