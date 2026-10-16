"""Ladder verdict + gate-policy tests.

The point of the ladder is that a WoW64 guest can be red while every gate stays
green, so these lean on the two ways that silence can be produced: a guest that
ran and failed, and a media that never carried a guest at all. Conflating them is
the bug -- one is a regression, the other is a lane that was never staged.
"""

from __future__ import annotations

import os
import shutil
import subprocess
import time
from pathlib import Path

import pytest

from tests.qemu.cli import ladder


def _serial(**guests: int) -> str:
    return "".join(f"[WINLOGON] {g}.exe exit = 0x{c:04x}\n" for g, c in guests.items())


def _all_pass() -> str:
    return _serial(**ladder.LADDER)


def _red(text: str) -> dict:
    """The failure dict, asserting there IS one — so a test that means to check a
    red verdict cannot quietly pass against a green ladder."""
    got = ladder.check(text)
    assert got is not None, "expected a ladder failure, got a clean pass"
    return got


# --- verdicts ---------------------------------------------------------------


def test_every_guest_passing_is_the_only_green():
    assert ladder.check(_all_pass()) is None


def test_a_red_rung_names_the_guest_and_both_codes():
    text = _all_pass().replace("thread32.exe exit = 0x600d", "thread32.exe exit = 0xc0000001")
    got = _red(text)
    assert got["classification"] == "LADDER_FAIL"
    assert any("thread32" in s and "0xc0000001" in s and "0x600d" in s for s in got["salient"])


def test_gui32_readback_mismatch_is_red():
    """gui32's soft pass 0x600E (drew, GetPixel readback mismatched) was the grade
    while arm64 could not read back a 32-bit draw; it is a regression now."""
    others = {g: c for g, c in ladder.LADDER.items() if g != "gui32"}
    assert _red(_serial(**others) + _serial(gui32=0x600E))["classification"] == "LADDER_FAIL"
    assert ladder.check(_all_pass()) is None


def test_x64_guests_are_owed_only_where_the_media_carries_them():
    """STAGED_LADDER: graded when staged. Absent from the media, absent from
    the serial, and the ladder is green -- an arm64-only tree builds none of
    them. Staged and silent is a hang, and red."""
    assert ladder.check(_all_pass()) is None
    got = ladder.check(_all_pass(), optional_staged=["file64"])
    assert got is not None and any("file64" in s for s in got["salient"])
    assert ladder.check(_all_pass() + _serial(file64=0x600D), optional_staged=["file64"]) is None


def test_a_red_x64_guest_is_red_even_unasked():
    """A verdict on the serial is graded whether or not the media read found it:
    the guest ran, so its answer counts."""
    got = ladder.check(_all_pass() + _serial(reg64=0xBAD3))
    assert got is not None and any("reg64" in s and "0xbad3" in s for s in got["salient"])


def test_one_guest_silent_is_a_fail_not_a_staging_report():
    """Partial silence is a regression: the others proved the media was staged."""
    text = "".join(line + "\n" for line in _all_pass().splitlines() if "selfmod32" not in line)
    got = _red(text)
    assert got["classification"] == "LADDER_FAIL"
    assert any("selfmod32" in s for s in got["salient"])


def test_total_silence_is_staging_and_says_how_to_stage():
    got = _red("[WINLOGON] explorer started\n")
    assert got["classification"] == "LADDER_NOT_STAGED"
    assert any("VIBEOS_FEX=1" in s for s in got["salient"])


def test_a_later_verdict_wins_over_an_earlier_one():
    """Guests are launched once, but a longer boot must not let a stale PASS mask
    a re-run's failure."""
    text = _all_pass() + _serial(hello32=0xDEAD)
    assert ladder.verdicts(text)["hello32"] == 0xDEAD
    assert _red(text)["classification"] == "LADDER_FAIL"


def test_non_parity_guests_are_reported_but_not_required():
    """arm64 stages fexprobe/wow64probe; they must not become a parity demand."""
    text = _all_pass() + _serial(fexprobe=0x600D)
    assert ladder.check(text) is None
    assert "non-parity" in ladder.summary(text)
    assert "fexprobe" in ladder.summary(text)


# --- gate policy ---------------------------------------------------------


def _manifest(
    tmp_path: Path, lines: list[str], arch: str = "amd64", cpu_layer: bool = True
) -> Path:
    """A repo whose build-<arch>/boot/livecd.cmake.lst holds these entries."""
    lst = tmp_path / f"build-{arch}" / "boot" / "livecd.cmake.lst"
    lst.parent.mkdir(parents=True, exist_ok=True)
    if cpu_layer:
        lines = [*lines, "reactos/system32/wow64cpu.dll=/tmp/vibeos/x/wow64cpu.dll"]
    lst.write_text("\n".join(lines) + "\n")
    return tmp_path


# A real manifest line: "<cd path>=<source path>".
_SRC = "/tmp/vibeos/build-fex-hello/hello32.exe"


def test_gate_asks_for_the_ladder_when_a_guest_is_staged(tmp_path):
    repo = _manifest(tmp_path, [f"reactos/syswow64/hello32.exe={_SRC}"])
    extra, staged, why = ladder.gate_decision(repo, "amd64")
    assert extra == ["--ladder"]
    assert staged
    assert str(len(ladder.LADDER)) in why


def test_a_media_that_merely_MENTIONS_a_guest_does_not_count(tmp_path):
    """The regression that motivated reading the manifest instead of scanning the
    image. `hello32.exe` appears in EVERY amd64 livecd — winlogon's own
    diagnostic carries it in plain ASCII ("CreateProcessW(hello32.exe) FAILED")
    — so a byte-scan says PRESENT on a boot with zero staged guests. Measured on
    build-amd64 at 0x9c2ffe4, and it passed a boot that then reported not a
    single guest verdict."""
    repo = _manifest(
        tmp_path,
        [
            "reactos/system32/winlogon.exe=/tmp/vibeos/build-amd64/base/system/"
            "winlogon/winlogon.exe",
            "reactos/system32/wow64cpu.dll=/tmp/vibeos/build-amd64/dll/win32/wow64cpu/wow64cpu.dll",
        ],
    )
    extra, staged, why = ladder.gate_decision(repo, "amd64")
    assert extra == []
    assert not staged
    assert "NOT checked" in why


def test_a_guest_outside_syswow64_is_not_a_staged_guest(tmp_path):
    """Only the path winlogon actually launches from counts."""
    repo = _manifest(tmp_path, [f"reactos/system32/hello32.exe={_SRC}"])
    assert ladder.gate_decision(repo, "amd64")[1] is False


def test_a_partial_stage_opts_IN_so_the_gap_cannot_pass_quietly(tmp_path):
    """One guest staged is enough: the boot then reports the other eight as
    missing and check() turns that into a LADDER_FAIL naming them."""
    repo = _manifest(tmp_path, [f"reactos/syswow64/gui32.exe={_SRC}"])
    assert ladder.gate_decision(repo, "amd64")[1] is True


def test_no_manifest_declines_rather_than_deciding_the_gate(tmp_path):
    """A tree that was never configured has no evidence either way."""
    extra, staged, _ = ladder.gate_decision(tmp_path, "amd64")
    assert extra == [] and not staged


