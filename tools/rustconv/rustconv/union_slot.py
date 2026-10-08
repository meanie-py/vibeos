"""A C union member re-slotted onto its neighbour's name.

`Registers->Status = base + 7` came back as `(*Registers).__bindgen_anon_1.Error = ...`, and
`return ATA_READ(Regs.Status)` as `ATA_READ(Regs.__bindgen_anon_1.Error)`. Both compile:
every member of the anonymous unions in IDE_REGISTERS is a `PUCHAR`, so a member of the WRONG
union type-checks. pciidex sent every ATA command to I/O port 0 and read the error register as
status, and the --ata-stack boot found no disks.

Decidable without knowing the struct: the model chose the member's NAME, and a member the C
function never spells is one it invented. So each `__bindgen_anon_N.M` in a Rust function must
have `M` among the identifiers of its C function. A nested `__bindgen_anon_M` is bindgen's, and
a `__lit.` receiver is the zero-initialiser `port` generates for a C `{0}` literal.

When the C names BOTH members the name test is blind (PciIdeInitTaskFileIoResources sets
`Error` and `Status`; the model wrote `Error` twice). So a member the Rust ASSIGNS through a
`__bindgen_anon_N` more often than the C assigns `.M`/`->M` is reported too. Writes only:
C's `x |= y` is Rust's `x = x | y`, and reads double whenever a macro is expanded inline.
And only when some other member the C writes lost a write in the Rust, since goto elimination
copies a label's tail (`IoStatus.Status = Status`) into every arm that jumped to it.

A C macro that expands to a member access can still produce a false report; the report names
the function and never edits.
"""

from __future__ import annotations

import re
from collections import Counter
from collections.abc import Mapping
from dataclasses import dataclass
from typing import Protocol

from .logical_not import _fn_bodies


class _Decls(Protocol):
    # Read-only, so a frozen dataclass (pointer_depth.CFunctionDecls) satisfies it.
    @property
    def identifiers(self) -> frozenset[str]: ...
    @property
    def member_uses(self) -> tuple[tuple[str, int], ...]: ...  # C writes per member


@dataclass(frozen=True)
class Reslot:
    fn: str
    member: str
    line: int


_WRITE = re.compile(r"\.\s*([A-Za-z_]\w*)\s*(?:[-+*/%&|^]|<<|>>)?=(?!=)")
_ANON = re.compile(r"(\b__lit\s*\.\s*)?\b__bindgen_anon_\d+\s*\.\s*([A-Za-z_]\w*)")


def scan(rust: str, decls: Mapping[str, _Decls]) -> list[Reslot]:
    found: list[Reslot] = []
    for name, lo, hi in _fn_bodies(rust):
        d = decls.get(name)
        if d is None:
            continue
        seen: set[str] = set()
        c_uses = dict(d.member_uses)
        rust_uses: dict[str, list[int]] = {}
        # Blank string literals: an ASSERT carries its condition's text as a byte string.
        code = re.sub(r'b?"(?:[^"\\]|\\.)*"', lambda q: " " * len(q.group(0)), rust[:hi])
        for m in _ANON.finditer(code, lo, hi):
            member = m.group(2)
            if m.group(1) or member.startswith("__bindgen_anon"):
                continue
            line = rust.count("\n", 0, m.start()) + 1
            if re.match(r"\s*(?:[-+*/%&|^]|<<|>>)?=(?!=)", code[m.end() : m.end() + 4]):
                rust_uses.setdefault(member, []).append(line)
            if member in d.identifiers or member in seen:
                continue
            seen.add(member)
            found.append(Reslot(name, member, line))
        # A re-slot MOVES a write: one member gains it and another loses it. A gain alone is
        # goto elimination copying a label's tail into each arm (`IoStatus.Status = Status`).
        writes = Counter(_WRITE.findall(code, lo, hi))
        lost = any(writes[m] < n for m, n in c_uses.items())
        for member, lines in rust_uses.items():
            if lost and member not in seen and member in c_uses and len(lines) > c_uses[member]:
                found.append(Reslot(name, member, lines[c_uses[member]]))
    return found
