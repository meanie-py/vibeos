"""Build-bench adapters and real child orchestration against a disposable fake build."""

import argparse
import json
import os
import subprocess
import sys
from pathlib import Path

import pytest

from . import buildbench

REPO = Path(__file__).resolve().parents[3]
TOOL = REPO / "tools/build-infra"


def test_proc_reader_facades_and_parser(tmp_path):
    meminfo = tmp_path / "meminfo"
    meminfo.write_text("MemTotal: 100 kB\nHugePages_Total: 2\n")
    assert buildbench._read_meminfo(meminfo) == {"MemTotal": 102400, "HugePages_Total": 2}
    vmstat = tmp_path / "vmstat"
    vmstat.write_text("oom_kill 4\npgmajfault 9\npgfault 100\n")
    assert buildbench._read_vmstat(vmstat) == {"oom_kill": 4, "pgmajfault": 9}
    pressure = buildbench._read_pressure(tmp_path)
    assert all(
        v == {"available": False, "reason": "kernel PSI file absent"} for v in pressure.values()
    )
    assert buildbench._media_path(tmp_path, "arm64") == tmp_path / "qemu/ramdisk.img"
    parser = argparse.ArgumentParser()
    sub = parser.add_subparsers()
    buildbench.add_parser(sub)
    args = parser.parse_args(["build-bench", "--arches", "amd64", "i386"])
    assert args.arches == ["amd64", "i386"]
    assert args.repeat == 1
    assert args.fn is buildbench.run


def _fixture_repo(tmp_path):
    repo = tmp_path / "source"
    repo.mkdir()
    subprocess.run(["git", "init", "-q"], cwd=repo, check=True)
    (repo / ".gitignore").write_text("build-*/\nqemu/\n.cache/\n")
    (repo / "source.c").write_text("before\n")
    cli = repo / "vibeos-cli"
    cli.write_text(
        """#!/bin/sh
set -eu
kind="$1"
shift
arch=""
while [ "$#" -gt 0 ]; do
  case "$1" in --arch) arch="$2"; shift;; esac
  shift
done
if [ "$kind" = build ]; then
  if [ "${MUTATE_SOURCE:-}" = 1 ]; then printf changed > source.c; fi
  sleep 0.5
  if [ "${FAIL_BUILD:-}" = 1 ]; then exit 7; fi
  if [ "$arch" = arm64 ]; then
    mkdir -p qemu; printf ramdisk > qemu/ramdisk.img; printf boot > qemu/boot.img
  else
    mkdir -p "build-$arch"; printf media > "build-$arch/livecd.iso"
  fi
fi
if [ "${NO_SUMMARY:-}" != 1 ]; then
  mkdir -p "$VIBEOS_ARTIFACT_ROOT/run"
  printf '{"verdict":"OK"}\\n' > "$VIBEOS_ARTIFACT_ROOT/run/summary.json"
fi
"""
    )
    cli.chmod(0o755)
    subprocess.run(["git", "add", "."], cwd=repo, check=True)
    subprocess.run(
        [
            "git",
            "-c",
            "user.email=fixture@example.invalid",
            "-c",
            "user.name=Fixture",
            "commit",
            "-q",
            "-m",
            "fixture",
        ],
        cwd=repo,
        check=True,
    )
    return repo


