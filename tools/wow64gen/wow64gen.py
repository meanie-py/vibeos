#!/usr/bin/env python3
"""wow64gen — re-runnable WoW64 syscall thunk generator.

Reads a Wine/ReactOS `.spec` file (our single source of truth for the Nt*/Zw*
syscall surface and, crucially, each argument's *type*) and emits the C thunk
table wow64.dll dispatches through: for every syscall, a 32->64 marshaller that

  * reads the guest's 32-bit argument dwords off the guest stack,
  * widens each argument to the host (arm64/amd64) 64-bit ABI, and
  * calls the real 64-bit Nt* (imported from the native ntdll),

returning the NTSTATUS straight back into guest EAX.

WHY A TOOL (and why not bindgen / rustconv):
  * bindgen reads C headers -> Rust *type* declarations.
  * rustconv translates C *bodies* -> Rust via an LLM.
  * wow64gen reads a *table* (the .spec) -> mechanical marshaller codegen.
    It is a spec2def-class tool: deterministic, re-runnable, no LLM. Add a
    syscall to the .spec, re-run wow64gen, and the thunk table regrows. That is
    exactly the "make it easy for ourselves later" property we want as more
    APIs come online.

THE KEY WOW64 SIMPLIFICATION: a 32-bit guest pointer is already a valid 64-bit
pointer (guest VAs live below 4GB), so pointer marshalling is just a
zero-extension. That makes the flat majority of Nt* thunkable by pure codegen.
Arguments that need DEEP marshalling (a 32-bit struct whose *layout* differs
from the 64-bit struct, or one containing embedded pointers/handles/SIZE_T)
cannot be handled by width-widening alone; wow64gen emits those as explicit
`WOW64_NEEDS_MANUAL` stubs so they are impossible to miss, never silently wrong.

Usage:
    python3 tools/wow64gen/wow64gen.py \
        --spec dll/ntdll/def/ntdll.spec \
        --arch arm64 \
        --out-c sdk/lib/rust/.../wow64_thunks.gen.c
"""

from __future__ import annotations

import argparse
import re
import sys
from dataclasses import dataclass


# ---- .spec argument-type vocabulary ---------------------------------------
# Maps a Wine .spec arg token to how many 32-bit guest dwords it occupies and
# how it widens into the host 64-bit ABI. `manual=True` means width-widening is
# NOT sufficient (deep marshalling required) -> emit a loud stub.
@dataclass(frozen=True)
class ArgKind:
    dwords: int  # guest stack slots consumed (4 bytes each)
    c_type: str  # host C parameter type
    widen: str  # C expression template; {s} = first slot index
    manual: bool = False


ARG_KINDS: dict[str, ArgKind] = {
    # Every 1-dword arg is SIGN-extended and passed 64-bit wide. Rationale:
    #  * a callee reading the 32-bit subregister (ULONG/LONG params) sees the
    #    exact low 32 bits either way;
    #  * a callee reading the full 64-bit register is a pointer or HANDLE —
    #    guest pointers live below 2GB (sign == zero extension), and HANDLE
    #    pseudo-values (NtCurrentProcess() == -1, -2, ...) are only correct
    #    sign-extended. Zero-extending (or a 32-bit C param, whose upper reg
    #    bits are UNSPECIFIED in AAPCS64) breaks every pseudo-handle arg —
    #    the .spec types handles as plain 'long', so this must hold for all.
    "long": ArgKind(1, "ULONG_PTR", "(ULONG_PTR)(LONG_PTR)(LONG)a[{s}]"),
    "s_long": ArgKind(1, "LONG_PTR", "(LONG_PTR)(LONG)a[{s}]"),
    "word": ArgKind(1, "ULONG_PTR", "(ULONG_PTR)(USHORT)a[{s}]"),
    "ptr": ArgKind(1, "PVOID", "(PVOID)(LONG_PTR)(LONG)a[{s}]"),
    "str": ArgKind(1, "PVOID", "(PVOID)(LONG_PTR)(LONG)a[{s}]"),
    "wstr": ArgKind(1, "PVOID", "(PVOID)(LONG_PTR)(LONG)a[{s}]"),
    # 64-bit scalar -> combine two guest dwords (little-endian: low then high).
    "int64": ArgKind(2, "ULONGLONG", "((ULONGLONG)a[{s}] | ((ULONGLONG)a[{s1}] << 32))"),
    "double": ArgKind(2, "double", "wow64_bits_to_double(a[{s}], a[{s1}])", manual=False),
    # 128-bit -> genuinely needs a by-ref marshal; flag it.
    "int128": ArgKind(4, "PVOID", "", manual=True),
}


