"""Static data tables come from the C, verbatim — never from a model's transcription.

A lookup table is the one construct where a model has *nothing* to contribute and every
opportunity to be subtly wrong: it is pure data, it is long, and an error in it is invisible
in review. msvcrt's `mbcs.c` proved the point three separate times across two runs of the
same file:

* `mbbtombc_932` — C has 158 elements; one run emitted 190 (32 invented trailing zeros),
  the next declared `[u8; 187]` over 158 initialisers.
* `mbctombb_932_punct` — correct 88 values under a declared `[u8; 80]`, then `[u8; 72]`.

Note what those have in common: the VALUES were right both times, and the LENGTH was wrong
every time. A declared length that disagrees with the initialiser count does not compile, so
that half is loud. The other half — a padded or truncated table that still compiles — is
exactly the silent-wrongness class this repo keeps paying for, and `_mbbtombc_l` indexes
`mbbtombc_932` with attacker-adjacent input.

So this pass does not "check" the tables, it **replaces** them with the C's own text. Values
and length both come from the source of truth; the model's copy is discarded wholesale rather
than diffed. What cannot be matched to a C array is left alone and reported, never guessed.
"""

from __future__ import annotations

import re
from dataclasses import dataclass

#: `static const unsigned char name[] = { ... };` at C file scope. Sized declarations
#: (`name[16]`) are deliberately excluded: their length is already explicit in the C and
#: carrying it across is the translator's job, not a repair's.
_C_ARRAY = re.compile(
    r"static\s+(?:const\s+)?"
    r"(?:unsigned\s+char|signed\s+char|char|unsigned\s+short|short|unsigned\s+int|int|"
    r"WCHAR|BYTE|WORD|DWORD)\s+"
    r"(\w+)\s*\[\s*\]\s*=\s*\{(.*?)\}\s*;",
    re.S,
)

#: `pub static mut NAME: [T; N] = [ ... ];`
_RUST_ARRAY = re.compile(
    r"(?P<head>(?:pub\s+)?static\s+(?:mut\s+)?(?P<name>\w+)\s*:\s*\[\s*(?P<ty>[\w:]+)\s*;\s*)"
    r"(?P<len>\d+)"
    r"(?P<mid>\s*\]\s*=\s*\[)(?P<body>.*?)(?P<tail>\]\s*;)",
    re.S,
)

_PER_ROW = 16

#: Rust array bodies this pass understands: a FLAT list of scalar literals, nothing else.
#: Two shapes look like comma-separated data and are not, and both were mis-"repaired" the
#: first time this ran on mbcs.c:
#:
#: * `[0; 257]` is a repeat expression, not a list — counting commas gives 1 element and
#:   "correcting" the length to 1 silently shrinks a 257-byte table to a single byte.
#: * `[cp_extra_info_t { cp: 932, TrailBytes: [..] }, ..]` is an array of STRUCTS —
#:   counting commas gives 42 for 7 elements, and its data lives in the C as an
#:   initialiser list this pass cannot linearise anyway.
#:
#: So the rule is whitelist, not blacklist: every element must be a plain integer literal.
#: Anything else is left completely alone, with no finding — a valid construct is not a
#: defect, and a pass that cries wolf on `[0; N]` would get switched off.
_SCALAR_ELEMENT = re.compile(r"^[+-]?(?:0[xX][0-9a-fA-F]+|\d+)$")


def _flat_scalar_elements(body: str) -> list[str] | None:
    """Elements of a flat integer-literal array body, or ``None`` if it is anything else."""
    if any(ch in body for ch in "{}[]();"):
        return None
    elements = [e.strip() for e in body.split(",") if e.strip()]
    if not elements or not all(_SCALAR_ELEMENT.match(e) for e in elements):
        return None
    return elements


@dataclass(frozen=True)
class TableFinding:
    name: str
    detail: str

    def __str__(self) -> str:
        return f"table {self.name}: {self.detail}"


def c_arrays(c_source: str) -> dict[str, list[str]]:
    """``{name: [element text, ...]}`` for every unsized static array in *c_source*."""
    out: dict[str, list[str]] = {}
    for match in _C_ARRAY.finditer(c_source):
        elements = [e.strip() for e in match.group(2).split(",")]
        # A trailing comma before `}` is idiomatic C and is not an element.
        out[match.group(1)] = [e for e in elements if e]
    return out


def _same(a: list[str], b: list[str]) -> bool:
    """Numeric equality, so `0x40` and `64` compare equal; falls back to text."""
    try:
        return [int(x, 0) for x in a] == [int(x, 0) for x in b]
    except ValueError:
        return a == b


def repair_tables(rust: str, c_source: str) -> tuple[str, list[TableFinding]]:
    """Rewrite every Rust static array from its C counterpart. Returns (rust, findings).

    A finding is reported for each table actually changed — silence would make this
    indistinguishable from a run where the model got them right, and the whole reason the
    pass exists is that it does not.
    """
    tables = c_arrays(c_source)
    findings: list[TableFinding] = []

    def replace(match: re.Match[str]) -> str:
        name = match.group("name")
        body = _flat_scalar_elements(match.group("body"))
        if body is None:
            return match.group(0)  # repeat expression, struct array, … — not ours

        values = tables.get(name)
        if values is None:
            # Not a C table we can source. Still catch the guaranteed compile error of a
            # declared length that disagrees with the initialisers actually present.
            if len(body) != int(match.group("len")):
                findings.append(
                    TableFinding(
                        name,
                        f"declared [{match.group('ty')}; {match.group('len')}] over "
                        f"{len(body)} initialisers, and no C array of that name to source "
                        "from — declared length corrected, VALUES NOT VERIFIED",
                    )
                )
                return (
                    f"{match.group('head')}{len(body)}{match.group('mid')}"
                    f"{match.group('body')}{match.group('tail')}"
                )
            return match.group(0)

        declared = int(match.group("len"))
        if declared == len(values) == len(body) and _same(body, values):
            return match.group(0)  # already faithful

        problems = []
        if declared != len(body):
            problems.append(f"declared {declared} over {len(body)} initialisers")
        if len(body) != len(values):
            problems.append(f"{len(body)} elements vs the C's {len(values)}")
        elif not _same(body, values):
            problems.append("values differ from the C")
        findings.append(
            TableFinding(name, "; ".join(problems) + " — REPLACED with the C's own data")
        )

        rows = [",".join(values[i : i + _PER_ROW]) for i in range(0, len(values), _PER_ROW)]
        rendered = "\n" + "\n".join(f"    {row}," for row in rows) + "\n"
        return (
            f"{match.group('head')}{len(values)}{match.group('mid')}"
            f"{rendered}{match.group('tail')}"
        )

    return _RUST_ARRAY.sub(replace, rust), findings