def test_native_x86_needs_no_special_case(tmp_path):
    """An i386 livecd has no SysWOW64 to stage into, so the same read declines
    without the policy ever naming an arch."""
    repo = _manifest(
        tmp_path,
        ["reactos/system32/ntoskrnl.exe=/tmp/vibeos/build-i386/ntoskrnl/ntoskrnl.exe"],
        arch="i386",
        cpu_layer=False,
    )
    assert ladder.gate_decision(repo, "i386")[1] is False


# --- the drain ------------------------------------------------------------


class _FakeSerial:
    """Yields one chunk per read, then nothing."""

    def __init__(self, chunks: list[str]):
        self.chunks = list(chunks)
        self.reads = 0

    def read_available(self, timeout: float = 0.5) -> str:
        self.reads += 1
        return self.chunks.pop(0) if self.chunks else ""


class _FakeVM:
    def __init__(self, chunks: list[str]):
        self.serial = _FakeSerial(chunks)


def test_a_guest_already_seen_is_not_waited_for_again():
    """The measured bug: the boot's settle read consumed gui32's verdict, so the
    drain sat out its whole 420s budget waiting for a marker that had gone past
    — on a run where all nine PASSED. `seen` is what stops that."""
    vm = _FakeVM([_serial(**{g: c for g, c in ladder.LADDER.items() if g != "gui32"})])
    out = ladder.watch(vm, seconds=5, seen=_serial(gui32=0x600D))
    assert ladder.check(out + _serial(gui32=0x600D)) is None
    assert vm.serial.reads == 1, "should stop the moment the remaining guests report"


def test_all_guests_already_seen_drains_nothing_at_all():
    vm = _FakeVM([])
    assert ladder.watch(vm, seconds=5, seen=_all_pass()) == ""
    assert vm.serial.reads == 0, "no read at all when nothing is outstanding"


def test_a_staged_optional_guest_is_waited_for_before_its_launch_line():
    """The measured race: the last parity verdict arrives, the optional guest's
    launch line has not been printed yet, and a serial-only drain stops in that
    gap. check() -- which asks the media -- then fails lua32 while it is running.
    With the media answer the drain keeps reading until lua32 reports."""
    vm = _FakeVM(
        [
            _all_pass(),
            "[WINLOGON] lua32.exe (i386 Lua stress guest) created (PID 944) - waiting\n",
            "[WINLOGON] lua32.exe exit = 0x600d (0x600D = PASS)\n",
        ]
    )
    out = ladder.watch(vm, seconds=5, seen="", optional_staged=["lua32"])
    assert "lua32.exe exit = " in out
    assert ladder.check(out, optional_staged=["lua32"]) is None


def test_an_optional_guest_launched_during_settle_is_still_in_flight():
    """The settle read consumed lua32's LAUNCH line; the drain must still wait
    for its verdict rather than looking only at the newly drained text."""
    vm = _FakeVM([_all_pass(), "[WINLOGON] lua32.exe exit = 0x600d (0x600D = PASS)\n"])
    seen = "[WINLOGON] lua32.exe (i386 Lua stress guest) created (PID 944) - waiting\n"
    out = ladder.watch(vm, seconds=5, seen=seen)
    assert "lua32.exe exit = " in out


def test_a_sweep_holds_the_drain_open_until_it_says_complete():
    """The measured bug: the drain ended during `user32:--list` because nothing
    GRADED was outstanding, and the boot went green having run zero subtests.
    Every chunk below ends at a point where the old drain would have stopped."""
    vm = _FakeVM(
        [
            _all_pass() + "[WT] user32:--list: started (PID 132)\n",
            "[WT] user32:--list: exit = 0 failure(s)\n",
            "[WT] user32:combo: started (PID 140)\n",
            "[WT] user32:combo: exit = 23 failure(s)\n",
            "[WT] user32: sweep complete, 1 subtests, 0 quarantined, 47 not selected\n",
        ]
    )
    out = ladder.watch(vm, seconds=5, seen="")
    assert "sweep complete" in out
    assert ladder.check(out) is None


def test_a_sweep_that_never_completes_is_a_red_ladder():
    """Its unrun subtests never printed a start line, so nothing else can see
    them -- the unfinished sweep itself has to be the finding."""
    text = _all_pass() + "[WT] user32:--list: started (PID 132)\n"
    red = _red(text)
    assert red["classification"] == "LADDER_FAIL"
    assert any("user32:--list" in s for s in red["salient"])


def test_a_sweep_that_listed_nothing_is_finished_not_hung():
    text = (
        _all_pass()
        + "[WT] user32:--list: started (PID 132)\n"
        + "[WT] user32: --list produced no subtest names - suite NOT swept\n"
    )
    assert ladder.sweeps_unfinished(text) == []


def test_the_drain_waits_for_winlogon_to_say_it_is_done():
    """Every graded verdict is in and nothing is in flight -- the exact state the
    old drain stopped in, twice, one launch before the next thing started.
    With the media's winlogon known to print PROGRAM_DONE, that state is not
    'done'; the drain reads on until winlogon says so."""
    vm = _FakeVM(
        [
            _all_pass(),
            "[WT] user32:--list: started (PID 132)\n",
            "[WT] user32: sweep complete, 1 subtests, 0 quarantined, 47 not selected\n",
            ladder.PROGRAM_DONE + "\n",
        ]
    )
    out = ladder.watch(vm, seconds=5, seen="", program_done_expected=True)
    assert out.endswith(ladder.PROGRAM_DONE + "\n")
    assert vm.serial.reads == 4


def test_program_done_ends_the_drain_even_with_guests_missing():
    """i386 has no WoW64 guests, so their markers never arrive. Once winlogon
    says the graded program is over, waiting for them is waiting for nothing --
    the drain stops, and check() reports the absence."""
    vm = _FakeVM(["[WT] native/user32:combo: exit = 23 failure(s)\n", ladder.PROGRAM_DONE + "\n"])
    t0 = time.monotonic()
    ladder.watch(vm, seconds=5, seen="", program_done_expected=True)
    assert time.monotonic() - t0 < 2, "should stop at PROGRAM_DONE, not the ceiling"


def test_program_done_matches_what_winlogon_prints():
    """The two halves are in different languages; pin the text across them."""
    src = (
        Path(__file__).resolve().parents[3] / "sdk/lib/rust/winlogon-rust/src/winlogon.rs"
    ).read_text()
    assert ladder.PROGRAM_DONE + "\\n" in src


def test_a_named_upgrade_is_not_a_red_rung(monkeypatch):
    """The measured false red: gui32's desktop readback started working while it
    was graded on its soft pass, it exited 0x600D, and the ladder failed it for
    passing better. ALSO_ACCEPTED names such upgrades."""
    monkeypatch.setitem(ladder.ALSO_ACCEPTED, "gui32", frozenset({0x600F}))
    others = {g: c for g, c in ladder.LADDER.items() if g != "gui32"}
    assert ladder.check(_serial(**others) + _serial(gui32=0x600F)) is None


def test_gui32_still_fails_on_a_real_error_code():
    others = {g: c for g, c in ladder.LADDER.items() if g != "gui32"}
    text = _serial(**others) + _serial(gui32=0xBAD1)
    assert _red(text)["classification"] == "LADDER_FAIL"


