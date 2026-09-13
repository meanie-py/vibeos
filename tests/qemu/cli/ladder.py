"""WoW64 probe-ladder verdict check (`smoke --ladder`).

WHY THIS EXISTS. The ladder guests are spawned by winlogon *after* explorer,
and the smoke's success condition IS the desktop marker — so the ordinary
`smoke`/`gate` stops watching before a single guest has run. A ladder guest can
therefore be RED on one arch indefinitely and every gate stays green. That is
not hypothetical: it happened twice.

  * 2026-07-25 (roadmap, the `_MmCopyFromCaller` entry) noted in passing that
    "the smoke gate stays GREEN through this: the boot marker fires long before
    the ladder runs".
  * 2026-07-26 (E19) found `selfmod32.exe` had been exiting 0xBAD9 on amd64
    since the day E17 landed it — its rung-5 fixture writes under %WINDIR%,
    which is a writable ramdisk on arm64 and a read-only CD on the amd64
    livecd. Nobody saw it because seeing it required knowing to type
    `smoke --arch amd64 --until "selfmod32.exe exit"`.

Per AGENTS.md, a rule with no command behind it is a rule you will break, so
"remember to run --until on both arches after touching a guest" becomes this
flag instead.

SHAPE. This is a bounded EXTRA watch after the normal boot marker, not a change
to the boot path: with `--ladder` off, nothing here runs and the boot is
byte-for-byte what it was. The guests self-verify (each checks values, not
survival) and report a machine-readable verdict, so all this has to do is
collect them and insist every one is present and passing.

WHAT THIS DOES **NOT** COVER, stated so the flag is not read as more than it is.
winlogon runs the nine self-verifying guests FIRST and then a longer robustness
ladder — winmine, doom, and ~15 stock i386 applets. The watch below stops the
moment the ninth guest reports, so none of that is checked here. That is
deliberate rather than an oversight: those launches are "silent no-op unless
staged" and their pass criterion is "still running after 8-10s", which is not a
machine-checkable verdict in the sense this file means. Covering them properly
needs them to grow real verdicts first. Until then, a green `--ladder` says the
nine guests passed, not that the stock applets did.
"""

from __future__ import annotations

import re
import time

# guest -> the exit code that means PASS. gui32 reports 0x600e (it is the
# GDI/draw rung and has its own verdict code); everything else uses 0x600d.
# These 9 are the arches' SHARED ladder — the parity claim is that the same
# guest passes on a JIT host and on a native compat-mode host, whose arrival
# paths share nothing below wow64.dll. arm64-only extras (fexprobe,
# wow64probe) are reported when seen but are not required, so this table stays
# the definition of parity rather than of "whatever arm64 happens to stage".
LADDER: dict[str, int] = {
    "hello32": 0x600D,
    "gui32": 0x600E,
    "win32": 0x600D,
    "file32": 0x600D,
    "async32": 0x600D,
    "marshal32": 0x600D,
    "thread32": 0x600D,
    "fault32": 0x600D,
    "selfmod32": 0x600D,
}

# Optional guests: reported when seen, never required. This table is the PARITY
# set -- every entry must report or the ladder fails -- so a guest that is built
# on demand does not belong in it. sqlite32 is the third-party stress guest
# (tools/stress/sqlite/build.sh); it is absent from a normal image, and putting
# it above would make every ladder run wait out LADDER_TIMEOUT_SECONDS and then
# fail for a guest nobody built.
OPTIONAL: dict[str, int] = {
    "sqlite32": 0x600D,
}

# [WINLOGON] selfmod32.exe exit = 0x600d (0x600D = PASS)
_VERDICT = re.compile(r"\[WINLOGON\] ([A-Za-z0-9_]+)\.exe exit = 0x([0-9a-fA-F]+)")

# How long to keep draining serial after the desktop marker for the ladder to
# finish. The guests run in sequence with per-guest winlogon watch windows
# (8-10s each for the stock applets), so this is generous on purpose: a
# too-short window would turn a slow host into a fake ladder failure, which is
# the exact class of false signal this file exists to remove.
LADDER_TIMEOUT_SECONDS = 420.0

# Extra drain granted once an optional guest is seen to launch. Matches the
# winlogon watch window for the slowest one (sqlite32: 300s) plus slack.
OPTIONAL_EXTRA_SECONDS = 360.0


