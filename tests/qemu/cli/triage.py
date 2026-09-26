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
    # RETIRED 2026-09-25 — arm64_eresource_notowned_0xe3 (#43) and
    # arm64_smp39_zero_pc_fetch_abort (#39), together. #43's root fix landed
    # 2026-09-24 (fastfat's C shim tested IRP_CONTEXT_FLAG_DISABLE_DIRTY as the WAIT
    # bit, so FCB acquires ran non-blocking and FatCommonCleanup released a page
    # fault's lock; field manual "0xE3 RESOURCE_NOT_OWNED (#43) was a fastfat shim").
    # Verify-gone: 102 arm64 MP kmtest rounds at e2928af9a3c, zero bugchecks of ANY
    # code, where the same corpus had 0xE3 in 18/336 and 0x1E in 10/336 before it.
    # #39's own root is NOT proven: one capture's stack ran through
    # ExReleaseResourceForThreadLite, so it is plausibly the same corruption, and
    # retiring it is the 0xF4 precedent above — a recurrence must now BLOCK as a
    # fresh BUGCHECK 0x1E with its capture preserved, which is how the next lead
    # arrives. Both signatures, kept for a re-add if either recurs:
    #   0xE3, P3/P4 "0x0000000000000000, 0x0000000000000003)"
    #   0x1E with "[KEXC]" + "PC=0x0000000000000000" + "ESR=0x0000000086000007"
    #        + "Spsr=0x0000000020000345" (the whole shape: SEH tests print ESRs on
    #        purpose and pass)
    # Do NOT re-add 0x03: KiSetAffinityThread raises a genuine 0x03 with ALL-ZERO
    # parameters, which is a different bug and must keep blocking.
    #
    # MP-default beta (2026-09-12, user's call): the fleet boots the MP kernel by
    # default and eats this one while it is hunted in the open. DELIBERATELY narrow
    # — the retired `arm64_lpcexitthread_race` entry above is the warning: a broad
    # signature masked real bugs for months. Retire it the moment its root fix
    # lands and a re-gate shows the signature gone.
    {
        "name": "i386_mp_win32k_av_0x1e",
        "when": "bugcheck",
        "code": "0x0000001E",
        # Pinned to the faulting RVA the banner resolves, so a DIFFERENT win32k
        # fault still blocks. Found 1-in-200 on the first real i386 MP soak.
        "any": ["win32k.sys+0x65EE1"],
        "note": "i386 MP boot: win32k AV at winlogon desktop bringup (1/204=0.49%); retry",
    },
]


def _flake_note(flake: dict | None) -> list[str]:
    """The flake's human note, as a (possibly empty) salient-line list.

    `note` is optional metadata: indexing it directly turned an entry that merely
    omitted it into a KeyError that took down the whole triage, on the one path that
    DOES consult the flake list. Read it defensively.
    """
    if not flake:
        return []
    note = flake.get("note")
    return [note] if note else []


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
        # A documented flake does not stop being one because it fired a few hundred
        # milliseconds later. This arm used to hardcode known_flake=None, so the
        # identical crash was retried when it landed before the marker and REJECTED
        # the lane when it landed after it — which is how a known arm64 ERESOURCE
        # 0xE3 bounced a green-gated batch out of the merge queue.
        flake = _match_known_flake(serial, code, "bugcheck")
        return {
            "classification": "KNOWN_FLAKE" if flake else "BUGCHECK",
            "salient": salient + _flake_note(flake),
            "known_flake": flake["name"] if flake else None,
            **fields,
        }
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
            "salient": salient + _flake_note(flake),
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
            "salient": salient + _flake_note(flake),
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
# Logs with [KRNLBASE] use the runtime base from the loader list. Logs without
# that marker still have no known ARM64/amd64 relocation and must decline.
#
# A stale hardcoded value CANNOT produce a wrong answer here: it would push every
# lookup outside the index, the offset guard above fires, and the result is "no
# symbol" rather than a confident lie. That is why hardcoding is acceptable.
_RELOC_BY_ARCH = {"i386": "0x800c3000", "i386-uefi": "0x800c3000"}

