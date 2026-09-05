#!/usr/bin/env python3
"""ioctlaudit — which device IOCTLs carry a payload that changes shape at 32/64?

wow64.dll marshals the syscall SURFACE (marshalgen, stage 2), but
NtDeviceIoControlFile/NtFsControlFile defeat that by construction: their in/out
buffers are typed by the IoControlCode, not by the prototype. wow64_manual.c
therefore passed them through flat and said so:

    "The in/out buffers are IOCTL-specific and OPAQUE to us ... We can't know
     which from here"

The premise is wrong in a useful way. The code IS knowable — it is argument 5 of
the syscall — so the payload type is derivable, and the same clang record-layout
diff marshalgen already trusts settles whether that payload diverges. This tool
does that derivation for AFD (every socket a 32-bit guest opens) and prints the
authoritative table.

It reads the LIVE driver, which is Rust (10 of 11 afd/*.c are cohab-flipped to
afd-rust; the .c on disk are inert dead source per the never-delete rule). Three
inputs, none of them hand-maintained:

  1. IOCTL codes + transfer method   <- afd-rust/src/main.rs `afd_ctl(op, method)`
  2. code -> handler                 <- afd-rust/src/main.rs dispatch match
  3. handler -> payload struct(s)    <- `as PAFD_<T>` casts in the handler body,
                                        keeping only T that the SHARED header
                                        declares. That header is the wire
                                        contract msafd and afd agree on, so a
                                        type from it is a payload; a type from
                                        the driver's private afd.h (PAFD_FCB,
                                        PAFD_MAPBUF, ...) is driver state and is
                                        correctly ignored.

Layouts come from clang (-fdump-record-layouts-complete) over the real header
stack at both widths, exactly as marshalgen does -- never from reading a header
by eye. Every struct-width bug this project has shipped came from a human
reading a header; none came from a layout diff.

Usage:
    python3 tools/wow64gen/ioctlaudit.py report
    python3 tools/wow64gen/ioctlaudit.py check   # gate: 0 new vs the baseline
"""

from __future__ import annotations

import argparse
import re
import sys
from dataclasses import dataclass
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from marshalgen import (
    REPO,
    Record,
    diff_struct,
    dump_layouts,
    extract_flags,
)

AFD_RUST = REPO / "sdk/lib/rust/afd-rust/src"
SHARED_H = REPO / "sdk/include/reactos/drivers/afd/shared.h"
BASELINE = REPO / "tools/lint-baseline/wow64_ioctl_audit.txt"

# The layout TU: afd.h's own include prologue, minus the driver-private tail.
# The three defines are afd.h's, not ours: they suppress the usermode windows
# headers that <winsock2.h> would otherwise drag into a kernel TU. Drop them and
# minwinbase.h redeclares RTL_CRITICAL_SECTION against ntifs.h.
LAYOUT_TU = """
#include <ntifs.h>
#include <ndk/obtypes.h>
#include <tdi.h>
#include <tcpioctl.h>
#define _WINBASE_
#define _WINDOWS_H
#define _INC_WINDOWS
#include <windef.h>
#include <winsock2.h>
#include <afd/shared.h>
"""

METHOD_NAMES = {
    0: "METHOD_BUFFERED",
    1: "METHOD_IN_DIRECT",
    2: "METHOD_OUT_DIRECT",
    3: "METHOD_NEITHER",
}


# ---------------------------------------------------------------------------
# 1. The wire contract: which struct names does the SHARED header declare?
# ---------------------------------------------------------------------------

_TYPEDEF_STRUCT = re.compile(r"^\s*typedef\s+struct\s+(\w+)?\s*\{", re.M)
_TYPEDEF_TAIL = re.compile(r"^\s*\}\s*([A-Za-z_]\w*)\s*(?:,\s*\*\s*(P[A-Za-z_]\w*))?\s*;", re.M)


def wire_structs(header: Path) -> dict[str, str]:
    """-> {PAFD_FOO: AFD_FOO} for every struct the shared header declares.

    Keyed by the POINTER typedef because that is what the driver casts to."""
    text = header.read_text()
    out: dict[str, str] = {}
    for m in _TYPEDEF_TAIL.finditer(text):
        name, pname = m.group(1), m.group(2)
        if pname:
            out[pname] = name
    return out


