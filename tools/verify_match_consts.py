#!/usr/bin/env python3
"""
verify_match_consts.py — catch the "match on an undefined constant silently
becomes a catch-all variable binding" bug class in the Rust ntoskrnl translation.

THE BUG (caused a real BUGCHECK 0x1E):
    C:    switch (x) { case FOO: ...; case BAR: ...; }
    Rust: match x { FOO => ..., BAR => ..., }
If `FOO` is NOT an in-scope `const` / `static` / imported enum variant, Rust does
NOT compare by value. It treats `FOO` as a *fresh irrefutable binding* that matches
EVERYTHING. The first such arm swallows every input; later arms become dead. rustc
emits only a `unreachable_patterns` warning (plus maybe `non_upper_case_globals`),
never an error — so it compiles green and silently does the wrong thing at runtime.

WHAT THIS TOOL FLAGS:
A `match` arm whose pattern is a single bare path-free identifier in CONST_CASE or
PascalCase (^[A-Z][A-Za-z0-9_]*$), used as a pattern (left of `=>`), that is NOT
resolvable to an in-scope const/static/enum-variant. Such an arm is a silent binding.

RESOLUTION / CONFIDENCE (tuned to minimize false positives):
  - In-file `const IDENT` / `static IDENT` / `let IDENT` / explicit `use ...::IDENT`
    / local `enum { ... IDENT ... }`  -> RESOLVED, not flagged. (This is what makes
    pnpnotify.rs's `EventCategoryDeviceInterfaceChange =>` SAFE: it has a local
    `const EventCategoryDeviceInterfaceChange` at the top of the file.)
  - File has glob imports (`use X::*;`) AND the identifier exists *somewhere* in the
    Rust workspace as a `pub const` / enum variant  -> LOW confidence (probably
    resolved through a glob; reported only with --include-low).
  - Not resolved in-file, and NOT found anywhere in the workspace as a const/variant
    -> HIGH confidence hazard (this is the real bug shape).
  - Not resolved in-file, found in workspace, but file has NO glob imports
    -> MEDIUM confidence (can't reach it without an import).

CORROBORATING SMELL:
A risky arm immediately followed by additional arms (which rustc would call
`unreachable`) raises confidence — that's the exact runtime footprint of the bug.

Patterns that are NORMAL Rust bindings and are NEVER flagged:
  Some(x), Ok/Err/None, tuple/struct/slice patterns, `n @ range`, ranges `a..=b`,
  guards `if`, `|` alternations containing literals/paths, `_`, lowercase idents,
  paths with `::`, calls `IDENT(...)`, struct literals `IDENT { .. }`.
"""

import argparse
import os
import re
import sys

# ---------------------------------------------------------------------------
# Identifier classification
# ---------------------------------------------------------------------------

CONSTISH = re.compile(r'^[A-Z][A-Za-z0-9_]*$')

# Prelude enum variants / unit types that are always in scope and are NOT the
# undefined-const bug (they really do match by value). Never flag these.
ALLOW_BINDINGS = {
    'None', 'Some', 'Ok', 'Err',
    'Ordering', 'Less', 'Greater', 'Equal',
    'Cow', 'Borrowed', 'Owned',
}


def looks_const(ident: str) -> bool:
    return bool(CONSTISH.match(ident))


# ---------------------------------------------------------------------------
# Comment / string stripping (line-oriented, good enough for arm detection)
# ---------------------------------------------------------------------------

def strip_line_comment(line: str) -> str:
    # remove // ... but not inside a string. Simple state machine.
    out = []
    i = 0
    in_str = False
    in_char = False
    n = len(line)
    while i < n:
        c = line[i]
        if in_str:
            out.append(c)
            if c == '\\' and i + 1 < n:
                out.append(line[i + 1]); i += 2; continue
            if c == '"':
                in_str = False
            i += 1; continue
        if in_char:
            out.append(c)
            if c == '\\' and i + 1 < n:
                out.append(line[i + 1]); i += 2; continue
            if c == "'":
                in_char = False
            i += 1; continue
        if c == '"':
            in_str = True; out.append(c); i += 1; continue
        if c == '/' and i + 1 < n and line[i + 1] == '/':
            break
        out.append(c); i += 1
    return ''.join(out)


# ---------------------------------------------------------------------------
# Build the workspace const/static/enum-variant universe
# ---------------------------------------------------------------------------

RE_PUB_CONST = re.compile(r'\b(?:pub(?:\([^)]*\))?\s+)?(?:const|static)\s+(?:mut\s+)?([A-Z][A-Za-z0-9_]*)\b')


