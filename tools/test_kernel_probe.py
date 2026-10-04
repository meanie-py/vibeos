"""Host regressions for the shipping Rust kernel pointer helpers."""

import subprocess
import sys
from pathlib import Path

import pytest


@pytest.mark.skipif(sys.platform != "linux", reason="uses Linux process_vm_readv")
def test_kernel_probe(tmp_path: Path) -> None:
    repo = Path(__file__).resolve().parent.parent
    binary = tmp_path / "kernel-probe-tests"
    subprocess.run(
        [
            "rustc",
            "--edition=2021",
            "--test",
            str(repo / "tests/host/kernel_probe.rs"),
            "-o",
            str(binary),
        ],
        cwd=repo,
        check=True,
        capture_output=True,
        text=True,
    )
    result = subprocess.run([str(binary)], check=True, capture_output=True, text=True)
    assert "9 passed; 0 failed" in result.stdout
