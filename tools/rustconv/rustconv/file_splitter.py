"""Win32k pretranslation — giant-file split *analyzer* (analyze-only).

rustconv's giant-file failure is a *batch* problem (silent stubbing);
`translate` mode already chunks function-by-function. So physically
splitting a giant C file is NOT needed for translation *fidelity* — it
is worth it where *clean* because the **output** would otherwise be one
multi-thousand-LOC `.rs` module (bad for review, diffs, cgu-firewall
granularity, maintenance forever).

The only hazard is **file-scope `static` coupling**: cutting between a
`static` (data or function) and a referencing function across a TU
boundary either fails to link (static fn / data → undefined symbol) or,
worse, *duplicates mutable state* (a `static` table copied into both
split files = two instances = silent correctness bug). So this is
Pass-0-before-Pass-1 discipline applied to file-splitting: **analyze the
static-reference graph, find cuts that cross zero statics, classify —
do not chop blindly.** This module only analyzes + classifies; the
emitter (and any static-promotion) is a separate, smoke-gated step.

Classification per file:
  CLEAN_SPLIT     — a function partition exists with ≥2 groups and
                    *zero* file-scope statics referenced across any cut.
                    Free win; physically split.
  NEEDS_PROMOTION — no zero-crossing partition, but every blocking
                    static is promotable (a `static` fn, or `const`
                    data → de-`static` + `extern` in a `<file>_priv.h`).
                    Worth it only for very large files.
  LEAVE_WHOLE     — a mutable `static` is referenced file-wide, or no
                    usable cut. Don't split; let the translation phase
                    use `translate` (chunked) mode and accept a larger
                    `.rs`.

CLI: `python -m rustconv split-plan [path] [--min-loc N]
      [--target-loc N] [--json]`.
"""

from __future__ import annotations

import re
from dataclasses import dataclass, field
from pathlib import Path

from .callout_annotate import _line_of, _mask_literals

# Translation scope (win32k.sys → Rust). freetype.c is excluded by the
# caller note, not here — it's link-against 3rd-party FFI, never
# translated (audit), so it should not even be a split candidate.
SCOPE_DIRS = ("gdi/ntgdi", "gdi/eng", "gdi/dib", "user/ntuser", "reactx/ntddraw", "user/rtl")
NEVER_SPLIT = ("freetype.c",)  # 3rd-party FFI, not translated


@dataclass
class Func:
    name: str
    start: int
    end: int
    loc: int


@dataclass
class Static:
    name: str
    kind: str  # "fn" | "const" | "mutable"
    line: int
    refs: list[int] = field(default_factory=list)  # function indices


@dataclass
class Plan:
    path: Path
    loc: int
    nfuncs: int
    classification: str  # CLEAN_SPLIT | NEEDS_PROMOTION | LEAVE_WHOLE
    groups: list[tuple[int, int, int]]  # (first_fn, last_fn, group_loc)
    blocking: list[str]  # static names that block all clean cuts
    note: str = ""


def _enumerate_functions(text: str, masked: str) -> list[Func]:
    """Ordered top-level function definitions, via the depth-0
    `)`-then-`{` body-open heuristic shared with callout_annotate."""
    funcs: list[Func] = []
    depth = 0
    i = 0
    n = len(masked)
    while i < n:
        c = masked[i]
        if c == "{":
            if depth == 0:
                j = i - 1
                while j >= 0 and masked[j] in " \t\r\n":
                    j -= 1
                if j >= 0 and masked[j] == ")":
                    # name = ident before the signature's `(`
                    pd = 0
                    k = j
                    while k >= 0:
                        if masked[k] == ")":
                            pd += 1
                        elif masked[k] == "(":
                            pd -= 1
                            if pd == 0:
                                break
                        k -= 1
                    nm = re.search(r"([A-Za-z_]\w*)\s*$", masked[:k])
                    # start = just after previous depth-0 ; } or file start
                    s = k
                    bd = 0
                    while s > 0:
                        ch = masked[s - 1]
                        if bd == 0 and ch in ";}":
                            break
                        if ch == ")":
                            bd += 1
                        elif ch == "(":
                            bd -= 1
                        s -= 1
                    while s < k and masked[s] in " \t\r\n":
                        s += 1
                    # end = matching `}`
                    d = 0
                    e = i
                    while e < n:
                        if masked[e] == "{":
                            d += 1
                        elif masked[e] == "}":
                            d -= 1
                            if d == 0:
                                e += 1
                                break
                        e += 1
                    if nm:
                        funcs.append(Func(nm.group(1), s, e, text.count("\n", s, e) + 1))
                    # whole balanced body consumed → depth net unchanged
                    i = e
                    continue
            depth += 1
        elif c == "}":
            depth -= 1
        i += 1
    return funcs


_RE_STATIC = re.compile(r"^[ \t]*static\b[^\n;{]*", re.MULTILINE)