def harvest_universe(roots):
    """Return a set of all const/static names and enum-variant names defined
    anywhere under the given roots. Over-approximates resolvability (safe for FP)."""
    consts = set()
    rs_files = []
    for root in roots:
        for dp, _dn, fns in os.walk(root):
            if 'target' in dp.split(os.sep):
                continue
            for fn in fns:
                if fn.endswith('.rs'):
                    rs_files.append(os.path.join(dp, fn))

    enum_body = re.compile(r'enum\s+\w+[^{]*\{(.*?)\}', re.DOTALL)
    variant = re.compile(r'\b([A-Z][A-Za-z0-9_]*)\b')
    for path in rs_files:
        try:
            text = open(path, encoding='utf-8', errors='replace').read()
        except OSError:
            continue
        for m in RE_PUB_CONST.finditer(text):
            consts.add(m.group(1))
        # enum variants
        for em in enum_body.finditer(text):
            body = em.group(1)
            # crude: each variant token before ',' or '(' or '{' or '='
            for vm in re.finditer(r'(^|,)\s*([A-Z][A-Za-z0-9_]*)\s*(?:[,({=]|$)', body):
                consts.add(vm.group(2))
    return consts


# ---------------------------------------------------------------------------
# Per-file local-scope resolution
# ---------------------------------------------------------------------------

def file_local_names(text: str):
    """Names that are definitely resolvable inside this file:
    local const/static, local enum variants, explicit `use`d names, and `let`
    bindings (shadowing). Returns a set."""
    names = set()
    has_glob = False

    for m in RE_PUB_CONST.finditer(text):
        names.add(m.group(1))

    # local enum variants
    for em in re.finditer(r'enum\s+\w+[^{]*\{(.*?)\}', text, re.DOTALL):
        for vm in re.finditer(r'(^|,)\s*([A-Z][A-Za-z0-9_]*)\s*(?:[,({=]|$)', em.group(1)):
            names.add(vm.group(2))

    # explicit `use` imports (capture identifiers, ignore globs but record presence)
    for um in re.finditer(r'\buse\s+([^;]+);', text):
        clause = um.group(1)
        if '*' in clause:
            has_glob = True
        # braced group: use a::b::{X, Y as Z, ...}
        for grp in re.finditer(r'\{([^}]*)\}', clause):
            for tok in grp.group(1).split(','):
                tok = tok.strip()
                if not tok:
                    continue
                # handle `X as Y`
                parts = [p.strip() for p in tok.split(' as ')]
                last = parts[-1].split('::')[-1].strip()
                if CONSTISH.match(last):
                    names.add(last)
                first = parts[0].split('::')[-1].strip()
                if CONSTISH.match(first):
                    names.add(first)
        # tail of an unbraced path: use a::b::CONST;  or use a::b::CONST as C;
        tail = clause.split('{')[0]
        if tail.strip() and '*' not in tail:
            tparts = [p.strip() for p in tail.split(' as ')]
            cand = tparts[-1].split('::')[-1].strip()
            if CONSTISH.match(cand):
                names.add(cand)
            cand0 = tparts[0].split('::')[-1].strip()
            if CONSTISH.match(cand0):
                names.add(cand0)

    # let bindings that shadow with a const-looking name (rare but real)
    for lm in re.finditer(r'\blet\s+(?:mut\s+)?([A-Z][A-Za-z0-9_]*)\b', text):
        names.add(lm.group(1))

    return names, has_glob


# ---------------------------------------------------------------------------
# Match-arm scanning
# ---------------------------------------------------------------------------

# A bare-identifier arm pattern. We require the *whole* pattern (everything left
# of the top-level `=>`) to be exactly one const-ish identifier, optionally with a
# trailing guard? No — guards make it refutable-ish but a bare ident binding +
# guard is still a binding; however to be conservative we skip arms with `if`
# guards and `|`. We also require it not be a path (no `::`), not a call, not a
# struct literal, not `@`, not a range.

def iter_match_arms(lines):
    """Yield (lineno, pattern_text, raw_line) for candidate arm lines.
    Heuristic line-based: a line containing `=>` whose left side (after stripping
    comments and leading whitespace) is a single bare identifier."""
    for i, raw in enumerate(lines, start=1):
        line = strip_line_comment(raw)
        if '=>' not in line:
            continue
        left = line.split('=>', 1)[0].strip()
        if not left:
            continue
        yield i, left, raw


def is_risky_pattern(left: str):
    """Return (ident, reason) if `left` is a single bare const-ish binding pattern,
    else (None, None)."""
    # Disqualifiers that mean it's a legit pattern, not a lone binding:
    if left == '_' or left.endswith(','):
        # trailing comma can appear; strip it
        pass
    s = left.rstrip(',').strip()
    if not s:
        return None, None
    # guards / alternations / ranges / bindings / paths / calls / structs / refs
    if any(tok in s for tok in ('::', '|', '@', '..', '(', ')', '{', '}', '[', ']',
                                ' if ', '&', '"', "'", ' as ')):
        return None, None
    if ' ' in s:
        return None, None
    if not CONSTISH.match(s):
        return None, None
    if s == '_':
        return None, None
    if s in ALLOW_BINDINGS:
        return None, None
    return s, 'bare const-ish identifier used as match pattern'


# ---------------------------------------------------------------------------
# Driver
# ---------------------------------------------------------------------------

