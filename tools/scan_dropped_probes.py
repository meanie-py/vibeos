#!/usr/bin/env python3
"""
scan_dropped_probes.py — the C validated the user pointer. The translation kept
that validation as a comment and dereferenced the pointer anyway.

THE POINT. Same oracle as `scan_dropped_prints`, opposite stakes. rustconv keeps
the original C as trailing comments in the same `.rs`, so a function whose
commented C probes a user-mode buffer, and whose live Rust reaches the same
buffer with no validation, has stated its own safety requirement and dropped it —
visible in one file, with no header catalog, no C build, no cross-file reasoning.

WHY THIS IS NOT THE PRINTS SCANNER AGAIN. A dropped print costs a debugging
cycle. A dropped probe is a SECURITY BUG, and under DUMMY_PSEH it is not even a
contained one. `NtSetValueKey` (task #79) is the specimen: the C probes `Data`
with `ProbeForRead(Data, DataSize, 1)` inside one `_SEH2_TRY` and copies it
inside a second, and BOTH regions were dropped. What shipped was a bare
`Status = STATUS_SUCCESS;` where the probe belongs, followed by
`copy_nonoverlapping` straight off the caller's raw pointer — which also left
both `if (!NT_SUCCESS(Status))` arms below it unreachable, checking a status
nothing could set. Unprivileged callers reach it directly: a bad `Data` faults
with no SEH to catch it and takes the box down, and a `Data` pointing INTO
KERNEL SPACE is copied into a registry value the same caller reads back with
`NtQueryValueKey`. That is an arbitrary kernel-memory disclosure reachable from
user mode, and it sat in the tree looking like ordinary code.

WHY NOTHING ELSE SEES THIS CLASS. It is an ABSENCE, not a shape.
`scan_silent_failures` matches `loop {}`, `.unwrap()`, `Err(_) => {}` — things
that are present and wrong. A function that copies from a raw pointer is
*correct code* in any other context; only the commented C saying "I probed this
first" makes it a bug. rustc cannot see it either: the raw-pointer deref is
`unsafe` and both spellings compile. No build, no boot, and no fuzzer positioned
against these entry points would report it.

DUMMY_PSEH IS WHY DROPPING IT IS FATAL RATHER THAN MERELY WRONG. Upstream's
probe raises a software exception the surrounding `_SEH2_TRY` catches, turning a
bad pointer into `STATUS_ACCESS_VIOLATION`. We build with DUMMY_PSEH, so there
is nothing to catch anything: the correct Rust shape is VALIDATE-FIRST — check
the range, then act, returning `Err(NTSTATUS)` — per
`docs/engineering/dummy-pseh-verdict.md`. That is what `ntkrnl_sys::probe`
provides and what the live vocabulary below is drawn from.

WHAT COUNTS AS VALIDATION on the live side, and why the list is this wide. Any
of these means the pointer was checked before use:

  * the raw upstream API kept as FFI — `ProbeForRead(`, `ProbeForWriteUlong(`,
    and the tree's `Safe*`/`_rs_*`/`w32r_*`/`Csr*` wrappers around it;
  * the Rust substrate — `probe_user_buffer`, `probe_read_user`,
    `probe_write_user`, `write_user`, `safe_copy_from_user`, and friends;
  * `ProbeAndCapture*`, which validates AND copies into pool, so a callee
    receiving the captured copy needs no probe of its own;
  * `MmProbeAndLock*`, the MDL path;
  * a hand-rolled range check against `mm_user_probe_address`. That is the
    genuine article written out longhand, and counting it is what keeps the
    scanner from demanding a specific spelling of a correct thing.

DELIBERATELY NOT COUNTED, because the word "probe" means something else there:
hardware enumeration (`BusLogic_ProbeHostAdapter`, `acpi_ec_ecdt_probe`), DLL
manifest lookup (`LdrSetDllManifestProber`, `BasepProbeForDllManifest`), and
local helpers that merely have the substring (`printf_probe`, `strtod_probe`,
`probe_entry_lr`). Miscounting one of those as a probe would silence a real
finding in the same file, which is the expensive direction of the error.

PAIRING is by function NAME, exact first and case-insensitively second, and only
when BOTH sides exist — identical to `scan_dropped_prints`, including the
`_impl`/`_body` suffix folding, because translation routinely splits one C
function into an exported wrapper plus the real body.

TWO KINDS.

  ALL      the C probes and the live function never validates anything. This is
           the high-confidence kind and the one to burn down first.
  PARTIAL  live validates, but fewer times than the C does. ADVISORY — read it
           before believing it. One legitimate cause dominates: the C probes a
           structure field by field where the Rust probes the whole buffer once,
           so 4 becomes 1 with nothing lost. Kept anyway, because it is also the
           shape of a function that probes the header and forgets the body.

FALSE POSITIVES ARE EXPECTED AND ARE THE POINT OF THE BASELINE. The commonest is
a callee whose caller already captured the buffer into pool — `CmpSetValueKeyNew`
is exactly that, its `ValueName` arriving as kernel memory from
`ProbeAndCaptureUnicodeString` one frame up. That is a real answer, not a dodge,
but it can only be settled by FOLLOWING THE POINTER to its caller. Do that, then
suppress on the signature line with the reason:

    unsafe fn Foo(..) {  // nolint: dropped-probe (caller captures into pool)

Usage:
    python3 tools/scan_dropped_probes.py                 # full report
    python3 tools/scan_dropped_probes.py --lint          # terse (CLI baseline gate)
    python3 tools/scan_dropped_probes.py --self-test     # prove the scanner can SEE
    python3 tools/scan_dropped_probes.py <file.rs ...>   # just these files
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from scan_dropped_prints import (
    GENERATED_RE,
    REPO,
    base_names,
    c_function_bodies,
    rust_files,
    rust_function_items,
    split_regions,
)

NOLINT_RE = re.compile(r"\bnolint\b\s*[:(\[]?\s*([a-z0-9_-]*)", re.I)
NOLINT_IDS = {"", "all", "dropped-probe", "dropped_probe", "scan_dropped_probes"}

# Identifiers that VALIDATE a user pointer, on either side of the oracle. Ordered
# longest-alternative-first where prefixes overlap so the regex cannot match a
# short alternative and strand the rest of the name.
PROBE_ALTS = (
    r"[A-Za-z0-9_]*ProbeFor[A-Za-z0-9_]*",
    r"[A-Za-z0-9_]*ProbeAndCapture[A-Za-z0-9_]*",
    r"[A-Za-z0-9_]*ProbeAndLock[A-Za-z0-9_]*",
    r"ProbeArrayFor[A-Za-z0-9_]*",
    r"ProbeAndZeroHandle",
    r"probe_user_buffer",
    r"probe_read_user",
    r"probe_write_user",
    r"probe_read_ulong",
    r"probe_write_ulong",
    r"probe_write_handle",
    r"probe_write_boolean",
    r"probe_write_size_t",
    r"probe_read_generic",
    r"probe_write_generic",
    r"probe_capture_unicode_string_header",
    r"write_user_nofail",
    r"write_user",
    r"safe_copy_from_user",
    r"safe_copy_to_user",
    r"safe_copy_raw",
    r"mm_user_probe_address",
    # win32k-rust/src/safe_user_io.rs: the Result probes win32k uses in place of
    # ntkrnl_sys::probe (a separate .sys; same range/alignment arithmetic).
    r"probe_for_read",
    r"probe_for_write",
    r"copy_from_user",
    r"copy_to_user",
    r"capture_user",
    r"probe_and_capture_unicode_string",
    # dbgk-rust DbgkpPostFakeModuleMessages: a local macro over probe_user_buffer.
    r"probe_or_done!",
)
# The trailing `[({]` is what makes this a CALL rather than a mention — `ProbeMode`,
# `ProbeFlags` and `probe_size` are all variables in this tree and must not count. The
# optional turbofish between name and paren is not decoration: the substrate API is
# generic, so `probe_read_user::<u32>(h)` is the ordinary spelling, and without this the
# scanner scored every generic probe as absent and reported its function as ALL.
PROBE_RE = re.compile(
    r"(?<![A-Za-z0-9_])(?:" + "|".join(PROBE_ALTS) + r")\s*(?:::\s*<[^;{}()]*>\s*)?[({]",
)

# Names the regex above matches but that do not validate a user pointer. Each is
# the "probe" of a different domain; see the docstring.
NOT_A_PROBE_RE = re.compile(
    r"(?<![A-Za-z0-9_])(?:BasepProbeForDllManifest|LdrpManifestProberRoutine"
    r"|LdrSetDllManifestProber)\s*\(",
)

# Cheap byte prefilter: a file whose commented C never probes has nothing to compare.
PREFILTER_RE = re.compile(rb"Probe(?:For|And|Array)")


def count_probes(text: str) -> int:
    """Validation calls in a span, with the other senses of "probe" removed."""
    return len(PROBE_RE.findall(text)) - len(NOT_A_PROBE_RE.findall(text))


def suppressed(text: str) -> bool:
    return any(m.group(1).lower() in NOLINT_IDS for m in NOLINT_RE.finditer(text))


def rust_probe_counts(src: bytes, live: str) -> dict[str, tuple[int, int, int]]:
    """{fn name: (validation count, decl_start, body_end)}, folded over base names."""
    out: dict[str, tuple[int, int, int]] = {}
    for name, node, body in rust_function_items(src):
        n = count_probes(live[body.start_byte : body.end_byte])
        for key in base_names(name):
            prev = out.get(key)
            if prev is None:
                out[key] = (n, node.start_byte, body.end_byte)
            else:
                out[key] = (
                    prev[0] + n,
                    min(prev[1], node.start_byte),
                    max(prev[2], body.end_byte),
                )
    return out


def scan_file(path: Path, rel: str) -> list[tuple[str, str, str, str]]:
    """[(rel, fn name, kind, detail)] for one translated file."""
    src = path.read_bytes()
    if not PREFILTER_RE.search(src):
        return []
    text = src.decode("utf8", "replace")
    if GENERATED_RE.search(text[:4096]):
        return []

    live, craw = split_regions(src)

    cfuncs: dict[str, int] = {}
    for name, body in c_function_bodies(craw):
        cfuncs[name] = cfuncs.get(name, 0) + count_probes(body)
    if not any(v > 0 for v in cfuncs.values()):
        return []

    rfuncs = rust_probe_counts(src, live)

    # Exact name first; a case-insensitive fallback for translations that
    # re-cased, but only when it is unambiguous on both sides.
    ci_r: dict[str, list[str]] = {}
    for k in rfuncs:
        ci_r.setdefault(k.lower(), []).append(k)
    ci_c: dict[str, list[str]] = {}
    for k in cfuncs:
        ci_c.setdefault(k.lower(), []).append(k)

    out = []
    for cname, cn in sorted(cfuncs.items()):
        if cn <= 0:
            continue
        rname = cname
        if rname not in rfuncs:
            cand = ci_r.get(cname.lower(), [])
            if len(cand) != 1 or len(ci_c.get(cname.lower(), [])) != 1:
                continue
            rname = cand[0]
        rn, decl, end = rfuncs[rname]
        if rn >= cn:
            continue
        if suppressed(text[max(0, decl - 240) : end]):
            continue
        kind = "ALL" if rn == 0 else "PARTIAL"
        detail = f"C validates {cn}x, live {rn}x"
        out.append((rel, cname, kind, detail))
    return out


# Each case is a real shape from the tree: the source, and the findings it must
# produce. A scanner that cannot be shown to SEE is worth nothing as a gate.
SELF_TEST: list[tuple[str, str, list[str]]] = [
    (
        "the #79 specimen: both SEH regions dropped, copy left unguarded",
        """
