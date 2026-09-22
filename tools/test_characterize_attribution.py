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
CLEAN = "1 failed, 3 skipped in 1807.32s (0:30:07)\n"


def test_a_bugcheck_is_evidence() -> None:
    assert should_quarantine("0x0000001E", CLEAN) is True


def test_the_watchdog_killing_the_vm_is_not_the_subtests_fault() -> None:
    """The regression that recurred: a timer ended the boot, not the guest."""
    assert should_quarantine(None, WATCHDOG) is False


def test_a_bugcheck_still_counts_even_if_the_watchdog_also_fired() -> None:
    """A guest that crashed AND then got reaped is still a guest that crashed.

    Ordering matters here: the harness output is a whole run's log, so a late
    watchdog line must not retroactively excuse a real banner.
    """
    assert should_quarantine("0x0000001E", WATCHDOG) is True


def test_a_boot_that_simply_stopped_is_still_attributable() -> None:
    """Without a harness-kill sign, an incomplete sweep IS the guest's doing.

    This keeps the fix honest: it must not become "never quarantine anything",
    which would make the whole characterisation loop inert.
    """
    assert should_quarantine(None, CLEAN) is True


def test_pytest_timeout_is_also_a_harness_kill() -> None:
    assert should_quarantine(None, "E   Failed: Timeout >7800.0s\n") is False
