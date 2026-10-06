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

import os
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
# A verdict BETTER than the one a rung is graded on. gui32's 0x600E means "drew,
# but GetPixel readback mismatched" -- the soft pass it earned when desktop
# readback did not work. Once readback started working it exited 0x600D, a full
# PASS, and an exact-match grade failed the rung for improving: LADDER_FAIL
# "gui32.exe exit = 0x600d (want 0x600e)" on a tree whose gui32 log said
# "PASS: 32-bit GDI drew on the host desktop and read it back". Its source never
# changed; the OS got better. The graded value stays 0x600E so an arch where
# readback still mismatches stays green; this names the upgrade.
ALSO_ACCEPTED: dict[str, frozenset[int]] = {
    "gui32": frozenset({0x600D}),
}


def verdict_ok(guest: str, got: int, want: int) -> bool:
    """`got` satisfies a rung graded `want`: exactly, or a named upgrade."""
    return got == want or got in ALSO_ACCEPTED.get(guest, frozenset())


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
    # Promoted from DIAGNOSTIC 2026-09-19, the day it first reported 0x600D.
    # It was built to MEASURE guest CreateProcess while that gap was open; a
    # diagnostic guest that starts passing has finished its job, and leaving it
    # ungraded would mean a regression in the thing it proved could go unnoticed.
    "spawn32": 0x600D,
    # The first x86-64 guest (BTCpu64 M1, docs/engineering/wow64-x64-guest-design.md
    # Fork B). Parity in the sense this table means, one width up: the SAME binary
    # runs under FEX in 64-bit mode on arm64 and natively on amd64. Not a WoW64
    # guest, so it stages to system32 rather than syswow64 -- every check here is
    # keyed by name, and media_has_guests needs only SOME guest found, so the
    # different directory costs nothing.
    "hello64": 0x600D,
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

# DIAGNOSTIC guests measure a gap we KNOW is open. They report a verdict and
# never fail a gate -- grading one would redden every lane in the fleet for a
# hole we already have a task for, which is how a known gap turns into a
# disabled check. They are graded by a HUMAN reading the summary line.
#
# The value is the PASS code, not the failure code we currently expect. Writing
# today's failure in here would enshrine it: the guest would then go quiet on
# the day the failure mode CHANGES, which is exactly the day worth hearing
# about. Instead `summary` prints whatever code came back, and shouts if it is
# PASS -- because a diagnostic guest that starts passing means the gap closed
# and the guest should be promoted into LADDER, where a regression can red it.
#
# A diagnostic guest must be cheap to abandon: it shares the ladder's drain
# budget, and one guest that hangs has cost four later ones their verdicts
# before now (marshal32, six days stale, ate 420s). spawn32 caps its own wait
# at 30s for that reason.
# Empty, and that is a good state rather than an unused mechanism: the machinery
# below is what let spawn32 measure an open gap for a day without reddening any
# lane's gate, and the x86-64 ladder went through it on the way to STAGED_LADDER.
DIAGNOSTIC: dict[str, int] = {}

# The x86-64 ladder (BTCpu64 M2, docs/engineering/wow64-x64-guest-design.md):
# the 32-bit ladder's own sources rebuilt for x86-64, run under FEX on arm64
# and natively on amd64. GRADED WHEN STAGED: a guest the media carries must
# report PASS, exactly like a LADDER guest; one it does not carry is not owed.
#
# Not in LADDER itself, because LADDER is owed by EVERY image that runs the
# ladder, and these need a whole second build to exist at all: their import
# libraries and their guest DLL set (arm64's sysamd64) come from build-amd64.
# An arm64-only tree builds hello64 (no imports) and none of these, and
# mkramdisk.sh stages none of them unless it staged sysamd64 first -- so
# "present on the media" already means "runnable", and grading by media is
# the whole guarantee. All ten passed under FEX on arm64 and natively on amd64
# on 2026-10-06 (the run dirs are in the design doc).
STAGED_LADDER: dict[str, int] = {
    g: 0x600D
    for g in (
        "gui64", "win64", "file64", "async64", "marshal64", "fault64",
        "thread64", "selfmod64", "reg64", "spawn64",
    )
}
# Where the x86-64 guests are staged on both arches: an x86-64 exe is not a
# WoW64 guest, so system32, not syswow64.
X64_STAGED_DIR = "system32"