# ---------------------------------------------------------------------------
# 2. IOCTL codes and the dispatch, from the LIVE (Rust) driver
# ---------------------------------------------------------------------------

_IOCTL_DEF = re.compile(r"^const (IOCTL_AFD_\w+): u32 = afd_ctl\((\d+),\s*(\d+)\);", re.M)
_DISPATCH = re.compile(r"^\s*(IOCTL_AFD_\w+) =>\s*\{\s*\n\s*return (\w+)\(", re.M)


def parse_ioctls(main_rs: Path) -> dict[str, tuple[int, int]]:
    text = main_rs.read_text()
    return {m.group(1): (int(m.group(2)), int(m.group(3))) for m in _IOCTL_DEF.finditer(text)}


def parse_dispatch(main_rs: Path) -> dict[str, str]:
    text = main_rs.read_text()
    return {m.group(1): m.group(2) for m in _DISPATCH.finditer(text)}


# ---------------------------------------------------------------------------
# 3. handler -> payload struct(s), from `as PAFD_<T>` casts in the body
# ---------------------------------------------------------------------------

_FN_START = re.compile(r"^(?:pub )?(?:unsafe )?(?:extern \"[A-Za-z0-9_-]+\" )?fn (\w+)\s*\(", re.M)


def function_bodies(src_dir: Path) -> dict[str, str]:
    """Crude but adequate: a body runs from its `fn NAME(` to the next `fn ` at
    column 0. Rust in this tree is generated with one function per top level.

    types_*.rs is skipped on purpose: those are generated FFI DECLARATIONS
    (`pub fn AfdSelect(...)` inside an extern block, no body). Letting one match
    would shadow the real body with an empty one and silently report the handler
    as payload-free -- the exact false-negative this audit exists to prevent."""
    bodies: dict[str, str] = {}
    for path in sorted(src_dir.glob("*.rs")):
        if path.name.startswith("types"):
            continue
        text = path.read_text()
        starts = [(m.start(), m.group(1)) for m in _FN_START.finditer(text)]
        for i, (pos, name) in enumerate(starts):
            end = starts[i + 1][0] if i + 1 < len(starts) else len(text)
            bodies[name] = text[pos:end]
    return bodies


def payloads_of(body: str, wire: dict[str, str]) -> list[str]:
    """Wire struct names the body casts an IRP buffer to, in first-seen order."""
    seen: list[str] = []
    for m in re.finditer(r"\bas (P[A-Z][A-Z0-9_]*)\b", body):
        pname = m.group(1)
        struct = wire.get(pname)
        if struct and struct not in seen:
            seen.append(struct)
    # Also the declared-type form: `let X: PAFD_FOO`
    for m in re.finditer(r":\s*(P[A-Z][A-Z0-9_]*)\b", body):
        struct = wire.get(m.group(1))
        if struct and struct not in seen:
            seen.append(struct)
    return seen


# ---------------------------------------------------------------------------
# 4. Report
# ---------------------------------------------------------------------------


@dataclass
class Row:
    ioctl: str
    method: str
    handler: str
    struct: str
    verdict: str
    detail: str
    size32: int
    size64: int

    def line(self) -> str:
        span = f"{self.size32}->{self.size64}" if self.size32 or self.size64 else "?"
        s = f"{self.ioctl} [{self.method}] {self.handler} {self.struct} {self.verdict} {span}"
        return f"{s} ({self.detail})" if self.detail else s


