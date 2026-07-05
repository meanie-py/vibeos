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
        if _CENSUS_STUCK.search(ln) or _LOCKDEP_HIT.search(ln)
    ]
    return {
        "census_stuck": sum(1 for ln in lines if _CENSUS_STUCK.search(ln)),
        "lockdep": sum(1 for ln in lines if _LOCKDEP_HIT.search(ln)),
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
        if ev["census_stuck"] or ev["lockdep"]:
            fields["wedge_confirmed"] = True
            fields["census_stuck"] = ev["census_stuck"]
            fields["lockdep_hits"] = ev["lockdep"]
            salient.insert(
                0,
                f"kernel self-report: {ev['census_stuck']} [CENSUS] STUCK, "
                f"{ev['lockdep']} [LOCKDEP] — wedge CONFIRMED, not contention",
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


def symbolicate_pc(repo: Path, arch: str, serial: str) -> str | None:
    """Best-effort: resolve a faulting PC via the offline index. Skips silently
    if the index/tool is missing (graceful degradation)."""
    m = _PC.search(serial)
    if not m:
        return None
    pc = next(g for g in m.groups() if g)
    tool = repo / "qemu" / "addr2sym.sh"
    if not tool.is_file():
        return None
    try:
        cp = subprocess.run(
            ["bash", str(tool), arch, pc], cwd=repo, capture_output=True, text=True, timeout=30
        )
        out = cp.stdout.strip()
        return f"{pc} -> {out}" if out else None
    except (OSError, subprocess.SubprocessError):
        return None