def test_a_guest_that_never_reports_still_uses_the_budget():
    """The ceiling must stay a real ceiling — a silent guest is what it is for."""
    vm = _FakeVM([_serial(hello32=0x600D)])
    out = ladder.watch(vm, seconds=1.0, seen="")
    assert _red(out)["classification"] == "LADDER_FAIL"


def test_each_optional_guest_earns_its_own_drain_extension(monkeypatch):
    """Two optional guests must not share ONE extension.

    winlogon watches each optional guest for 300s independently, so a flat
    one-shot extension covers the first and abandons the second -- which then
    reads as "staged but never reported", i.e. a RED verdict against a guest
    that was merely still running. Assert the deadline grows once PER launched
    guest, not once in total."""
    assert len(ladder.OPTIONAL) >= 2, "this invariant only bites with 2+ optional guests"
    a, b = sorted(ladder.OPTIONAL)[:2]
    # Both launch, neither reports: the loop must grant two extensions before
    # giving up, so it outlives a budget sized for one.
    launched = f"[WINLOGON] {a}.exe (x) created\n[WINLOGON] {b}.exe (x) created\n"
    monkeypatch.setattr(ladder, "OPTIONAL_EXTRA_SECONDS", 0.25)
    vm = _FakeVM([_all_pass() + launched])
    started = time.monotonic()
    ladder.watch(vm, seconds=0.05, seen="")
    waited = time.monotonic() - started
    assert waited >= 0.5, (
        f"drained {waited:.2f}s — two launched optional guests must extend twice "
        "(0.05 base + 2x0.25), not once"
    )


def test_built_but_unstaged_says_so_and_names_the_fix(tmp_path):
    """The 45-minute case: guests on disk, none on the media, because staging is
    snapshotted at configure time and a warm build does not reconfigure. A bare
    "not staged" sends the reader looking for a build that already happened."""
    repo = _manifest(tmp_path, ["reactos/system32/winlogon.exe=/tmp/vibeos/x"])
    guests = repo / "build-fex-hello"
    guests.mkdir()
    for g in ladder.staged_guests():
        (guests / f"{g}.exe").write_bytes(b"MZ")
    extra, staged, why = ladder.gate_decision(repo, "amd64")
    assert extra == [] and not staged
    assert "BUILT but none is staged" in why
    # The remedy must match THIS media's staging mechanism, not the other arch's.
    assert "./vibeos-cli build --arch amd64" in why and "VIBEOS_FEX" not in why
    # No `touch` recipe any more: the guests are configure dependencies, so the
    # plain build restages them. A remedy that still said touch would be a rule
    # with no command behind it.
    assert "touch" not in why


def test_partially_built_names_the_missing_guest_not_a_restage(tmp_path):
    """Adding a guest puts every existing tree in this state for one build, and
    the advice that fits the other two states is actively wrong here: staging is
    all-or-nothing, so restaging a tree that is missing ONE guest changes
    nothing and hides the real cause. ONE absent doom-i386.exe already kept all
    nine guests off the livecd once."""
    repo = _manifest(tmp_path, ["reactos/system32/winlogon.exe=/tmp/vibeos/x"])
    guests = repo / "build-fex-hello"
    guests.mkdir()
    for g in ladder.staged_guests():
        if g != "reg32":
            (guests / f"{g}.exe").write_bytes(b"MZ")
    extra, staged, why = ladder.gate_decision(repo, "amd64")
    assert extra == [] and not staged
    assert "reg32.exe missing" in why
    assert "build-hello-compute.sh" in why
    # Must NOT give the restage advice: the media is not the problem.
    assert "BUILT but none is staged" not in why


# A fixed clock for the staleness fixtures. Every mtime below is an offset from
# it, so the ordering under test is explicit rather than a race with the clock.
_T0 = time.time()


def _all_sources():
    """Every source a staged guest's freshness is dated against."""
    return [*ladder.GUEST_SOURCE.values(), *(h for hs in ladder.GUEST_HELPERS.values() for h in hs)]


def _built_tree(tmp_path, *, sources=True):
    """A repo with a staged manifest, every guest binary, and their sources."""
    repo = _manifest(tmp_path, [f"reactos/syswow64/{g}.exe=/tmp/x" for g in ladder.staged_guests()])
    guests = repo / "build-fex-hello"
    guests.mkdir()
    (repo / "tools" / "fex").mkdir(parents=True)
    (repo / ladder.GUEST_BUILDER).write_text("#!/bin/sh\n")
    if sources:
        for rel in _all_sources():
            src = repo / rel
            src.parent.mkdir(parents=True, exist_ok=True)
            src.write_text("int start(void){return 0;}\n")
    for g in ladder.staged_guests():
        (guests / f"{g}.exe").write_bytes(b"MZ")
    # Every binary newer than every source -- the fresh state. Times are set
    # EXPLICITLY and far apart rather than with utime(None): the real gap is
    # days, and a test that relies on "now is later than a moment ago" turns
    # filesystem mtime granularity into a flake.
    for rel in _all_sources():
        os.utime(repo / rel, (_T0 - 1000, _T0 - 1000))
    os.utime(repo / ladder.GUEST_BUILDER, (_T0 - 1000, _T0 - 1000))
    for g in ladder.staged_guests():
        os.utime(guests / f"{g}.exe", (_T0 - 500, _T0 - 500))
    return repo


def test_fresh_guests_are_not_called_stale(tmp_path):
    """The positive control's other half: a correctly built tree must still run
    the ladder. A staleness check that fires on everything disables the gate."""
    repo = _built_tree(tmp_path)
    assert ladder.stale_guests(repo) == []
    extra, staged, why = ladder.gate_decision(repo, "amd64")
    assert extra == ["--ladder"] and staged
    assert "older than their sources" not in why


def test_a_guest_older_than_its_source_skips_loudly(tmp_path):
    """THE incident: marshal32.exe built 35 minutes before a fix to its own
    socket rung, hanging there for a week and costing the four guests that run
    after it their verdicts -- while the lane that built it saw nothing wrong,
    because every input it could see was internally consistent. It must SKIP,
    not fail: a stale binary is our bookkeeping, not an OS regression, and a RED
    here blames the innocent guests downstream of it."""
    repo = _built_tree(tmp_path)
    src = repo / ladder.GUEST_SOURCE["marshal32"]
    os.utime(src, (_T0, _T0))  # source edited after the binary was built
    src_rel = ladder.GUEST_SOURCE["marshal32"]
    assert ladder.stale_guests(repo) == [f"marshal32 ({src_rel} is newer)"]
    extra, staged, why = ladder.gate_decision(repo, "amd64")
    assert extra == [] and not staged
    assert "marshal32" in why and ladder.GUEST_BUILDER in why


def test_absolute_age_would_not_have_caught_it(tmp_path):
    """Guards the design choice rather than the code path. The stale binary was
    SIX DAYS old and any age threshold loose enough not to nag would have passed
    it; what makes it wrong is only that its source is newer. So an ancient
    binary whose source is older still must read as FRESH."""
    repo = _built_tree(tmp_path)
    ancient = _T0 - 60 * 60 * 24 * 90
    for rel in _all_sources():
        os.utime(repo / rel, (ancient - 100, ancient - 100))
    os.utime(repo / ladder.GUEST_BUILDER, (ancient - 100, ancient - 100))
    for g in ladder.staged_guests():
        os.utime(repo / "build-fex-hello" / f"{g}.exe", (ancient, ancient))
    assert ladder.stale_guests(repo) == []