@dataclass
class Syscall:
    name: str
    args: list[str]  # raw .spec arg tokens
    ordinal: int = -1  # assigned service index
    arch: str | None = None  # -arch= filter, or None for all
    manual_reason: str = ""
    slots: int = 0  # total guest dwords consumed

    @property
    def needs_manual(self) -> bool:
        return bool(self.manual_reason)


_SPEC_LINE = re.compile(
    r"^\s*@\s+stdcall\s+"
    r"(?:-arch=(?P<arch>\S+)\s+)?"
    r"(?P<name>[A-Za-z_][A-Za-z0-9_]*)"
    r"\s*\((?P<args>[^)]*)\)"
)

# ntoskrnl/include/sysfuncs.h: `SVC_(AcceptConnectPort, 6)` — THE service-number
# source of truth. The i386 ntdll stubs (ntoskrnl/ntdll.S -> syscalls.inc) and
# every arch's KiServiceTable assign numbers in this file's order, so the guest's
# EAX carries the INDEX INTO THIS LIST — never the .spec order (which drifts:
# 299 spec Nt* vs 296 SVC_ entries at time of writing).
_SYSFUNCS_LINE = re.compile(r"^\s*SVC_\(\s*(?P<name>[A-Za-z_][A-Za-z0-9_]*)\s*,\s*(?P<argc>\d+)\s*\)")


def parse_sysfuncs(path: str) -> list[tuple[str, int]]:
    """Return [(Nt-prefixed name, stack dword count)] in service-number order."""
    out: list[tuple[str, int]] = []
    with open(path, encoding="utf-8") as fh:
        for line in fh:
            m = _SYSFUNCS_LINE.match(line)
            if m:
                out.append(("Nt" + m.group("name"), int(m.group("argc"))))
    return out


def load_manual_list(path: str | None) -> set[str]:
    """Names forced to WOW64_NEEDS_MANUAL regardless of their flat spec types.

    A `ptr` in the .spec is a 32-bit pointer VALUE that zero-extends cleanly —
    but says NOTHING about the pointed-to data. When it points at a struct whose
    32-bit and 64-bit layouts differ (embedded pointers / HANDLE / SIZE_T /
    ULONG_PTR fields — OBJECT_ATTRIBUTES, IO_STATUS_BLOCK.Information, LARGE
    context blocks, ...), passing the guest struct straight through is WRONG.
    Those syscalls need a hand-written deep marshaller; list them here so the
    generator emits a loud stub instead of a silently-broken auto thunk.
    """
    manual: set[str] = set()
    if not path:
        return manual
    with open(path, encoding="utf-8") as fh:
        for line in fh:
            line = line.split("#", 1)[0].strip()
            if line:
                manual.add(line)
    return manual


def parse_spec(path: str, arch: str, manual: set[str] | None = None) -> list[Syscall]:
    """Extract stdcall Nt* entries applicable to `arch`, in file order.

    Zw* aliases are skipped (same syscall number as their Nt* twin). Entries
    gated to a different arch are skipped. Service indices are assigned in the
    order the applicable Nt* entries appear — the same ordering our i386 ntdll
    numbers its stubs, which is what the guest's EAX will carry.
    """
    out: list[Syscall] = []
    with open(path, encoding="utf-8", errors="replace") as fh:
        for line in fh:
            m = _SPEC_LINE.match(line)
            if not m:
                continue
            name = m.group("name")
            if not name.startswith("Nt"):
                continue  # Zw* aliases + non-syscalls handled by their Nt twin
            entry_arch = m.group("arch")
            if entry_arch:
                # A -arch= list is comma-separated; arch applies if listed
                # (a leading '!' negates: -arch=!i386 => all but i386).
                spec_arches = entry_arch.split(",")
                negate = spec_arches and spec_arches[0].startswith("!")
                bare = [x.lstrip("!") for x in spec_arches]
                listed = arch in bare
                if (negate and listed) or (not negate and not listed):
                    continue
            raw = m.group("args").strip()
            args = raw.split() if raw else []
            sc = Syscall(name=name, args=args, arch=entry_arch)
            _classify(sc)
            if manual and name in manual and not sc.needs_manual:
                sc.manual_reason = "listed in --manual-list (struct layout)"
            out.append(sc)
    for i, sc in enumerate(out):
        sc.ordinal = i
    return out