# ...but for the arches above, none of that guessing is needed when the bugcheck
# banner already resolved the module itself:
#     ***   P2 = ntoskrnl.exe+0x3788
# That offset IS the RVA, so the index address is just RVA + the PE ImageBase —
# no runtime base, no reloc, immune to where the image happened to land. The
# comment above ("arm64/amd64 ... cannot be recovered from a saved log") was
# wrong: it is right there on the next line. Until this was used, every arm64
# bugcheck declined to symbolicate, and the one that mattered — a -smp 1 0x1E —
# needed hand-computing to reach `KiSynchronousExceptionEntry+4`.
_MODULE_OFFSET = re.compile(r"^\*\*\*\s+P\d\s*=\s*(\S+?)\+0x([0-9a-fA-F]+)\s*$", re.MULTILINE)
_PE_IMAGE_BASE = 0x400000

_KERNEL_BASE = re.compile(r"\[KRNLBASE\] ntoskrnl runtime base=(?:0x)?([0-9a-fA-F]+)")


def kernel_relocation(arch: str, serial: str) -> str | None:
    """Use this boot's loader-reported base when available."""
    matches = _KERNEL_BASE.findall(serial)
    if matches:
        return hex(int(matches[-1], 16) - _PE_IMAGE_BASE)
    return _RELOC_BY_ARCH.get(arch)


def faulting_module_offset(serial: str) -> tuple[str, int] | None:
    """(module, RVA) from the bugcheck banner's resolved parameter, or None.

    Takes the LAST occurrence for the same reason `faulting_pc` does: the crash
    being triaged is the one the boot died on.
    """
    hits = _MODULE_OFFSET.findall(serial)
    if not hits:
        return None
    module, rva = hits[-1]
    return module, int(rva, 16)


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


def _resolve_addr(
    repo: Path, arch: str, pc: str, *, mp: bool = False, reloc: str | None = None
) -> str | None:
    """One address -> "0xADDR -> symbol+off" via the offline index, or None.

    Skips silently if the index/tool is missing (graceful degradation), and
    REFUSES rather than returning a match whose offset proves it is meaningless.
    """
    tool = repo / "qemu" / "addr2sym.sh"
    if not tool.is_file():
        return None
    argv = ["bash", str(tool), arch]
    if reloc:
        argv += ["--reloc", reloc]
    # ntkrnlmp.exe is a different binary from ntoskrnl.exe, so an MP address
    # resolved against the UP index names the wrong function — and it does so
    # silently, because some symbol always sorts below the address.
    if mp:
        argv.append("--mp")
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


def symbolicate_pc(repo: Path, arch: str, serial: str) -> str | None:
    """Best-effort: resolve a faulting PC via the offline index."""
    pc = faulting_pc(serial)
    # Prefer the banner's own module+RVA over a guessed reloc: it is exact, needs
    # no runtime base, and works on the arches that have no reloc entry at all --
    # nor a PC at all, since the RVA alone locates the fault.
    mod_off = faulting_module_offset(serial)
    if not pc:
        if not mod_off:
            return None
        pc = f"{mod_off[0]}+{mod_off[1]:#x}"
    if mod_off and mod_off[0].lower() not in _KERNEL_IMAGES:
        # The banner named a DRIVER (win32k.sys, a kmtest .sys). The kernel index
        # holds only ntoskrnl, and some kernel symbol always sorts below any RVA,
        # so resolving there named a confidently wrong function. Resolve against
        # the named module's own symbols, or say it is not indexed.
        return _resolve_in_module(repo, arch, pc, *mod_off)
    reloc = None if mod_off else kernel_relocation(arch, serial)
    if mod_off:
        pc = hex(_PE_IMAGE_BASE + mod_off[1])
        mp = mod_off[0].lower() == "ntkrnlmp.exe"
    else:
        # freeldr announces which kernel it loaded, so read it off the serial.
        mp = "ntkrnlmp.exe" in serial
    return _resolve_addr(repo, arch, pc, mp=mp, reloc=reloc)


_KERNEL_IMAGES = frozenset({"ntoskrnl.exe", "ntkrnlmp.exe"})


def _resolve_in_module(repo: Path, arch: str, pc: str, module: str, rva: int) -> str:
    from . import symbolize

    mods = [m for m in symbolize.load_modules(repo, arch) if m.name.lower() == module.lower()]
    if not mods:
        return (
            f"{pc} -> {module}+{rva:#x} (module not in the {arch} user index; "
            f"run qemu/build-user-index.sh {arch}, then ./vibeos-cli sym)"
        )
    return f"{pc} = " + symbolize.resolve(repo, arch, mods[0].base + rva, want_module=module)


