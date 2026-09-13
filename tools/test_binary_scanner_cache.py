"""Exercise cached shipping scanners through real subprocesses and baseline edits."""

import os
import subprocess
import sys
from pathlib import Path

import pytest

REPO = Path(__file__).resolve().parents[1]


def test_self_recursion_cache_rechecks_baseline_and_changed_binary(tmp_path):
    tool = tmp_path / "bin/llvm-objdump"
    tool.parent.mkdir()
    counter = tmp_path / "calls"
    tool.write_text(
        f"#!{sys.executable}\n"
        "import pathlib, sys\n"
        f"p = pathlib.Path({str(counter)!r})\n"
        "p.write_text(p.read_text() + 'x' if p.exists() else 'x')\n"
        "print('00001000 <loop>:\\n    1000: ' + "
        "('jmp 0x1000 <loop>' if pathlib.Path(sys.argv[-1]).read_text() == 'bad' else 'nop'))\n"
    )
    tool.chmod(0o755)
    build = tmp_path / "build-amd64"
    build.mkdir()
    binary = build / "fixture.exe"
    binary.write_text("bad")
    baseline = tmp_path / "tools/lint-baseline/scan_self_recursive.txt"
    baseline.parent.mkdir(parents=True)
    env = dict(os.environ, PATH=f"{tool.parent}:{os.environ['PATH']}")
    env.pop("VIBEOS_BINARY_CACHE", None)

    def check(expected):
        cp = subprocess.run(
            [sys.executable, str(REPO / "tools/scan_self_recursive.py"), "--check", str(build)],
            env=env,
            capture_output=True,
            text=True,
        )
        assert cp.returncode == expected, cp.stdout + cp.stderr

    check(1)
    baseline.write_text("fixture.exe: loop\n")
    check(0)
    assert counter.read_text() == "x", "baseline edits must reuse facts, not an old verdict"
    baseline.write_text("")
    check(1)
    before = binary.stat()
    binary.write_text("nop")
    os.utime(binary, ns=(before.st_atime_ns, before.st_mtime_ns))
    check(0)
    assert counter.read_text() == "xx"


@pytest.mark.parametrize("function", ["scan_ud2", "scan_driver_binary", "find_rust_binaries"])
def test_ud2_failed_tools_never_report_clean(tmp_path, monkeypatch, function):
    from tools import scan_ud2

    binary = tmp_path / "fixture.sys"
    binary.write_bytes(b"invalid PE")

    def failed(cmd, **kwargs):
        assert kwargs["check"] is True
        raise subprocess.CalledProcessError(1, cmd, stderr="invalid PE")

    monkeypatch.setattr(scan_ud2.subprocess, "run", failed)
    with pytest.raises(subprocess.CalledProcessError):
        if function == "find_rust_binaries":
            scan_ud2.find_rust_binaries(tmp_path, "nm")
        else:
            getattr(scan_ud2, function)(binary, "objdump", "i386")