def order_by_sysfuncs(
    sysfuncs: list[tuple[str, int]], spec_syscalls: list[Syscall]
) -> list[Syscall]:
    """Re-key the thunk table into REAL service-number order.

    `sysfuncs` (from parse_sysfuncs) provides the authoritative (name, dword
    count, index) triple; the .spec provides only the argument TYPES. Any
    disagreement is emitted as a loud manual stub, never a silently-misaligned
    thunk:
      * name in sysfuncs but not in the .spec -> no type info -> manual;
      * .spec slot count != sysfuncs dword count -> our type parse (or the
        table) is wrong -> manual.
    """
    by_name = {sc.name: sc for sc in spec_syscalls}
    out: list[Syscall] = []
    for i, (name, argc) in enumerate(sysfuncs):
        sc = by_name.get(name)
        if sc is None:
            sc = Syscall(name=name, args=[], slots=argc)
            sc.manual_reason = "in sysfuncs.h but not in .spec (no type info)"
        elif not sc.needs_manual and sc.slots != argc:
            sc.manual_reason = f"spec arg dwords {sc.slots} != sysfuncs.h argcount {argc}"
            sc.slots = argc
        sc.ordinal = i
        out.append(sc)
    return out


def _classify(sc: Syscall) -> None:
    slots = 0
    for tok in sc.args:
        kind = ARG_KINDS.get(tok)
        if kind is None:
            sc.manual_reason = f"unknown arg token '{tok}'"
            return
        if kind.manual:
            sc.manual_reason = f"arg '{tok}' needs deep marshalling"
        slots += kind.dwords
    sc.slots = slots


# ---- C emitter -------------------------------------------------------------
C_HEADER = """\
/* GENERATED by tools/wow64gen/wow64gen.py — DO NOT EDIT BY HAND.
 * Re-run: python3 tools/wow64gen/wow64gen.py --spec @@SPEC@@ --arch @@ARCH@@ \\
 *             --sysfuncs ntoskrnl/include/sysfuncs.h --out-c @@OUT@@
 *
 * Table index == REAL service number (sysfuncs.h order — what guest EAX
 * carries); the .spec contributes argument types only.
 *
 * WoW64 32->64 syscall thunk table for arch=@@ARCH@@. Each thunk reads the
 * guest's 32-bit argument dwords (Arguments[]) and calls the real 64-bit Nt*.
 * A 32-bit guest pointer is a valid 64-bit pointer (guest VA < 4GB), so
 * pointer args are zero-extended. Deep-marshalling cases are emitted as loud
 * WOW64_NEEDS_MANUAL stubs — implement them by hand in wow64.c and remove the
 * generated stub from the table. Count: @@NAUTO@@ auto, @@NMANUAL@@ manual.
 */

#ifndef WOW64_THUNKS_GEN_H_INCLUDED
/* wow64.c provides: PVOID resolve_ntdll_export(const char *name);
 *                   ULONG DbgPrint(const char *fmt, ...);
 * and the WOW64 basic types (ULONG, PVOID, ULONG_PTR, NTSTATUS, ...). */
#endif

typedef NTSTATUS (*WOW64_THUNK)(ULONG *a);

typedef struct _WOW64_THUNK_ENTRY {
    const char *Name;    /* Nt* export name (resolved lazily from ntdll)      */
    WOW64_THUNK Thunk;   /* marshaller, or 0 for a not-yet-implemented manual */
    unsigned char Slots; /* guest arg dwords consumed (sanity/debug)          */
} WOW64_THUNK_ENTRY;

static inline double wow64_bits_to_double(ULONG lo, ULONG hi)
{
    union { ULONGLONG u; double d; } v;
    v.u = ((ULONGLONG)lo) | (((ULONGLONG)hi) << 32);
    return v.d;
}

"""