# A diagnostic guest that can BUGCHECK must be kept off the media entirely, not
# merely left ungraded, and spawn32 is where that was learned. For one day it
# was both: a 32-bit CreateProcessW took the kernel down with a 0x1E
# (c0000005) AFTER NtCreateProcessEx had already succeeded, and "report, never
# red" buys nothing against a dead machine -- it reds the boot for every lane
# in the fleet. The answer was to not stage it at all unless asked (built under
# VIBEOS_WOW64_SPAWN=1, staged `if(EXISTS)`, run last of all guests so its
# crash could not cost anyone else a verdict).
#
# All of that is history now: the bugcheck was a dropped _SEH2_EXCEPT, the gap
# it measured is closed, and spawn32 is an ordinary parity guest built and
# staged with the rest. Kept written down because the next diagnostic guest
# aimed at a crashing gap needs the same three precautions -- and because the
# ordering rule INVERTS on promotion: a graded guest must run BEFORE sqlite32,
# never after it.


def staged_guests() -> list[str]:
    """The guests CMake stages UNCONDITIONALLY (`_wow64_ladder`).

    Only LADDER. That set is all-or-nothing (`_wow64_guest_ok`), so a missing
    member costs every other guest its staging -- which is why absence is a
    hole here and merely a skip for the opt-in guests. DIAGNOSTIC is
    deliberately not included: it is staged `if(EXISTS)` like the stress
    guests, so an absent spawn32 must cost the parity set nothing."""
    return list(LADDER)


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

# A Wine conformance subtest winlogon has started: TryStartWinetest gives each
# one 180s before it kills it and prints TIMED OUT, and runs them one after
# another. check() FAILS a subtest that started and never reported (see
# winetests_lost), so the drain has to wait for it. It did not, and the gate
# went red on a guest that reported cleanly 8 lines after the drain stopped.
WINETEST_EXTRA_SECONDS = 200.0

# Env override: keep draining for at least this many seconds even after every
# graded guest has reported.
#
# The drain stops the moment the graded set is complete, which is right for a
# gate and WRONG for an investigation: the STOCK applets (winmine, notepad,
# mspaint, wordpad, ...) run AFTER the parity set and are not graded at all, so
# nothing in the normal path ever waits for them. A `--ladder` run therefore
# ends at ~87s with the applet phase entirely unobserved -- which is why the
# 58-fault execute-at-NULL cascade those three take could be recorded in the
# bug registry and then not reproduced by the very flag meant to watch the
# ladder. This makes that phase reachable without hand-rolling a QEMU boot.
#
# It only ever EXTENDS the drain, never shortens it, so it cannot turn a slow
# host into a fake failure. Unset means the old behaviour exactly.
DRAIN_FLOOR_ENV = "VIBEOS_LADDER_DRAIN_FLOOR"


def drain_floor() -> float:
    """Seconds from DRAIN_FLOOR_ENV; 0.0 when unset.

    An unparseable or negative value is a HARD ERROR rather than a silent 0:
    the whole point of setting it is to observe a phase that is invisible
    without it, so quietly ignoring a typo would hand back a run that looks
    like it watched and did not.
    """
    raw = os.environ.get(DRAIN_FLOOR_ENV)
    if raw is None or raw == "":
        return 0.0
    try:
        val = float(raw)
    except ValueError:
        raise ValueError(f"{DRAIN_FLOOR_ENV}={raw!r} is not a number of seconds") from None
    if val < 0:
        raise ValueError(f"{DRAIN_FLOOR_ENV}={raw!r} must not be negative")
    return val


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
    "spawn32": "tools/fex/hello-compute/Spawn32.c",
    "hello64": "tools/fex/hello-compute/Hello64.c",
    "gui64": "tools/fex/hello-compute/Gui32.c",
    "win64": "tools/fex/hello-compute/Win32App.c",
    "file64": "tools/fex/hello-compute/File32.c",
    "async64": "tools/fex/hello-compute/Async32.c",
    "marshal64": "tools/fex/hello-compute/Marshal32.c",
    "thread64": "tools/fex/hello-compute/Thread32.c",
    "fault64": "tools/fex/hello-compute/Fault32.c",
    "selfmod64": "tools/fex/hello-compute/Selfmod32.c",
    "reg64": "tools/fex/hello-compute/Reg32.c",
    "spawn64": "tools/fex/hello-compute/Spawn32.c",
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
    for guest in staged_guests():
        exe = repo / "build-fex-hello" / f"{guest}.exe"
        if not exe.is_file():
            continue
        rel = GUEST_SOURCE.get(guest)
        if rel is None:
            out.append(f"{guest} (no source mapping)")
            continue
        out.extend(_newer_than(repo, guest, exe, [rel, GUEST_BUILDER]))

    # DIAGNOSTIC guests are opt-in (VIBEOS_WOW64_SPAWN=1), so absent is the
    # normal state and must not read as a hole -- but a built one is compared
    # exactly like a parity guest, because it lives in build-fex-hello/ and
    # goes stale the same way.
    for guest in [*DIAGNOSTIC, *STAGED_LADDER]:
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


