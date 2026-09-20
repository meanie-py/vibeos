#!/usr/bin/env python3
"""Find near-identical sibling functions in sdk/lib/rust that diverge on one line.

No oracle needed: this does not compare against upstream ReactOS. It looks for pairs
of Rust functions in the SAME FILE whose names are conventionally paired (Shared/
Exclusive, Acquire/Release, Nt<X>/Zw<X>, <X>/<X>Ex), or that are textually identical
twins gated on different `#[cfg(target_arch = ...)]` / `#[cfg(target_pointer_width =
...)]` attributes, or that are two loop constructs adjacent inside one function.
A/W (ANSI/wide) and Get/Set pairs were tried and dropped -- see the report for why.
It normalises each pair's bodies (so the pair's own names and the token that
legitimately differs between them collapse to a placeholder) and reports the
residual diff. A residual diff on an otherwise-identical body is exactly the shape
of bug this tool exists to find: a guard, a variable, or a call argument that is
present on one side and not the other for no principled reason.

This is a heuristic source scanner, not a Rust parser. It is deliberately tuned to
produce few, high-confidence candidates rather than exhaustive coverage -- see
docs/engineering/active-lanes/sibling-drift-findings.md for the false-positive
classes and the hand-verified results.

Usage:
    scan_sibling_drift.py [--root DIR] [--top N] [--min-lines N] [--json] [-v]

Read-only: never writes, never touches git, never builds anything.
"""

from __future__ import annotations

import argparse
import difflib
import re
import sys
from dataclasses import dataclass
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from source_walk import source_files

# ---------------------------------------------------------------------------
# Function extraction (regex + brace counting -- good enough for our own Rust,
# which is rustfmt'd and has no raw strings containing unbalanced braces in
# function bodies we care about).
# ---------------------------------------------------------------------------

FN_SIG_RE = re.compile(
    r"^(?P<indent>[ \t]*)"
    r"(?P<attrs>(?:#\[[^\]]*\]\s*\n[ \t]*)*)"
    r"(?:pub(?:\([^)]*\))?\s+)?"
    r"(?:unsafe\s+)?(?:extern\s+\"[^\"]*\"\s+)?(?:unsafe\s+)?"
    r"fn\s+(?P<name>[A-Za-z_][A-Za-z0-9_]*)\s*[<(]",
    re.MULTILINE,
)

CFG_ATTR_RE = re.compile(r"#\[cfg\(([^\]]*)\)\]")

ORPHAN_BANNER_RE = re.compile(r"^\s*//!\s*ORPHAN-BY-DESIGN:", re.MULTILINE)

# Generated / mechanically-produced files are not hand-written sibling pairs: a
# bindgen dump or a per-arch `types_*.rs` table has thousands of trivial one-line
# accessor fns whose name-pairing is coincidental, and naive O(n^2) pairing over
# 7000 functions is also just too slow to be worth it. Skip them outright.
GENERATED_NAME_RE = re.compile(r"^(bindgen_|hal_bindgen_|types_)")

# Safety valve: if a file still has an implausible number of functions after the
# above filter, name-pair generation is O(n^2) and not worth paying for -- skip
# name-pairing (arch-gate and adjacent-loop pairing, which are not O(n^2) over
# ALL pairs in practice, still run).
MAX_FUNCS_FOR_NAME_PAIRING = 400


@dataclass
class Fn:
    name: str
    file: Path
    start_line: int  # 1-based, line of `fn`
    end_line: int  # 1-based, line of closing brace
    body: str  # text strictly between the outermost { and }
    cfg: str | None  # raw contents of the nearest #[cfg(...)] right above it, if any
    preceding_attrs: str


def strip_comments(src: str) -> str:
    """Remove // and /* */ comments. Leaves strings alone on a best-effort basis
    (good enough here: this codebase doesn't embed `//` inside string literals in
    the functions we scan, and a false non-removal only weakens normalisation,
    it never mis-brace-matches since we don't touch braces here)."""
    out = []
    i = 0
    n = len(src)
    while i < n:
        if src[i : i + 2] == "//":
            j = src.find("\n", i)
            i = n if j == -1 else j
            continue
        if src[i : i + 2] == "/*":
            j = src.find("*/", i + 2)
            i = n if j == -1 else j + 2
            continue
        out.append(src[i])
        i += 1
    return "".join(out)


