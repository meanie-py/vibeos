"""Ladder verdict + gate-policy tests.

The point of the ladder is that a WoW64 guest can be red while every gate stays
green, so these lean on the two ways that silence can be produced: a guest that
ran and failed, and a media that never carried a guest at all. Conflating them is
the bug -- one is a regression, the other is a lane that was never staged.
"""

from __future__ import annotations

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


# --- gate policy ------------------------------------------------------------


@pytest.fixture
def staged(monkeypatch):
    def _set(missing: list[str]):
        monkeypatch.setattr(
            ladder.provenance, "probes_missing_from_media", lambda *_a, **_k: missing
        )

    return _set


def test_gate_asks_for_the_ladder_when_the_media_carries_guests(staged):
    staged([])
    extra, is_staged, why = ladder.gate_decision(Path("/repo"), "amd64")
    assert extra == ["--ladder"]
    assert is_staged
    assert str(len(ladder.LADDER)) in why


def test_gate_declines_and_says_so_when_it_does_not(staged):
    staged([f"{ladder.SENTINEL!r} appears 0 times in livecd.iso"])
    extra, is_staged, why = ladder.gate_decision(Path("/repo"), "i386")
    assert extra == []
    assert not is_staged
    # The skip has to be legible: a gate that quietly stops checking is the bug.
    assert ladder.SENTINEL in why and "NOT checked" in why


def test_the_probe_reads_the_media_not_the_trees_that_fed_it(monkeypatch):
    """Staging happens at CONFIGURE time, so build-i386 existing now says nothing
    about what went onto an ISO built before it. Assert the question asked is
    about the boot media, with the arch that will open it."""
    asked: list[tuple] = []

    def _spy(repo, arch, literals):
        asked.append((repo, arch, tuple(literals)))
        return []

    monkeypatch.setattr(ladder.provenance, "probes_missing_from_media", _spy)
    ladder.gate_decision(Path("/repo"), "amd64")
    assert asked == [(Path("/repo"), "amd64", (ladder.SENTINEL,))]


def test_an_unreadable_media_declines_rather_than_deciding_the_gate(monkeypatch):
    def _boom(*_a, **_k):
        raise OSError("half-written image")

    monkeypatch.setattr(ladder.provenance, "probes_missing_from_media", _boom)
    extra, is_staged, _ = ladder.gate_decision(Path("/repo"), "amd64")
    assert extra == [] and not is_staged
