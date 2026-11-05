"""Integration contracts that native-only controller fixtures cannot exercise."""

from __future__ import annotations

import json
import os
import subprocess
from pathlib import Path
from types import SimpleNamespace
from typing import Any

import pytest

from tests.qemu import test_stortest
from tests.qemu.vm import QemuVM, make_usb_stick_image
from tools.qemu_infra_compat import DedicatedSession, call
from tools.qemu_infra_runner import Bindings


def _git(repo, *args):
    subprocess.run(["git", *args], cwd=repo, check=True, capture_output=True)


@pytest.mark.parametrize("arch", ["i386", "amd64", "arm64"])
def test_native_conformance_round_crosses_real_ledger_callback(tmp_path, monkeypatch, arch):
    repo = tmp_path / "lane"
    build = repo / f"build-{arch}"
    build.mkdir(parents=True)
    (repo / ".build-tools/llvm-mingw").mkdir(parents=True)
    payload = repo / "tools/payloads/kmtest"
    payload.mkdir(parents=True)
    (payload / f"kmtest-sweep-{arch}.exe").write_bytes(b"fixture payload")
    (repo / "qemu").mkdir()
    for name in ["QEMU_EFI.fd", "efivars.fd"]:
        (repo / "qemu" / name).write_bytes(b"fixture firmware")
    (repo / "source.c").write_text("int fixture;\n")
    (repo / ".gitignore").write_text("build-*/\n.build-tools/\n")
    (build / "livecd.iso").write_bytes(b"fixture media")
    (build / "payload-selection.json").write_text('{"payloads":["kmtest"]}')
    _git(repo, "init", "-q")
    _git(repo, "config", "user.name", "fixture")
    _git(repo, "config", "user.email", "fixture@example.invalid")
    _git(repo, "add", "-A")
    _git(repo, "commit", "-qm", "fixture")
    fleet = tmp_path / "fleet"
    fleet.mkdir()
    monkeypatch.setenv("VIBEOS_ROOT", str(fleet))
    environment = {**os.environ, "VIBEOS_ARTIFACT_STORE_ROOT": str(tmp_path / "store")}
    config = {
        "boot_media_rel": f"build-{arch}/livecd.iso",
        "staged_bins_rel": [],
        "needs_mkramdisk": False,
    }
    stamp = call(
        "provenance_make_stamp",
        {"repo": repo, "arch": arch, "cfg": config, "cold": True, "ts": 1, "env": environment},
        repo,
    )
    call("provenance_stamp_write", {"build_dir": build, "stamp": stamp}, repo)
    out = tmp_path / "round"
    out.mkdir()
    bindings = Bindings(repo, None)
    callbacks = bindings.callbacks
    callbacks["create_run_dir"] = lambda _a: out

    def pytest_authority(arguments):
        argv = arguments["argv"]
        assert "pytest" in argv and "--skip-clean-build" in argv
        guest_arch = argv[argv.index("--arch") + 1]
        artifact_dir = Path(arguments["env"]["VIBEOS_ARTIFACT_DIR"])
        (artifact_dir / f"{guest_arch}_serial.log").write_text(
            "[KMTEST-SWEEP] launch A\n[KMTEST:END A pass=10 fail=0 skip=0]\n"
            "[KMTEST-SWEEP] exit A code=0\n[KMTEST-SWEEP] done total=1\n"
        )
        return {"returncode": 0, "timed_out": False}

    # Only the external pytest command is a fixture. The shipping Rust controller,
    # callback routing, authoritative repo selection and durable ledger writer run.
    callbacks["command"] = pytest_authority
    options = {
        "args": {
            "arch": arch,
            "no_build": True,
            "repeat": 1,
            "timeout": 1,
            "write_baseline": False,
            "smp": 0,
            "gate": False,
        },
        "env": environment,
        "cfgs": {arch: config},
        "farm_dir": str(tmp_path / "farm"),
    }
    with DedicatedSession(root=repo) as session:
        result = session.call("conformance_run", options, callbacks)
    assert result["code"] == 0
    rows = [json.loads(line) for line in (fleet / "vibeos-ledger.jsonl").read_text().splitlines()]
    assert len(rows) == 1 and rows[0]["cmd"] == "conformance"
    assert rows[0]["arch"] == arch and rows[0]["worktree"] == repo.name
    assert rows[0]["launched"] == 1 and rows[0]["ok"] == 1


@pytest.mark.parametrize("arch", ["i386", "amd64", "arm64"])
@pytest.mark.parametrize("backend", ["windows-host", "native"])
def test_stortest_grades_the_image_from_the_native_staging_handle(tmp_path, arch, backend):
    work = tmp_path / "linux-originals"
    staging = tmp_path / "windows-copies"
    work.mkdir()
    staging.mkdir()
    files = {"WRITETST.TXT": test_stortest._WRITE_CONTENT}
    # The Linux original deliberately has no guest write on Windows-host.
    make_usb_stick_image(work / "usbdisk.qcow2", files=files if backend == "native" else {})
    if backend == "windows-host":
        make_usb_stick_image(staging / "usbdisk.qcow2", files=files)
    vm: Any = object.__new__(QemuVM)
    vm.config = SimpleNamespace(arch=arch)
    vm.work_dir = work
    vm._backend = backend
    vm._boot_output = "[STORTEST] write+readback OK (43 bytes)\n[STORTEST] ALL OK drive E:\n"
    vm.serial_output = lambda timeout=1: ""
    # This is the same handle DTO consumed after native vm_start.
    vm._sync(
        {
            "live_log": str(work / "serial.log"),
            "staging": str(staging) if backend == "windows-host" else None,
        }
    )
    classification, _, _ = test_stortest._grade(vm)
    assert classification == ""
    assert vm.scratch_path.parent == (staging if backend == "windows-host" else work)
