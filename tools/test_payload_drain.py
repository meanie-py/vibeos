"""The payload drain budget is WALL CLOCK, not read iterations.

This oracle exists because the bug it pins was invisible to every other check
for months and cost real evidence. `launch_drain_secs` was consumed by a
`for _ in range(budget)` loop around a read that returns the INSTANT bytes are
already waiting, so the budget was only spent at one-per-second while the guest
was SILENT. The chattier the payload, the shorter the real drain — exactly
inverted, since the payloads that need the longest drain are the ones producing
the most output.

The apitest sweep's declared 7200s ended after 12-28 minutes, every run, and the
characterisation driver on top then blamed whichever subtest was in flight when
the harness stopped reading. Seven innocent subtests were quarantined as
crashers before anyone noticed that their serials ended with their own passing
summary line.

A chatty-payload case is the one that matters: a test that only ever feeds an
idle guest cannot tell the two implementations apart.
"""

from __future__ import annotations

from tests.qemu.test_payload import drain_for_marker


class FakeClock:
    """Advances only when the code under test says time passed."""

    def __init__(self) -> None:
        self.t = 0.0

    def __call__(self) -> float:
        return self.t


def test_a_chatty_guest_does_not_shrink_the_budget() -> None:
    """The regression itself: bytes always waiting, so each read is ~free.

    Under the old `for _ in range(budget)` the loop ran out after `budget`
    reads, which here is 20 wall-clock seconds out of a declared 600. The
    wall-clock loop keeps reading until the clock actually says 600.
    """
    clock = FakeClock()
    reads = 0

    def read(timeout: float = 1.0) -> str:
        nonlocal reads
        reads += 1
        clock.t += 0.02  # data already buffered: returns immediately
        return "chatter\n"

    out = drain_for_marker(read, "", "[DONE]", "*** BUGCHECK", 600.0, now=clock)

    assert "[DONE]" not in out
    assert clock.t >= 600.0, f"gave up after {clock.t}s of a 600s budget"
    # The old loop stopped after `budget` reads — 600 of them, 12 seconds of
    # guest time. Anything near that number means the iteration count is back.
    assert reads > 25000, f"only {reads} reads: budget is being counted, not timed"


def test_an_idle_guest_still_spends_the_budget_once() -> None:
    """The case that always worked — kept so a 'fix' cannot regress it."""
    clock = FakeClock()

    def read(timeout: float = 1.0) -> str:
        clock.t += timeout  # nothing buffered: waits the full timeout
        return ""

    drain_for_marker(read, "", "[DONE]", "*** BUGCHECK", 60.0, now=clock)
    assert clock.t >= 60.0


def test_the_marker_ends_the_drain_early() -> None:
    clock = FakeClock()

    def read(timeout: float = 1.0) -> str:
        clock.t += 0.02
        return "[DONE]\n"

    out = drain_for_marker(read, "", "[DONE]", "*** BUGCHECK", 600.0, now=clock)
    assert "[DONE]" in out
    assert clock.t < 1.0, "a found marker must not keep draining"


def test_a_bugcheck_ends_the_drain_early() -> None:
    """A crashed guest emits nothing more, so draining on is pure waste."""
    clock = FakeClock()

    def read(timeout: float = 1.0) -> str:
        clock.t += 0.02
        return "*** BUGCHECK: 0x0000001E\n"

    out = drain_for_marker(read, "", "[DONE]", "*** BUGCHECK", 7200.0, now=clock)
    assert "[DONE]" not in out
    assert clock.t < 1.0, "a crashed guest must not burn the whole budget"


def test_output_already_holding_the_marker_is_returned_untouched() -> None:
    clock = FakeClock()

    def read(timeout: float = 1.0) -> str:
        raise AssertionError("must not read when the marker is already present")

    # The caller guards this, but the helper must not depend on that guard.
    out = drain_for_marker(read, "[DONE]\n", "[DONE]", "*** BUGCHECK", 10.0, now=clock)
    assert out == "[DONE]\n"
