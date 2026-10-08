#!/usr/bin/env python3
"""Census of the rustc warnings a build emitted, sorted by whether they can be a BUG.

Most of the workspace's warnings are translation noise -- dead helpers, FFI names that
are not snake_case, unused imports -- and a raw count buries the few that are not. This
tool reads the warnings out of `vibeos-cli build` logs (`/tmp/vibeos-cli/<ts>-build-<arch>/
full.log`) and sorts every one into a RISK tier, so the work starts at the top:

  UB      the program as written is undefined or miscompiles (`invalid_value` on a struct
          left uninitialized, `deref_nullptr`, a self-recursive body, an FFI type with no
          stable layout).
  LOGIC   the code does something its author plainly did not intend: a statement with no
          effect (`STATUS;` where `STATUS()` was meant), a comparison that is always true or
          discarded (`FAILED(hr);` with its `if` lost), a pointer null-check that can never
          fire, an extern redeclared with a different signature.
  NOISE   style and dead code. Real cleanup, no behaviour.

A lint this file has not classified is reported as UNCLASSIFIED, never folded into NOISE:
a new rustc lint arrives unranked, and defaulting it to "harmless" is how a real one
would hide.

WHAT A LOG CAN AND CANNOT TELL YOU. A warm build recompiles only what changed, so its log
carries warnings for those crates alone -- cargo replays a fresh unit's cached diagnostics
only when cargo is invoked for it, and ninja does not invoke it. The census therefore
reports which crates it SAW; a crate missing from the log is missing coverage, not a
clean crate. `vibeos-cli build --arch <a> --clean` gives a complete log.

WHY THE LINT NAME IS INFERRED. rustc attaches the `#[warn(lint)]` note to the FIRST
occurrence of each lint per crate only, so most blocks carry no name. The name is
recovered from the message: every annotated block teaches its message TEMPLATE (backticked
spans and numbers blanked), and an un-annotated block takes the lint its template was
taught. A template no annotated block taught is UNATTRIBUTED and reported as such.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from collections import defaultdict
from dataclasses import dataclass, field
from pathlib import Path

UB = {
    "invalid_value",
    "deref_nullptr",
    "unconditional_recursion",
    "invalid_reference_casting",
    "dangling_pointers_from_temporaries",
    "dangling_pointers_from_locals",
    "improper_ctypes",
    "improper_ctypes_definitions",
    "static_mut_refs",
    "invalid_null_arguments",
    "ptr_to_integer_transmute_in_consts",
    "integer_to_ptr_transmutes",
    "invalid_from_utf8",
    "uninhabited_static",
    "undefined_naked_function_abi",
}
LOGIC = {
    "path_statements",
    "unused_comparisons",
    "unused_must_use",
    "useless_ptr_null_checks",
    "unreachable_code",
    "unreachable_patterns",
    "clashing_extern_declarations",
    "unpredictable_function_pointer_comparisons",
    "ambiguous_wide_pointer_comparisons",
    "overflowing_literals",
    "unused_assignments",
    "dropping_references",
    "dropping_copy_types",
    "forgetting_references",
    "forgetting_copy_types",
    "invalid_nan_comparisons",
    "let_underscore_drop",
    "while_true",
    "irrefutable_let_patterns",
    "bindings_with_variant_name",
    "no_mangle_generic_items",
    "overlapping_range_endpoints",
    "non_contiguous_range_endpoints",
    "unused_results",
    "arithmetic_overflow",
    "unconditional_panic",
}
NOISE = {
    "dead_code",
    "unused_imports",
    "unused_variables",
    "unused_mut",
    "unused_labels",
    "unused_macros",
    "unused_doc_comments",
    "unused_unsafe",
    "unused_parens",
    "unused_braces",
    "unused_attributes",
    "unused_features",
    "unused_allocation",
    "unused_macro_rules",
    "non_snake_case",
    "non_camel_case_types",
    "non_upper_case_globals",
    "bad_asm_style",
    "unexpected_cfgs",
    "unnecessary_transmutes",
    "private_interfaces",
    "private_bounds",
    "hidden_glob_reexports",
    "ambiguous_glob_reexports",
    "special_module_name",
    "redundant_semicolons",
    "ellipsis_inclusive_range_patterns",
    "deprecated",
    "unstable_name_collisions",
    "non_local_definitions",
    "renamed_and_removed_lints",
    "unknown_lints",
    "stable_features",
    "unused_qualifications",
    "repr_transparent_external_private_fields",
    "missing_abi",
    "uncommon_codepoints",
    "confusable_idents",
    "mixed_script_confusables",
}
TIERS = ("UB", "LOGIC", "NOISE", "UNCLASSIFIED")


def tier_of(lint: str) -> str:
    if lint in UB:
        return "UB"
    if lint in LOGIC:
        return "LOGIC"
    if lint in NOISE:
        return "NOISE"
    return "UNCLASSIFIED"


_HEAD = re.compile(r"^warning: (?P<msg>.+)$")
_SUMMARY = re.compile(
    r"^warning: `(?P<crate>[^`]+)` \((?P<kind>[^)]+)\) generated (?P<n>\d+) warnings?"
)
_LOC = re.compile(r"^\s*--> (?P<path>[^:\s]+):(?P<line>\d+):(?P<col>\d+)")
_NOTE = re.compile(r"#\[warn\((?P<lint>[a-z0-9_]+)\)\]")
_BACKTICK = re.compile(r"`[^`]*`")
_NUM = re.compile(r"\b\d+\b")


_LIST = re.compile(r"`_`(?:(?:, | and |, and | or |, or )`_`)+")
_PLURAL = re.compile(
    r"\b(import|variant|field|function|method|constant|associated item|macro|label)s\b"
)


def template(msg: str) -> str:
    """The message with its specifics blanked: `unused imports: `A`, `B`, and `C`` and
    `unused import: `D`` are one shape, so a list collapses and a plural folds."""
    t = _NUM.sub("N", _BACKTICK.sub("`_`", msg))
    return _PLURAL.sub(r"\1", _LIST.sub("`_`", t)).strip()


# Messages whose first occurrence in a log may carry no note at all (the note went to an
# earlier invocation of the same crate, or the shape varies by pluralization in a way no
# template catches). Consulted ONLY when no annotated block taught the template -- a
# taught template always wins, so this cannot override what rustc itself said.
_FALLBACK = [
    (re.compile(r"^unused imports?: "), "unused_imports"),
    (re.compile(r" is never read$"), "unused_assignments"),
    (re.compile(r"should have an upper camel case name$"), "non_camel_case_types"),
    (re.compile(r"should have a snake case name$"), "non_snake_case"),
    (re.compile(r"should have an upper case name$"), "non_upper_case_globals"),
    (re.compile(r" (is|are) never (used|constructed|read)"), "dead_code"),
    (re.compile(r"^unnecessary parentheses"), "unused_parens"),
    (re.compile(r"^unnecessary braces"), "unused_braces"),
    (re.compile(r"^unused variables?: "), "unused_variables"),
    (re.compile(r"^variable does not need to be mutable$"), "unused_mut"),
    (re.compile(r"^unnecessary `unsafe` block$"), "unused_unsafe"),
    (re.compile(r"^unused label$"), "unused_labels"),
    (re.compile(r"^unnecessary transmute$"), "unnecessary_transmutes"),
]


@dataclass
class Site:
    path: str
    line: int
    col: int
    msg: str
    lint: str | None
    snippet: str = ""
    arches: set[str] = field(default_factory=set)

    @property
    def crate(self) -> str:
        return self.path.split("/", 1)[0]

    def key(self) -> tuple:
        return (self.path, self.line, self.col, template(self.msg))


def parse_log(text: str) -> tuple[list[Site], dict[str, int]]:
    """(warning sites in order, {crate: warnings rustc says it generated})."""
    sites: list[Site] = []
    claimed: dict[str, int] = {}
    cur: Site | None = None
    want_snippet = False
    for raw in text.splitlines():
        line = raw.rstrip()
        m = _SUMMARY.match(line)
        if m:
            cur = None
            # The same crate is reported once per cargo invocation that builds it; keep
            # the largest claim rather than summing repeats of one crate.
            claimed[m["crate"]] = max(claimed.get(m["crate"], 0), int(m["n"]))
            continue
        m = _HEAD.match(line)
        if m:
            cur = Site("", 0, 0, m["msg"], None)
            want_snippet = False
            continue
        if line.startswith("error"):
            cur = None
            continue
        if cur is None:
            continue
        if not cur.path:
            m = _LOC.match(line)
            if m:
                cur.path, cur.line, cur.col = m["path"], int(m["line"]), int(m["col"])
                sites.append(cur)
                want_snippet = True
            continue
        if want_snippet:
            sm = re.match(rf"^\s*{cur.line}\s+\|\s?(.*)$", line)
            if sm:
                cur.snippet = sm.group(1).strip()[:160]
                want_snippet = False
        if cur.lint is None:
            m = _NOTE.search(line)
            if m:
                cur.lint = m["lint"]
    return sites, claimed


def attribute(sites: list[Site]) -> None:
    """Fill each un-annotated site's lint from the template an annotated one taught.

    A template taught two DIFFERENT lints is ambiguous and teaches nothing: guessing
    between them would be a confident wrong answer."""
    taught: dict[str, set[str]] = defaultdict(set)
    for s in sites:
        if s.lint:
            taught[template(s.msg)].add(s.lint)
    for s in sites:
        if s.lint is None:
            lints = taught.get(template(s.msg), set())
            if len(lints) == 1:
                s.lint = next(iter(lints))
            elif not lints:
                s.lint = next((lint for rx, lint in _FALLBACK if rx.search(s.msg)), None)


def merge(per_arch: dict[str, list[Site]]) -> list[Site]:
    out: dict[tuple, Site] = {}
    for arch, sites in per_arch.items():
        for s in sites:
            k = s.key()
            if k not in out:
                out[k] = Site(s.path, s.line, s.col, s.msg, s.lint, s.snippet)
            elif out[k].lint is None and s.lint:
                out[k].lint = s.lint
            out[k].arches.add(arch)
    return sorted(out.values(), key=lambda s: (s.path, s.line, s.col))


def arch_of(log: Path) -> str:
    m = re.search(r"-build-(i386|amd64|arm64)", str(log))
    return m.group(1) if m else log.parent.name


def main(argv: list[str]) -> int:
    ap = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    ap.add_argument("logs", nargs="+", type=Path, help="vibeos-cli build full.log file(s)")
    ap.add_argument("--tier", choices=TIERS, help="list the sites of one tier")
    ap.add_argument("--lint", help="list the sites of one lint")
    ap.add_argument("--crate", help="restrict to one crate (path prefix, e.g. kernel32-rust)")
    ap.add_argument("--json", action="store_true", help="machine-readable sites")
    args = ap.parse_args(argv)

    per_arch: dict[str, list[Site]] = defaultdict(list)
    claimed_all: dict[str, int] = {}
    for log in args.logs:
        if not log.is_file():
            print(f"MISSING LOG: {log}", file=sys.stderr)
            return 2
        sites, claimed = parse_log(log.read_text(errors="replace"))
        attribute(sites)
        per_arch[arch_of(log)].extend(sites)
        for c, n in claimed.items():
            claimed_all[c] = max(claimed_all.get(c, 0), n)
    sites = merge(per_arch)
    # Teach once more across arches: a template annotated only on amd64 still names
    # the lint of the same message on i386.
    attribute(sites)
    if args.crate:
        sites = [s for s in sites if s.crate == args.crate]

    def label(s: Site) -> str:
        return s.lint or "UNATTRIBUTED"

    if args.json:
        json.dump(
            [
                {
                    "path": s.path,
                    "line": s.line,
                    "col": s.col,
                    "lint": s.lint,
                    "tier": tier_of(s.lint) if s.lint else "UNCLASSIFIED",
                    "msg": s.msg,
                    "snippet": s.snippet,
                    "arches": sorted(s.arches),
                }
                for s in sites
            ],
            sys.stdout,
            indent=1,
        )
        print()
        return 0

    if args.tier or args.lint:
        for s in sites:
            t = tier_of(s.lint) if s.lint else "UNCLASSIFIED"
            if (args.tier and t != args.tier) or (args.lint and s.lint != args.lint):
                continue
            arches = ",".join(sorted(s.arches))
            print(f"{s.path}:{s.line}:{s.col}  [{label(s)}] [{arches}]  {s.msg}")
            if s.snippet:
                print(f"      | {s.snippet}")
        return 0

    by_tier: dict[str, dict[str, list[Site]]] = {t: defaultdict(list) for t in TIERS}
    for s in sites:
        t = tier_of(s.lint) if s.lint else "UNCLASSIFIED"
        by_tier[t][label(s)].append(s)
    print(f"{len(sites)} distinct warning sites across {', '.join(sorted(per_arch))}\n")
    for t in TIERS:
        lints = by_tier[t]
        total = sum(len(v) for v in lints.values())
        print(f"== {t}: {total}")
        for lint, ss in sorted(lints.items(), key=lambda kv: -len(kv[1])):
            crates = sorted({s.crate for s in ss})
            more = f" +{len(crates) - 4}" if len(crates) > 4 else ""
            print(f"   {len(ss):6d}  {lint:44s} {', '.join(crates[:4])}{more}")
        print()
    seen = {s.crate for s in sites}
    print(
        f"coverage: {len(seen)} crate(s) emitted warnings in these logs; "
        f"{len(claimed_all)} crate summaries read. A crate absent here was NOT compiled "
        "by these builds -- that is missing coverage, not a clean crate.",
        file=sys.stderr,
    )
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
