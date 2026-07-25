"""Failure triage (plan §14, §20.1). Cheap tier today: classify from serial +
debugcon, surface RUST_PANIC file:line and user-mode panics, match a known-flake
catalog, symbolicate via the offline index. Full gdb attach is P2 (smoke's
QemuVM doesn't expose a gdbstub yet); this never blocks and degrades gracefully.
"""

from __future__ import annotations

import re
import subprocess
from pathlib import Path

# Matches every banner spelling in the tree: x86 `BugCheck 0xNN`, the Rust/arm64
# `*** BUGCHECK: 0xNN`, and `*** STOP: 0xNN`. The arm64 spelling was unmatched
# until 2026-07-04 — a real 0xF4 classified as HANG_TIMEOUT (which the old gate
# policy would then have auto-retried as a "flake").
_BUGCHECK = re.compile(
    r"BugCheck\s+(0x[0-9A-Fa-f]+)|\*\*\*\s*(?:STOP|BUGCHECK):?\s*(0x[0-9A-Fa-f]+)"
)
_RUST_PANIC = re.compile(r"RUST PANIC HIT.*?PANIC LOC:\s*(\S+)", re.S)
_RUST_PANIC2 = re.compile(r"panicked at\s+'([^']*)',\s*(\S+)")
# Kernel-mode Rust panic: the kernel-rust/rtl-rust panic handlers emit
# `PANIC:<file>:<line>` to debugcon (port 0xE9) ONLY — serial goes silent, so
# without this a deterministic kernel panic classifies as HANG_TIMEOUT (and a
# HANG_TIMEOUT can be gate-retried as if it were host contention). Matched
# against the RAW debugcon bytes, not the decoded milestone report (the report
# truncates its Raw: line, which can cut the panic string in half).
_KERNEL_PANIC = re.compile(r"PANIC:([^\s:]+):(\d+)")
# Wedge self-reports from the in-kernel sync instrumentation (census watchdog,
# ke/census.rs; lockdep Phase A+B, ke/lockdep.rs). These print to SERIAL on any
# boot where threads are stuck >10s in a gate wait — evidence that a timeout is
# a genuine wedge, not a slow/contended boot, and on green boots a trendable
# health signal (e.g. the Mm WS self-deadlock strands eventlog every boot).
# Lockdep violation records are `[LOCKDEP]   SELF-DEADLOCK #n …` (Phase A) and
# `[LOCKDEP]   ABBA #n …` (Phase B); header/reverse-order lines are context,
# not counted. Lockdep is feature-gated (build --lockdep), default OFF.
_CENSUS_STUCK = re.compile(r"\[CENSUS\] STUCK\b")
_LOCKDEP_HIT = re.compile(r"\[LOCKDEP\][^\n]*(?:SELF-DEADLOCK|ABBA)")
# cswitchdep (ke/cswitchdep.rs, build --cswitchdep): context-switch hand-off
# violations, loud (`[CSWITCH] KIND …`) or census (`[CSWITCH]   #n KIND …`).
# The plain `[CSWITCH] census:` counters line is context, not counted.
_CSWITCH_HIT = re.compile(
    r"\[CSWITCH\][^\n]*(?:HIGH_IRQL_AT_SWITCH|IRQL_UNBALANCED_SWITCH|SWAP_IRQL"
    r"|SWITCH_FROM_DPC|CURTHREAD_INCOHERENT|STACK_OOB)"
)
_PC = re.compile(r"\bPC[:=]\s*(0x[0-9A-Fa-f]+)|\b(?:RIP|EIP)[:=]\s*(0x[0-9A-Fa-f]+)")

