"""Ladder verdict + gate-policy tests.

The point of the ladder is that a WoW64 guest can be red while every gate stays
green, so these lean on the two ways that silence can be produced: a guest that
ran and failed, and a media that never carried a guest at all. Conflating them is
the bug -- one is a regression, the other is a lane that was never staged.
"""

from __future__ import annotations

import shutil
import subprocess
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


def test_gui32_wants_its_own_verdict_code():
    """gui32 reports 0x600e; grading it against the common 0x600d would fail a
    green ladder on every boot."""
    assert ladder.LADDER["gui32"] != ladder.LADDER["hello32"]
    assert ladder.check(_all_pass()) is None


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
    out = ladder.watch(vm, seconds=5, seen=_serial(gui32=0x600E))
    assert ladder.check(out + _serial(gui32=0x600E)) is None
    assert vm.serial.reads == 1, "should stop the moment the remaining guests report"


def test_all_guests_already_seen_drains_nothing_at_all():
    vm = _FakeVM([])
    assert ladder.watch(vm, seconds=5, seen=_all_pass()) == ""
    assert vm.serial.reads == 0, "no read at all when nothing is outstanding"


def test_a_guest_that_never_reports_still_uses_the_budget():
    """The ceiling must stay a real ceiling — a silent guest is what it is for."""
    vm = _FakeVM([_serial(hello32=0x600D)])
    out = ladder.watch(vm, seconds=1.0, seen="")
    assert _red(out)["classification"] == "LADDER_FAIL"


def test_built_but_unstaged_says_so_and_names_the_fix(tmp_path):
    """The 45-minute case: guests on disk, none on the media, because staging is
    snapshotted at configure time and a warm build does not reconfigure. A bare
    "not staged" sends the reader looking for a build that already happened."""
    repo = _manifest(tmp_path, ["reactos/system32/winlogon.exe=/tmp/vibeos/x"])
    guests = repo / "build-fex-hello"
    guests.mkdir()
    for g in ladder.LADDER:
        (guests / f"{g}.exe").write_bytes(b"MZ")
    extra, staged, why = ladder.gate_decision(repo, "amd64")
    assert extra == [] and not staged
    assert "BUILT but none is staged" in why
    # The remedy must match THIS media's staging mechanism, not the other arch's.
    assert "wow64cpu/CMakeLists.txt" in why and "VIBEOS_FEX" not in why


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
    for g in ladder.LADDER:
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