def test_a_newer_build_script_invalidates_every_guest(tmp_path):
    """The script carries the link lines. When --no-insert-timestamp was added
    to all nine guests, nothing about any .c changed -- but every binary was out
    of date with respect to how it is now built."""
    repo = _built_tree(tmp_path)
    os.utime(repo / ladder.GUEST_BUILDER, (_T0, _T0))
    stale = ladder.stale_guests(repo)
    assert len(stale) == len(ladder.staged_guests())
    assert all(ladder.GUEST_BUILDER in entry for entry in stale)


def test_an_unmapped_guest_reads_as_stale_not_fresh(tmp_path):
    """A guest added to LADDER without a GUEST_SOURCE entry is a HOLE in this
    check. Silence there would report freshness it never verified, which is the
    same failure shape the check exists to catch."""
    repo = _built_tree(tmp_path)
    (repo / "build-fex-hello" / "newguest32.exe").write_bytes(b"MZ")
    ladder.LADDER["newguest32"] = 0x600D
    try:
        assert "newguest32 (no source mapping)" in ladder.stale_guests(repo)
    finally:
        del ladder.LADDER["newguest32"]


def test_an_unbuilt_guest_is_not_stale(tmp_path):
    """Missing is not stale -- that is the partially-built branch's job, and it
    has a different remedy. Two diagnoses for one state is how a reader ends up
    running the wrong command."""
    repo = _built_tree(tmp_path)
    (repo / "build-fex-hello" / "reg32.exe").unlink()
    assert ladder.stale_guests(repo) == []


def _with_optional(repo, guest="lua32", fetched="tools/stress/lua/src/lua.c"):
    """Add an OPTIONAL guest: our two sources, its .exe, and a fetched
    third-party file whose mtime is NEWER than everything (as a re-fetch
    leaves it)."""
    exe_rel, srcs = ladder.OPTIONAL_SOURCE[guest]
    for rel in [*srcs, exe_rel, fetched]:
        f = repo / rel
        f.parent.mkdir(parents=True, exist_ok=True)
        f.write_text("x")
    for rel in srcs:
        os.utime(repo / rel, (_T0 - 1000, _T0 - 1000))
    os.utime(repo / exe_rel, (_T0 - 500, _T0 - 500))
    os.utime(repo / fetched, (_T0, _T0))  # curl ran just now
    return repo


def test_fetched_upstream_source_does_not_make_a_guest_stale(tmp_path):
    """THE trap. tools/stress/lua/src/ and tools/stress/sqlite/amalg/ are
    gitignored and re-fetched on demand, so their mtime records WHEN CURL RAN,
    not when anything changed. A build on a machine that had to re-fetch leaves
    upstream newer than a binary that is perfectly current. Keying on it would
    red a healthy guest on a schedule set by the network."""
    repo = _with_optional(_built_tree(tmp_path))
    assert ladder.stale_guests(repo) == []
    extra, staged, _ = ladder.gate_decision(repo, "amd64")
    assert extra == ["--ladder"] and staged


def test_our_own_optional_source_does_make_it_stale(tmp_path):
    """The positive control for the trap above: the check must still FIRE on
    the two files we own, or excluding upstream would have excluded everything.
    lua32 is the newest binary in the tree today and the stalest in a
    fortnight, for exactly the reason marshal32 was."""
    repo = _with_optional(_built_tree(tmp_path))
    os.utime(repo / "tools/stress/lua/Lua32.c", (_T0, _T0))
    assert ladder.stale_guests(repo) == ["lua32 (tools/stress/lua/Lua32.c is newer)"]


def test_an_optional_guests_build_script_also_invalidates_it(tmp_path):
    repo = _with_optional(_built_tree(tmp_path))
    os.utime(repo / "tools/stress/lua/build.sh", (_T0, _T0))
    assert ladder.stale_guests(repo) == ["lua32 (tools/stress/lua/build.sh is newer)"]


def test_an_absent_optional_guest_is_not_stale(tmp_path):
    """OPTIONAL guests are built by a script that fetches from the network, so
    they are absent in most trees. Absent means nobody built it --
    media_optional_staged handles that by not grading it. The LADDER rule that
    an unmapped guest reads as stale is right there and wrong here."""
    repo = _built_tree(tmp_path)
    for exe_rel, _ in ladder.OPTIONAL_SOURCE.values():
        assert not (repo / exe_rel).exists()
    assert ladder.stale_guests(repo) == []


def test_never_built_keeps_the_plain_reason(tmp_path):
    """No guests on disk is the ordinary case — most worktrees — and must not be
    dressed up as a staging fault."""
    repo = _manifest(tmp_path, ["reactos/system32/winlogon.exe=/tmp/vibeos/x"])
    _, staged, why = ladder.gate_decision(repo, "amd64")
    assert not staged
    assert "BUILT but" not in why


# --- arm64: a ramdisk, not a cmake manifest -------------------------------


def _ramdisk(tmp_path: Path, guests: list[str], cpu_layer: bool = True) -> Path:
    """A real FAT image with real mcopy'd guests — the arm64 media shape."""
    img = tmp_path / "qemu" / "ramdisk.img"
    img.parent.mkdir(parents=True, exist_ok=True)
    subprocess.run(
        ["dd", "if=/dev/zero", f"of={img}", "bs=1M", "count=64", "status=none"], check=True
    )
    subprocess.run(["mformat", "-i", str(img), "::"], check=True)
    stub = tmp_path / "stub.bin"
    stub.write_bytes(b"MZ")

    def _put(dirname: str, name: str) -> None:
        subprocess.run(["mmd", "-i", str(img), "::/reactos"], check=False)
        subprocess.run(["mmd", "-i", str(img), f"::/reactos/{dirname}"], check=False)
        subprocess.run(
            ["mcopy", "-o", "-i", str(img), str(stub), f"::/reactos/{dirname}/{name}"], check=True
        )

    if cpu_layer:
        _put("system32", "libwow64fex.dll")
    for g in guests:
        _put("syswow64", f"{g}.exe")
    return tmp_path


mtools = pytest.mark.skipif(
    shutil.which("mdir") is None or shutil.which("mformat") is None, reason="mtools absent"
)


@mtools
def test_arm64_reads_the_ramdisk_because_it_has_no_manifest(tmp_path):
    """The reported defect: arm64 stages via mcopy and leaves no cmake manifest,
    so a manifest-only probe answered False while the guests were aboard —
    silently withholding ladder coverage from the FEX host."""
    repo = _ramdisk(tmp_path, list(ladder.LADDER))
    assert ladder.media_has_guests(repo, "arm64") is True
    assert ladder.gate_decision(repo, "arm64")[0] == ["--ladder"]