# Documented intermittent flakes single-run gates have masked (see
# docs/engineering/deterministic-builds-plan.md). Matching one re-labels a
# failure KNOWN_FLAKE so a *documented* flake is instantly told apart from a
# fresh regression (an unmatched/unknown red still classifies and blocks).
# Each entry: `when` ("bugcheck"|"hang", default "bugcheck"), optional `code`,
# `any` (all substrings present), `none` (all substrings ABSENT). Signatures are
# specific; RETIRE an entry (delete it) once its root fix lands and a re-gate
# confirms the signature no longer appears (verify-gone).
KNOWN_FLAKES = [
    # RETIRED 2026-07-03: `tWinMain_0xF4_race` (0xF4 after `[EXPLORER] _tWinMain`).
    # Did not reproduce in 36 desktop-reaching windows-host TCG boots (sequential +
    # 3-wide contended) — under heavy contention boots die EARLIER at the
    # service-startup storage stall / nettest handoff (separately-tracked flakes:
    # nettest_explorer_race + the amd64-net EventLog/DcomLaunch stall), before ever
    # reaching the post-explorer window this 0xF4 lived in. Gating the nettest spawn
    # (now landed, see below) removes that shared explorer-handoff disruptor. With no
    # live repro to root-cause and the signature verify-gone-by-absence, the entry is
    # removed so any recurrence BLOCKS as a fresh BUGCHECK 0xF4 (we WANT to catch it)
    # rather than being silently auto-retried. History: field-manual §"0xF4(0x3) at
    # _tWinMain entered". Re-add a signature here only if a real recurrence is caught.
    #
    # RETIRED 2026-07-03 — nettest_explorer_race: winlogon spawned nettest.exe
    # unconditionally every boot and raced explorer's csrss/win32k handoff
    # (~1/3 i386 boots hung; arm64 variant too). Root fix landed: netstack.sys
    # publishes NicPresent and winlogon gates the spawn on it, so plain smokes
    # (no NIC) never spawn nettest — the marker "[WINLOGON] nettest.exe created"
    # can no longer appear on a plain boot, so the signature is unreachable.
    # Verify-gone: i386 smoke ×8 = 8/8 green; cold gate ×3 all-arch green.
    #
    # RETIRED 2026-07-03 — arm64_lpcexitthread_race. The signature ("LpcExitThread"
    # in serial, no _tWinMain) was a broad catch-all — that UART trace fires on
    # every arm64 system-thread exit, so it matched any arm64 hang after an early
    # thread exit. Its deterministic root causes were fixed earlier (fastfat
    # FILE_CORRUPT constants 65c82a286a0, 0xC2 double-free fddff701ac9, fastfat
    # recursion 0x50 3b3f9744c0a); the residual intermittent exit-path hang was
    # arm64's per-thread kernel stack being mis-sized at i386's 12KB while the
    # 64-bit sibling amd64 uses 48KB (deep Rust stacks + no arm64 exception stack
    # + no GUI large-stack switch on arm64 → intermittent guard-page overflow on
    # the exit teardown). Fixed by bumping arm64 KERNEL_STACK_SIZE to 0xC000
    # (xdk/arm64/ke.h + ARM3/helpers.rs + the stack-bounds sites). An arm64 exit
    # hang now classifies HANG_TIMEOUT (still gate-retryable) — the honest signal,
    # no longer masked as an expected flake.
    #
    # arm64 ERESOURCE release-not-owned (thread-teardown family): the rare
    # non-deterministic 0x3 — KeBugCheckEx(0x03, Resource, Thread, OwnerTable, 3)
    # from ex-rust/src/resource.rs (owner-table scan miss on a SHARED resource;
    # both live hits had OwnerTable == NULL, hence the P3=0 in the signature),
    # firing around thread-exit rundown (`PspExitThread: about to call
    # LpcExitThread` context) under load. Documented: roadmap "Known bugs &
    # flakes" 2026-07-18 + the 2026-07-16 conformance-quest 0x3 entry (same
    # param shape, pre-dating this entry by two days). Density measured
    # 2026-07-18: 2/2 under 3-arch-parallel gate load, 0/3 solo arm64 —
    # load-sensitive, and the raiser predates the tree that first showed it in
    # a gate (the crates rebuilt that night are user-mode; no WoW64 guest had
    # started in either hit). RETIRE on the ERESOURCE root fix (verify-gone).
    {
        "name": "arm64_eresource_notowned_0x3",
        "when": "bugcheck",
        "code": "0x00000003",
        "any": ["0x0000000000000000, 0x0000000000000003)"],
    },
]


