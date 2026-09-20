"""The census must FIRE on the exact shape that froze the guest on 2026-09-19.

A gate nobody has watched trip is not a gate. The case that matters is one
character: removing the `#` from a CMake source-list line re-enables a
cohab-flipped `.c` next to its Rust twin, and no boot test can see the result.

These tests build their OWN fixture tree via `--root`. An earlier version edited
`ntoskrnl/ntos.cmake` in place and restored it in a `finally`, which passed here
and FAILED inside the merge queue's integration worktree -- a test that mutates
shared tracked state to prove a point is not hermetic, and the failure cost a
landing to diagnose. The only test that still touches the real tree is the one
whose whole subject IS the real tree.
"""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
TOOL = REPO / "tools" / "cohab_census.py"
CENSUS = REPO / "tools" / "cohab-census.txt"

FLIP = "    #${REACTOS_SOURCE_DIR}/ntoskrnl/lpc/reply.c\n"


def _run(*args: str) -> subprocess.CompletedProcess:
    return subprocess.run(
        [sys.executable, str(TOOL), *args], capture_output=True, text=True, cwd=REPO
    )


def _fixture(tmp_path: Path, body: str) -> Path:
    """A minimal tree with one CMake file, plus a snapshot of itself."""
    (tmp_path / "ntoskrnl").mkdir(parents=True, exist_ok=True)
    (tmp_path / "ntoskrnl" / "ntos.cmake").write_text(body, encoding="utf-8")
    seed = _run("--root", str(tmp_path), "--write")
    assert seed.returncode == 0, seed.stderr
    return tmp_path


def test_snapshot_is_committed_and_current():
    """The checked-in snapshot must match the real tree, or the gate is meaningless."""
    assert CENSUS.is_file(), "tools/cohab-census.txt missing -- run --write"
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
    root = _fixture(tmp_path, f"list(APPEND SOURCE\n{FLIP})\n")
    assert _run("--root", str(root)).returncode == 0, "fixture should start clean"

    (root / "ntoskrnl" / "ntos.cmake").write_text(
        f"list(APPEND SOURCE\n{FLIP.replace('#', '', 1)})\n", encoding="utf-8"
    )
    cp = _run("--root", str(root))
    assert cp.returncode == 1
    assert "[CENSUS-REMOVED]" in cp.stdout
    assert "lpc/reply.c" in cp.stdout
    # and it tells the reader which direction is dangerous, and what to do
    assert "dangerous direction" in cp.stdout
    assert "--write" in cp.stdout


def test_a_new_flip_is_reported_as_routine(tmp_path):
    """Adding a comment-out is ADDED, and must not be described as dangerous."""
    root = _fixture(tmp_path, "list(APPEND SOURCE\n)\n")
    (root / "ntoskrnl" / "ntos.cmake").write_text(
        f"list(APPEND SOURCE\n{FLIP})\n", encoding="utf-8"
    )
    cp = _run("--root", str(root))
    assert cp.returncode == 1
    assert "[CENSUS-ADDED]" in cp.stdout
    assert "Routine" in cp.stdout
    assert "dangerous direction" not in cp.stdout