@mtools
def test_an_arm64_ramdisk_without_guests_is_a_real_negative(tmp_path):
    """mdir exits 0 on 'File not found', so this must come from stdout, not $?."""
    repo = _ramdisk(tmp_path, [], cpu_layer=True)
    assert ladder.media_has_guests(repo, "arm64") is False


def test_no_image_and_no_manifest_is_still_a_decline(tmp_path):
    """Neither mechanism can answer — decline rather than guess either way."""
    assert ladder.media_has_guests(tmp_path, "arm64") is False


@mtools
def test_the_restage_remedy_matches_the_arm64_mechanism(tmp_path):
    """arm64 restages with VIBEOS_FEX=1, not the x86 configure touch. Naming the
    wrong one is worse than naming none — it sends the reader to a command that
    cannot help on their arch."""
    repo = _ramdisk(tmp_path, [], cpu_layer=True)
    guests = repo / "build-fex-hello"
    guests.mkdir(exist_ok=True)
    for g in ladder.staged_guests():
        (guests / f"{g}.exe").write_bytes(b"MZ")
    _, staged, why = ladder.gate_decision(repo, "arm64")
    assert not staged
    assert "VIBEOS_FEX" in why and "wow64cpu/CMakeLists.txt" not in why


@mtools
def test_optional_guests_are_visible_on_arm64_too(tmp_path):
    """media_optional_staged arrived from the wow64-amd64 lane reading only
    <stem>.cmake.lst, with a docstring saying it could not answer on arm64. Now
    that both readers are one, it inherits the ramdisk path — so this asserts the
    capability that version documented as impossible."""
    repo = _ramdisk(tmp_path, [], cpu_layer=True)
    img = repo / ladder.RAMDISK_REL
    stub = tmp_path / "s.bin"
    stub.write_bytes(b"MZ")
    subprocess.run(["mmd", "-i", str(img), "::/reactos/syswow64"], check=False)
    for g in ladder.OPTIONAL:
        subprocess.run(
            ["mcopy", "-o", "-i", str(img), str(stub), f"::/reactos/syswow64/{g}.exe"], check=True
        )
    assert ladder.media_optional_staged(repo, "arm64") == sorted(ladder.OPTIONAL)


def test_optional_guests_absent_is_an_empty_list(tmp_path):
    repo = _manifest(tmp_path, ["reactos/syswow64/hello32.exe=/tmp/vibeos/x"])
    assert ladder.media_optional_staged(repo, "amd64") == []


def test_a_sibling_medium_manifest_must_not_answer_for_this_one(tmp_path):
    """arm64's media is boot.img, so its stem is "boot" — and a loose `{stem}*.lst`
    glob matches bootcd.*.lst, a DIFFERENT medium. That made the manifest probe
    answer "no guests" for arm64 and shadow the ramdisk probe that had the right
    answer, breaking the None contract this function documents. Caught on real
    staged media by the wow64-amd64 lane."""
    boot = tmp_path / "build-arm64" / "boot"
    boot.mkdir(parents=True)
    # A populated sibling manifest, of the kind that fooled the loose glob.
    (boot / "bootcd.cmake.lst").write_text("reactos/system32/ntoskrnl.exe=/tmp/vibeos/x\n")
    (boot / "bootcdregtest.Debug.lst").write_text("reactos/system32/hal.dll=/tmp/vibeos/y\n")
    # No boot.*.lst exists, so the manifest probe must DECLINE (None), not answer.
    assert ladder._staged_per_manifest(tmp_path, "arm64", "syswow64", {"hello32.exe"}) is None


def test_an_unbuilt_tree_says_unknown_not_unsupported(tmp_path):
    """A freshly-provisioned box has no build dir at all, so neither probe can
    answer. That must NOT come back as "32-bit guests cannot run here": on amd64
    that is a false architectural claim, and it is the reassuring direction to be
    wrong in, so nobody downstream would question it. `_read_media` and both
    probes keep a None for exactly this, and `bool()` used to discard it."""
    assert ladder.supports_wow64(tmp_path, "amd64") is None
    _, staged, why = ladder.gate_decision(tmp_path, "amd64")
    assert not staged
    assert "UNKNOWN, not" in why
    assert "cannot run here" not in why
    assert "./vibeos-cli build --arch amd64" in why


def test_a_real_native_x86_negative_still_reads_as_unsupported(tmp_path):
    """The positive control: i386's livecd HAS a manifest and no SysWOW64 to
    stage into, so the probe answers honestly and the verdict is final rather
    than fixable. Turning the no-media case into None must not soften this one."""
    repo = _manifest(tmp_path, [], arch="i386", cpu_layer=False)
    assert ladder.supports_wow64(repo, "i386") is False
    _, staged, why = ladder.gate_decision(repo, "i386")
    assert not staged
    assert "cannot run here" in why


def test_the_ramdisk_never_answers_for_an_iso_arch(tmp_path):
    """The image path carries no arch, so a present qemu/ramdisk.img used to
    answer for amd64 — arm64's media speaking for x86's. Hidden whenever a
    build-amd64/ manifest existed (that probe runs first), it surfaced on a
    restored box whose only media was a two-week-old ramdisk: amd64 was told its
    media had no WoW64 CPU layer, on arm64's evidence."""
    (tmp_path / "qemu").mkdir()
    (tmp_path / ladder.RAMDISK_REL).write_bytes(b"\0" * 512)
    for arch in ("i386", "amd64"):
        assert ladder._staged_per_ramdisk(tmp_path, arch, "system32", ladder.CPU_LAYERS) is None
        assert ladder.supports_wow64(tmp_path, arch) is None
        assert "UNKNOWN, not" in ladder.gate_decision(tmp_path, arch)[2]


# DIAGNOSTIC is EMPTY today — spawn32, the guest it was built for, was promoted
# into LADDER the day it passed. These tests therefore drive the mechanism with
# a synthetic guest rather than a real one. That is deliberate: the machinery is
# what the next guest aimed at a known-open gap will need, and deleting its
# coverage because the table happens to be empty would leave it to rot
# unexercised until exactly the moment someone depends on it.
_DIAG = "probe32"


@pytest.fixture
def one_diagnostic(monkeypatch):
    """Register a synthetic diagnostic guest for the duration of one test."""
    monkeypatch.setitem(ladder.DIAGNOSTIC, _DIAG, 0x600D)
    return _DIAG


def test_a_failing_diagnostic_guest_does_not_red_the_gate(one_diagnostic):
    """The whole reason DIAGNOSTIC exists. A diagnostic guest measures a gap we
    already have a task for; grading it would red every lane in the fleet for a
    hole nobody introduced, and the fleet's answer to a check that reds for a
    known reason is to disable the check."""
    text = _all_pass() + _serial(**{one_diagnostic: 0xBAE1})
    assert ladder.check(text) is None


def test_a_failing_diagnostic_guest_still_reports_its_code(one_diagnostic):
    """Not grading it must not mean not SAYING it — an unreported measurement is
    not a measurement. The exact code is what triages the rung."""
    out = ladder.summary(_all_pass() + _serial(**{one_diagnostic: 0xBAE1}))
    assert f"{one_diagnostic}=0xbae1" in out