def _match_known_flake(serial: str, code: str | None, when: str) -> dict | None:
    for fl in KNOWN_FLAKES:
        if fl.get("when", "bugcheck") != when:
            continue
        # entry keyed on a bugcheck code → require a matching code
        if fl.get("code") and (not code or code.lower() != fl["code"].lower()):
            continue
        if not all(s in serial for s in fl.get("any", [])):
            continue
        if any(s in serial for s in fl.get("none", [])):
            continue
        return fl
    return None


def wedge_evidence(serial: str) -> dict:
    """Count sync-instrumentation self-reports in serial. Returns
    {census_stuck, lockdep, lines} — lines carries the first few raw reports
    for the summary. Zero-cost when the instrumentation isn't in the build."""
    lines = [
        ln.strip()
        for ln in serial.splitlines()
        if _CENSUS_STUCK.search(ln) or _LOCKDEP_HIT.search(ln) or _CSWITCH_HIT.search(ln)
    ]
    return {
        "census_stuck": sum(1 for ln in lines if _CENSUS_STUCK.search(ln)),
        "lockdep": sum(1 for ln in lines if _LOCKDEP_HIT.search(ln)),
        "cswitch": sum(1 for ln in lines if _CSWITCH_HIT.search(ln)),
        "lines": lines[:3],
    }


def post_boot_check(serial: str, debugcon_raw: str | None = None) -> dict | None:
    """Scan a boot that REACHED its marker (including the settle-window serial)
    for fatal signals the marker race can hide: a BugCheck or Rust panic that
    fired after the marker, or a kernel panic that only reached debugcon.
    Returns a triage-shaped failure dict, or None if nothing fatal was seen.
    (Without this, a crash during taskbar build / desktop settle is a false
    green — the marker already matched before the crash.)"""
    salient: list[str] = []
    fields: dict = {}
    mp = _RUST_PANIC.search(serial) or _RUST_PANIC2.search(serial)
    if mp:
        loc = mp.groups()[-1]
        return {
            "classification": "RUST_PANIC",
            "salient": [f"RUST PANIC at {loc} (after boot marker)"],
            "known_flake": None,
            "panic_loc": loc,
        }
    kp = _KERNEL_PANIC.search(debugcon_raw or "")
    if kp:
        loc = f"{kp.group(1)}:{kp.group(2)}"
        return {
            "classification": "RUST_PANIC",
            "salient": [f"KERNEL RUST PANIC at {loc} (debugcon, after boot marker)"],
            "known_flake": None,
            "panic_loc": loc,
        }
    mb = _BUGCHECK.search(serial)
    if mb:
        code = mb.group(1) or mb.group(2)
        fields["bugcheck"] = code
        salient.append(f"BugCheck {code} (after boot marker)")
        line = next((ln.strip() for ln in serial.splitlines() if "BugCheck" in ln), "")
        if line:
            salient.append(line[:140])
        return {"classification": "BUGCHECK", "salient": salient, "known_flake": None, **fields}
    return None