def test_real_child_phases_resource_samples_and_refusals(tmp_path):
    repo = _fixture_repo(tmp_path)
    output = tmp_path / "report.json"
    command = [
        str(TOOL),
        "--root",
        str(repo),
        "build-bench",
        "--arches",
        "amd64",
        "i386",
        "--clean",
        "--output",
        str(output),
    ]
    proc = subprocess.run(command, text=True, capture_output=True, check=False)
    assert proc.returncode == 0, (proc.stdout, proc.stderr)
    report = json.loads(output.read_text())
    repeat = report["repeats"][0]
    assert report["status"] == "OK"
    assert repeat["artifacts"]["i386"]["exists"]
    assert any(row["name"] == "build-amd64-restage" for row in repeat["children"])
    samples = [json.loads(row) for row in Path(repeat["samples_jsonl"]).read_text().splitlines()]
    assert any(row["process_tree"]["processes"] for row in samples)
    assert all(row["meminfo_bytes"]["MemAvailable"] > 0 for row in samples)
    assert subprocess.run(command, capture_output=True, check=False).returncode != 0

    fail = tmp_path / "failure.json"
    proc = subprocess.run(
        [str(TOOL), "--root", str(repo), "build-bench", "--arches", "i386", "--output", str(fail)],
        env={**os.environ, "FAIL_BUILD": "1"},
        text=True,
        capture_output=True,
        check=False,
    )
    assert proc.returncode != 0
    result = json.loads(fail.read_text())
    assert result["status"] == "FAILED"
    assert any(row["status"] == "SKIPPED" for row in result["repeats"][0]["children"])

    no_summary = tmp_path / "missing-summary.json"
    proc = subprocess.run(
        [
            str(TOOL),
            "--root",
            str(repo),
            "build-bench",
            "--arches",
            "i386",
            "--output",
            str(no_summary),
        ],
        env={**os.environ, "NO_SUMMARY": "1"},
        text=True,
        capture_output=True,
        check=False,
    )
    assert proc.returncode != 0
    result = json.loads(no_summary.read_text())
    assert any(
        "summary" in row.get("artifact_error", "") for row in result["repeats"][0]["children"]
    )

    changed = tmp_path / "changed.json"
    proc = subprocess.run(
        [
            str(TOOL),
            "--root",
            str(repo),
            "build-bench",
            "--arches",
            "i386",
            "--output",
            str(changed),
        ],
        env={**os.environ, "MUTATE_SOURCE": "1"},
        text=True,
        capture_output=True,
        check=False,
    )
    assert proc.returncode != 0
    result = json.loads(changed.read_text())
    assert "git state changed" in result["repeats"][0]["error"]


def test_worker_receipt_preserves_signal_and_launch_errors(tmp_path):
    receipt = tmp_path / "signal.json"
    proc = subprocess.run(
        [
            str(TOOL),
            "--root",
            str(tmp_path),
            "build-bench-worker",
            str(receipt),
            "/bin/sh",
            "-c",
            "kill -TERM $$",
        ],
        text=True,
        capture_output=True,
        check=False,
    )
    assert proc.returncode == 143
    assert json.loads(receipt.read_text())["exit_code"] == -15
    missing = tmp_path / "missing.json"
    proc = subprocess.run(
        [
            str(TOOL),
            "--root",
            str(tmp_path),
            "build-bench-worker",
            str(missing),
            str(tmp_path / "missing-executable"),
        ],
        text=True,
        capture_output=True,
        check=False,
    )
    assert proc.returncode == 127
    assert json.loads(missing.read_text())["exit_code"] == 127
    assert "missing-executable" in proc.stderr
    assert "launch_error" in json.loads(missing.read_text())


def test_child_status_and_missing_summary_refusals(tmp_path):
    missing = buildbench._run_child(
        "missing", [str(tmp_path / "no-executable")], tmp_path, tmp_path / "missing.log"
    )
    assert missing["exit_code"] == 127
    assert "launch_error" in missing
    assert Path(missing["log"]).is_file()
    signaled = buildbench._run_child(
        "signal", ["/bin/sh", "-c", "kill -TERM $$"], tmp_path, tmp_path / "signal.log"
    )
    assert signaled["exit_code"] == -15
    no_summary = buildbench._run_child("zero", ["/bin/true"], tmp_path, tmp_path / "zero.log")
    assert no_summary["exit_code"] == 1
    assert "summary" in no_summary["artifact_error"]


def test_killed_worker_does_not_claim_child_signal(tmp_path):
    row = buildbench._run_child(
        "worker-loss",
        ["/bin/sh", "-c", "kill -KILL $PPID; sleep 1"],
        tmp_path,
        tmp_path / "worker-loss.log",
    )
    assert row["exit_code"] == 1
    assert row["worker_exit_code"] == -9
    assert "status receipt" in row["artifact_error"]