# What the guest builder links against. Every graded guest imports kernel32, and
# the builder skips a guest (it does not fail) when its importlib is missing -- so
# without this probe a tree with no build-i386 would "refresh" into a partial set,
# which is the all-or-nothing hole `gate_decision` reports.
GUEST_PREREQ = "build-i386/dll/win32/kernel32/CMakeFiles/libkernel32.dir/libkernel32.a"
GUEST_TOOLCHAIN = ".build-tools/llvm-mingw/bin/i686-w64-mingw32-clang"


def guests_to_refresh(repo: Path) -> list[str]:
    """Graded guests the builder must (re)make: absent, or older than a source.

    Only the set the builder owns -- LADDER + DIAGNOSTIC in build-fex-hello/.
    The OPTIONAL stress guests come from their own fetching build.sh and are
    deliberately NOT built on anyone's behalf.
    """
    out: list[str] = []
    stale = stale_guests(repo)
    for guest in [*LADDER, *DIAGNOSTIC, *STAGED_LADDER]:
        exe = repo / "build-fex-hello" / f"{guest}.exe"
        if guest in LADDER and not exe.is_file():
            out.append(f"{guest} (absent)")
        else:
            out.extend(s for s in stale if s.startswith(f"{guest} ("))
    return out


def refresh_blocker(repo: Path) -> str | None:
    """Why the guests cannot be built in this tree, or None if they can."""
    if not (repo / GUEST_TOOLCHAIN).is_file():
        return f"no i386 toolchain at {GUEST_TOOLCHAIN}"
    if not (repo / GUEST_PREREQ).is_file():
        return (
            "no build-i386 importlibs to link the guests against — "
            "build i386 first: ./vibeos-cli build --arch i386"
        )
    return None


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
    # DIAGNOSTIC included: it is not unconditionally staged, but when it IS
    # on the media the probe must see it.
    return {f"{g}.exe".lower() for g in [*LADDER, *DIAGNOSTIC]}


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


def uses_ramdisk(arch: str) -> bool:
    """Does this arch stage into the FAT ramdisk rather than an assembled ISO?

    Asked of the ARCH, never of the disk. `RAMDISK_REL` is a fixed path with no
    arch in it, so "does qemu/ramdisk.img exist" is not the same question and
    answering with it makes arm64's media speak for amd64. That substitution
    has now produced two separate bugs from one root -- a probe that read
    arm64's ramdisk on amd64's behalf, and a restage remedy that told an amd64
    tree to run `VIBEOS_FEX=1 build --arch arm64` -- so the derivation lives
    here once instead of at each site that needs it.
    """
    from . import archcfg

    try:
        media = Path(archcfg.arch_build(arch).boot_media_rel)
    except (KeyError, AttributeError):
        return False
    return not (media.parts and media.parts[0].startswith("build-"))


def _staged_per_ramdisk(repo: Path, arch: str, subdir: str, names: set[str]) -> set[str] | None:
    """arm64 media, filled by mcopy. None = cannot tell (no image, no mtools).

    Reads stdout, never the exit code: `mdir` exits 0 on "File not found", which
    is the same $?-is-not-the-answer trap AGENTS.md names twice.

    Declines for an arch that does not USE a ramdisk, and that is not a
    formality. The image lives at a fixed repo path with no arch in its name, so
    this probe used to read arm64's ramdisk and answer on behalf of amd64. It
    was invisible while a tree had a build-amd64/ manifest -- that probe runs
    first and answers -- and surfaced on a freshly-restored box where the only
    media present was a two-week-old qemu/ramdisk.img: amd64 was told "no WoW64
    CPU layer on this arch's media", sourced entirely from arm64's media.
    Derived from archcfg rather than an arch list: x86 media is an ISO assembled
    under build-<arch>/, arm64's is mcopy-filled and lives outside it, and that
    IS the mechanism split this function's None contract is about.
    """
    if not uses_ramdisk(arch):
        return None  # assembled ISO -- the manifest is this arch's mechanism

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


