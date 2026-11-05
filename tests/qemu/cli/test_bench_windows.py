"""Portable bench services must also work as standalone Windows processes."""

import shutil
import subprocess
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[3]


@pytest.mark.skipif(not shutil.which("powershell.exe"), reason="requires Windows interop")
def test_standalone_windows_http_and_tftp():
    built = subprocess.run(
        [str(ROOT / "tools/bench-infra"), "--windows-path"],
        cwd=ROOT,
        capture_output=True,
        text=True,
        timeout=600,
        check=True,
    )
    binary = Path(built.stdout.strip())
    assert binary.is_file()
    fixture = Path(__file__).with_name("data") / "bench-services-windows.ps1"
    paths = [
        subprocess.check_output(["wslpath", "-w", str(p)], text=True).strip()
        for p in (fixture, binary)
    ]
    run = subprocess.run(
        [
            "powershell.exe",
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-File",
            paths[0],
            "-Binary",
            paths[1],
        ],
        cwd=ROOT,
        capture_output=True,
        text=True,
        timeout=60,
    )
    assert run.returncode == 0, run.stdout + run.stderr
    assert "WINDOWS_BENCH_SERVICES_OK" in run.stdout