def test_a_diagnostic_guest_that_starts_passing_is_loud(one_diagnostic):
    """The day the gap closes is the day this line matters, and a bare 0x600d
    among the other hex codes is a result nobody reads. It must say so in words
    and name the next move, because a diagnostic guest that passes should be
    promoted into LADDER where a REGRESSION can red it again. This is not
    hypothetical: it is how spawn32 left this table."""
    out = ladder.summary(_all_pass() + _serial(**{one_diagnostic: 0x600D}))
    assert f"{one_diagnostic} PASSES" in out and "promote it to LADDER" in out


def test_a_diagnostic_guest_is_not_counted_in_the_parity_score(one_diagnostic):
    """It is not a parity guest, so it must move neither the numerator nor the
    denominator — and it must not fall through to the '+N non-parity' tail
    either, which is for guests nobody has classified."""
    out = ladder.summary(_all_pass() + _serial(**{one_diagnostic: 0xBAE1}))
    n = len(ladder.LADDER)
    assert out.startswith(f"{n}/{n} guests PASS")
    assert "non-parity" not in out


def test_an_absent_diagnostic_guest_says_nothing_at_all(tmp_path):
    """Silence when it did not run: a gate on media that never staged it must
    read exactly as it did before the guest existed."""
    out = ladder.summary(_all_pass())
    assert "diagnostic" not in out
    assert ladder.check(_all_pass()) is None


def test_an_absent_diagnostic_guest_costs_the_parity_set_nothing(tmp_path, one_diagnostic):
    """A diagnostic guest is built on demand, so absent is its normal state —
    the doom-i386 rule (one missing .exe keeps all the others off the media)
    must NOT apply to it. spawn32 was the original case and is no longer one:
    it is a parity guest now, so this drives a synthetic diagnostic instead.
    The asymmetry is the point — a guest absent from `staged_guests()` may go
    missing for free, and a LADDER member may not."""
    repo = _built_tree(tmp_path)  # every parity guest built, incl. spawn32
    assert one_diagnostic not in ladder.staged_guests()
    assert not (repo / "build-fex-hello" / f"{one_diagnostic}.exe").exists()
    assert ladder.stale_guests(repo) == []
    extra, staged, _ = ladder.gate_decision(repo, "amd64")
    assert extra == ["--ladder"] and staged


def test_a_promoted_guest_is_graded_and_shares_the_all_or_nothing_set(tmp_path):
    """The other half of the promotion, and the reason it is not free. spawn32
    is now a LADDER member, so `check` grades it and `staged_guests` demands it
    — and that set is all-or-nothing in CMake, so a tree that fails to build it
    loses the whole ladder rather than just one verdict.

    Note what is NOT asserted here: `gate_decision` cannot see a missing
    build-fex-hello/spawn32.exe, because it asks the MEDIA what was staged
    rather than the build tree. The enforcement lives in CMake's
    `_wow64_guest_ok`. What ladder.py owns is grading and staleness, so that is
    what this pins."""
    assert "spawn32" in ladder.LADDER
    assert "spawn32" in ladder.staged_guests()
    # Graded now: a bad code reds, where as a DIAGNOSTIC it was reported only.
    assert ladder.check(_all_pass()) is None
    bad = _all_pass().replace(
        "[WINLOGON] spawn32.exe exit = 0x600d",
        "[WINLOGON] spawn32.exe exit = 0xbae5",
    )
    assert ladder.check(bad) is not None
    # And a stale binary now skips the ENTIRE ladder, not just its own rung.
    repo = _built_tree(tmp_path)
    exe = repo / "build-fex-hello" / "spawn32.exe"
    os.utime(exe, (_T0 - 500, _T0 - 500))
    src = repo / ladder.GUEST_SOURCE["spawn32"]
    src.parent.mkdir(parents=True, exist_ok=True)
    src.write_text("int start(void){return 0;}\n")
    os.utime(src, (_T0, _T0))
    _extra, staged, why = ladder.gate_decision(repo, "amd64")
    assert not staged and "spawn32" in why


def test_a_built_diagnostic_guest_is_still_staleness_checked(tmp_path):
    """It lives in build-fex-hello/ and rots like any other guest, which is the
    marshal32 failure mode. Written when spawn32 was opt-in, on the principle
    that opt-in must not mean unchecked; it now guards a parity guest, and the
    assertion is the same either way."""
    repo = _built_tree(tmp_path)
    exe = repo / "build-fex-hello" / "spawn32.exe"
    exe.write_bytes(b"MZ")
    os.utime(exe, (_T0 - 500, _T0 - 500))
    src = repo / ladder.GUEST_SOURCE["spawn32"]
    src.parent.mkdir(parents=True, exist_ok=True)
    src.write_text("int start(void){return 0;}\n")
    os.utime(src, (_T0, _T0))
    assert ladder.stale_guests(repo) == ["spawn32 (tools/fex/hello-compute/Spawn32.c is newer)"]


def test_the_restage_remedy_is_chosen_by_arch_not_by_a_stray_ramdisk(tmp_path):
    """Same root cause as the arch-blind probe, second symptom. The remedy used
    to key on `qemu/ramdisk.img` EXISTING, so an amd64 tree that happened to
    have a ramdisk on disk — a restored backup, an arm64 build in the same
    worktree — was told to restage with `VIBEOS_FEX=1 build --arch arm64`. The
    reader follows that and rebuilds the wrong arch, and the guests are still
    not staged on the one they asked about."""
    repo = _manifest(tmp_path, ["reactos/system32/winlogon.exe=/tmp/vibeos/x"])
    (repo / "qemu").mkdir(exist_ok=True)
    (repo / ladder.RAMDISK_REL).write_bytes(b"\0" * 512)
    guests = repo / "build-fex-hello"
    guests.mkdir()
    for g in ladder.staged_guests():
        (guests / f"{g}.exe").write_bytes(b"MZ")
    _, staged, why = ladder.gate_decision(repo, "amd64")
    assert not staged
    assert "BUILT but none is staged" in why
    assert "--arch amd64" in why
    assert "VIBEOS_FEX" not in why and "--arch arm64" not in why


def test_uses_ramdisk_answers_from_the_arch_alone(tmp_path):
    """No repo argument on purpose — the mechanism is a property of the arch's
    media, and taking a path would re-open the door to answering from the disk."""
    assert ladder.uses_ramdisk("arm64") is True
    assert ladder.uses_ramdisk("amd64") is False
    assert ladder.uses_ramdisk("i386") is False


def test_the_staged_line_counts_what_is_actually_staged(tmp_path):
    """It read `len(LADDER)` and so said "10 guests staged" while 11 sat on the
    media. Harmless-looking, but it is the number a reader checks the serial
    against, and a count that silently omits the diagnostic guest teaches them
    to ignore its line."""
    repo = _built_tree(tmp_path)
    extra, staged, why = ladder.gate_decision(repo, "amd64")
    assert extra == ["--ladder"] and staged
    assert f"{len(ladder.LADDER)} parity + {len(ladder.DIAGNOSTIC)} diagnostic" in why


def test_drain_floor_is_zero_and_inert_when_unset(monkeypatch):
    monkeypatch.delenv(ladder.DRAIN_FLOOR_ENV, raising=False)
    assert ladder.drain_floor() == 0.0
    vm = _FakeVM([])
    assert ladder.watch(vm, seconds=5, seen=_all_pass()) == ""
    assert vm.serial.reads == 0, "unset floor must not change the old fast path"