pub unsafe fn NtSetValueKey(Data: PVOID, DataSize: ULONG) -> NTSTATUS {
    let mut Status = STATUS_SUCCESS;
    core::ptr::copy_nonoverlapping(Data as *const u8, DataCopy as *mut u8, DataSize as usize);
    Status
}
// NTSTATUS NtSetValueKey(PVOID Data, ULONG DataSize)
// {
//     _SEH2_TRY {
//         ProbeForRead(Data, DataSize, 1);
//     } _SEH2_END;
//     RtlCopyMemory(DataCopy, Data, DataSize);
// }
""",
        ["NtSetValueKey ALL"],
    ),
    (
        "validation restored via the Rust substrate clears the finding",
        """
pub unsafe fn NtSetValueKey(Data: PVOID, DataSize: ULONG) -> NTSTATUS {
    probe_user_buffer(Data as *const c_void, DataSize as usize, 1, PreviousMode)?;
    core::ptr::copy_nonoverlapping(Data as *const u8, DataCopy as *mut u8, DataSize as usize);
    STATUS_SUCCESS
}
// NTSTATUS NtSetValueKey(PVOID Data, ULONG DataSize)
// {
//     ProbeForRead(Data, DataSize, 1);
//     RtlCopyMemory(DataCopy, Data, DataSize);
// }
""",
        [],
    ),
    (
        "the raw FFI spelling counts as validation just as the wrapper does",
        """