def find_matching_brace(src: str, open_idx: int) -> int | None:
    """src[open_idx] must be '{'. Returns index of the matching '}' or None."""
    depth = 0
    i = open_idx
    n = len(src)
    in_str = False
    in_char = False
    while i < n:
        c = src[i]
        if in_str:
            if c == "\\":
                i += 2
                continue
            if c == '"':
                in_str = False
        elif in_char:
            if c == "\\":
                i += 2
                continue
            if c == "'":
                in_char = False
        elif c == '"':
            in_str = True
        elif c == "'" and i + 1 < n and src[i + 1] != "\\" and src.find("'", i + 1, i + 3) != -1:
            in_char = True
        elif c == "{":
            depth += 1
        elif c == "}":
            depth -= 1
            if depth == 0:
                return i
        i += 1
    return None


def extract_functions(path: Path, text: str) -> list[Fn]:
    """Extract top-level-ish fn bodies. Nested fns inside a matched body are not
    separately reported (their divergence is covered as part of the parent diff)."""
    funcs: list[Fn] = []
    stripped = strip_comments(text)
    for m in FN_SIG_RE.finditer(stripped):
        name = m.group("name")
        # Find the opening brace of the body (skip past return-type / where-clause).
        sig_end = m.end()
        brace_idx = stripped.find("{", sig_end)
        if brace_idx == -1:
            continue
        # Guard against matching into a following item if there's a `;` (trait decl
        # with no body) before the brace on the same "statement".
        semi_idx = stripped.find(";", sig_end)
        if semi_idx != -1 and semi_idx < brace_idx:
            # Could still be legit (e.g. a `where` clause with a semicolon-free
            # generic bound is rare); heuristically trust the brace anyway only if
            # it's on the very next non-trivial line.
            between = stripped[sig_end:brace_idx]
            if ";" in between:
                continue
        close_idx = find_matching_brace(stripped, brace_idx)
        if close_idx is None:
            continue
        body = stripped[brace_idx + 1 : close_idx]
        start_line = stripped.count("\n", 0, m.start()) + 1
        end_line = stripped.count("\n", 0, close_idx) + 1

        # Look at the attrs captured directly above the fn for a #[cfg(...)].
        attrs_text = m.group("attrs") or ""
        cfg_match = CFG_ATTR_RE.search(attrs_text)
        cfg = cfg_match.group(1) if cfg_match else None

        funcs.append(
            Fn(
                name=name,
                file=path,
                start_line=start_line,
                end_line=end_line,
                body=body,
                cfg=cfg,
                preceding_attrs=attrs_text,
            )
        )
    return funcs


# ---------------------------------------------------------------------------
# Name-pairing rules
# ---------------------------------------------------------------------------


def name_pair_reason(a: str, b: str) -> str | None:
    """Return a short reason string if `a` and `b` look like a conventional sibling
    pair, else None. Order-independent (caller dedups by frozenset).

    NOTE: A/W (ANSI/wide) and Get/Set pairs are deliberately NOT included here.
    Both were tried and dropped after hand-checking the scanner's own output: A/W
    siblings routinely have genuinely different bodies (narrow vs wide string
    marshalling, different buffer types) so a token-swap normalisation produces
    mostly noise -- see "false positive classes" in the write-up. Shared/Exclusive,
    Acquire/Release, Nt/Zw and X/XEx are kept because in this codebase they are
    near-body-identical by construction (a lock acquired two ways, a kernel-mode
    vs Zw wrapper, an Ex overload), which is exactly the "only one line legitimately
    differs" shape this tool targets.
    """
    if a == b:
        return None

    # Shared/Exclusive, Acquire/Release as substring swaps.
    for x, y in (("Shared", "Exclusive"), ("Acquire", "Release")):
        if a.replace(x, "\0") == b.replace(y, "\0") and x in a and y in b:
            return f"{x}/{y}"
        if a.replace(y, "\0") == b.replace(x, "\0") and y in a and x in b:
            return f"{x}/{y}"

    # Nt<X> / Zw<X>
    if a.startswith("Nt") and b.startswith("Zw") and a[2:] == b[2:]:
        return "Nt/Zw"
    if b.startswith("Nt") and a.startswith("Zw") and b[2:] == a[2:]:
        return "Nt/Zw"

    # <X> / <X>Ex
    if a + "Ex" == b or b + "Ex" == a:
        return "X/XEx"

    return None


