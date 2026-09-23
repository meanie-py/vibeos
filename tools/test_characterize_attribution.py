"""A characterisation round may only accuse a test on POSITIVE fault evidence.

This is the oracle for the most expensive bug the apitest harness has had, and
the one that came back after being fixed once. `characterize.py` reruns a sweep
that stopped early and quarantines whichever subtest was in flight. That
reasoning is only sound if the guest is what stopped -- and three separate
bounds could end the boot instead:

  1. the drain loop counting read iterations rather than seconds,
  2. pytest's --timeout,
  3. the QEMU VM lifetime watchdog (30 min, and long unreachable from pytest).

Seven innocent subtests were quarantined via (1). When (1) was fixed, (2)/(3)
produced four more on the very next run -- same mechanism, different timer. So
the durable fix is not raising budgets, which only moves the next-lowest bound
into place; it is refusing to accuse anyone without a guest-side fault.
"""

from __future__ import annotations

import importlib.util
from pathlib import Path

_spec = importlib.util.spec_from_file_location(
    "characterize", Path(__file__).resolve().parent / "payloads" / "apitest" / "characterize.py"
)
assert _spec and _spec.loader
characterize = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(characterize)

should_quarantine = characterize.should_quarantine

WATCHDOG = (
    "[QemuVM] WATCHDOG: amd64-uefi VM exceeded 30-min hard cap — killing pid=2103805\n"
    "E           ConnectionError: QEMU exited with code -15\n"
)
BUDGET = "[payload] drain-end: budget\n1 failed, 3 skipped in 7262.33s (2:01:02)\n"
IDLE = "[payload] drain-end: idle\n1 failed, 3 skipped in 912.11s (0:15:12)\n"


def test_a_bugcheck_is_evidence() -> None:
    assert should_quarantine("0x0000001E", BUDGET) is True


def test_a_guest_that_went_silent_is_evidence() -> None:
    """cmd_apitest's three real hangs were caught exactly this way."""
    assert should_quarantine(None, IDLE) is True


def test_the_rig_running_out_of_time_is_NOT_evidence() -> None:
    """The original bug: seven innocent subtests quarantined on this shape."""
    assert should_quarantine(None, BUDGET) is False


def test_the_watchdog_killing_the_vm_is_not_the_subtests_fault() -> None:
    """The recurrence: one timer fixed, the next one down took over."""
    assert should_quarantine(None, WATCHDOG) is False


def test_a_bugcheck_still_counts_even_if_the_rig_also_gave_up() -> None:
    """A guest that crashed AND then got reaped is still a guest that crashed."""
    assert should_quarantine("0x0000001E", WATCHDOG) is True


def test_silence_beats_a_later_watchdog_line() -> None:
    """A wedge detected by the idle cutoff stands even if the VM is reaped after."""
    assert should_quarantine(None, IDLE + WATCHDOG) is True


def test_the_default_is_INNOCENT() -> None:
    """No signal at all must not accuse anyone.

    This is the polarity the original rule had backwards: it accused by default
    and required nothing to do so.
    """
    assert should_quarantine(None, "") is False