def test_worker_stops_owned_child_when_owner_exits(tmp_path):
    import time

    child_pid_file = tmp_path / "child.pid"
    receipt = tmp_path / "worker-receipt.json"
    launcher = """import subprocess, sys
p = subprocess.Popen([
    sys.argv[1], "--root", sys.argv[2], "build-bench-worker", sys.argv[3],
    "/bin/sh", "-c", 'echo $$ > "$1"; exec sleep 30', "_", sys.argv[4],
])
p.wait()
"""
    owner = subprocess.Popen(
        [
            sys.executable,
            "-c",
            launcher,
            str(TOOL),
            str(tmp_path),
            str(receipt),
            str(child_pid_file),
        ],
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )
    try:
        deadline = time.monotonic() + 10
        while time.monotonic() < deadline and not child_pid_file.exists():
            assert owner.poll() is None
            time.sleep(0.02)
        assert child_pid_file.exists(), "worker did not launch child"
        pid = int(child_pid_file.read_text())
        owner.terminate()
        owner.wait(timeout=5)
        while time.monotonic() < deadline:
            try:
                state = Path(f"/proc/{pid}/stat").read_text().rsplit(")", 1)[1].split()[0]
            except FileNotFoundError:
                break
            if state == "Z":
                break
            time.sleep(0.02)
        else:
            raise AssertionError("benchmark child survived its owner")
    finally:
        if owner.poll() is None:
            owner.terminate()
            owner.wait(timeout=5)


def test_facade_routes_args_and_inherited_descriptors(tmp_path, monkeypatch):
    from types import SimpleNamespace

    captured = {}

    def call(argv, **kwargs):
        captured.update(argv=argv, kwargs=kwargs)
        return 7

    monkeypatch.setattr(buildbench.subprocess, "call", call)
    monkeypatch.setattr(buildbench, "_inherited_descriptors", lambda: (5, 8))
    args = SimpleNamespace(
        arches=["amd64", "i386"], clean=True, repeat=2, output=tmp_path / "r.json"
    )
    assert buildbench.run(args, tmp_path) == 7
    assert captured["kwargs"]["pass_fds"] == (5, 8)
    assert captured["argv"][1:] == [
        "--root",
        str(tmp_path),
        "build-bench",
        "--arches",
        "amd64",
        "i386",
        "--clean",
        "--repeat",
        "2",
        "--output",
        str(tmp_path / "r.json"),
    ]


def _fixture_alive(identity):
    try:
        fields = Path(f"/proc/{identity['pid']}/stat").read_text().rsplit(")", 1)[1].split()
        return fields[19] == identity["start_ticks"] and fields[0] not in ("Z", "X")
    except FileNotFoundError:
        return False


# Identity files are published with os.replace: the waiters poll exists(), and a plain
# open(...,"w").write() made the file visible empty first (JSONDecodeError, batch 145).
_SLEEP_CHILD = """import os,sys,json,time,signal
if sys.argv[2]=='stubborn': signal.signal(signal.SIGTERM,signal.SIG_IGN)
identity={'pid':os.getpid(),'start_ticks':open('/proc/self/stat').read().rsplit(')',1)[1].split()[19]}
open(sys.argv[1]+'.tmp','w').write(json.dumps(identity))
os.replace(sys.argv[1]+'.tmp',sys.argv[1])
time.sleep(60)
"""


def _child_family(pidfile, stubborn, kill_runner):
    program = """import os,subprocess,sys,time,signal
p=subprocess.Popen([sys.executable,'-c',sys.argv[1],sys.argv[2],sys.argv[3]])
while not os.path.exists(sys.argv[2]): time.sleep(.01)
if sys.argv[4]=='kill': os.kill(os.getppid(),signal.SIGKILL)
p.wait()
"""
    return [
        sys.executable,
        "-c",
        program,
        _SLEEP_CHILD,
        str(pidfile),
        "stubborn" if stubborn else "normal",
        "kill" if kill_runner else "wait",
    ]