# ---------------------------------------------------------------------------
# Normalisation: collapse the tokens that are SUPPOSED to differ between a
# sibling pair down to a common placeholder, so the diff shows only the
# unexplained residue.
# ---------------------------------------------------------------------------

TOKEN_RE = re.compile(r"[A-Za-z_][A-Za-z0-9_]*|[0-9]+|\S")

# Pairs of literal words that legitimately swap between conventional siblings.
GENERIC_SWAPS = [
    ("Shared", "Exclusive"),
    ("Acquire", "Release"),
    ("Acquired", "Released"),
    ("Get", "Set"),
    ("Nt", "Zw"),
]


def normalise_tokens(body: str, name_a: str, name_b: str) -> list[str]:
    toks = TOKEN_RE.findall(body)
    out = []
    for t in toks:
        if t in (name_a, name_b):
            out.append("<SELF>")
            continue
        replaced = t
        for x, y in GENERIC_SWAPS:
            if x in replaced and y not in replaced:
                cand = replaced.replace(x, "<SWAP>")
                # only accept if the swapped form actually appears as the sibling
                # word somewhere reasonable -- cheap heuristic, just do it.
                replaced = cand
            elif y in replaced:
                replaced = replaced.replace(y, "<SWAP>")
        out.append(replaced)
    return out


@dataclass
class Finding:
    kind: str
    file: Path
    name_a: str
    name_b: str
    line_a: int
    line_b: int
    score: int
    reasons: list[str]
    diff_snippet: str
    ratio: float


KEYWORD_WEIGHTS = {
    "continue": 5,
    "return": 4,
    "break": 3,
    "lock": 4,
    "unlock": 4,
    "Lock": 4,
    "null": 2,
    "NULL": 2,
    "Owner": 3,
    "if": 2,
}

MIN_RATIO = 0.55  # below this, bodies are "wholly different" -> excluded
MAX_RATIO = 0.999  # at/above this with zero opcodes of interest -> identical, skip


def diff_bodies(
    body_a: str, body_b: str, name_a: str, name_b: str
) -> tuple[float, list[str], int, list[str]] | None:
    toks_a = normalise_tokens(body_a, name_a, name_b)
    toks_b = normalise_tokens(body_b, name_a, name_b)
    if not toks_a or not toks_b:
        return None
    sm = difflib.SequenceMatcher(a=toks_a, b=toks_b, autojunk=False)
    ratio = sm.ratio()
    if ratio < MIN_RATIO or ratio >= MAX_RATIO:
        return None

    score = 0
    reasons: list[str] = []
    snippets: list[str] = []
    replace_single_token_hits = 0

    for tag, i1, i2, j1, j2 in sm.get_opcodes():
        if tag == "equal":
            continue
        chunk_a = toks_a[i1:i2]
        chunk_b = toks_b[j1:j2]
        text_a = " ".join(chunk_a)[:80]
        text_b = " ".join(chunk_b)[:80]
        snippets.append(f"  [{tag}] A: {text_a!r}  B: {text_b!r}")

        combined = chunk_a + chunk_b
        for kw, w in KEYWORD_WEIGHTS.items():
            if any(tok == kw for tok in combined):
                score += w
                if kw not in reasons:
                    reasons.append(f"keyword:{kw}")

        # "different variable passed to the same callee in otherwise identical
        # calls": a `replace` op where exactly one token differs and both sides
        # are identifiers (not punctuation/keywords already scored above).
        if tag == "replace" and len(chunk_a) == 1 and len(chunk_b) == 1:
            ta, tb = chunk_a[0], chunk_b[0]
            if ta.isidentifier() and tb.isidentifier() and ta != tb:
                replace_single_token_hits += 1
                score += 6
                reasons.append(f"single-token-swap:{ta}!={tb}")

        # guard present in one, absent in other: an insert/delete opcode whose
        # chunk contains "if" and "{" is very likely a whole conditional/guard.
        one_sided = chunk_a if tag == "delete" else chunk_b if tag == "insert" else None
        if one_sided and "if" in one_sided and "{" in one_sided:
            score += 8
            reasons.append("one-sided-guard")

    if score == 0:
        return None
    return ratio, snippets, score, reasons


