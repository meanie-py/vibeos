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

    out, reason = drain_for_marker(read, "", "[DONE]", "*** BUGCHECK", 600.0, now=clock)

    assert "[DONE]" not in out
    assert reason == "budget"
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

    _out, reason = drain_for_marker(read, "", "[DONE]", "*** BUGCHECK", 60.0, now=clock)
    assert reason == "budget"
    assert clock.t >= 60.0


def test_the_marker_ends_the_drain_early() -> None:
    clock = FakeClock()

    def read(timeout: float = 1.0) -> str:
        clock.t += 0.02
        return "[DONE]\n"

    out, reason = drain_for_marker(read, "", "[DONE]", "*** BUGCHECK", 600.0, now=clock)
    assert "[DONE]" in out
    assert reason == "marker"
    assert clock.t < 1.0, "a found marker must not keep draining"


def test_a_bugcheck_ends_the_drain_early() -> None:
    """A crashed guest emits nothing more, so draining on is pure waste."""
    clock = FakeClock()

    def read(timeout: float = 1.0) -> str:
        clock.t += 0.02
        return "*** BUGCHECK: 0x0000001E\n"

    out, reason = drain_for_marker(read, "", "[DONE]", "*** BUGCHECK", 7200.0, now=clock)
    assert "[DONE]" not in out
    assert reason == "fail"
    assert clock.t < 1.0, "a crashed guest must not burn the whole budget"


def test_output_already_holding_the_marker_is_returned_untouched() -> None:
    clock = FakeClock()

    def read(timeout: float = 1.0) -> str:
        raise AssertionError("must not read when the marker is already present")

    # The caller guards this, but the helper must not depend on that guard.
    out, reason = drain_for_marker(read, "[DONE]\n", "[DONE]", "*** BUGCHECK", 10.0, now=clock)
    assert out == "[DONE]\n"
    assert reason == "marker"


def test_a_guest_that_goes_silent_is_caught_by_the_IDLE_cutoff() -> None:
    """The wedge case, and the reason it is worth a separate signal.

    A wedged guest used to cost the ENTIRE budget: the sweep died at 19:01 and
    the rig kept politely reading nothing until 21:01, two hours later, three
    rounds running. Silence while the rig is still reading is also the only
    thing that positively implicates the GUEST rather than the rig, so it is
    both the speedup and the attribution signal.
    """
    clock = FakeClock()

    def read(timeout: float = 1.0) -> str:
        clock.t += timeout  # nothing ever arrives
        return ""

    out, reason = drain_for_marker(
        read, "", "[DONE]", "*** BUGCHECK", 7200.0, idle_secs=600.0, now=clock
    )
    assert reason == "idle"
    assert "[DONE]" not in out
    assert clock.t < 700.0, f"burned {clock.t}s of a 7200s budget on a wedged guest"


def test_a_slow_but_PROGRESSING_sweep_is_not_mistaken_for_a_wedge() -> None:
    """The converse, which is what makes the cutoff safe to enable.

    Output arriving in bursts further apart than a subtest's own runtime must
    not trip the idle cutoff, or every long sweep gets killed as 'wedged' and
    the attribution swings wrong in the other direction.
    """
    clock = FakeClock()
    bursts = [0]

    def read(timeout: float = 1.0) -> str:
        clock.t += timeout
        bursts[0] += 1
        # A line every ~120s: the sweep's own per-subtest timeout cap.
        return "still working\n" if bursts[0] % 120 == 0 else ""

    _out, reason = drain_for_marker(
        read, "", "[DONE]", "*** BUGCHECK", 3600.0, idle_secs=600.0, now=clock
    )
    assert reason == "budget", "a progressing sweep must not read as idle"