def test_drain_floor_keeps_draining_past_the_graded_set(monkeypatch):
    """The stock applets (winmine/notepad/mspaint/wordpad) run AFTER the parity
    set and are not graded, so the drain's "every guest reported" exit throws
    their phase away. The floor is what makes it observable."""
    monkeypatch.setenv(ladder.DRAIN_FLOOR_ENV, "0.5")
    vm = _FakeVM([_all_pass()])
    started = time.monotonic()
    ladder.watch(vm, seconds=0.05, seen="")
    waited = time.monotonic() - started
    assert waited >= 0.5, (
        f"drained {waited:.2f}s — a floor must hold the drain open past the "
        "graded set, which is the whole point of setting it"
    )


def test_drain_floor_applies_even_when_nothing_is_outstanding(monkeypatch):
    """`seen` already carrying every verdict is exactly the investigation case:
    the graded set finished in the settle read and the applets are still to
    come. The early return must not skip the floor."""
    monkeypatch.setenv(ladder.DRAIN_FLOOR_ENV, "0.4")
    vm = _FakeVM([])
    started = time.monotonic()
    ladder.watch(vm, seconds=0.01, seen=_all_pass())
    assert time.monotonic() - started >= 0.4


def test_drain_floor_never_shortens_the_budget(monkeypatch):
    """It may only EXTEND: a floor below the ceiling must leave a silent guest
    still using the full ceiling, or it would manufacture fake failures."""
    monkeypatch.setenv(ladder.DRAIN_FLOOR_ENV, "0.01")
    vm = _FakeVM([_serial(hello32=0x600D)])
    out = ladder.watch(vm, seconds=1.0, seen="")
    assert _red(out)["classification"] == "LADDER_FAIL"


@pytest.mark.parametrize("bad", ["abc", "-5", "1,5"])
def test_a_malformed_drain_floor_is_loud(monkeypatch, bad):
    """Silently reading a typo as 0 hands back a run that looks like it watched
    the applet phase and did not."""
    monkeypatch.setenv(ladder.DRAIN_FLOOR_ENV, bad)
    with pytest.raises(ValueError):
        ladder.drain_floor()


# --- Wine conformance suites ------------------------------------------------
# The counts are the ORACLE grading us and start non-zero by construction, so
# they are reported and never gated; a subtest that starts and vanishes is OUR
# regression and is gated exactly like every other rung.

_WT_OK = "[WT] user32:wsprintf: exit = 0 failure(s), 900 bytes of output\n"
_WT_RED = "[WT] user32:class: exit = 7 failure(s), 4096 bytes of output\n"
_WT_START = "[WT] user32:class: started (PID 901)\n"


def test_winetest_failure_counts_are_parsed():
    got = ladder.winetests(_all_pass() + _WT_OK + _WT_RED)
    assert got == {"user32:wsprintf": 0, "user32:class": 7}


def test_winetest_failures_do_not_red_the_ladder():
    """A failing assertion is Wine telling us about the OS, not about the run.
    Gating on it would paint the ladder permanently red and get it ignored."""
    assert ladder.check(_all_pass() + _WT_START + _WT_RED) is None


def test_a_winetest_that_started_and_vanished_is_a_failure():
    """No exit line = the guest crashed or hung, which is ours, not Wine's."""
    red = _red(_all_pass() + _WT_START)
    assert red["classification"] == "LADDER_FAIL"
    assert any("user32:class" in s and "never reported" in s for s in red["salient"])


def test_winetest_counts_reach_the_summary():
    line = ladder.summary(_all_pass() + _WT_OK + _WT_RED)
    assert "user32:class=7" in line and "user32:wsprintf=0" in line


def test_summary_says_nothing_about_winetests_when_none_ran():
    assert "winetest" not in ladder.summary(_all_pass())


_WT_CRASH_LINE = "[WT] user32:class: CRASHED, exit = 0xc0000005, 2043 bytes of output\n"


def test_a_crash_is_not_read_as_a_failure_count():
    """0xC0000005 as "3221225477 failures" is a number that looks like data and
    is not — the suite died partway and every later assertion went unrun."""
    text = _all_pass() + _WT_START + _WT_CRASH_LINE
    assert ladder.winetests(text) == {}
    assert ladder.winetests_crashed(text) == {"user32:class": "0xc0000005"}


def test_a_crashed_subtest_counts_as_having_reported():
    """It crashed, which is ours — but it is not LOST, and conflating the two
    would hide a hang behind a crash."""
    assert ladder.winetests_lost(_all_pass() + _WT_START + _WT_CRASH_LINE) == []


def test_a_crash_reaches_the_summary_as_a_crash():
    line = ladder.summary(_all_pass() + _WT_CRASH_LINE)
    assert "user32:class=CRASH(0xc0000005)" in line


def test_the_native_control_label_is_parsed_alongside_the_wow64_one():
    """`native/user32:class` is the SAME binary on bare i386. Losing it to a
    regex that forbids `/` would silently delete the control half of the A/B
    and leave every WoW64 failure unattributable."""
    text = (
        _all_pass()
        + "[WT] user32:class: CRASHED, exit = 0xc0000005, 100 bytes of output\n"
        + "[WT] native/user32:class: exit = 3 failure(s), 200 bytes of output\n"
    )
    assert ladder.winetests(text) == {"native/user32:class": 3}
    assert ladder.winetests_crashed(text) == {"user32:class": "0xc0000005"}
    assert ladder.winetests_lost(text) == []


def test_a_timeout_is_its_own_outcome():
    """We killed it with 0xDEAD, so "57005 failure(s)" is our number wearing the
    oracle's clothes."""
    text = _all_pass() + "[WT] native/user32:class: TIMED OUT, 635 bytes of output\n"
    assert ladder.winetests(text) == {}
    assert ladder.winetests_timed_out(text) == ["native/user32:class"]
    assert "native/user32:class=TIMEOUT" in ladder.summary(text)


def test_the_drain_waits_for_a_started_winetest(monkeypatch):
    """check() fails a winetest that started and never reported, so watch()
    must not stop while one is running. It did: the gate went LADDER_FAIL on
    user32:wsprintf, which reported `exit = 0` eight serial lines after the
    drain had already ended on the last parity marker."""
    monkeypatch.setattr(ladder, "WINETEST_EXTRA_SECONDS", 5.0)
    started = "[WT] user32:wsprintf: started (PID 1148)\n"
    done = "[WT] user32:wsprintf: exit = 0 failure(s), 272 bytes of output\n"
    vm = _FakeVM([_all_pass() + started, "", done])
    out = ladder.watch(vm, seconds=0.01, seen="")
    assert done in out, "the drain stopped before the running winetest reported"
    assert ladder.check(out) is None


def test_a_winetest_started_in_the_settle_read_is_still_waited_for(monkeypatch):
    monkeypatch.setattr(ladder, "WINETEST_EXTRA_SECONDS", 5.0)
    started = "[WT] user32:class: started (PID 901)\n"
    vm = _FakeVM(["", _WT_RED])
    out = ladder.watch(vm, seconds=0.01, seen=_all_pass() + started)
    assert _WT_RED in out
    assert vm.serial.reads == 2