def emit_c(syscalls: list[Syscall], spec: str, arch: str, out: str) -> str:
    n_manual = sum(1 for s in syscalls if s.needs_manual)
    n_auto = len(syscalls) - n_manual
    header = (
        C_HEADER.replace("@@SPEC@@", spec)
        .replace("@@ARCH@@", arch)
        .replace("@@OUT@@", out)
        .replace("@@NAUTO@@", str(n_auto))
        .replace("@@NMANUAL@@", str(n_manual))
    )
    buf: list[str] = [header]

    # Per-syscall: a generic extern decl of the 64-bit Nt* + the thunk body.
    for sc in syscalls:
        if sc.needs_manual:
            buf.append(
                f"/* {sc.name}: WOW64_NEEDS_MANUAL — {sc.manual_reason}. "
                f"Implement by hand; table entry left 0. */\n"
            )
            continue
        params = ", ".join(_c_param_type(t) for t in sc.args) or "void"
        buf.append(f"extern NTSTATUS {sc.name}({params});")
        buf.append(f"static NTSTATUS wow64_thunk_{sc.name}(ULONG *a)")
        buf.append("{")
        buf.append(f"    (void)a;\n    return {sc.name}({_c_call_args(sc)});")
        buf.append("}")
        buf.append("")

    # The dispatch table, indexed by service ordinal.
    buf.append("static const WOW64_THUNK_ENTRY Wow64ThunkTable[] = {")
    for sc in syscalls:
        if sc.needs_manual:
            buf.append(
                f'    {{ "{sc.name}", 0, {sc.slots} }}, '
                f"/* [{sc.ordinal}] MANUAL: {sc.manual_reason} */"
            )
        else:
            buf.append(
                f'    {{ "{sc.name}", wow64_thunk_{sc.name}, {sc.slots} }}, /* [{sc.ordinal}] */'
            )
    buf.append("};")
    buf.append("")
    buf.append(
        "static const unsigned Wow64ThunkCount = "
        "sizeof(Wow64ThunkTable) / sizeof(Wow64ThunkTable[0]);"
    )
    buf.append("")
    return "\n".join(buf)


def _c_param_type(tok: str) -> str:
    return ARG_KINDS[tok].c_type


def _c_call_args(sc: Syscall) -> str:
    parts: list[str] = []
    slot = 0
    for tok in sc.args:
        kind = ARG_KINDS[tok]
        expr = kind.widen.format(s=slot, s1=slot + 1)
        parts.append(expr)
        slot += kind.dwords
    return ", ".join(parts)


def main(argv: list[str]) -> int:
    ap = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    ap.add_argument("--spec", required=True, help="path to the .spec file")
    ap.add_argument(
        "--arch",
        required=True,
        choices=["arm64", "amd64", "i386"],
        help="host arch the thunks run on",
    )
    ap.add_argument(
        "--sysfuncs",
        required=True,
        help="path to ntoskrnl/include/sysfuncs.h — the service-number "
        "ordering source of truth (the guest's EAX indexes THIS list)",
    )
    ap.add_argument("--out-c", help="write generated C here (default: stdout)")
    ap.add_argument(
        "--manual-list",
        help="file of Nt* names forced to WOW64_NEEDS_MANUAL "
        "(struct args needing deep marshalling)",
    )
    ap.add_argument("--stats", action="store_true", help="print a coverage summary to stderr")
    args = ap.parse_args(argv)

    manual = load_manual_list(args.manual_list)
    spec_syscalls = parse_spec(args.spec, args.arch, manual)
    if not spec_syscalls:
        print(f"wow64gen: no Nt* syscalls found in {args.spec}", file=sys.stderr)
        return 2
    sysfuncs = parse_sysfuncs(args.sysfuncs)
    if not sysfuncs:
        print(f"wow64gen: no SVC_ entries found in {args.sysfuncs}", file=sys.stderr)
        return 2
    syscalls = order_by_sysfuncs(sysfuncs, spec_syscalls)

    text = emit_c(syscalls, args.spec, args.arch, args.out_c or "-")
    if args.out_c:
        with open(args.out_c, "w", encoding="utf-8") as fh:
            fh.write(text)
    else:
        sys.stdout.write(text)

    if args.stats or args.out_c:
        n_manual = sum(1 for s in syscalls if s.needs_manual)
        print(
            f"wow64gen: {len(syscalls)} syscalls "
            f"({len(syscalls) - n_manual} auto, {n_manual} manual) "
            f"for arch={args.arch}",
            file=sys.stderr,
        )
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
