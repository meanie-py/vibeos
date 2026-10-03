"""`vibeos-cli debug start` records a session that outlives the CLI as the VM's owner."""

from __future__ import annotations

import os
import subprocess
import sys

from tests.qemu.cli import debugcmd


def test_a_non_shell_start_is_its_own_owner():
    assert debugcmd._session_owner(os.getpid()) == os.getpid()


def test_owner_walks_past_a_live_shell(tmp_path):
    marker = tmp_path / "owner"
    script = (
        f'{sys.executable} -c "import os,sys; sys.path.insert(0, {str(os.getcwd())!r}); '
        "from tests.qemu.cli import debugcmd; "
        f"open({str(marker)!r}, 'w').write(str(debugcmd._session_owner(os.getppid())))\""
    )
    subprocess.run(["bash", "-c", script + "; true"], check=True)
    assert int(marker.read_text()) == os.getpid()


def test_start_passes_the_owner_to_start_sh(monkeypatch, tmp_path):
    (tmp_path / "qemu").mkdir()
    (tmp_path / "qemu" / "start.sh").write_text("")
    seen = {}

    def fake_run(cmd, cwd=None, env=None):
        seen["env"] = env
        return subprocess.CompletedProcess(cmd, 0)

    monkeypatch.delenv("VIBEOS_QEMU_OWNER_PID", raising=False)
    monkeypatch.setattr(debugcmd.subprocess, "run", fake_run)
    args = type("A", (), {"verb": "start", "rest": ["arm64"]})()
    assert debugcmd.debug(args, tmp_path) == 0
    assert seen["env"]["VIBEOS_QEMU_OWNER_PID"].isdigit()
