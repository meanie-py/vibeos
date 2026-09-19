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
winlogon runs the self-verifying guests FIRST and then a longer robustness
ladder — winmine, doom, and ~15 stock i386 applets. The watch below stops the
moment the last parity guest reports, so none of that is checked here. That is
deliberate rather than an oversight: those launches are "silent no-op unless
staged" and their pass criterion is "still running after 8-10s", which is not a
machine-checkable verdict in the sense this file means. Covering them properly
needs them to grow real verdicts first. Until then, a green `--ladder` says the
parity guests passed, not that the stock applets did.
"""

from __future__ import annotations

import re
import subprocess
import time
from pathlib import Path

# guest -> the exit code that means PASS. gui32 reports 0x600e (it is the
# GDI/draw rung and has its own verdict code); everything else uses 0x600d.
# These are the arches' SHARED ladder — the parity claim is that the same
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
    "reg32": 0x600D,
}

# Optional guests: reported when seen, never required. This table is the PARITY
# set -- every entry must report or the ladder fails -- so a guest that is built
# on demand does not belong in it. sqlite32 is the third-party stress guest
# (tools/stress/sqlite/build.sh); it is absent from a normal image, and putting
# it above would make every ladder run wait out LADDER_TIMEOUT_SECONDS and then
# fail for a guest nobody built.
OPTIONAL: dict[str, int] = {
    "sqlite32": 0x600D,
    "lua32": 0x600D,
}

# [WINLOGON] selfmod32.exe exit = 0x600d (0x600D = PASS)
_VERDICT = re.compile(r"\[WINLOGON\] ([A-Za-z0-9_]+)\.exe exit = 0x([0-9a-fA-F]+)")

# How long to keep draining serial after the desktop marker for the ladder to
# finish. The guests run in sequence with per-guest winlogon watch windows
# (8-10s each for the stock applets), so this is generous on purpose: a
# too-short window would turn a slow host into a fake ladder failure, which is
# the exact class of false signal this file exists to remove.
LADDER_TIMEOUT_SECONDS = 420.0

# Extra drain granted per optional guest seen to launch. Matches the winlogon
# watch window each of them gets (300s) plus slack.
#
# PER GUEST, and granted once each -- not one flat extension for the whole set.
# It was a single one-shot bool while sqlite32 was the only optional guest, which
# was correct for a set of one and silently wrong the moment lua32 joined: two
# guests budgeted 300s apiece in winlogon would have shared a single 360s
# extension, so a slow second guest gets cut off mid-run and reads as "staged but
# never reported" -- a false RED, and the most expensive kind, because the guest
# it accuses did nothing wrong.
OPTIONAL_EXTRA_SECONDS = 360.0


# Where add_cd_file stages the guests (dll/win32/wow64cpu/CMakeLists.txt on
# amd64, qemu/mkramdisk.sh on arm64). Matched on the CD path, not the source
# path: the guests come from sibling trees (build-fex-hello, build-i386) that sit
# outside the build dir every other manifest entry points into.
STAGED_DIR = "syswow64"


# arm64's media is not built by `add_cd_file` at all: `qemu/mkramdisk.sh` fills a
# FAT ramdisk with ~36 `mcopy` calls, into the same `reactos/syswow64` path shape.
# `qemu/ramdisk.img` is what it writes (boot.img then embeds it), so it is the
# arm64 analogue of the cmake manifest: the record of what was staged.
RAMDISK_REL = "qemu/ramdisk.img"


# guest -> the .c it is built from. Explicit rather than derived: win32 is built
# from Win32App.c, so a name-mangling convention would silently check nothing for
# it -- and a staleness check that quietly verifies nothing is worse than none,
# because it reports "fresh".
GUEST_SOURCE: dict[str, str] = {
    "hello32": "tools/fex/hello-compute/Hello32.c",
    "gui32": "tools/fex/hello-compute/Gui32.c",
    "win32": "tools/fex/hello-compute/Win32App.c",
    "file32": "tools/fex/hello-compute/File32.c",
    "async32": "tools/fex/hello-compute/Async32.c",
    "marshal32": "tools/fex/hello-compute/Marshal32.c",
    "thread32": "tools/fex/hello-compute/Thread32.c",
    "fault32": "tools/fex/hello-compute/Fault32.c",
    "selfmod32": "tools/fex/hello-compute/Selfmod32.c",
    "reg32": "tools/fex/hello-compute/Reg32.c",
}

# The script carries the link lines, so a change to it (a new flag, a different
# importlib) invalidates every guest just as a source edit invalidates one.
GUEST_BUILDER = "tools/fex/build-hello-compute.sh"

# The OPTIONAL guests live somewhere else entirely: their own dir under
# tools/stress/, their own build.sh, and their .exe beside the source rather
# than in build-fex-hello/. Value is (exe, [OUR sources]).
#
# Only OUR two files per guest are listed, and that is the whole subtlety.
# Each dir also holds fetched third-party code -- tools/stress/lua/src/ (Lua
# 5.4.7) and tools/stress/sqlite/amalg/ (the amalgamation) -- both gitignored,
# both re-fetched on demand by build.sh. Their mtime records WHEN CURL RAN, not
# when anything changed, so a build on a machine that had to re-fetch leaves
# upstream source newer than a binary that is perfectly current. Including them
# would red a healthy guest on a schedule set by the network. The question this
# check asks is "did OUR source move after we built", and upstream cannot
# answer it.
OPTIONAL_SOURCE: dict[str, tuple[str, list[str]]] = {
    "lua32": (
        "tools/stress/lua/lua32.exe",
        ["tools/stress/lua/Lua32.c", "tools/stress/lua/build.sh"],
    ),
    "sqlite32": (
        "tools/stress/sqlite/sqlite32.exe",
        ["tools/stress/sqlite/Sqlite32.c", "tools/stress/sqlite/build.sh"],
    ),
}


def stale_guests(repo: Path) -> list[str]:
    """Guests whose .exe is OLDER than the source it was built from.

    The guests are NOT in git -- `.gitignore` excludes `build-fex-*`, and
    `vibeos-cli build` does not rebuild them -- so the ladder's verdict depends
    on three inputs of which only one is versioned: the commit, these binaries,
    and the build-i386 import closure. Two lanes at the SAME commit can therefore
    grade different guest code, and did: a marshal32.exe built 35 minutes before
    a fix to its own socket rung hung there for a week, costing the four guests
    that run after it their verdicts on every boot, while the lane that built it
    saw nothing wrong -- every input it could see was internally consistent.

    Keyed on source-vs-binary mtime, deliberately NOT on absolute age. That
    binary was six days old; any age threshold loose enough not to nag would
    have passed it. What makes it detectably wrong is only that Marshal32.c is
    NEWER than marshal32.exe.

    A guest with no binary is not stale -- it is unbuilt, which the
    partially-built branch of `gate_decision` already reports with its own
    remedy. Only guests present on disk are compared. A guest missing from
    GUEST_SOURCE is reported as stale rather than skipped: an unmapped guest is
    a gap in this check, and it must not read as freshness.
    """
    out: list[str] = []
    for guest in LADDER:
        exe = repo / "build-fex-hello" / f"{guest}.exe"
        if not exe.is_file():
            continue
        rel = GUEST_SOURCE.get(guest)
        if rel is None:
            out.append(f"{guest} (no source mapping)")
            continue
        out.extend(_newer_than(repo, guest, exe, [rel, GUEST_BUILDER]))

    # OPTIONAL guests are built on demand by a script that FETCHES, so they are
    # absent in most trees. Absent means "nobody built it", which
    # media_optional_staged already handles by not grading it -- so unlike a
    # LADDER guest, an absent one is not a hole in this check and must not read
    # as stale. Only a binary that EXISTS is compared.
    for guest, (exe_rel, srcs) in OPTIONAL_SOURCE.items():
        exe = repo / exe_rel
        if not exe.is_file():
            continue
        out.extend(_newer_than(repo, guest, exe, srcs))
    return out


def _newer_than(repo: Path, guest: str, exe: Path, sources: list[str]) -> list[str]:
    """Which of `sources` postdates `exe`. A source that does not exist is
    reported, not skipped: the map naming a file the tree does not have is a
    broken mapping, and silence about it would read as freshness."""
    built = exe.stat().st_mtime
    out: list[str] = []
    for rel in sources:
        src = repo / rel
        if not src.is_file():
            out.append(f"{guest} (source {rel} missing)")
        elif src.stat().st_mtime > built:
            out.append(f"{guest} ({rel} is newer)")
    return out


def _guest_names() -> set[str]:
    return {f"{g}.exe".lower() for g in LADDER}


def _staged_per_manifest(repo: Path, arch: str, subdir: str, names: set[str]) -> set[str] | None:
    """x86 media, written by `add_cd_file`. None = this media has no manifest,
    which is not the same as 'no guests' — see `_staged_per_ramdisk`."""
    from . import archcfg

    try:
        base = archcfg.base_arch(arch)
        stem = Path(archcfg.arch_build(arch).boot_media_rel).stem
        # Read EVERY <stem>*.lst, not just <stem>.cmake.lst. add_cd_file(TARGET ...)
        # lands in the cmake one as an UNEXPANDED generator expression
        # ("reactos/system32/$<TARGET_FILE_NAME:wow64cpu>"); only add_cd_file(FILE ...)
        # is literal there. The expanded twin (<stem>.<CONFIG>.lst, written by
        # file(GENERATE)) carries the real names. Reading one of them answers
        # correctly for half the entries and silently wrongly for the other half --
        # which is the same mistake as reading a manifest that does not describe
        # this media at all.
        # `{stem}.*.lst`, NOT `{stem}*.lst`. arm64's media is qemu/boot.img, so
        # stem is "boot", and the loose glob matches bootcd.cmake.lst and
        # bootcdregtest.cmake.lst — manifests for a DIFFERENT medium that merely
        # share the prefix. `text` was then non-empty, so this probe believed it
        # could answer, returned an empty set, and _read_media took that as the
        # verdict without ever consulting _staged_per_ramdisk (which finds the
        # guests correctly). A missing manifest masqueraded as a missing file,
        # which is exactly what the None contract here exists to prevent.
        lsts = sorted((repo / f"build-{base}" / "boot").glob(f"{stem}.*.lst"))
        text = "\n".join(f.read_text(errors="ignore") for f in lsts)
    except (OSError, KeyError, AttributeError):
        return None
    if not text:
        return None

    found: set[str] = set()
    for line in text.splitlines():
        cd_path = line.split("=", 1)[0].strip().replace("\\", "/").lower()
        parts = cd_path.split("/")
        if len(parts) >= 2 and parts[-2] == subdir and parts[-1] in names:
            found.add(parts[-1])
    return found


def _staged_per_ramdisk(repo: Path, arch: str, subdir: str, names: set[str]) -> set[str] | None:
    """arm64 media, filled by mcopy. None = cannot tell (no image, no mtools).

    Reads stdout, never the exit code: `mdir` exits 0 on "File not found", which
    is the same $?-is-not-the-answer trap AGENTS.md names twice.
    """
    img = repo / RAMDISK_REL
    if not img.is_file():
        return None
    try:
        cp = subprocess.run(
            ["mdir", "-b", "-i", str(img), f"::/reactos/{subdir}"],
            capture_output=True,
            text=True,
            timeout=60,
        )
    except (OSError, subprocess.SubprocessError):
        return None  # mtools absent — no verdict, not a negative one
    return {
        base
        for line in cp.stdout.splitlines()
        if (base := line.strip().rsplit("/", 1)[-1].lower()) in names
    }


# The per-arch WoW64 CPU layer. winlogon gates every guest spawn on exactly this
# (`Wow64SupportStaged()`), so it is the honest test for "can this media run a
# 32-bit guest at all" — and using the launcher's own signal keeps an arch list
# out of here.
CPU_LAYERS = {"wow64cpu.dll", "libwow64fex.dll"}


def _read_media(repo: Path, arch: str, subdir: str, names: set[str]) -> set[str] | None:
    """True/False from whichever staging mechanism this media uses; None if
    neither can answer. Each probe returns None for 'not my mechanism', so a
    missing manifest can never masquerade as a missing file."""
    for probe in (_staged_per_manifest, _staged_per_ramdisk):
        answer = probe(repo, arch, subdir, names)
        if answer is not None:
            return answer
    return None


def supports_wow64(repo: Path, arch: str) -> bool:
    """Is a WoW64 CPU layer on this media? False on a native-x86 livecd, which is
    why 'no guests staged' is the final answer there rather than a fixable state."""
    return bool(_read_media(repo, arch, "system32", CPU_LAYERS))


def media_has_guests(repo: Path, arch: str) -> bool:
    """Does the media this arch's boot would OPEN actually carry the guests?

    Two mechanisms, because the tree has two. x86 media is assembled by
    `add_cd_file` and leaves a `boot/<media>.cmake.lst` manifest; arm64's is a FAT
    ramdisk filled by `mcopy` and leaves none. Asking only the manifest made arm64
    answer False WHILE THE GUESTS WERE ABOARD — a false negative that silently
    withheld ladder coverage from the FEX host, the arch it matters most on. The
    bug was conflating "no manifest" with "no guests"; each probe now returns None
    for "not my mechanism" and only a real reading returns False.

    NOT a byte-scan of the image, and that is the point. Scanning for
    "hello32.exe" hits on EVERY amd64 image staged or not, because winlogon's own
    diagnostic carries the name in plain ASCII
    ("[WINLOGON] CreateProcessW(hello32.exe) FAILED (error %lu)"). That false
    positive was measured, and it made `gate` demand a ladder of boots that had
    none — 420s burned, then innocent lanes blamed.

    False for i386 without special-casing it: its livecd has a manifest and no
    SysWOW64 to stage into, so the first probe answers honestly.
    """
    return bool(_read_media(repo, arch, STAGED_DIR, _guest_names()))


def media_optional_staged(repo: Path, arch: str) -> list[str]:
    """Which OPTIONAL guests this arch's media actually carries.

    Same read as `media_has_guests`, for the reason the wow64-amd64 lane gave when
    it added this: the serial is not a reliable source here. An optional guest runs
    AFTER the parity set, so the drain can finish before its launch line is even
    emitted, and a hung guest then reads as absent — that silently passed a run as
    "N/N guests PASS" while sqlite32 was wedged. The media knows before the boot
    starts.

    Its original form read only `<stem>.cmake.lst` and documented that it could not
    answer on arm64, recommending a manifest from mkramdisk.sh. That is no longer
    needed: `_read_media` answers on both media types and reads every `<stem>*.lst`,
    so this inherits the ramdisk path and the generator-expression fix instead of
    carrying a second reader with its own blind spots.
    """
    found = _read_media(repo, arch, STAGED_DIR, {f"{g}.exe" for g in OPTIONAL})
    if not found:
        return []
    return sorted(g for g in OPTIONAL if f"{g}.exe".lower() in found)


def gate_decision(repo: Path, arch: str) -> tuple[list[str], bool, str]:
    """What a gate should do about the ladder for one arch.

    Returns (extra smoke argv, staged?, one-line reason). Kept here rather than
    in gate.py so the policy is testable without standing up a gate run, and so
    the reason a gate gives for NOT checking the guests is written next to the
    check it is declining to run.
    """
    staged = media_has_guests(repo, arch)
    if staged:
        # Stale guests SKIP, they do not fail. A stale binary is our own
        # bookkeeping error, not a regression in the OS, and grading it produces
        # a RED that blames the guests running AFTER the stale one -- which is
        # how one six-day-old marshal32.exe read as four broken guests. The
        # remedy is one command and belongs in the message, not in a bisect.
        stale = stale_guests(repo)
        if stale:
            return (
                [],
                False,
                f"staged guest binaries are older than their sources "
                f"({', '.join(stale)}) — the guests are NOT in git and a normal "
                f"build does not rebuild them, so the ladder would grade stale "
                f"code; rebuild with: {GUEST_BUILDER}",
            )
        return (
            ["--ladder"],
            True,
            f"{len(LADDER)} WoW64 guests staged — boot watched past the desktop marker",
        )
    # A native-x86 boot has no CPU layer, so it has no SysWOW64 to stage into and
    # "not staged" is the final answer, not a fixable state. Checked by asking the
    # media for the layer winlogon itself gates on, so no arch list appears here.
    if not supports_wow64(repo, arch):
        return (
            [],
            False,
            "no WoW64 CPU layer on this arch's media — 32-bit guests cannot run here, "
            "so the ladder does not apply",
        )
    # Built-but-unstaged is a DIFFERENT situation from never-built, and the
    # difference is one command. Staging is snapshotted when the media is
    # assembled, so guests built afterwards sit on disk while the media has none --
    # silently. That cost 45 minutes tonight. The remedy differs by mechanism, so
    # name the one that matches this media rather than the x86 one everywhere.
    built = sorted(g for g in LADDER if (repo / "build-fex-hello" / f"{g}.exe").is_file())
    # Partially built is its own state, and restaging cannot fix it. The CMake
    # ladder list is all-or-nothing (`_wow64_guest_ok`), so ONE missing guest
    # keeps every other one off the media -- the same failure doom caused once
    # by being absent. A tree that predates a newly added guest lands here, and
    # telling it to restage would send it round a build that changes nothing.
    if built and len(built) < len(LADDER):
        absent = ", ".join(f"{g}.exe" for g in LADDER if g not in built)
        return (
            [],
            False,
            f"{len(built)}/{len(LADDER)} ladder guests built — {absent} missing, and "
            f"staging is all-or-nothing, so NOTHING is staged; build the guests with: "
            f"tools/fex/build-hello-compute.sh",
        )
    if built:
        fix = (
            "VIBEOS_FEX=1 ./vibeos-cli build --arch arm64"
            if (repo / RAMDISK_REL).is_file()
            else "touch dll/win32/wow64cpu/CMakeLists.txt && ./vibeos-cli build --arch <a>"
        )
        return (
            [],
            False,
            f"{len(built)}/{len(LADDER)} guests are BUILT but none is staged on this "
            f"arch's media — staging is snapshotted when the media is assembled; "
            f"restage with: {fix}",
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


def _optional_in_flight(buf: str) -> list[str]:
    """Optional guests winlogon has LAUNCHED but that have not reported yet.

    An optional guest is absent from most images, so it cannot be a required
    marker -- but once serial shows winlogon created it, its verdict is exactly
    what the run is for, and stopping the drain before it reports throws the
    answer away. Detected from the launch line rather than from staging state,
    because the harness cannot see inside the media."""
    return [g for g in OPTIONAL if f"{g}.exe (" in buf and f"{g}.exe exit = " not in buf]


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
    if not need and not _optional_in_flight(seen):
        return ""  # every guest already reported before we were called
    buf = ""
    deadline = time.monotonic() + seconds
    # A launched optional guest gets its own extension on top of the parity
    # budget: it is third-party code doing real work under emulation, so it is
    # slow by nature, and charging it to the parity budget is what turned a
    # green ladder into a fake selfmod32 failure the first time one was added.
    extended: set[str] = set()
    while time.monotonic() < deadline:
        buf += vm.serial.read_available(timeout=0.5) or ""
        pending = _optional_in_flight(buf)
        if all(m in buf for m in need) and not pending:
            break
        for g in pending:
            if g not in extended:
                deadline += OPTIONAL_EXTRA_SECONDS
                extended.add(g)
    return buf


def verdicts(text: str) -> dict[str, int]:
    """Last reported exit code per guest (a guest is launched once, but take the
    last match so a re-run in a longer boot cannot be masked by an earlier one)."""
    out: dict[str, int] = {}
    for name, code in _VERDICT.findall(text):
        out[name] = int(code, 16)
    return out


def check(text: str, optional_staged: list[str] | None = None) -> dict | None:
    """None if every ladder guest reported PASS; else a failure dict with the
    same {classification, salient} shape the smoke triage stages use.

    `optional_staged` names OPTIONAL guests the MEDIA carries (see
    media_optional_staged). Those are then required to report, because a staged
    guest that never reports has hung -- which is exactly what a stress guest is
    for and must not read as absent."""
    seen = verdicts(text)
    missing = [g for g in LADDER if g not in seen]
    wrong = [(g, seen[g], want) for g, want in LADDER.items() if g in seen and seen[g] != want]
    # An optional guest is ignored when absent and enforced when present: the
    # reason to build one is to find out whether it passes, so a red one that
    # got staged must not be silently tolerated.
    wrong += [(g, seen[g], want) for g, want in OPTIONAL.items() if g in seen and seen[g] != want]
    # Staged but never reported is NOT the same as absent, and the serial cannot
    # settle it: an optional guest runs after the parity set, so the drain can
    # end before its launch line appears. Ask the media instead. (The in-flight
    # check still catches one that launched inside the drained window.)
    for g in optional_staged or []:
        if g not in seen and g not in missing:
            missing.append(g)
    missing += [g for g in _optional_in_flight(text) if g not in missing]

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
