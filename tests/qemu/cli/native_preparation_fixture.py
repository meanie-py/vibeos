"""External tool/process fixtures for native preparation and launch ordering."""

from __future__ import annotations

import os
import shutil
import sys
from pathlib import Path

import pytest

from tools.qemu_infra_compat import DedicatedSession


@pytest.fixture
def preparation_tree(tmp_path, monkeypatch):
    for name in list(os.environ):
        if name.startswith("VIBEOS_QEMU_") or name in {
            "VIBEOS_BOOT_MEDIA",
            "VIBEOS_WRITABLE_ROOT",
            "VIBEOS_TRIPWIRE",
            "VIBEOS_AHCI_CD",
            "VIBEOS_TCG_THREAD",
            "VIBEOS_BOOT_TIMEOUT_SCALE",
        }:
            monkeypatch.delenv(name, raising=False)
    real_qemu_img = shutil.which("qemu-img")
    assert real_qemu_img, "native media controls require the project's qemu-img dependency"
    repo = tmp_path / "fixture-repo"
    firmware = repo / "qemu"
    firmware.mkdir(parents=True)
    for name in ("QEMU_EFI.fd", "efivars.fd", "OVMF32_CODE.fd", "OVMF32_VARS.fd"):
        (firmware / name).write_bytes(bytes(4096))
    (firmware / "boot.img").write_bytes(b"fixture boot media")
    for arch in ("i386", "amd64"):
        build = repo / f"build-{arch}"
        build.mkdir()
        (build / "livecd.iso").write_bytes(b"fixture ISO media")
    bin_dir = tmp_path / "bin"
    bin_dir.mkdir()
    trace = tmp_path / "trace"
    trace.mkdir()
    convert = bin_dir / "qemu-img"
    convert.write_text(f"""#!{sys.executable}
import fcntl, os, sys, time
from pathlib import Path
if len(sys.argv)>1 and sys.argv[1]=="convert":
    mode=os.environ.get("FAKE_PREPARATION", "normal")
    if mode=="fail":
        print("fixture media staging failed", file=sys.stderr)
        sys.exit(3)
    if mode=="barrier":
        root=Path(os.environ["FAKE_PREPARATION_TRACE"])
        lock=Path(os.environ["VIBEOS_HOSTLOCK_DIR"])/"qemu-port-handshake.lock"
        lock.parent.mkdir(parents=True,exist_ok=True)
        with lock.open("a+") as stream:
            fcntl.flock(stream,fcntl.LOCK_SH|fcntl.LOCK_NB)
            (root/f"prepare-{{os.getppid()}}").write_text(str(time.monotonic_ns()))
            deadline=time.monotonic()+10
            while len(list(root.glob("prepare-*")))<2:
                if time.monotonic()>deadline: raise RuntimeError("preparations failed to overlap")
                time.sleep(.01)
os.execv({real_qemu_img!r},[{real_qemu_img!r},*sys.argv[1:]])
""")
    convert.chmod(0o700)
    executable = bin_dir / "qemu-system-aarch64"
    original = Path(__file__).with_name("fake_qemu.py").read_text()
    prefix = f"""#!{sys.executable}
import fcntl as _fixture_fcntl, os as _fixture_os, time as _fixture_time
from pathlib import Path as _FixturePath
if _fixture_os.environ.get("FAKE_PREPARATION")=="barrier":
    _trace=_FixturePath(_fixture_os.environ["FAKE_PREPARATION_TRACE"])
    _lock=_FixturePath(_fixture_os.environ["VIBEOS_HOSTLOCK_DIR"])/"qemu-port-handshake.lock"
    with _lock.open("a+") as _stream:
        try: _fixture_fcntl.flock(_stream,_fixture_fcntl.LOCK_SH|_fixture_fcntl.LOCK_NB)
        except BlockingIOError: pass
        else: raise RuntimeError("QEMU started without the exclusive handshake lease")
    (_trace/f"launch-start-{{_fixture_os.getpid()}}").write_text(str(_fixture_time.monotonic_ns()))
"""
    # A completion stamp precedes the capabilities reply, hence precedes lease release.
    original = original.replace(
        '        stream.write((json.dumps({"return": result}) + "\\n").encode())',
        """        if command=="qmp_capabilities" and os.environ.get("FAKE_PREPARATION")=="barrier":
            (Path(os.environ["FAKE_PREPARATION_TRACE"])/f"launch-end-{os.getpid()}").write_text(str(time.monotonic_ns()))
            time.sleep(.1)
        stream.write((json.dumps({"return": result}) + "\\n").encode())""",
    )
    assert "launch-end-" in original, "fixture injection must follow the real QMP command parser"
    executable.write_text(prefix + original)
    executable.chmod(0o700)
    powershell = bin_dir / "powershell.exe"
    powershell.write_text(f"""#!{sys.executable}
from pathlib import Path
Path({str(trace / "unexpected-port-selection")!r}).write_text("called")
raise RuntimeError("media preparation selected a Windows port")
""")
    powershell.chmod(0o700)
    settings = {
        "PATH": f"{bin_dir}:{os.environ['PATH']}",
        "VIBEOS_HOSTLOCK_DIR": str(tmp_path / "host"),
        "VIBEOS_QEMU_LOCK_DIR": str(tmp_path / "locks"),
        "VIBEOS_BUILD_POLICY_DIR": str(tmp_path / "policy"),
        "VIBEOS_HOST_BROKER": "0",
        "VIBEOS_QEMU_MAX_LIFETIME": "30",
        "FAKE_QEMU_EVENTS": str(trace / "events"),
        "FAKE_PREPARATION_TRACE": str(trace),
    }
    for name, value in settings.items():
        monkeypatch.setenv(name, value)
    # Provision before thread/overlap timing. This is a fixture transport operation.
    with DedicatedSession(root=repo) as session:
        assert session.call("test_version", {})["native"]
    return repo, trace


def request(repo, work, *, arch="arm64", backend="native"):
    return {
        "arch": arch,
        "work_dir": str(work),
        "backend": backend,
        "env": dict(os.environ),
        "overrides": {
            "boot_marker": "READY",
            "boot_timeout": 5,
            "desktop_settle_seconds": 0,
            "min_boot_bytes": 0,
        },
    }