# ── the amd64 build makes its own guests ────────────────────────────────────
# The merge queue's integration tree had no build-fex-hello/, so every batch
# skipped the ladder "not staged" and no WoW64 regression could ever red a
# queue gate. `build --arch amd64` now builds what is absent or stale.


def _buildable(repo: Path) -> None:
    for rel in (ladder.GUEST_TOOLCHAIN, ladder.GUEST_PREREQ):
        (repo / rel).parent.mkdir(parents=True, exist_ok=True)
        (repo / rel).write_bytes(b"")


def test_a_fresh_tree_has_nothing_to_refresh(tmp_path):
    """A rebuilt guest re-runs the amd64 configure, so refreshing a current set
    would cost every build a reconfigure for nothing."""
    assert ladder.guests_to_refresh(_built_tree(tmp_path)) == []


def test_absent_and_stale_guests_are_both_refreshed(tmp_path):
    repo = _built_tree(tmp_path)
    (repo / "build-fex-hello" / "reg32.exe").unlink()
    os.utime(repo / ladder.GUEST_SOURCE["spawn32"], (_T0, _T0))
    todo = ladder.guests_to_refresh(repo)
    assert "reg32 (absent)" in todo
    assert any(t.startswith("spawn32 (") for t in todo)
    assert len(todo) == 2


def test_refresh_names_what_it_is_missing(tmp_path):
    repo = _built_tree(tmp_path)
    assert "toolchain" in (ladder.refresh_blocker(repo) or "")
    (repo / ladder.GUEST_TOOLCHAIN).parent.mkdir(parents=True)
    (repo / ladder.GUEST_TOOLCHAIN).write_bytes(b"")
    assert "--arch i386" in (ladder.refresh_blocker(repo) or "")
    _buildable(repo)
    assert ladder.refresh_blocker(repo) is None


def test_only_x64_guests_are_deferred_and_only_without_amd64_importlibs(tmp_path):
    repo = tmp_path
    todo = ["selfmod64 (absent)", "apc64 (tools/fex/hello-compute/Apc64.c is newer)"]
    assert ladder.x64_refresh_deferred(repo, todo) == todo
    # A 32-bit guest is never deferred, whatever build-amd64 holds.
    assert ladder.x64_refresh_deferred(repo, [*todo, "file32 (absent)"]) == todo
    assert ladder.x64_refresh_deferred(repo, []) == []
    lib = repo / ladder.GUEST_PREREQ_X64
    lib.parent.mkdir(parents=True)
    lib.write_bytes(b"")
    assert ladder.x64_refresh_deferred(repo, todo) == []


class _FakeRun:
    def __init__(self):
        self.stages = []
        self.classification = None

    def add(self, stage):
        self.stages.append(stage)

    def run(self, cmd, cwd=None, env=None, stage=None, timeout=None):
        return subprocess.run(cmd, cwd=cwd, capture_output=True, text=True, timeout=timeout)


def test_build_stage_fails_loud_when_the_builder_leaves_guests_behind(tmp_path):
    """The builder SKIPS a guest whose importlib is missing and still exits 0 --
    so exit status alone would call a partial set built, and the all-or-nothing
    staging would then quietly stage none of it."""
    from tests.qemu.cli import build

    repo = _built_tree(tmp_path)
    _buildable(repo)
    (repo / "build-fex-hello" / "reg32.exe").unlink()
    (repo / ladder.GUEST_BUILDER).write_text("echo '[!] reg32.exe skipped'\n")
    run = _FakeRun()
    assert build._refresh_wow64_guests(run, repo) is False
    assert run.classification == "GUEST_BUILD_FAIL"
    assert "reg32 (absent)" in run.stages[-1].detail


def test_build_stage_builds_then_reports_ok(tmp_path):
    from tests.qemu.cli import build

    repo = _built_tree(tmp_path)
    _buildable(repo)
    (repo / "build-fex-hello" / "reg32.exe").unlink()
    (repo / ladder.GUEST_BUILDER).write_text("printf MZ > build-fex-hello/reg32.exe\n")
    # Rewriting the builder makes it newer than every guest -- all stale -- which
    # is right for the real script and wrong for this fixture.
    os.utime(repo / ladder.GUEST_BUILDER, (_T0 - 1000, _T0 - 1000))
    run = _FakeRun()
    assert build._refresh_wow64_guests(run, repo) is True
    assert run.stages[-1].status == "ok" and "reg32" in run.stages[-1].detail


def _stale_x64_guest(repo: Path) -> None:
    exe = repo / "build-fex-hello" / "apc64.exe"
    exe.write_bytes(b"MZ")
    os.utime(exe, (_T0 - 5000, _T0 - 5000))
    assert any(t.startswith("apc64 (") for t in ladder.guests_to_refresh(repo))


def test_build_stage_defers_x64_guests_before_amd64_has_importlibs(tmp_path):
    """A fresh build-amd64 has no importlibs before its first ninja, so the
    builder skips the x86-64 guests. That is a deferral, not a failure: it failed
    every first amd64 build in a fresh worktree with an edited x64 guest source."""
    from tests.qemu.cli import build

    repo = _built_tree(tmp_path)
    _buildable(repo)
    _stale_x64_guest(repo)
    (repo / ladder.GUEST_BUILDER).write_text("echo '[!] apc64.exe skipped'\n")
    os.utime(repo / ladder.GUEST_BUILDER, (_T0 - 10000, _T0 - 10000))
    run = _FakeRun()
    deferred = build._refresh_wow64_guests(run, repo)
    assert isinstance(deferred, list) and any(d.startswith("apc64 (") for d in deferred)
    assert run.classification is None
    assert run.stages[-1].status == "ok" and "deferred" in run.stages[-1].detail


def test_build_stage_fails_on_x64_guests_once_amd64_has_importlibs(tmp_path):
    """With build-amd64's importlibs present, a skipped x86-64 guest is a real
    failure again -- the deferral must not become a permanent excuse."""
    from tests.qemu.cli import build

    repo = _built_tree(tmp_path)
    _buildable(repo)
    prereq = repo / ladder.GUEST_PREREQ_X64
    prereq.parent.mkdir(parents=True, exist_ok=True)
    prereq.write_bytes(b"")
    _stale_x64_guest(repo)
    (repo / ladder.GUEST_BUILDER).write_text("echo '[!] apc64.exe skipped'\n")
    os.utime(repo / ladder.GUEST_BUILDER, (_T0 - 10000, _T0 - 10000))
    run = _FakeRun()
    assert build._refresh_wow64_guests(run, repo) is False
    assert run.classification == "GUEST_BUILD_FAIL" and "apc64" in run.stages[-1].detail


def test_build_stage_skips_loudly_without_an_i386_tree(tmp_path):
    """Not this build's failure -- amd64 media is complete without guests -- but
    the stage that could have built them says why it did not."""
    from tests.qemu.cli import build

    repo = _built_tree(tmp_path)
    (repo / "build-fex-hello" / "reg32.exe").unlink()
    run = _FakeRun()
    assert build._refresh_wow64_guests(run, repo) is True
    assert run.stages[-1].status == "skip" and "toolchain" in run.stages[-1].detail
