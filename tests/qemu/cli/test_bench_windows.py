"""Portable bench services must also work as standalone Windows processes."""

import shutil
import subprocess
from pathlib import Path

import pytest

from .. import backend

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
    # Through run_interop: on 2026-11-07 a gate failed here with powershell.exe never having
    # started (`UtilAcceptVsock: accept4 failed 110`), which says nothing about the services.
    run = backend.run_interop(
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


def test_interop_launch_failure_is_retried_once_and_nothing_else_is(monkeypatch):
    """A failed interop LAUNCH never ran anything on Windows, so one more try is safe. Any
    other failure is the program's own answer and must come back unretried."""
    vsock = "<3>WSL (2552400 - ) ERROR: UtilAcceptVsock:281: accept4 failed 110\n"
    answers: list[subprocess.CompletedProcess] = []
    calls: list[list[str]] = []

    def fake(cmd, **_kw):
        calls.append(cmd)
        return answers.pop(0)

    monkeypatch.setattr(backend.subprocess, "run", fake)

    answers[:] = [
        subprocess.CompletedProcess([], 1, "", vsock),
        subprocess.CompletedProcess([], 0, "ok", ""),
    ]
    assert backend.run_interop(["powershell.exe"], capture_output=True, text=True).stdout == "ok"
    assert len(calls) == 2

    calls.clear()
    answers[:] = [subprocess.CompletedProcess([], 1, "", "Copy-Item: access denied")]
    assert backend.run_interop(["powershell.exe"], capture_output=True, text=True).returncode == 1
    assert len(calls) == 1

    calls.clear()
    answers[:] = [subprocess.CompletedProcess([], 1, "", vsock)] * 2
    with pytest.raises(subprocess.CalledProcessError):
        backend.run_interop(["powershell.exe"], capture_output=True, text=True, check=True)
    assert len(calls) == 2