def _file_scope_statics(text: str, masked: str, funcs: list[Func]) -> list[Static]:
    """`static` declarations at file scope (outside any function span).
    Classified fn / const / mutable — the fix path differs per kind."""
    spans = [(f.start, f.end) for f in funcs]
    out: list[Static] = []
    for m in _RE_STATIC.finditer(masked):
        pos = m.start()
        if any(a <= pos < b for a, b in spans):
            continue  # a `static` local, not file scope
        head = masked[pos : pos + 200]
        # identifier: `static [const] <type...> NAME ( | = | [ | ;`
        idm = re.search(r"\bstatic\b[\w\s\*]*?\b([A-Za-z_]\w*)\s*[\(\[=;]", head)
        if not idm:
            continue
        name = idm.group(1)
        after = head[idm.end() - 1 :]
        if after.lstrip().startswith("("):
            kind = "fn"
        elif re.search(r"\bconst\b", head[: idm.start(1)]):
            kind = "const"
        else:
            kind = "mutable"
        out.append(Static(name, kind, _line_of(text, pos)))
    # ref graph: a func references S if \bS\b appears in its masked body
    for s in out:
        rx = re.compile(rf"\b{re.escape(s.name)}\b")
        s.refs = [i for i, f in enumerate(funcs) if rx.search(masked[f.start : f.end])]
    return out


def _clean_cuts(funcs: list[Func], statics: list[Static]) -> set[int]:
    """Cut indices i (between func i and i+1) that cross *zero* statics.
    A static S blocks cut i iff some referencing func ≤ i and some > i."""
    blocked: set[int] = set()
    for s in statics:
        if len(s.refs) < 2:
            continue
        lo, hi = min(s.refs), max(s.refs)
        for i in range(lo, hi):  # cuts lo..hi-1 split S's refs
            blocked.add(i)
    return set(range(len(funcs) - 1)) - blocked


def analyze_file(path: Path, target_loc: int = 900) -> Plan:
    text = path.read_text(errors="replace")
    masked = _mask_literals(text)
    funcs = _enumerate_functions(text, masked)
    loc = text.count("\n") + 1
    if len(funcs) < 2:
        return Plan(
            path, loc, len(funcs), "LEAVE_WHOLE", [], [], "fewer than 2 top-level functions"
        )

    statics = _file_scope_statics(text, masked, funcs)
    clean = _clean_cuts(funcs, statics)
    # A split is only *useful* if every resulting group is itself
    # sub-giant — otherwise rustconv still chokes on the big chunk.
    ceiling = max(int(target_loc * 1.5), 1400)

    # Greedy: at/after target_loc, close at the next clean cut. Fold a
    # tiny trailing remainder back into the previous group.
    groups: list[tuple[int, int, int]] = []
    gstart = acc = 0
    for i, f in enumerate(funcs):
        acc += f.loc
        if i == len(funcs) - 1:
            if groups and acc < target_loc // 3 and groups[-1][2] + acc <= ceiling:
                a, _, loc = groups.pop()
                groups.append((a, i, loc + acc))
            else:
                groups.append((gstart, i, acc))
            break
        if acc >= target_loc and i in clean:
            groups.append((gstart, i, acc))
            gstart, acc = i + 1, 0

    oversized = [g for g in groups if g[2] > ceiling]
    if len(groups) >= 2 and not oversized:
        return Plan(
            path,
            loc,
            len(funcs),
            "CLEAN_SPLIT",
            groups,
            [],
            f"{len(groups)} groups, each ≤{ceiling}L, zero cross-group statics",
        )

    # A split exists but a static forces a still-giant chunk. Identify
    # the statics whose ref-span straddles the oversized group's range
    # (those are what block a finer cut there).
    span_lo = oversized[0][0] if oversized else 0
    span_hi = oversized[-1][1] if oversized else len(funcs) - 1
    cross = [
        s for s in statics if len(s.refs) >= 2 and min(s.refs) <= span_lo < span_hi <= max(s.refs)
    ]
    mutable_wide = [s for s in cross if s.kind == "mutable"]
    promotable = [s for s in cross if s.kind in ("fn", "const")]
    if mutable_wide:
        return Plan(
            path,
            loc,
            len(funcs),
            "LEAVE_WHOLE",
            groups,
            sorted({s.name for s in mutable_wide}),
            "mutable static(s) span the file — duplicating splits state; translate-chunk instead",
        )
    if promotable:
        return Plan(
            path,
            loc,
            len(funcs),
            "NEEDS_PROMOTION",
            groups,
            sorted({s.name for s in promotable}),
            f"split leaves a >{ceiling}L chunk; blocking statics "
            f"are de-static-able (fn/const) → <file>_priv.h",
        )
    if len(groups) >= 2 and not oversized:
        return Plan(path, loc, len(funcs), "CLEAN_SPLIT", groups, [], "")
    return Plan(
        path,
        loc,
        len(funcs),
        "LEAVE_WHOLE",
        groups,
        [],
        "no clean cut yields all-sub-giant groups; translate-chunk",
    )


@dataclass
class Emit:
    path: Path
    classification: str
    fragments: list[tuple[Path, str]]  # (fragment path, content)
    new_base: str  # rewritten <base>.c content
    note: str = ""


