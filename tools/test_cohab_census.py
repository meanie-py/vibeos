"""The census must FIRE on the exact shape that froze the guest on 2026-09-19.

A gate nobody has watched trip is not a gate. The case that matters is one character:
removing the `#` from a CMake source-list line re-enables a cohab-flipped `.c` next to
its Rust twin, and no boot test can see the result.
"""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
TOOL = REPO / "tools" / "cohab_census.py"
CENSUS = REPO / "tools" / "cohab-census.txt"


def _run() -> subprocess.CompletedProcess:
    return subprocess.run([sys.executable, str(TOOL)], capture_output=True, text=True, cwd=REPO)


def test_snapshot_is_committed_and_current():
    """The checked-in snapshot must match the tree, or the gate is meaningless."""
    assert CENSUS.is_file(), "tools/cohab-census.txt missing — run --write"
    cp = _run()
    assert cp.returncode == 0, f"census drifted from the tree:\n{cp.stdout}"
    assert cp.stdout.strip() == ""


def test_snapshot_records_a_known_flip():
    """lpc/reply.c is flipped: on disk, commented out, owned by lpc-rust."""
    body = CENSUS.read_text(encoding="utf-8")
    assert "ntoskrnl/lpc/reply.c" in body
    assert (REPO / "ntoskrnl" / "lpc" / "reply.c").is_file()


def test_uncommenting_a_source_line_is_a_finding(tmp_path):
    """Drop one `#` and the gate must say so. This is the 2026-09-19 failure shape."""
    target = REPO / "ntoskrnl" / "ntos.cmake"
    original = target.read_text(encoding="utf-8")
    needle = "#${REACTOS_SOURCE_DIR}/ntoskrnl/lpc/reply.c"
    assert needle in original, "fixture drifted: reply.c no longer flipped in ntos.cmake"
    try:
        target.write_text(original.replace(needle, needle[1:], 1), encoding="utf-8")
        cp = _run()
    finally:
        target.write_text(original, encoding="utf-8")
    assert cp.returncode == 1
    assert "[CENSUS-REMOVED]" in cp.stdout
    assert "lpc/reply.c" in cp.stdout
    # and the tree is back to clean
    assert _run().returncode == 0