def load_orphan_ranges(text_stripped: str) -> bool:
    return bool(ORPHAN_BANNER_RE.search(text_stripped))


def collect_file_functions(path: Path) -> list[Fn] | None:
    try:
        text = path.read_text(encoding="utf-8", errors="replace")
    except OSError:
        return None
    stripped = strip_comments(text)
    if load_orphan_ranges(stripped):
        return None
    return extract_functions(path, text)


def gen_arch_gate_pairs(funcs: list[Fn]) -> list[tuple[Fn, Fn, str]]:
    by_name: dict[str, list[Fn]] = {}
    for f in funcs:
        by_name.setdefault(f.name, []).append(f)
    out = []
    for _name, group in by_name.items():
        if len(group) < 2:
            continue
        gated = [
            f
            for f in group
            if f.cfg and ("target_arch" in f.cfg or "target_pointer_width" in f.cfg)
        ]
        if len(gated) < 2:
            continue
        for i in range(len(gated)):
            for j in range(i + 1, len(gated)):
                out.append((gated[i], gated[j], f"arch-gate:{gated[i].cfg}|{gated[j].cfg}"))
    return out


def gen_name_pairs(funcs: list[Fn]) -> list[tuple[Fn, Fn, str]]:
    out = []
    n = len(funcs)
    for i in range(n):
        for j in range(i + 1, n):
            a, b = funcs[i], funcs[j]
            if a.name == b.name:
                continue  # handled by arch-gate pairing
            reason = name_pair_reason(a.name, b.name)
            if reason:
                out.append((a, b, f"name-pair:{reason}"))
    return out


def gen_adjacent_loop_pairs(funcs: list[Fn]) -> list[tuple[Fn, Fn, str]]:
    """Within a single function, find two top-level loop constructs (for/while/loop)
    whose bodies are substantial and similar in size -- the fsrtl "shared loop
    directly above the exclusive loop" shape. We synthesise pseudo-Fn entries for
    the two loop bodies so they can flow through the same diff machinery."""
    loop_re = re.compile(r"\b(for|while|loop)\b[^{]*\{")
    out: list[tuple[Fn, Fn, str]] = []
    for f in funcs:
        body = f.body
        loops: list[tuple[int, str]] = []
        last_end = -1  # end of the last ACCEPTED top-level loop's span
        for m in loop_re.finditer(body):
            if m.start() < last_end:
                # Nested inside the loop we just accepted -- not a sibling, skip it.
                # (Nested loops routinely have deliberately different bodies, e.g.
                # IopFixupResourceListWithRequirements's outer alternative-list loop
                # vs its inner descriptor loop; pairing them as "siblings" is pure
                # noise, hand-checked and dropped.)
                continue
            brace_idx = body.rfind("{", 0, m.end())
            if brace_idx == -1 or body[brace_idx] != "{":
                brace_idx = m.end() - 1
            close_idx = find_matching_brace(body, brace_idx)
            if close_idx is None:
                continue
            inner = body[brace_idx + 1 : close_idx]
            last_end = close_idx
            if inner.count("\n") < 4:
                continue  # trivially short, excluded per spec
            loops.append((m.start(), inner))
        for i in range(len(loops)):
            for j in range(i + 1, len(loops)):
                start_a, inner_a = loops[i]
                start_b, inner_b = loops[j]
                # only adjacent-ish pairs: nothing large in between them structurally
                # -- cap how many loops apart we compare to keep this cheap and precise
                if j - i > 1:
                    continue
                line_a = f.start_line + body.count("\n", 0, start_a)
                line_b = f.start_line + body.count("\n", 0, start_b)
                fake_a = Fn(f"{f.name}#loop{i}", f.file, line_a, line_a, inner_a, None, "")
                fake_b = Fn(f"{f.name}#loop{j}", f.file, line_b, line_b, inner_b, None, "")
                out.append((fake_a, fake_b, f"adjacent-loop-in:{f.name}"))
    return out


