"""Shared native execution must supervise trees without losing evidence or concurrency."""

import json
import sys
import time
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

import pytest

from tools.build_infra_compat import NativeError

from . import artifacts


def alive(pid):
    try:
        return Path(f"/proc/{pid}/stat").read_text().split(") ")[1].split()[0] not in ("Z", "X")
    except FileNotFoundError:
        return False


@pytest.mark.parametrize("timeout", [False, True])
def test_owned_descendant_cannot_hold_output_pipes_open(tmp_path, monkeypatch, timeout):
    monkeypatch.setattr(artifacts, "ARTIFACT_ROOT", tmp_path / "runs")
    run = artifacts.Run("fixture", "all")
    pidfile = tmp_path / "descendant"
    script = (
        "import os,sys,time; from pathlib import Path; "
        "pid=os.fork(); "
        "Path(sys.argv[1]).write_text(str(pid)) if pid else None; "
        "print('partial witness',flush=True); "
        "time.sleep(60) if pid==0 or sys.argv[2]=='timeout' else sys.exit(17)"
    )
    start = time.monotonic()
    result = run.run(
        [sys.executable, "-c", script, str(pidfile), "timeout" if timeout else "exit"],
        timeout=2 if timeout else 10,
    )
    assert time.monotonic() - start < 15
    assert result.returncode == (124 if timeout else 17) and result.timed_out == timeout
    assert "partial witness" in result.stdout and "partial witness" in run.full_log.read_text()
    assert not alive(int(pidfile.read_text()))
    run.add(artifacts.Stage("owned-child", exit=result.returncode, status="fail"))
    assert run.finalize() == 1
    summary = json.loads((run.dir / "summary.json").read_text())
    assert summary["command_timings"][0]["timed_out"] == timeout
    assert summary["command_timings"][0]["exit"] == result.returncode


def test_parallel_commands_reach_each_other_before_either_can_finish(tmp_path, monkeypatch):
    monkeypatch.setattr(artifacts, "ARTIFACT_ROOT", tmp_path / "runs")
    run = artifacts.Run("fixture", "all")
    script = """import sys,time
from pathlib import Path
mine,other=map(Path,sys.argv[1:])
mine.touch()
end=time.monotonic()+10
while not other.exists():
    assert time.monotonic()<end,'shared metadata channel serialized commands'
    time.sleep(.01)
print(mine.name)
"""

    def command(i):
        return run.run(
            [sys.executable, "-c", script, str(tmp_path / str(i)), str(tmp_path / str(1 - i))],
            stage=f"parallel-{i}",
            timeout=15,
        )

    with ThreadPoolExecutor(max_workers=2) as pool:
        results = list(pool.map(command, range(2)))
    assert [r.returncode for r in results] == [0, 0]
    rows = [json.loads(line) for line in run.commands_log.read_text().splitlines()]
    assert len(rows) == 2 and {r["stage"] for r in rows} == {"parallel-0", "parallel-1"}
    assert all(Path(r["stdout_path"]).read_text().strip() in ("0", "1") for r in rows)


def test_external_adapter_distinguishes_failed_launch_from_program_exit_127(tmp_path):
    from . import gate

    with pytest.raises(OSError, match="cannot launch"):
        gate._execute([str(tmp_path / "absent-command")], cwd=tmp_path)
    result = gate._execute([sys.executable, "-c", "raise SystemExit(127)"], cwd=tmp_path)
    assert result.returncode == 127


def test_out_of_range_timeout_refuses_without_panicking(tmp_path, monkeypatch):
    monkeypatch.setattr(artifacts, "ARTIFACT_ROOT", tmp_path / "runs")
    run = artifacts.Run("fixture", "all")
    with pytest.raises(NativeError, match="timeout exceeds supported duration"):
        run.run([sys.executable, "-c", "raise SystemExit(0)"], timeout=1e30)
    assert run.run([sys.executable, "-c", "print('still alive')"]).stdout.strip() == "still alive"
