"""`--deterministic-only` must make ZERO paid calls. Proven by running it without keys.

This is deliberately an end-to-end subprocess test rather than a unit test of the guard,
because a unit test would have passed on the broken version. The first implementation set
`paid_deps = []` — which reads like it disables paying and does not, since the translation
loops iterate `sorted_deps` and `paid_deps` only feeds the cost banner. It spent $0.90 on
mbcs.c while reporting that it was skipping everything.

So the property under test is not "a flag is checked somewhere", it is "no money leaves the
building". The only honest way to assert that is to remove the API keys and confirm the run
still completes: any attempted call would fail loudly on auth instead of returning a body.
"""

from __future__ import annotations

import os
import re
import subprocess
import sys
from pathlib import Path

import pytest

REPO_ROOT = Path(__file__).resolve().parents[3]
RUSTCONV_DIR = REPO_ROOT / "tools" / "rustconv"
# 12 functions, 10 of which lower deterministically -> both branches are exercised.
SUBJECT = REPO_ROOT / "dll/win32/msvcrt/mathf.c"


def _configured_msvcrt() -> bool:
    """A build-<arch> whose compile database already lists msvcrt's C.

    translate derives the TU's macro set from that database (macro_authority), so without it
    the run refuses before the property under test is reached. The merge gate runs this suite
    BESIDE its `--clean` builds, which delete compile_commands.json for minutes: that window
    turned origin/master "red" with nothing wrong (2026-10-09) and held the whole queue.
    """
    for db in REPO_ROOT.glob("build-*/compile_commands.json"):
        try:
            if "dll/win32/msvcrt/" in db.read_text(errors="replace"):
                return True
        except OSError:
            continue
    return False


@pytest.mark.skipif(not SUBJECT.is_file(), reason="msvcrt sources not present")
@pytest.mark.skipif(
    not _configured_msvcrt(),
    reason="no configured build-<arch> lists msvcrt (./vibeos-cli build --arch i386)",
)
def test_deterministic_only_spends_nothing_and_still_assembles(tmp_path):
    env = {
        **os.environ,
        # Both backends unusable. If the run tries to pay, it cannot succeed quietly.
        "ANTHROPIC_API_KEY": "",
        "DEEPSEEK_API_KEY": "",
        "RUSTCONV_RESPONSES_DIR": str(tmp_path / "responses"),
        "PYTHONPATH": str(RUSTCONV_DIR) + os.pathsep + os.environ.get("PYTHONPATH", ""),
    }
    out = tmp_path / "mathf.rs"
    proc = subprocess.run(
        [
            sys.executable, "-m", "rustconv", "translate",
            str(SUBJECT.relative_to(REPO_ROOT)),
            "--target", "msvcrt",
            "--output-crate", "msvcrt-crt-rust",
            "--deterministic-only",
            "--no-resume",  # no cache to hide behind: a paid call would have to be made
            "-o", str(out),
        ],
        cwd=REPO_ROOT,
        env=env,
        capture_output=True,
        text=True,
        timeout=300,
    )
    combined = proc.stdout + proc.stderr
    assert proc.returncode == 0, combined

    # 1. Nothing was spent, and nothing was archived (an archived unit IS a paid call).
    spent = re.search(r"Total spent this run: \$0\.00\s+\(0 unit\(s\) archived\)", combined)
    assert spent, combined
    assert "Total tokens: 0 in / 0 out" in combined, combined

    # 2. The paid band was skipped explicitly rather than silently succeeding.
    assert "SKIPPED (--deterministic-only)" in combined, combined
    assert "[preamble] SKIPPED (--deterministic-only)" in combined, combined

    # 3. The free band was still produced -- the flag takes work, it does not just abort.
    assert "DETERMINISTIC ($0)" in combined, combined
    assert out.is_file() and out.read_text().strip(), "nothing assembled"