pub unsafe fn NtFoo(p: PVOID) -> NTSTATUS {
    ProbeForWrite(p, 4, 4);
    STATUS_SUCCESS
}
// NTSTATUS NtFoo(PVOID p) { ProbeForWrite(p, 4, 4); return 0; }
""",
        [],
    ),
    (
        "a hand-rolled range check is the genuine article, not a missing probe",
        """
pub unsafe fn NtBar(p: PVOID) -> NTSTATUS {
    if (p as usize) >= mm_user_probe_address() { return STATUS_ACCESS_VIOLATION; }
    STATUS_SUCCESS
}
// NTSTATUS NtBar(PVOID p) { ProbeForRead(p, 4, 4); return 0; }
""",
        [],
    ),
    (
        "probes the header and forgets the body — the PARTIAL that matters",
        """
pub unsafe fn NtBaz(h: PVOID, b: PVOID) -> NTSTATUS {
    probe_read_user::<u32>(h)?;
    core::ptr::copy_nonoverlapping(b as *const u8, out, n);
    STATUS_SUCCESS
}
// NTSTATUS NtBaz(PVOID h, PVOID b)
// {
//     ProbeForRead(h, 4, 4);
//     ProbeForRead(b, n, 1);
//     return 0;
// }
""",
        ["NtBaz PARTIAL"],
    ),
    (
        "hardware enumeration is a different sense of the word and must not report",
        """