def plan_emit(path: Path, target_loc: int = 900) -> Emit:
    """Build the #include-back split for a CLEAN_SPLIT file. The base .c
    keeps the prologue + group-0 functions and `#include`s the remaining
    groups as fragment files. The C preprocessor reassembles the IDENTICAL
    translation unit (provably semantics-preserving — same tokens, same
    order, plus one inserted comment) so there are NO CMake changes and NO
    linkage/static hazard. rustconv sees small files; the build sees one TU.
    """
    pl = analyze_file(path, target_loc)
    if pl.classification != "CLEAN_SPLIT" or len(pl.groups) < 2:
        return Emit(path, pl.classification, [], "", "not CLEAN_SPLIT — nothing to emit")
    text = path.read_text(errors="replace")
    masked = _mask_literals(text)
    funcs = _enumerate_functions(text, masked)

    # Cut offsets at each non-final group's last-function end.
    cuts = [funcs[b].end for (_a, b, _loc) in pl.groups[:-1]]
    bounds = [*cuts, len(text)]

    stem = path.stem
    suffix = path.suffix
    base_keep = text[: cuts[0]]
    fragments: list[tuple[Path, str]] = []
    includes: list[str] = []
    prev = cuts[0]
    for n, end in enumerate(bounds[1:], start=2):
        slice_ = text[prev:end]
        prev = end
        fname = f"{stem}.split{n}{suffix}"
        hdr = (
            f"/* {fname} — #include fragment of {stem}{suffix}, carved "
            f"for pretranslation so rustconv gets a small file. NOT a "
            f"standalone TU; NOT in CMake; do not compile directly. "
            f"The preprocessor reassembles the original {stem}{suffix} "
            f"TU verbatim. See "
            f"docs/engineering/win32k-pretranslation-audit.md */\n"
        )
        fragments.append((path.with_name(fname), hdr + slice_))
        includes.append(f'#include "{fname}"')

    banner = (
        f"\n/* --- pretranslation split: the remaining function "
        f"groups live in {stem}.split2..{len(pl.groups)}{suffix} and "
        f"are #included below so this stays ONE translation unit "
        f"(zero CMake/linkage change). Reassembled verbatim by cpp. "
        f"--- */\n"
    )
    new_base = base_keep + banner + "\n".join(includes) + "\n"

    # Oracle (by construction, asserted): stripping each fragment's added
    # header line and concatenating after base_keep == the original file.
    recon = base_keep + "".join(c.split("\n", 1)[1] for _p, c in fragments)
    if recon != text:
        return Emit(
            path,
            pl.classification,
            [],
            "",
            "RECONSTRUCTION MISMATCH — refusing to emit (byte-exact reassembly check failed)",
        )
    return Emit(
        path,
        "CLEAN_SPLIT",
        fragments,
        new_base,
        f"{len(pl.groups)} parts, reconstruction byte-exact",
    )


def apply_emit(em: Emit) -> None:
    for fp, content in em.fragments:
        fp.write_text(content)
    em.path.write_text(em.new_base)


def _scope(p: Path) -> bool:
    s = p.as_posix()
    return any(f"/win32ss/{d}/" in s for d in SCOPE_DIRS) and p.name not in NEVER_SPLIT


def analyze_scope(win32ss_root: Path, min_loc: int, target_loc: int) -> list[Plan]:
    plans: list[Plan] = []
    for c in sorted(win32ss_root.rglob("*.c")):
        if not _scope(c):
            continue
        if (c.read_text(errors="replace").count("\n") + 1) < min_loc:
            continue
        plans.append(analyze_file(c, target_loc))
    return sorted(plans, key=lambda p: -p.loc)


def print_report(plans: list[Plan]) -> None:
    print(f"\n{'=' * 72}")
    print(f"Win32k giant-file split plan — {len(plans)} files ≥ threshold")
    print(f"{'=' * 72}")
    print(f"{'LOC':>5} {'fns':>4}  {'class':<15} file")
    for p in plans:
        rel = p.path.as_posix().split("/win32ss/")[-1]
        print(f"{p.loc:>5} {p.nfuncs:>4}  {p.classification:<15} {rel}")
        if p.classification == "CLEAN_SPLIT":
            parts = ", ".join(f"[fn{a}-{b} ~{loc}L]" for a, b, loc in p.groups)
            print(f"      → {len(p.groups)} files: {parts}")
        elif p.blocking:
            print(f"      blocking statics: {', '.join(p.blocking[:10])}  ({p.note})")
        elif p.note:
            print(f"      {p.note}")
    n_clean = sum(p.classification == "CLEAN_SPLIT" for p in plans)
    n_prom = sum(p.classification == "NEEDS_PROMOTION" for p in plans)
    n_whole = sum(p.classification == "LEAVE_WHOLE" for p in plans)
    saved = sum(p.loc for p in plans if p.classification == "CLEAN_SPLIT")
    print(
        f"\n  CLEAN_SPLIT {n_clean} ({saved} LOC of giants → small "
        f"modules, free) · NEEDS_PROMOTION {n_prom} · LEAVE_WHOLE "
        f"{n_whole} (translate-chunk)"
    )