def classify(
    serial: str,
    *,
    reached: bool,
    fail_marker: str | None,
    debugcon: str | None,
    debugcon_raw: str | None = None,
) -> dict:
    """Return {classification, salient[], known_flake, fields...}."""
    salient: list[str] = []
    fields: dict = {}

    mp = _RUST_PANIC.search(serial) or _RUST_PANIC2.search(serial)
    if mp:
        loc = mp.groups()[-1]
        fields["panic_loc"] = loc
        salient.append(f"RUST PANIC at {loc}")
        return {"classification": "RUST_PANIC", "salient": salient, "known_flake": None, **fields}

    # Kernel-mode panic: reaches debugcon only (x86; silent on arm64). Checked
    # before the hang branch so a deterministic kernel panic classifies as
    # RUST_PANIC (never retried) instead of HANG_TIMEOUT (retryable).
    kp = _KERNEL_PANIC.search(debugcon_raw or "")
    if kp:
        loc = f"{kp.group(1)}:{kp.group(2)}"
        fields["panic_loc"] = loc
        salient.append(f"KERNEL RUST PANIC at {loc} (debugcon)")
        return {"classification": "RUST_PANIC", "salient": salient, "known_flake": None, **fields}

    mb = _BUGCHECK.search(serial)
    code = None
    if mb:
        code = mb.group(1) or mb.group(2)
        fields["bugcheck"] = code
        salient.append(f"BugCheck {code}")
        line = next((ln.strip() for ln in serial.splitlines() if "BugCheck" in ln), "")
        if line:
            salient.append(line[:140])
        flake = _match_known_flake(serial, code, "bugcheck")
        return {
            "classification": "KNOWN_FLAKE" if flake else "BUGCHECK",
            "salient": salient + ([flake["note"]] if flake else []),
            "known_flake": flake["name"] if flake else None,
            **fields,
        }

    if fail_marker:
        salient.append(f"hit fail marker: {fail_marker}")
        return {"classification": "FAIL_MARKER", "salient": salient, "known_flake": None, **fields}

    if not reached:
        # hang/timeout — surface the last debugcon stage marker for pre-serial deaths
        if debugcon:
            tail = [ln for ln in debugcon.splitlines() if ln.strip()][-3:]
            salient += [f"debugcon: {ln}" for ln in tail]
        last = [ln for ln in serial.splitlines() if ln.strip()][-2:]
        salient += [f"serial tail: {ln[:120]}" for ln in last]
        # Census/lockdep self-reports turn "timed out" into "wedge, self-confirmed
        # by the kernel" — a real regression signature, never a contention retry.
        ev = wedge_evidence(serial)
        if ev["census_stuck"] or ev["lockdep"] or ev["cswitch"]:
            fields["wedge_confirmed"] = True
            fields["census_stuck"] = ev["census_stuck"]
            fields["lockdep_hits"] = ev["lockdep"]
            fields["cswitch_hits"] = ev["cswitch"]
            salient.insert(
                0,
                f"kernel self-report: {ev['census_stuck']} [CENSUS] STUCK, "
                f"{ev['lockdep']} [LOCKDEP], {ev['cswitch']} [CSWITCH] — "
                f"wedge CONFIRMED, not contention",
            )
            salient += ev["lines"][:2]
        # A hang whose serial matches a documented flake signature is labeled
        # KNOWN_FLAKE (told apart from a fresh regression); an unmatched hang
        # stays HANG_TIMEOUT and blocks as before.
        flake = _match_known_flake(serial, None, "hang")
        return {
            "classification": "KNOWN_FLAKE" if flake else "HANG_TIMEOUT",
            "salient": salient + ([flake["note"]] if flake else []),
            "known_flake": flake["name"] if flake else None,
            **fields,
        }

    return {"classification": "UNKNOWN", "salient": salient, "known_flake": None}


# `*** BUGCHECK: 0xNN (0xP1, 0xP2, 0xP3, 0xP4)` — the authoritative crash record.
_BUGCHECK_PARAMS = re.compile(
    r"\*\*\*\s*BUGCHECK:\s*(0x[0-9A-Fa-f]+)\s*\("
    r"\s*(0x[0-9A-Fa-f]+)\s*,\s*(0x[0-9A-Fa-f]+)\s*,"
    r"\s*(0x[0-9A-Fa-f]+)\s*,\s*(0x[0-9A-Fa-f]+)\s*\)"
)

# Which bugcheck parameter holds the faulting INSTRUCTION pointer, 1-based.
# Values are from the documented stop-code contracts, not guessed:
#   0x1E KMODE_EXCEPTION_NOT_HANDLED    P2 = address the exception occurred at
#   0x50 PAGE_FAULT_IN_NONPAGED_AREA    P3 = address that referenced memory
#   0x8E KERNEL_MODE_EXCEPTION_NOT_HDLD P3 = address the exception occurred at
#   0x7E SYSTEM_THREAD_EXCEPTION        P3 = exception-record address (best effort)
#   0x0A IRQL_NOT_LESS_OR_EQUAL         P4 = address that referenced memory
_PC_PARAM_BY_CODE = {0x1E: 2, 0x50: 3, 0x8E: 3, 0x7E: 3, 0x0A: 4}