pub unsafe fn AdapterInit() -> BOOLEAN { probe_entry_lr(); TRUE }
// BOOLEAN AdapterInit(void) { BusLogic_ProbeHostAdapter(a); return TRUE; }
""",
        [],
    ),
    (
        "the caller-captures answer, verified and suppressed",
        """
pub unsafe fn CmpSetValueKeyNew(n: PUNICODE_STRING) -> NTSTATUS {  // nolint: dropped-probe
    CmpCopyName(n);
    STATUS_SUCCESS
}
// NTSTATUS CmpSetValueKeyNew(PUNICODE_STRING n)
// {
//     _SEH2_TRY { ProbeForRead(n, 8, 1); CmpCopyName(n); } _SEH2_END;
// }
""",
        [],
    ),
    (
        "the wrapper/_impl split pairs to one function, not two findings",
        """
pub unsafe fn NtQux(p: PVOID) -> NTSTATUS { NtQux_impl(p) }
unsafe fn NtQux_impl(p: PVOID) -> NTSTATUS {
    probe_user_buffer(p, 4, 1, mode)?;
    STATUS_SUCCESS
}
// NTSTATUS NtQux(PVOID p) { ProbeForRead(p, 4, 1); return 0; }
""",
        [],
    ),
    (
        "ProbeAndCapture validates and copies, so the callee needs nothing more",
        """
pub unsafe fn NtOpenThing(oa: POBJECT_ATTRIBUTES) -> NTSTATUS {
    ProbeAndCaptureObjectAttributes(oa)?;
    STATUS_SUCCESS
}
// NTSTATUS NtOpenThing(POBJECT_ATTRIBUTES oa)
// { ProbeAndCaptureObjectAttributes(oa); return 0; }
""",
        [],
    ),
    (
        "a C function with no live counterpart is untranslated, not a finding",
        """
pub unsafe fn Unrelated() -> NTSTATUS { STATUS_SUCCESS }
// NTSTATUS SomeOtherFn(PVOID p) { ProbeForRead(p, 4, 1); return 0; }
""",
        [],
    ),
]


def self_test() -> int:
    import tempfile

    ok = 0
    for desc, src, want in SELF_TEST:
        with tempfile.TemporaryDirectory() as td:
            p = Path(td) / "t.rs"
            p.write_text(src)
            got = sorted(f"{n} {k}" for _, n, k, _ in scan_file(p, "t.rs"))
        if got == sorted(want):
            ok += 1
        else:
            print(f"FAIL: {desc}\n  want {sorted(want)}\n  got  {got}")
    print(f"self-test: {ok}/{len(SELF_TEST)}")
    return 0 if ok == len(SELF_TEST) else 1


def main() -> int:
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    lint = "--lint" in sys.argv
    if "--self-test" in sys.argv:
        return self_test()

    paths = [Path(a).resolve() for a in args] if args else rust_files()

    hits = []
    for p in paths:
        try:
            rel = p.relative_to(REPO).as_posix()
        except ValueError:
            rel = p.as_posix()
        hits.extend(scan_file(p, rel))

    hits.sort()
    for rel, name, kind, detail in hits:
        suffix = "" if lint else f"  ({detail})"
        print(f"{rel}: {name} [{kind}]{suffix}")
    if not lint and hits:
        by_kind: dict[str, int] = {}
        for h in hits:
            by_kind[h[2]] = by_kind.get(h[2], 0) + 1
        tally = ", ".join(f"{v} {k}" for k, v in sorted(by_kind.items()))
        print(f"\n{len(hits)} functions ({tally})")
    return 1 if hits else 0


if __name__ == "__main__":
    sys.exit(main())