def supports_wow64(repo: Path, arch: str) -> bool | None:
    """Is a WoW64 CPU layer on this media? False on a native-x86 livecd, which is
    why 'no guests staged' is the final answer there rather than a fixable state.

    None when the media cannot answer -- no build dir, no manifest, no ramdisk.
    Tri-state on purpose: `bool()` here would have thrown away the distinction
    that `_read_media` and both probes exist to preserve, and turned "I have not
    looked at any media" into the positive architectural claim "32-bit guests
    cannot run here". That is the reassuring direction to be wrong in, so nothing
    downstream would ever have questioned it -- an unbuilt amd64 tree would be
    told the ladder does not apply to amd64.
    """
    answer = _read_media(repo, arch, "system32", CPU_LAYERS)
    return None if answer is None else bool(answer)


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
    found = _read_media(repo, arch, STAGED_DIR, {f"{g}.exe" for g in OPTIONAL}) or set()
    staged = sorted(g for g in OPTIONAL if f"{g}.exe".lower() in found)
    # The x86-64 set is graded by the same rule -- present means owed -- and
    # read the same way, from its own directory.
    x64 = _read_media(repo, arch, X64_STAGED_DIR, {f"{g}.exe" for g in STAGED_LADDER}) or set()
    staged += sorted(g for g in STAGED_LADDER if f"{g}.exe".lower() in x64)
    return staged


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
            f"{len(LADDER)} parity + {len(DIAGNOSTIC)} diagnostic WoW64 guests staged — "
            f"boot watched past the desktop marker",
        )
    # A native-x86 boot has no CPU layer, so it has no SysWOW64 to stage into and
    # "not staged" is the final answer, not a fixable state. Checked by asking the
    # media for the layer winlogon itself gates on, so no arch list appears here.
    cpu_layer = supports_wow64(repo, arch)
    if cpu_layer is None:
        return (
            [],
            False,
            f"no media to read for {arch} — neither a build-{arch}/ manifest nor a "
            f"ramdisk this arch uses, so whether it can host 32-bit guests is UNKNOWN, "
            f"not no; build the arch first: ./vibeos-cli build --arch {arch}",
        )
    if not cpu_layer:
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
    built = sorted(g for g in staged_guests() if (repo / "build-fex-hello" / f"{g}.exe").is_file())
    # Partially built is its own state, and restaging cannot fix it. The CMake
    # ladder list is all-or-nothing (`_wow64_guest_ok`), so ONE missing guest
    # keeps every other one off the media -- the same failure doom caused once
    # by being absent. A tree that predates a newly added guest lands here, and
    # telling it to restage would send it round a build that changes nothing.
    if built and len(built) < len(staged_guests()):
        absent = ", ".join(f"{g}.exe" for g in staged_guests() if g not in built)
        return (
            [],
            False,
            f"{len(built)}/{len(staged_guests())} ladder guests built — {absent} missing, and "
            f"staging is all-or-nothing, so NOTHING is staged; build the guests with: "
            + (
                f"{GUEST_BUILDER}"
                if uses_ramdisk(arch)
                else f"./vibeos-cli build --arch {arch} (it runs {GUEST_BUILDER})"
            ),
        )
    if built:
        fix = (
            "VIBEOS_FEX=1 ./vibeos-cli build --arch arm64"
            if uses_ramdisk(arch)
            # The build restages by itself now: the guests are configure
            # dependencies, so the next build reconfigures on their creation.
            else f"./vibeos-cli build --arch {arch}"
        )
        return (
            [],
            False,
            f"{len(built)}/{len(staged_guests())} guests are BUILT but none is staged on this "
            f"arch's media — staging is snapshotted when the media is assembled; "
            f"restage with: {fix}",
        )
    return (
        [],
        False,
        f"no staged {STAGED_DIR}/ guest in this arch's media manifest — "
        f"WoW64 guests NOT checked by this gate"
        + ("" if uses_ramdisk(arch) else " (the amd64 build's wow64-guests stage says why)"),
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


# winlogon prints this once, after the last GRADED guest or winetest on its arch
# and before the ungraded stock-applet phase (winlogon.rs LadderProgramComplete).
PROGRAM_DONE = "[WINLOGON] graded program complete"


def watch(
    vm,
    seconds: float = LADDER_TIMEOUT_SECONDS,
    seen: str = "",
    optional_staged: list[str] | None = None,
    program_done_expected: bool = False,
) -> str:
    """Drain serial until every ladder guest has reported, or `seconds` elapse.

    `seen` is the serial the caller has ALREADY consumed. Without it this waits
    for markers that have gone past: the boot's own settle phase routinely reads
    a guest's verdict before this is called, and that one line is enough to make
    the loop sit out the entire ceiling. MEASURED -- a 9/9 PASS run drained
    420.2s of a 420s budget because `gui32.exe exit = 0x600e` landed in the settle
    read, so the verdict was correct and the wait was pure waste. The caller's
    `check()` sees both halves and never noticed.

    `optional_staged` is the SAME media answer `check()` grades with (see
    media_optional_staged), and the drain must wait on it for the same reason
    check() enforces it. An optional guest launches right AFTER the last parity
    guest reports, so with the serial as its only source the drain saw every
    required verdict, saw no optional guest yet, and stopped in the few lines
    between -- then check() read the staged, launched, unreported lua32 as a
    crash. MEASURED: spawn32's verdict at serial line 3214, lua32's launch at
    3217, a 73s drain against a 420s ceiling, LADDER_FAIL on a guest mid-run.
    The two halves of the ladder disagreed about what "done" means.

    `program_done_expected` is the real answer to that, and the others are its
    backstops. Inferring "done" from nothing being in flight is a race at EVERY
    launch -- after lua32 was fixed, the same gap reopened between sqlite32 and
    the winetest sweep. winlogon now SAYS when it is done (PROGRAM_DONE), and a
    caller booting media this tree built -- which smoke guarantees by refusing
    stale media -- waits for that line. The ceiling still bounds a machine that
    never gets there.

    Returns only the NEWLY drained text; the caller appends it to the boot serial,
    so a BUGCHECK during the ladder still lands in full.log.
    """
    floor = drain_floor()
    need = [m for m in markers() if m not in seen]
    staged = list(optional_staged or [])

    def outstanding(text: str) -> list[str]:
        # Launched-and-unreported, plus staged-and-unreported: a staged guest
        # is owed a verdict whether or not its launch line has appeared yet.
        out = _optional_in_flight(text)
        out += [g for g in staged if f"{g}.exe exit = " not in text and g not in out]
        return out

    if not floor and (
        (program_done_expected and PROGRAM_DONE in seen)
        or (
            not program_done_expected
            and not need
            and not outstanding(seen)
            and not winetests_lost(seen)
            and not sweeps_unfinished(seen)
        )
    ):
        return ""  # every guest already reported before we were called
    buf = ""
    start = time.monotonic()
    deadline = start + max(seconds, floor)
    # A launched optional guest gets its own extension on top of the parity
    # budget: it is third-party code doing real work under emulation, so it is
    # slow by nature, and charging it to the parity budget is what turned a
    # green ladder into a fake selfmod32 failure the first time one was added.
    # A started winetest is the same case: check() grades it, so the drain must
    # not end while one is still running.
    extended: set[str] = set()
    while time.monotonic() < deadline:
        buf += vm.serial.read_available(timeout=0.5) or ""
        # `seen + buf`, as winetests_lost below: a guest launched during the
        # settle read is in `seen`, and is still in flight.
        pending = outstanding(seen + buf)
        running = winetests_lost(seen + buf)
        sweeping = sweeps_unfinished(seen + buf)
        # Once winlogon has SAID it is done, nothing graded can still arrive:
        # a guest missing at that point is final, and check() grades it. So the
        # line ends the drain by itself -- an i386 native sweep, which has no
        # WoW64 guests to report, otherwise sat out the whole 420s ceiling
        # after winlogon had finished.
        if program_done_expected:
            finished = PROGRAM_DONE in seen + buf
        else:
            finished = all(m in buf for m in need) and not pending and not running and not sweeping
        if finished and time.monotonic() - start >= floor:
            break
        # The enumeration run gets one subtest's allowance; each subtest it
        # then starts earns its own below, so a sweep's budget grows with it.
        for suite in sweeping:
            key = f"{suite}:--list"
            if key not in extended:
                deadline += WINETEST_EXTRA_SECONDS
                extended.add(key)
        for g in pending:
            if g not in extended:
                deadline += OPTIONAL_EXTRA_SECONDS
                extended.add(g)
        for label in running:
            if label not in extended:
                deadline += WINETEST_EXTRA_SECONDS
                extended.add(label)
    return buf


def verdicts(text: str) -> dict[str, int]:
    """Last reported exit code per guest (a guest is launched once, but take the
    last match so a re-run in a longer boot cannot be masked by an earlier one)."""
    out: dict[str, int] = {}
    for name, code in _VERDICT.findall(text):
        out[name] = int(code, 16)
    return out


# [WT] user32:class: started (PID 123)  /  [WT] user32:class: exit = 4 failure(s), ...
# The label is <suite>:<subtest>, written by winlogon's TryStartWinetest, and
# carries a `native/` prefix for the bare-i386 control run of the same binary --
# the pair is what makes a red assertion attributable to the WoW64 seam rather
# than to user32.
_WT_STARTED = re.compile(r"\[WT\] ([A-Za-z0-9_/]+:[A-Za-z0-9_]+): started ")
_WT_VERDICT = re.compile(r"\[WT\] ([A-Za-z0-9_/]+:[A-Za-z0-9_]+): exit = (\d+) failure")
_WT_CRASH = re.compile(r"\[WT\] ([A-Za-z0-9_/]+:[A-Za-z0-9_]+): CRASHED, exit = (0x[0-9a-fA-F]+)")
_WT_TIMEOUT = re.compile(r"\[WT\] ([A-Za-z0-9_/]+:[A-Za-z0-9_]+): TIMED OUT,")


def winetests(text: str) -> dict[str, int]:
    """Failure count per Wine conformance subtest that reported.

    A winetest's exit code IS its failure count, so 0 means the suite agreed
    with real Windows on every assertion it made."""
    return {label: int(n) for label, n in _WT_VERDICT.findall(text)}


def winetests_crashed(text: str) -> dict[str, str]:
    """Subtests that died on an unhandled exception, and the NTSTATUS.

    Distinct from a failure count on purpose: a count is the oracle's verdict
    on the OS, a crash is the OS taking the oracle down with it and leaving
    every later assertion unrun. Only the second is a verdict about THIS run."""
    return dict(_WT_CRASH.findall(text))


def winetests_timed_out(text: str) -> list[str]:
    """Subtests we killed. A third outcome, kept apart from the other two for
    the same reason: the exit code is OURS (0xDEAD), so counting it as failures
    reports 57005 of them."""
    return _WT_TIMEOUT.findall(text)


def winetests_lost(text: str) -> list[str]:
    """Subtests that STARTED and never reported.

    This, and not the failure count, is what the ladder gates on. The counts
    are the oracle grading US: they start non-zero by construction and go down
    as the OS improves, so gating on zero would paint the ladder permanently
    red and teach everyone to ignore it. A subtest that vanishes mid-run has
    crashed or hung the guest, which is a regression in OUR code and is the
    same signal every other rung here carries."""
    done = set(winetests(text)) | set(winetests_crashed(text)) | set(winetests_timed_out(text))
    return [label for label in _WT_STARTED.findall(text) if label not in done]


# `[WT] user32:--list: started` opens a sweep; winlogon closes it with
# `[WT] user32: sweep complete, ...` or `[WT] user32: --list produced no subtest
# names - suite NOT swept`. The `--list` label is kept out of _WT_STARTED on
# purpose (it is bookkeeping, not a subtest), so nothing else sees a sweep run.
_SWEEP_STARTED = re.compile(r"\[WT\] ([A-Za-z0-9_/]+):--list: started ")
_SWEEP_DONE = re.compile(r"\[WT\] ([A-Za-z0-9_/]+): (?:sweep complete|--list produced no subtest)")


def sweeps_unfinished(text: str) -> list[str]:
    """Suites whose SWEEP started and has not said it finished.

    Without this a sweep boot did not wait for its own sweep. The drain ends when
    nothing graded is outstanding, and while `--list` runs -- and in the gap
    between one subtest's verdict and the next one's start line -- nothing graded
    is. MEASURED: the serial stopped mid-line during `user32:--list` and the boot
    reported green having run zero subtests. The workaround was to remember to
    export VIBEOS_LADDER_DRAIN_FLOOR near 2400, which is a recipe, not a tool."""
    done = set(_SWEEP_DONE.findall(text))
    return [s for s in _SWEEP_STARTED.findall(text) if s not in done]


def check(text: str, optional_staged: list[str] | None = None) -> dict | None:
    """None if every ladder guest reported PASS; else a failure dict with the
    same {classification, salient} shape the smoke triage stages use.

    `optional_staged` names OPTIONAL guests the MEDIA carries (see
    media_optional_staged). Those are then required to report, because a staged
    guest that never reports has hung -- which is exactly what a stress guest is
    for and must not read as absent."""
    seen = verdicts(text)
    missing = [g for g in LADDER if g not in seen]
    wrong = [
        (g, seen[g], want)
        for g, want in LADDER.items()
        if g in seen and not verdict_ok(g, seen[g], want)
    ]
    # An optional guest is ignored when absent and enforced when present: the
    # reason to build one is to find out whether it passes, so a red one that
    # got staged must not be silently tolerated.
    wrong += [(g, seen[g], want) for g, want in OPTIONAL.items() if g in seen and seen[g] != want]
    wrong += [(g, seen[g], want) for g, want in STAGED_LADDER.items() if g in seen and seen[g] != want]
    # Staged but never reported is NOT the same as absent, and the serial cannot
    # settle it: an optional guest runs after the parity set, so the drain can
    # end before its launch line appears. Ask the media instead. (The in-flight
    # check still catches one that launched inside the drained window.)
    for g in optional_staged or []:
        if g not in seen and g not in missing:
            missing.append(g)
    missing += [g for g in _optional_in_flight(text) if g not in missing]

    lost = winetests_lost(text)
    # A sweep that never finished lost every subtest after the point it died,
    # and only this can say so: those subtests never printed a start line.
    lost += [f"{suite}:--list" for suite in sweeps_unfinished(text)]

    if not missing and not wrong and not lost:
        return None

    salient: list[str] = []
    for label in lost:
        salient.append(f"winetest {label} started and never reported — the guest crashed or hung")
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
    ok = sum(1 for g, want in LADDER.items() if g in seen and verdict_ok(g, seen[g], want))
    extra = [g for g in seen if g not in LADDER and g not in DIAGNOSTIC and g not in STAGED_LADDER]
    x64 = [g for g in STAGED_LADDER if g in seen]
    x64_ok = sum(1 for g in x64 if seen[g] == STAGED_LADDER[g])
    tail = f" (+{len(extra)} non-parity: {', '.join(sorted(extra))})" if extra else ""
    wt = {f"{k}={v}" for k, v in winetests(text).items()}
    wt |= {f"{k}=CRASH({v})" for k, v in winetests_crashed(text).items()}
    wt |= {f"{k}=TIMEOUT" for k in winetests_timed_out(text)}
    wt_tail = ""
    if wt:
        # Reported, never gated — see winetests_lost() for why the failure
        # count is information and not a verdict.
        wt_tail = "; winetests " + ", ".join(sorted(wt))
    x64_tail = f"; x86-64 ladder {x64_ok}/{len(x64)} PASS" if x64 else ""
    return f"{ok}/{len(LADDER)} guests PASS{x64_tail}{tail}{_diagnostic_tail(seen)}{wt_tail}"


def _diagnostic_tail(seen: dict[str, int]) -> str:
    """The diagnostic guests' codes, appended to the roll-up. Never a verdict.

    A guest that came back PASS is called out in words rather than left as a
    hex code among others: it means the gap it measures has closed, and the
    next move is to promote it into LADDER so a regression can red it again. A
    silent 0x600d here would be a result nobody reads."""
    bits: list[str] = []
    for g, want in sorted(DIAGNOSTIC.items()):
        if g not in seen:
            continue
        got = seen[g]
        bits.append(
            f"{g} PASSES (0x{got:04x}) — gap closed, promote it to LADDER"
            if got == want
            else f"{g}=0x{got:04x}"
        )
    return f"; diagnostic: {', '.join(bits)}" if bits else ""
