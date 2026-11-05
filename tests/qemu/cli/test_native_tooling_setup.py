"""Scanner provisioning must precede the unit-test process and propagate failure."""

import importlib.util
import sys
from pathlib import Path

import pytest

from tests.qemu.cli import rustlints


@pytest.fixture
def tooling():
    path = Path(__file__).with_name("tooling.py")
    spec = importlib.util.spec_from_file_location("tests.qemu.cli._bench_tooling_candidate", path)
    assert spec and spec.loader
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def test_scanner_is_ready_before_its_control_process_starts(tmp_path, monkeypatch, tooling):
    ready = tmp_path / "prepared"

    def prepare(repo):
        assert repo == tmp_path
        ready.write_text("native scanner ready")

    monkeypatch.setattr(rustlints, "ensure", prepare)
    cmd = [
        sys.executable,
        "-c",
        "from pathlib import Path; "
        "assert Path('prepared').read_text()=='native scanner ready'; "
        "print('control ran')",
    ]
    label, ok, out = tooling._run("pytest:tools/test_lint_facts.py", cmd, tmp_path)
    assert label == "pytest:tools/test_lint_facts.py" and ok and out == "control ran"


def test_failed_provision_never_starts_the_control(tmp_path, monkeypatch, tooling):
    def fail(_repo):
        raise RuntimeError("compiler failed loudly")

    monkeypatch.setattr(rustlints, "ensure", fail)
    cmd = [sys.executable, "-c", "from pathlib import Path; Path('launched').touch()"]
    _label, ok, out = tooling._run("pytest:tools/test_lint_facts.py", cmd, tmp_path)
    assert not ok and "compiler failed loudly" in out
    assert not (tmp_path / "launched").exists()
