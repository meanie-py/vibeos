"""A typedef that is already a pointer must not be given a second star.

C's `_locale_t` IS `struct localeinfo_struct *`. The model does not know that — the star
is in a header, not in the spelling it is shown — so it reads `_locale_t locale` as a bare
type name and translates it the way it translates every other bare type name: by adding the
indirection the parameter appears to lack. On msvcrt's `mbcs.c` it did exactly that in **73
of 77** locale-taking signatures, emitting `locale: *mut _locale_t` where the C says
`_locale_t locale`.

That is a `**localeinfo_struct` where the caller passes a `*localeinfo_struct`.

## Why this needs a pass rather than a better prompt

The failure is *invisible at the ABI*. Both spellings are one machine pointer, so the
signature links, the call passes the identical bits, and nothing in the toolchain objects.
What breaks is every dereference in the body — and only some of those are loud. Here they
happened to be loud (`(*locale).mbcinfo` does not typecheck against `_locale_t`), which is
the ONLY reason this was caught at all. Write the same mistake at a position the body just
forwards, and it compiles, links, ships, and reads one indirection too deep at runtime.

So the model being right 4 times out of 77 is not a prompt problem to tune. It is a fact the
translator can *prove* and therefore must never delegate — the same argument that moved
constants, call targets, and data tables off the paid path.

## The rule, and why it cannot fire wrongly

Rewrite `*mut T` / `*const T` to `T` at a parameter position where **all** of:

* the C declares that parameter as `T` with **no** `*` of its own, and
* `rustconv.typedefs` proves `T` is a plain pointer typedef from the TU's own include
  closure, and
* the Rust spelling is exactly one star applied to **that same name**.

There is no C declaration `T x` whose faithful Rust is `*mut T` when `T` is itself a pointer
— the star is pure invention, and stripping it restores what the C said. The third condition
is what keeps this from touching a legitimately-different type: if the model wrote
`*mut c_void`, or `*mut localeinfo_struct` (the expansion, which is *correct*), the names do
not match and the pass leaves it alone.

Everything outside that shape is untouched and unreported. In particular the benign
re-spellings are none of this pass's business: `size_t`/`usize`, `unsigned int`/`c_uint`,
and `threadmbcinfo *`/`pthreadmbcinfo` are all *correct* translations that merely disagree
with a mechanical mapping, and 46 of them sit alongside the 73 real defects in the same file.
A pass that "corrected" those would be a regression wearing a fidelity badge.
"""

from __future__ import annotations

import re
from collections.abc import Mapping
from dataclasses import dataclass
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from .scanner import ScanResult

#: A translated definition's header. Deliberately anchored on the `extern "…"` form that
#: `abi_enforce` has already canonicalised, so this runs on a settled signature rather than
#: racing it. `[^)]*` is safe here: a Rust parameter list contains no `)` — a function-pointer
#: parameter would, but `Option<unsafe extern "C" fn(..)>` is not a shape this pass edits, and
#: a non-match is simply a function left alone.
_RUST_FN = re.compile(
    r"(?P<head>(?:pub\s+)?unsafe\s+extern\s+\"[^\"]+\"\s+fn\s+(?P<name>\w+)\s*\()"
    r"(?P<params>[^)]*)"
    r"(?P<tail>\))"
)

#: One `name: type` binding. `mut` is legal on a parameter pattern and must survive.
_PARAM = re.compile(r"^(?P<bind>(?:mut\s+)?\w+\s*:\s*)(?P<type>.+)$", re.S)

_STARRED = re.compile(r"^\*\s*(?:mut|const)\s+(?P<inner>[\w:]+)$")


@dataclass(frozen=True)
class SignatureFinding:
    function: str
    parameter: str
    detail: str

    def __str__(self) -> str:
        return f"{self.function}({self.parameter}): {self.detail}"


def _c_param_types(scan: ScanResult) -> dict[str, list[str]]:
    return {fn.name: [c_type for c_type, _ in fn.params] for fn in scan.functions}


def _bare(c_type: str) -> str:
    """The C type with qualifiers stripped, or `""` if it carries its own indirection."""
    if "*" in c_type or "[" in c_type:
        return ""
    words = [w for w in c_type.split() if w not in ("const", "volatile")]
    return words[0] if len(words) == 1 else ""


def repair_signatures(
    rust: str,
    scan: ScanResult,
    pointer_typedefs: Mapping[str, str],
) -> tuple[str, list[SignatureFinding]]:
    """Strip invented indirection from parameters typed by a proven pointer typedef.

    Returns `(rust, findings)`. A finding per parameter actually rewritten — silence must
    mean "the model got every one right", never "the pass could not tell".
    """
    if not pointer_typedefs:
        return rust, []

    c_params = _c_param_types(scan)
    findings: list[SignatureFinding] = []

    def fix_fn(match: re.Match[str]) -> str:
        name = match.group("name")
        declared = c_params.get(name)
        if declared is None:
            return match.group(0)  # no C counterpart: nothing to prove against

        parts = match.group("params").split(",")
        # A trailing comma leaves an empty tail element; keep it so the rendering is stable.
        bindings = [p for p in parts if p.strip()]
        if len(bindings) != len(declared):
            # `(void)` scans as one parameter, and a variadic tail has no C counterpart.
            # Either way the positions no longer line up, so nothing here is provable.
            return match.group(0)

        rebuilt: list[str] = []
        changed = False
        for binding, c_type in zip(bindings, declared, strict=True):
            param = _PARAM.match(binding.strip())
            bare = _bare(c_type)
            starred = _STARRED.match(param.group("type").strip()) if param else None
            if (
                param is None
                or starred is None
                or not bare
                or bare not in pointer_typedefs
                or starred.group("inner") != bare
            ):
                rebuilt.append(binding)
                continue
            leading = binding[: len(binding) - len(binding.lstrip())]
            rebuilt.append(f"{leading}{param.group('bind')}{bare}")
            findings.append(
                SignatureFinding(
                    name,
                    param.group("bind").split(":")[0].strip(),
                    f"C declares `{c_type}`, which is itself a pointer typedef — dropped the "
                    f"invented star from `{param.group('type').strip()}`",
                )
            )
            changed = True

        if not changed:
            return match.group(0)
        return f"{match.group('head')}{','.join(rebuilt)}{match.group('tail')}"

    return _RUST_FN.sub(fix_fn, rust), findings