@pytest.mark.parametrize("stubborn", [False, True])
def test_killed_runner_cannot_leave_a_grandchild_alive(tmp_path, stubborn):
    import os
    import signal

    pidfile = tmp_path / "grandchild.json"
    identity = None
    try:
        row = buildbench._run_child(
            "runner-loss",
            _child_family(pidfile, stubborn, True),
            tmp_path,
            tmp_path / "runner-loss.log",
        )
        identity = json.loads(pidfile.read_text())
        assert row["exit_code"] == 1 and row["worker_exit_code"] == -9
        assert not _fixture_alive(identity)
    finally:
        if identity and _fixture_alive(identity):
            os.kill(identity["pid"], signal.SIGKILL)


def test_controller_sigkill_stops_a_stubborn_grandchild(tmp_path):
    import os
    import signal
    import time

    pidfile = tmp_path / "grandchild.json"
    process = subprocess.Popen(
        [
            str(TOOL),
            "build-bench-worker",
            str(tmp_path / "receipt"),
            *_child_family(pidfile, True, False),
        ],
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )
    identity = None
    try:
        deadline = time.monotonic() + 10
        while not pidfile.exists() and time.monotonic() < deadline:
            assert process.poll() is None
            time.sleep(0.01)
        identity = json.loads(pidfile.read_text())
        process.kill()
        process.wait(timeout=5)
        deadline = time.monotonic() + 8
        while _fixture_alive(identity) and time.monotonic() < deadline:
            time.sleep(0.02)
        assert not _fixture_alive(identity)
    finally:
        if process.poll() is None:
            process.kill()
            process.wait(timeout=5)
        if identity and _fixture_alive(identity):
            os.kill(identity["pid"], signal.SIGKILL)


@pytest.mark.parametrize("code", [0, 7])
def test_detached_cache_service_keeps_its_separate_lifetime(tmp_path, code):
    import os
    import signal

    pidfile = tmp_path / "service.json"
    program = """import os,sys,time,json
from pathlib import Path
pid=os.fork()
if pid==0:
 os.setsid()
 null=os.open(os.devnull,os.O_RDWR)
 for fd in (0,1,2): os.dup2(null,fd)
 identity={'pid':os.getpid(),'start_ticks':open('/proc/self/stat').read().rsplit(')',1)[1].split()[19]}
 open(sys.argv[1]+'.tmp','w').write(json.dumps(identity))
 os.replace(sys.argv[1]+'.tmp',sys.argv[1])
 time.sleep(60)
 os._exit(0)
while not os.path.exists(sys.argv[1]): time.sleep(.01)
root=Path(os.environ['VIBEOS_ARTIFACT_ROOT'])/'run'
root.mkdir(parents=True)
(root/'summary.json').write_text('{"verdict":"OK"}')
sys.exit(int(sys.argv[2]))
"""
    identity = None
    try:
        row = buildbench._run_child(
            "service",
            [sys.executable, "-c", program, str(pidfile), str(code)],
            tmp_path,
            tmp_path / "service.log",
        )
        identity = json.loads(pidfile.read_text())
        assert row["exit_code"] == code and _fixture_alive(identity)
    finally:
        if identity and _fixture_alive(identity):
            os.kill(identity["pid"], signal.SIGKILL)


def test_worker_forwards_child_options_and_closes_private_receipt_fd(tmp_path):
    receipt = tmp_path / "receipt.json"
    program = """import os,sys
assert sys.argv[1:]==['--root','child-value','--client-pid','child-token']
for fd in os.listdir('/proc/self/fd'):
 try: path=os.readlink('/proc/self/fd/'+fd)
 except FileNotFoundError: continue
 assert path != os.environ['PRIVATE_RECEIPT_PATH'], (fd,path)
"""
    result = subprocess.run(
        [
            str(TOOL),
            "build-bench-worker",
            str(receipt),
            sys.executable,
            "-c",
            program,
            "--root",
            "child-value",
            "--client-pid",
            "child-token",
        ],
        env=dict(os.environ, PRIVATE_RECEIPT_PATH=str(receipt)),
        capture_output=True,
        text=True,
    )
    assert result.returncode == 0, result.stderr
    assert json.loads(receipt.read_text()) == {"exit_code": 0}