def scan_file(path, universe):
    text = open(path, encoding='utf-8', errors='replace').read()
    lines = text.splitlines()
    local_names, has_glob = file_local_names(text)
    findings = []

    arms = list(iter_match_arms(lines))
    # index arm linenos for "followed-by-more-arms" smell
    arm_linenos = [a[0] for a in arms]

    for idx, (lineno, left, raw) in enumerate(arms):
        ident, reason = is_risky_pattern(left)
        if ident is None:
            continue
        if ident in local_names:
            continue  # resolved in-file -> SAFE, not the bug

        in_universe = ident in universe

        # confidence model
        if not in_universe:
            confidence = 'HIGH'
            why = ('not defined in-file and not found anywhere in the Rust workspace '
                   'as a const/enum-variant -> silent catch-all binding')
        elif has_glob:
            confidence = 'LOW'
            why = ('found in workspace as a const/variant and this file has glob '
                   'imports -> probably resolved through a `use ...::*` (likely FP)')
        else:
            confidence = 'MEDIUM'
            why = ('defined elsewhere in the workspace but this file has NO glob '
                   'import that could bring it into scope -> may be a silent binding')

        # corroborating smell: are there subsequent arms in the same match block?
        # heuristic: another arm within the next ~80 lines at >= same indent.
        followed = False
        indent = len(raw) - len(raw.lstrip())
        for ln2 in arm_linenos:
            if lineno < ln2 <= lineno + 120:
                raw2 = lines[ln2 - 1]
                ind2 = len(raw2) - len(raw2.lstrip())
                if ind2 == indent:
                    followed = True
                    break

        findings.append({
            'path': path,
            'line': lineno,
            'ident': ident,
            'confidence': confidence,
            'why': why,
            'followed_by_arms': followed,
            'pattern': left,
        })
    return findings


def main():
    ap = argparse.ArgumentParser(description=__doc__,
                                 formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument('paths', nargs='*',
                    help='files/dirs to scan (default: sdk/lib/rust/ntoskrnl-rust/src)')
    ap.add_argument('--universe-root', action='append', default=None,
                    help='root(s) to harvest the const/enum-variant universe from '
                         '(default: sdk/lib/rust)')
    ap.add_argument('--include-low', action='store_true',
                    help='also report LOW-confidence (likely-FP, glob-resolvable) hits')
    ap.add_argument('--context', type=int, default=0,
                    help='print N lines of context around each hit')
    ap.add_argument('--check', action='store_true',
                    help='exit 1 if any HIGH (or MEDIUM) confidence hazard is found')
    args = ap.parse_args()

    repo = os.getcwd()
    targets = args.paths or [os.path.join('sdk', 'lib', 'rust', 'ntoskrnl-rust', 'src')]
    uni_roots = args.universe_root or [os.path.join('sdk', 'lib', 'rust')]

    # collect .rs files to scan
    scan_files = []
    for t in targets:
        if os.path.isfile(t):
            scan_files.append(t)
        else:
            for dp, _dn, fns in os.walk(t):
                if 'target' in dp.split(os.sep):
                    continue
                for fn in fns:
                    if fn.endswith('.rs'):
                        scan_files.append(os.path.join(dp, fn))

    sys.stderr.write(f'[verify_match_consts] harvesting const universe from {uni_roots} ...\n')
    universe = harvest_universe(uni_roots)
    sys.stderr.write(f'[verify_match_consts] {len(universe)} const/enum-variant names known; '
                     f'scanning {len(scan_files)} files ...\n')

    all_findings = []
    for path in sorted(scan_files):
        all_findings.extend(scan_file(path, universe))

    order = {'HIGH': 0, 'MEDIUM': 1, 'LOW': 2}
    all_findings.sort(key=lambda f: (order[f['confidence']],
                                     not f['followed_by_arms'],
                                     f['path'], f['line']))

    shown = [f for f in all_findings
             if args.include_low or f['confidence'] != 'LOW']

    counts = {'HIGH': 0, 'MEDIUM': 0, 'LOW': 0}
    for f in all_findings:
        counts[f['confidence']] += 1

    for f in shown:
        rel = os.path.relpath(f['path'], repo)
        smell = '  [+followed-by-arms: unreachable smell]' if f['followed_by_arms'] else ''
        print(f"{rel}:{f['line']}: [{f['confidence']}] `{f['ident']}` -- {f['why']}{smell}")
        if args.context:
            try:
                lines = open(f['path'], encoding='utf-8', errors='replace').read().splitlines()
            except OSError:
                lines = []
            lo = max(1, f['line'] - args.context)
            hi = min(len(lines), f['line'] + args.context)
            for ln in range(lo, hi + 1):
                marker = '>>' if ln == f['line'] else '  '
                print(f"    {marker} {ln}: {lines[ln-1]}")
            print()

    print()
    print(f"SUMMARY: HIGH={counts['HIGH']}  MEDIUM={counts['MEDIUM']}  "
          f"LOW(likely-FP)={counts['LOW']}  shown={len(shown)}")

    if args.check and (counts['HIGH'] + counts['MEDIUM']) > 0:
        return 1
    return 0


if __name__ == '__main__':
    sys.exit(main())
