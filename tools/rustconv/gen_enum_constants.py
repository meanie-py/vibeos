#!/usr/bin/env python3
"""Generate the CamelCase enum-constant ground truth used by fact sealing.

WHY THIS EXISTS
---------------
`rustconv.fact_seal` removes the job of copying constants from the model: every
constant occurrence in a C body becomes an opaque token the model must preserve,
and rustconv restores the exact source spelling afterwards.  Its symbol matcher
is `[A-Z][A-Z0-9_]+` — ALL_CAPS only — because that is what `#define`d Win32/NT
constants look like.

That misses an entire family.  DDK/PSDK **enum** constants are CamelCase:
`NonPagedPool`, `PagedPool`, `KernelMode`, `Executive`, `NotificationEvent`,
`MmNonCached`, and the whole `Ndis*` vocabulary.  Measured over the network
client DLLs and drivers, that is 345 unprotected value occurrences across 114
distinct names — every one of them a chance for the model to invent a plausible
sibling of the right enum.  Picking the wrong `POOL_TYPE` or the wrong
`KPROCESSOR_MODE` is exactly the failure mode that is invisible to rustc.

This is the AGENTS.md substrate rule — constants never go through the LLM — and
it is the same class the kernel just ate a 61-site sweep for (field manual
§9.113: a translated `KeBugCheck(SYMBOL)` came back as a *fabricated* number,
plausible and internally consistent, so review slid off it).

Sealing a CamelCase name is only safe because the Rust side spells these
IDENTICALLY to C: bindgen emits `pub const NonPagedPool: POOL_TYPE = 0;`, so
restoring the C spelling yields valid Rust.  (Verified against ~4,000 existing
usages in sdk/lib/rust.)

WHY A GENERATED FILE RATHER THAN A RUNTIME SCAN
-----------------------------------------------
Same reason `tools/gen_bugcodes.py` generates `ntkrnl_sys::bugcodes` from
bugcodes.mc: the value set gets ONE definition site, derived mechanically from
ground truth, auditable in review, and cheap at translation time.  Re-run this
after an SDK header bump.

Usage:
    python3 tools/rustconv/gen_enum_constants.py            # rewrite in place
    python3 tools/rustconv/gen_enum_constants.py --check     # CI: exit 1 if stale
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
# vendor/libdxg/include holds d3dkmthk.h, a DDK header until the vendored-code move put it
# under vendor/. Its D3DKMT_* CamelCase constants are still values the model must not be
# trusted to copy, so the header is scanned where it now lives rather than dropping out
# of the sealed set -- which is what the move did first, and this check caught.
HEADER_ROOTS = (
    "sdk/include/ddk",
    "sdk/include/ndk",
    "sdk/include/xdk",
    "sdk/include/psdk",
    "vendor/libdxg/include",
)
OUTPUT = Path(__file__).resolve().parent / "rustconv" / "ddk_enum_constants.py"

_ENUM_BODY_RE = re.compile(r"\btypedef\s+enum\b[^{]*\{(.*?)\}", re.S)
_LEADING_NAME_RE = re.compile(r"([A-Za-z_]\w*)")

# Object-like `#define Name <replacement>` (function-like macros excluded by (?!\().
_DEFINE_RE = re.compile(r"^\s*#\s*define\s+([A-Za-z_]\w*)(?!\()\s+(\S[^\n]*)$", re.M)

# A macro whose replacement text is a bare numeric literal is PROVABLY a value.
# This classifier is what makes the mixed-case #define family safe to seal at all.
# Measured over the network DLLs and drivers it selected 41 occurrences — every one
# a real constant (`NERR_Success`, `CmResourceTypePort`, ...) — and declined 180
# occurrences that would have been actively WRONG to seal, including the SEH
# control-flow macros `RpcTryExcept`/`RpcEndExcept` (114 of them) and the type
# aliases `LDAPControl`/`LDAPMod`/`IUnknown`.  Anything non-numeric stays declined:
# a missed seal costs one paid call, a sealed control-flow macro corrupts the body.
_NUMERIC_RHS_RE = re.compile(r"^\(?\s*-?\s*(?:0[xX][0-9A-Fa-f]+|\d+)[uUlL]*\s*\)?$")

# Names that are legitimate enum constants but are ALSO overwhelmingly likely to
# be a local variable, parameter, or struct field in real C.  Sealing a VARIABLE
# is the actively harmful direction: the model is told the token is an opaque
# value it must reproduce, when in fact it is storage it needs to read or assign.
# A miss is free (we just pay the model, as today), so anything ambiguous is
# excluded here by policy.  `fact_seal` additionally refuses to seal a name that
# is declared in the body being translated, but that check cannot see a field of
# a struct defined in another file, so this list is the belt to its braces.
AMBIGUOUS_WITH_STORAGE = frozenset(
    {
        "Blocked",
        "Buffer",
        "Count",
        "Data",
        "Default",
        "Device",
        "Disabled",
        "Enabled",
        "Error",
        "Event",
        "Failed",
        "Flags",
        "Handle",
        "Index",
        "Info",
        "Information",
        "Input",
        "Internal",
        "Invalid",
        "Length",
        "Level",
        "Maximum",
        "Minimum",
        "Name",
        "None",
        "Normal",
        "Output",
        "Owner",
        "Path",
        "Pending",
        "Position",
        "Reserved",
        "Result",
        "Running",
        "Size",
        "Started",
        "State",
        "Status",
        "Stopped",
        "Success",
        "Type",
        "Unknown",
        "Value",
        "Version",
    }
)


def collect(repo_root: Path) -> tuple[set[str], int]:
    """Return CamelCase enum-constant names and the number of headers scanned."""
    names: set[str] = set()
    scanned = 0
    for root in HEADER_ROOTS:
        directory = repo_root / root
        if not directory.is_dir():
            # Fail loud: a silently-skipped header root produces a quietly
            # incomplete envelope, which is the whole bug class we are closing.
            raise SystemExit(f"gen_enum_constants: header root missing: {directory}")
        for header in sorted(directory.rglob("*.h")):
            try:
                source = header.read_text(errors="replace")
            except OSError as exc:
                raise SystemExit(f"gen_enum_constants: cannot read {header}: {exc}") from exc
            scanned += 1
            source = re.sub(r"/\*.*?\*/", " ", source, flags=re.S)
            source = re.sub(r"//[^\n]*", " ", source)
            for body in _ENUM_BODY_RE.finditer(source):
                for item in body.group(1).split(","):
                    item = item.strip()
                    if not item:
                        continue
                    match = _LEADING_NAME_RE.match(item)
                    if match is None:
                        continue
                    name = match.group(1)
                    # ALL_CAPS enum members are already covered by fact_seal's
                    # _SYMBOL_RE; only the CamelCase family is new here.
                    if re.fullmatch(r"[A-Z][A-Z0-9_]*", name):
                        continue
                    if not name[0].isupper():
                        continue
                    if name in AMBIGUOUS_WITH_STORAGE:
                        continue
                    names.add(name)
            for define in _DEFINE_RE.finditer(source):
                name = define.group(1)
                replacement = define.group(2).split("/*")[0].split("//")[0].strip()
                if re.fullmatch(r"[A-Z][A-Z0-9_]*", name) or not name[0].isupper():
                    continue
                if name in AMBIGUOUS_WITH_STORAGE:
                    continue
                if _NUMERIC_RHS_RE.match(replacement):
                    names.add(name)
    return names, scanned


def render(names: set[str], scanned: int) -> str:
    lines = [
        '"""CamelCase DDK/PSDK enum-constant names — GENERATED, do not edit by hand.',
        "",
        f"Regenerate with `python3 tools/rustconv/gen_enum_constants.py` ({scanned} headers",
        "scanned).  See that script's docstring for why this exists: CamelCase enum",
        "constants are values the model must never be trusted to copy, and unlike",
        "ALL_CAPS #defines they do not match fact_seal's symbol pattern.",
        '"""',
        "",
        "from __future__ import annotations",
        "",
        "DDK_ENUM_CONSTANTS: frozenset[str] = frozenset(",
        "    {",
    ]
    for name in sorted(names):
        lines.append(f'        "{name}",')
    lines += ["    }", ")", ""]
    return "\n".join(lines)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="exit 1 if the file is stale")
    args = parser.parse_args()

    names, scanned = collect(REPO_ROOT)
    if not names:
        raise SystemExit("gen_enum_constants: found zero enum constants — refusing to write")
    rendered = render(names, scanned)

    if args.check:
        current = OUTPUT.read_text() if OUTPUT.exists() else ""
        if current != rendered:
            print(f"STALE: {OUTPUT} does not match the headers; re-run without --check")
            return 1
        print(f"OK: {len(names)} enum constants, up to date")
        return 0

    OUTPUT.write_text(rendered)
    print(f"wrote {OUTPUT} — {len(names)} CamelCase enum constants from {scanned} headers")
    return 0


if __name__ == "__main__":
    sys.exit(main())