def scan(root: Path, min_lines: int) -> list[Finding]:
    findings: list[Finding] = []
    rust_dir = root / "sdk" / "lib" / "rust"
    # source_files() prunes mutable output BEFORE scandir. Rolling our own walk is
    # what scan_unpruned_walk exists to catch, and it is the same defect that made
    # cohab_census die on a vanishing build dir earlier today.
    for path in sorted(source_files(rust_dir, "*.rs")):
        if "/target/" in str(path):
            continue
        if GENERATED_NAME_RE.match(path.name):
            continue
        funcs = collect_file_functions(path)
        if not funcs:
            continue
        candidates: list[tuple[Fn, Fn, str]] = []
        candidates += gen_arch_gate_pairs(funcs)
        if len(funcs) <= MAX_FUNCS_FOR_NAME_PAIRING:
            candidates += gen_name_pairs(funcs)
        candidates += gen_adjacent_loop_pairs(funcs)

        seen_pairs: set[tuple[str, str, int, int]] = set()
        for a, b, reason in candidates:
            key = (a.name, b.name, a.start_line, b.start_line)
            if key in seen_pairs:
                continue
            seen_pairs.add(key)

            lines_a = a.body.count("\n")
            lines_b = b.body.count("\n")
            if lines_a < min_lines or lines_b < min_lines:
                continue

            result = diff_bodies(a.body, b.body, a.name, b.name)
            if result is None:
                continue
            ratio, snippets, score, reasons = result
            if not snippets:
                continue

            # Adjacent-loop pairs iterate two DIFFERENT collections by construction,
            # so plain identifier drift (single-token-swap) is expected noise, not
            # signal -- almost every pair of parallel loops in this codebase trips
            # it. Only report one when it clears a much higher similarity bar AND
            # carries a real structural marker (a whole guard present on one side
            # and not the other), which is the actual shape of the fsrtl bug this
            # category is modelled on. Hand-verified: this drops ~600 candidates
            # to ~80 in this tree.
            if reason.startswith("adjacent-loop-in") and (
                ratio < 0.85 or "one-sided-guard" not in reasons
            ):
                continue

            findings.append(
                Finding(
                    kind=reason,
                    file=path,
                    name_a=a.name,
                    name_b=b.name,
                    line_a=a.start_line,
                    line_b=b.start_line,
                    score=score,
                    reasons=reasons,
                    diff_snippet="\n".join(snippets[:12]),
                    ratio=ratio,
                )
            )
    findings.sort(key=lambda f: (-f.score, str(f.file), f.line_a))
    return findings


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    ap.add_argument("--root", default=".", help="repo root (default: cwd)")
    ap.add_argument("--top", type=int, default=40, help="max findings to print (default: 40)")
    ap.add_argument(
        "--min-lines",
        type=int,
        default=6,
        help="minimum body length (newlines) for BOTH siblings to be considered (default: 6)",
    )
    ap.add_argument(
        "--min-score",
        type=int,
        default=12,
        help="drop candidates scoring below this (default: 12) -- keeps output small",
    )
    ap.add_argument("--json", action="store_true", help="emit JSON instead of text")
    ap.add_argument("-v", "--verbose", action="store_true", help="print full diff snippets")
    args = ap.parse_args(argv)

    root = Path(args.root).resolve()
    findings = scan(root, args.min_lines)
    findings = [f for f in findings if f.score >= args.min_score]

    top = findings[: args.top]
    if args.json:
        import json

        print(
            json.dumps(
                [
                    {
                        "kind": f.kind,
                        "file": str(f.file.relative_to(root))
                        if f.file.is_absolute()
                        else str(f.file),
                        "a": f"{f.name_a}:{f.line_a}",
                        "b": f"{f.name_b}:{f.line_b}",
                        "score": f.score,
                        "ratio": round(f.ratio, 3),
                        "reasons": f.reasons,
                    }
                    for f in top
                ],
                indent=2,
            )
        )
    else:
        print(f"# scan_sibling_drift: {len(findings)} candidate(s) total, showing top {len(top)}\n")
        for f in top:
            rel = f.file.relative_to(root) if f.file.is_absolute() else f.file
            print(f"score={f.score:3d} ratio={f.ratio:.2f} {f.kind}")
            print(f"  {rel}:{f.line_a}  {f.name_a}")
            print(f"  {rel}:{f.line_b}  {f.name_b}")
            print(f"  reasons: {', '.join(f.reasons)}")
            if args.verbose:
                print(f.diff_snippet)
            print()
    return 0


if __name__ == "__main__":
    sys.exit(main())
