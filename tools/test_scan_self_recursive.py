"""Binary-check tool failures must not look like clean disassembly."""

import subprocess
import sys
from pathlib import Path

import pytest

sys.path.insert(0, str(Path(__file__).resolve().parent))

import scan_self_recursive as scanner


@pytest.mark.parametrize("failure", ["missing", "timeout", "invalid"])
def test_disassembler_failure_is_not_clean(tmp_path, monkeypatch, failure, capsys):
    binary = tmp_path / "kernel.exe"
    binary.write_bytes(b"fixture")

    def execute(cmd, **kwargs):
        assert kwargs["check"] is True
        if failure == "missing":
            raise FileNotFoundError("objdump missing")
        if failure == "timeout":
            raise subprocess.TimeoutExpired(cmd, 180)
        raise subprocess.CalledProcessError(1, cmd, stderr="invalid PE")

    monkeypatch.setattr(scanner.subprocess, "run", execute)
    monkeypatch.setattr(scanner, "objdump", lambda _: sys.executable)
    monkeypatch.setattr(sys, "argv", ["scan_self_recursive", str(tmp_path), "--check"])
    assert scanner.main() == 2
    captured = capsys.readouterr()
    assert "cannot disassemble" in captured.err
    assert "clean" not in captured.out


def test_empty_build_is_not_clean(tmp_path, monkeypatch, capsys):
    monkeypatch.setattr(sys, "argv", ["scan_self_recursive", str(tmp_path), "--check"])
    assert scanner.main() == 2
    assert "no PE inputs" in capsys.readouterr().err


def test_self_jump_detection_survives_error_handling(tmp_path, monkeypatch):
    def execute(cmd, **kwargs):
        return subprocess.CompletedProcess(
            cmd, 0, "00001000 <loop>:\n    1000: jmp 0x1000 <loop>\n", ""
        )

    monkeypatch.setattr(scanner.subprocess, "run", execute)
    assert scanner.scan_pe("objdump", tmp_path / "kernel.exe") == [("kernel.exe", "loop")]