def markers() -> list[str]:
    """Serial substrings that must all appear before the ladder is complete."""
    return [f"{g}.exe exit = " for g in LADDER]


def _optional_in_flight(buf: str) -> list[str]:
    """Optional guests winlogon has LAUNCHED but that have not reported yet.

    An optional guest is absent from most images, so it cannot be a required
    marker -- but once serial shows winlogon created it, its verdict is exactly
    what the run is for, and stopping the drain before it reports throws the
    answer away. Detected from the launch line rather than from staging state,
    because the harness cannot see inside the media."""
    return [g for g in OPTIONAL if f"{g}.exe (" in buf and f"{g}.exe exit = " not in buf]


def watch(vm, seconds: float = LADDER_TIMEOUT_SECONDS) -> str:
    """Drain serial until every ladder guest has reported, or `seconds` elapse.

    Returns the drained text (the caller appends it to the boot serial, so a
    BUGCHECK during the ladder still lands in full.log)."""
    need = markers()
    buf = ""
    deadline = time.monotonic() + seconds
    # A launched optional guest gets its own extension on top of the parity
    # budget: it is third-party code doing real work under emulation, so it is
    # slow by nature, and charging it to the parity budget is what turned a
    # green ladder into a fake selfmod32 failure the first time one was added.
    extended = False
    while time.monotonic() < deadline:
        buf += vm.serial.read_available(timeout=0.5) or ""
        pending = _optional_in_flight(buf)
        if all(m in buf for m in need) and not pending:
            break
        if pending and not extended:
            deadline += OPTIONAL_EXTRA_SECONDS
            extended = True
    return buf


def verdicts(text: str) -> dict[str, int]:
    """Last reported exit code per guest (a guest is launched once, but take the
    last match so a re-run in a longer boot cannot be masked by an earlier one)."""
    out: dict[str, int] = {}
    for name, code in _VERDICT.findall(text):
        out[name] = int(code, 16)
    return out


def check(text: str) -> dict | None:
    """None if every ladder guest reported PASS; else a failure dict with the
    same {classification, salient} shape the smoke triage stages use."""
    seen = verdicts(text)
    missing = [g for g in LADDER if g not in seen]
    wrong = [(g, seen[g], want) for g, want in LADDER.items() if g in seen and seen[g] != want]
    # An optional guest is ignored when absent and enforced when present: the
    # reason to build one is to find out whether it passes, so a red one that
    # got staged must not be silently tolerated.
    wrong += [(g, seen[g], want) for g, want in OPTIONAL.items() if g in seen and seen[g] != want]
    # LAUNCHED but never reported is NOT the same as absent. Treating it as
    # absent renders a hung guest as a green ladder -- observed: sqlite32 wedged
    # at guest thread startup and the run still printed "9/9 guests PASS".
    missing += _optional_in_flight(text)

    if not missing and not wrong:
        return None

    salient: list[str] = []
    if len(missing) == len(LADDER):
        # Nothing ran at all — almost always staging, not a regression. Say so
        # with the recipe rather than letting it read as "the ladder broke".
        return {
            "classification": "LADDER_NOT_STAGED",
            "salient": [
                "no ladder guest reported a verdict — the media has no WoW64 guests staged",
                "arm64 stages them only under VIBEOS_FEX=1: "
                "`VIBEOS_FEX=1 ./vibeos-cli build --arch arm64`",
                "amd64 stages from dll/win32/wow64cpu/CMakeLists.txt; "
                "guests build via tools/fex/build-hello-compute.sh",
            ],
        }
    for g, got, want in wrong:
        salient.append(f"{g}.exe exit = 0x{got:04x} (want 0x{want:04x}) — see [{g}] serial lines")
    for g in missing:
        salient.append(f"{g}.exe never reported a verdict (crashed, hung, or not staged)")
    return {"classification": "LADDER_FAIL", "salient": salient}


def summary(text: str) -> str:
    """One-line roll-up for the stage detail."""
    seen = verdicts(text)
    ok = sum(1 for g, want in LADDER.items() if seen.get(g) == want)
    extra = [g for g in seen if g not in LADDER]
    tail = f" (+{len(extra)} non-parity: {', '.join(sorted(extra))})" if extra else ""
    return f"{ok}/{len(LADDER)} guests PASS{tail}"