# `info registers -a` groups its output per vCPU behind a bare `CPU#<n>` line,
# and prints the program counter UNPREFIXED under an arch-specific name: PC= on
# arm64, RIP= on amd64, EIP= on i386. Deliberately not `_PC` above — that one
# requires an `0x` prefix, which the monitor never emits.
_CPU_BANNER = re.compile(r"^CPU#(\d+)\b", re.MULTILINE)
_CPU_PC = re.compile(r"\b(?:PC|RIP|EIP)=([0-9A-Fa-f]{8,16})\b")
_CPU_SP = re.compile(r"\b(?:SP|RSP|ESP)=([0-9A-Fa-f]{8,16})\b")


def cpu_pcs(state: str) -> list[tuple[int, str]]:
    """[(cpu_index, pc_hex)] parsed out of a `capture_cpu_state()` blob.

    Slices the dump at each `CPU#n` banner and takes the FIRST program counter
    inside that slice, so a vCPU is credited only with its own register block.
    """
    banners = list(_CPU_BANNER.finditer(state))
    found = []
    for i, m in enumerate(banners):
        end = banners[i + 1].start() if i + 1 < len(banners) else len(state)
        pc = _CPU_PC.search(state, m.end(), end)
        if pc:
            found.append((int(m.group(1)), pc.group(1)))
    return found


# arm64 only: `info registers` prints PSTATE after the register file. M[3:0]
# is the exception level the vCPU is executing at — 0 = EL0t (user), 4/5 =
# EL1t/EL1h (kernel).
_CPU_PSTATE = re.compile(r"\bPSTATE=([0-9A-Fa-f]{8,16})\b")

# Below this, an arm64 VA is in the low (user) half of the address space.
_ARM64_USER_LIMIT = 0x0000_8000_0000_0000


def cpu_sps(state: str) -> list[tuple[int, str]]:
    """[(cpu_index, sp_hex)] parsed out of a `capture_cpu_state()` blob.

    Same slicing rule as cpu_pcs: a vCPU is credited only with its own block.
    `info registers` prints the stack pointer as `SP=` on arm64 (padded, hence
    the loose left boundary), `RSP=` on amd64 and `ESP=` on i386.
    """
    banners = list(_CPU_BANNER.finditer(state))
    found = []
    for i, m in enumerate(banners):
        end = banners[i + 1].start() if i + 1 < len(banners) else len(state)
        sp = _CPU_SP.search(state, m.end(), end)
        if sp:
            found.append((int(m.group(1)), sp.group(1)))
    return found


def cpu_kernel_at_user_pc(state: str) -> list[tuple[int, str]]:
    """[(cpu_index, pc_hex)] for every arm64 vCPU executing at EL1 with a PC in
    the USER half of the address space.

    That combination has exactly one benign reading — none. The kernel never
    executes user memory, and AArch64 will not let it (EL0-writable pages are
    implicitly PXN), so a vCPU parked there is taking the same instruction
    abort over and over: the handler resolves it as spurious and erets back to
    the same PC. Two samples of that read as FROZEN, because the PC never
    moves — but nothing is frozen; it is the busiest CPU on the box. This is
    the signature the 2026-09-05 arm64 boot wedge was hiding behind.
    """
    banners = list(_CPU_BANNER.finditer(state))
    found = []
    for i, m in enumerate(banners):
        end = banners[i + 1].start() if i + 1 < len(banners) else len(state)
        pc = _CPU_PC.search(state, m.end(), end)
        ps = _CPU_PSTATE.search(state, m.end(), end)
        if not pc or not ps:
            continue
        el = int(ps.group(1), 16) & 0xF
        if el != 0 and int(pc.group(1), 16) < _ARM64_USER_LIMIT:
            found.append((int(m.group(1)), pc.group(1)))
    return found


def symbolicate_cpu_state(
    repo: Path, arch: str, state: str, *, mp: bool = False, serial: str = ""
) -> list[str]:
    """One "CPU#n 0xADDR -> symbol" line per vCPU we could locate.

    This is the payoff of capturing the state at all: on a silent wedge the
    symbol here names the function the box died in, which serial cannot.
    """
    lines = []
    for idx, pc in cpu_pcs(state):
        sym = _resolve_addr(repo, arch, f"0x{pc}", mp=mp, reloc=kernel_relocation(arch, serial))
        lines.append(f"CPU#{idx} {sym or f'0x{pc} -> (unresolved)'}")
    return lines