# No function is a megabyte long. A symbol match reported at a larger offset is
# not a near miss, it is the index's last symbol matching an address that lies
# outside every known range — which is exactly how this returned
# `KTRAP_FRAME_LENGTH+8794868291436` for an arm64 fault and sent the hunt to the
# wrong subsystem (§9.124). Refuse rather than answer.
_MAX_SANE_SYMBOL_OFFSET = 0x100000
_SYM_OFFSET = re.compile(r"\+(\d+)\b")

# addr2sym's index holds FILE VAs; a bugcheck reports RUNTIME VAs. The difference
# is `kernel_runtime_base - ImageBase`, and without it every kernel PC lands past
# the end of the index (see §9.137).
#
# i386: base 0x804c3000, ImageBase 0x400000 -> 0x800c3000. That base is not a
# guess — four independently recorded `ntoskrnl+<off> (<addr>)` pairs from
# 2026-07-12 and three fresh crashes from 2026-07-25 all agree on it exactly,
# across 13 days of rebuilds.
#
# arm64/amd64: unknown. Their runtime base is never printed on serial and cannot
# be recovered from a saved log, so they get no reloc and will simply decline.
#
# A stale hardcoded value CANNOT produce a wrong answer here: it would push every
# lookup outside the index, the offset guard above fires, and the result is "no
# symbol" rather than a confident lie. That is why hardcoding is acceptable.
_RELOC_BY_ARCH = {"i386": "0x800c3000", "i386-uefi": "0x800c3000"}


def faulting_pc(serial: str) -> str | None:
    """The faulting PC, preferring the BUGCHECK record over loose `PC=` text.

    `_PC.search()` alone takes the FIRST `PC=`/`RIP=` anywhere in the log, which
    on a serial log containing any earlier trace (a usermode fault dump, a probe
    line) resolves something unrelated to the crash. The bugcheck parameters are
    the authoritative record, so consult them first and take the LAST one — the
    crash we are triaging is the one the boot died on, not the first it mentioned.
    """
    bc = _BUGCHECK_PARAMS.findall(serial)
    if bc:
        code_s, *params = bc[-1]
        try:
            idx = _PC_PARAM_BY_CODE.get(int(code_s, 16))
        except ValueError:
            idx = None
        if idx is not None:
            pc = params[idx - 1]
            if int(pc, 16):  # a zeroed parameter carries no address
                return pc
    # Fall back to loose PC=/RIP= text, but take the LAST occurrence.
    matches = _PC.findall(serial)
    for groups in reversed(matches):
        pc = next((g for g in groups if g), None)
        if pc:
            return pc
    return None


def symbolicate_pc(repo: Path, arch: str, serial: str) -> str | None:
    """Best-effort: resolve a faulting PC via the offline index. Skips silently
    if the index/tool is missing (graceful degradation), and REFUSES rather than
    returning a match whose offset proves it is meaningless."""
    pc = faulting_pc(serial)
    if not pc:
        return None
    tool = repo / "qemu" / "addr2sym.sh"
    if not tool.is_file():
        return None
    reloc = _RELOC_BY_ARCH.get(arch)
    argv = ["bash", str(tool), arch]
    if reloc:
        argv += ["--reloc", reloc]
    argv.append(pc)
    try:
        cp = subprocess.run(argv, cwd=repo, capture_output=True, text=True, timeout=30)
        out = cp.stdout.strip()
    except (OSError, subprocess.SubprocessError):
        return None
    if not out:
        return None
    off = _SYM_OFFSET.search(out)
    if off and int(off.group(1)) > _MAX_SANE_SYMBOL_OFFSET:
        # Common real cause: the PC is in a DRIVER (e.g. kmtest_drv.sys), whose
        # symbols are not in the kernel index at all. Say so instead of naming
        # whichever kernel symbol happened to sort below it.
        return f"{pc} -> (no symbol: nearest is {int(off.group(1)):#x} away — outside any function)"
    return f"{pc} -> {out}"
