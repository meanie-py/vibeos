"""WoW64 probe-ladder verdict check (`smoke --ladder`, and `gate` automatically).

WHY THIS EXISTS. The ladder guests are spawned by winlogon *after* explorer,
and the smoke's success condition IS the desktop marker — so a plain boot stops
watching before a single guest has run. A ladder guest could therefore be RED on
one arch indefinitely while every gate stayed green. That is not hypothetical:
it happened twice, and both are why `gate` now opts in on its own.

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
flag instead -- and since a flag nobody passes is the same rule again, `gate`
now passes it itself, for each arch whose media actually carries the guests
(`gate_decision` below).

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
from pathlib import Path

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

# [WINLOGON] selfmod32.exe exit = 0x600d (0x600D = PASS)
_VERDICT = re.compile(r"\[WINLOGON\] ([A-Za-z0-9_]+)\.exe exit = 0x([0-9a-fA-F]+)")

# How long to keep draining serial after the desktop marker for the ladder to
# finish. The guests run in sequence with per-guest winlogon watch windows
# (8-10s each for the stock applets), so this is generous on purpose: a
# too-short window would turn a slow host into a fake ladder failure, which is
# the exact class of false signal this file exists to remove.
LADDER_TIMEOUT_SECONDS = 420.0


# Where add_cd_file stages the guests (dll/win32/wow64cpu/CMakeLists.txt on
# amd64, qemu/mkramdisk.sh on arm64). Matched on the CD path, not the source
# path: the guests come from sibling trees (build-fex-hello, build-i386) that sit
# outside the build dir every other manifest entry points into.
STAGED_DIR = "syswow64"


def media_has_guests(repo: Path, arch: str) -> bool:
    """Does the media this arch's boot would OPEN actually carry the guests?

    Read from the generated `boot/<media>.cmake.lst` staging manifest, which is
    what `add_cd_file` wrote and therefore what the image contains.

    NOT a byte-scan of the image, and this is the whole point. A scan for
    "hello32.exe" returns a hit on EVERY amd64 image, staged or not, because
    winlogon's own diagnostic carries the name in its string table
    ("[WINLOGON] CreateProcessW(hello32.exe) FAILED (error %lu)") in plain ASCII.
    That false positive was measured, not imagined: it passed a boot with zero
    staged guests as ready for the ladder, which would have burned the full
    420s drain and then reported LADDER_NOT_STAGED -- the exact outcome this
    check exists to prevent. provenance.probes_missing_from_media is right for
    what it was built for (is this string COMPILED IN); "is this file staged at
    this path" is a different question and needs the manifest.

    False for every native-x86 boot without special-casing it -- an i386 livecd
    has no SysWOW64 to stage into -- so the arch list stays out of here.
    """
    from . import archcfg

    try:
        base = archcfg.base_arch(arch)
        stem = Path(archcfg.arch_build(arch).boot_media_rel).stem
        text = (repo / f"build-{base}" / "boot" / f"{stem}.cmake.lst").read_text(errors="ignore")
    except (OSError, KeyError, AttributeError):
        # No manifest (never configured) or an arch with no CD media. Absence of
        # evidence, so decline rather than demand -- the media checks report a
        # missing build far better than a boolean here could.
        return False

    guests = {f"{g}.exe" for g in LADDER}
    for line in text.splitlines():
        cd_path = line.split("=", 1)[0].strip().replace("\\", "/").lower()
        parts = cd_path.split("/")
        if len(parts) >= 2 and parts[-2] == STAGED_DIR and parts[-1] in guests:
            # Any staged guest opts in: winlogon can then run something, and a
            # PARTIAL set surfaces through check() as a LADDER_FAIL naming the
            # guests that never reported -- which is the right answer, since a
            # partial stage is itself a defect and must not pass quietly.
            return True
    return False


def gate_decision(repo: Path, arch: str) -> tuple[list[str], bool, str]:
    """What a gate should do about the ladder for one arch.

    Returns (extra smoke argv, staged?, one-line reason). Kept here rather than
    in gate.py so the policy is testable without standing up a gate run, and so
    the reason a gate gives for NOT checking the guests is written next to the
    check it is declining to run.
    """
    staged = media_has_guests(repo, arch)
    if staged:
        return (
            ["--ladder"],
            True,
            f"{len(LADDER)} WoW64 guests staged — boot watched past the desktop marker",
        )
    return (
        [],
        False,
        f"no staged {STAGED_DIR}/ guest in this arch's media manifest — "
        f"WoW64 guests NOT checked by this gate",
    )


def markers() -> list[str]:
    """Serial substrings that must all appear before the ladder is complete."""
    return [f"{g}.exe exit = " for g in LADDER]


def watch(vm, seconds: float = LADDER_TIMEOUT_SECONDS, seen: str = "") -> str:
    """Drain serial until every ladder guest has reported, or `seconds` elapse.

    `seen` is the serial the caller has ALREADY consumed. Without it this waits
    for markers that have gone past: the boot's own settle phase routinely reads
    a guest's verdict before this is called, and that one line is enough to make
    the loop sit out the entire ceiling. MEASURED -- a 9/9 PASS run drained
    420.2s of a 420s budget because `gui32.exe exit = 0x600e` landed in the settle
    read, so the verdict was correct and the wait was pure waste. The caller's
    `check()` sees both halves and never noticed.

    Returns only the NEWLY drained text; the caller appends it to the boot serial,
    so a BUGCHECK during the ladder still lands in full.log.
    """
    need = [m for m in markers() if m not in seen]
    if not need:
        return ""  # every guest already reported before we were called
    buf = ""
    deadline = time.monotonic() + seconds
    while time.monotonic() < deadline:
        buf += vm.serial.read_available(timeout=0.5) or ""
        if all(m in buf for m in need):
            break
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