def build_rows() -> list[Row]:
    wire = wire_structs(SHARED_H)
    ioctls = parse_ioctls(AFD_RUST / "main.rs")
    dispatch = parse_dispatch(AFD_RUST / "main.rs")
    bodies = function_bodies(AFD_RUST)

    # Both build dirs must be THIS worktree's. REPO comes from __file__, so there
    # is no sibling fallback to drift into — the verify_signatures incident
    # (a sibling's headers moved a mismatch count 987 -> 9 with no source change)
    # is only avoidable if a missing build dir stops the tool instead of
    # widening the search. Say what to run rather than raising from json.load.
    for arch in ("i386", "amd64"):
        if not (REPO / f"build-{arch}" / "compile_commands.json").is_file():
            sys.exit(
                f"ioctlaudit: no compile_commands.json in {REPO}/build-{arch} — the "
                f"32/64 record layouts come from clang using that build's real include "
                f"flags, and a layout is not guessable from source alone.\n"
                f"fix: ./vibeos-cli build --arch {arch}"
            )

    flags32 = extract_flags(REPO / "build-i386", "tdihelpers/tdi.c")
    flags64 = extract_flags(REPO / "build-amd64", "tdihelpers/tdi.c")
    db32 = dump_layouts(flags32, LAYOUT_TU)
    db64 = dump_layouts(flags64, LAYOUT_TU)

    rows: list[Row] = []
    for ioctl in sorted(ioctls):
        _op, method = ioctls[ioctl]
        handler = dispatch.get(ioctl)
        if not handler:
            rows.append(
                Row(
                    ioctl,
                    METHOD_NAMES.get(method, str(method)),
                    "-",
                    "-",
                    "UNDISPATCHED",
                    "no case in the dispatch match",
                    0,
                    0,
                )
            )
            continue
        body = bodies.get(handler, "")
        structs = payloads_of(body, wire) if body else []
        if not body:
            rows.append(
                Row(
                    ioctl,
                    METHOD_NAMES.get(method, str(method)),
                    handler,
                    "-",
                    "UNRESOLVED",
                    "handler body not found",
                    0,
                    0,
                )
            )
            continue
        if not structs:
            rows.append(
                Row(
                    ioctl,
                    METHOD_NAMES.get(method, str(method)),
                    handler,
                    "-",
                    "NOPAYLOAD",
                    "no wire struct cast in the body",
                    0,
                    0,
                )
            )
            continue
        for struct in structs:
            v = diff_struct(struct, db32, db64)
            r32: Record | None = v.rec32
            r64: Record | None = v.rec64
            rows.append(
                Row(
                    ioctl,
                    METHOD_NAMES.get(method, str(method)),
                    handler,
                    struct,
                    v.cls,
                    v.detail,
                    r32.size if r32 else 0,
                    r64.size if r64 else 0,
                )
            )
    return rows


def main() -> int:
    ap = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    ap.add_argument("mode", choices=("report", "check"), nargs="?", default="report")
    args = ap.parse_args()

    rows = build_rows()
    lines = [r.line() for r in rows]

    if args.mode == "report":
        for line in lines:
            print(line)
        div = [r for r in rows if r.verdict not in ("IDENTICAL", "NOPAYLOAD")]
        print()
        print(
            f"{len(rows)} IOCTL/payload pair(s); "
            f"{len(div)} carry a payload that is NOT width-identical"
        )
        print("A non-IDENTICAL row passed through flat is a guest writing one shape and")
        print("the driver reading another. WIDENING wants a field-copy marshaller;")
        print("SEMANTIC wants a hand-written one (nested arrays whose ELEMENT stride also")
        print("changes, so the header size alone understates it).")
        print()
        print("IDENTICAL is the only verdict that means safe. NOPAYLOAD and UNRESOLVED both")
        print("mean this tool could not SEE a payload -- the handler may take a raw buffer")
        print("it never casts to a wire struct. Neither is evidence of safety.")
        return 0

    if not BASELINE.exists():
        print(f"missing baseline {BASELINE}; run `report` and save it", file=sys.stderr)
        return 1
    base = [ln for ln in BASELINE.read_text().splitlines() if ln.strip() and not ln.startswith("#")]
    new = [ln for ln in lines if ln not in base]
    gone = [ln for ln in base if ln not in lines]
    for ln in new:
        print(f"NEW  {ln}")
    for ln in gone:
        print(f"GONE {ln}")
    if new:
        print(f"\n{len(new)} new IOCTL payload divergence(s); a new row means a payload changed")
        print("shape or a new IOCTL appeared. Marshal it in Wow64pIoControl, or rebaseline")
        print("with a reason.", file=sys.stderr)
        return 1
    if gone:
        print(f"\n{len(gone)} baseline row(s) no longer produced — rebaseline.", file=sys.stderr)
        return 1
    print(f"OK — {len(lines)} IOCTL payload rows, 0 new vs baseline")
    return 0


if __name__ == "__main__":
    sys.exit(main())
