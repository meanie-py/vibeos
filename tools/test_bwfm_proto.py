"""Host-run the Pi Wi-Fi driver's protocol tests (bwfm-rust's proto.rs).

SDPCM framing, BCDC commands and data headers, and firmware events are pure code over bytes.
On the board a mistake in any of them looks the same: the chip answers nothing, or answers
the wrong request. So the layouts are checked here, before there is a bus to try them on.
"""

import os
import shutil
import subprocess
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]
MODULE = "sdk/lib/rust/bwfm-rust/src/proto.rs"


def test_bwfm_proto(tmp_path: Path) -> None:
    source = REPO / MODULE
    assert source.is_file(), f"missing {MODULE}"
    rustc = shutil.which("rustc")
    assert rustc, "rustc is required for the protocol tests"
    binary = tmp_path / "tests"
    env = os.environ.copy()
    env.pop("RUSTC_WRAPPER", None)
    env.pop("RUSTC_WORKSPACE_WRAPPER", None)
    built = subprocess.run(
        [rustc, "--test", "--edition=2021", str(source), "-o", str(binary)],
        capture_output=True,
        text=True,
        env=env,
        timeout=180,
        check=False,
    )
    assert built.returncode == 0, built.stdout + built.stderr
    ran = subprocess.run([str(binary)], capture_output=True, text=True, timeout=60, check=False)
    assert ran.returncode == 0, ran.stdout + ran.stderr
    assert "running 0 tests" not in ran.stdout, ran.stdout
