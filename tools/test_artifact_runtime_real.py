"""Real isolated compilation and concurrent reuse across independent source roots."""

import concurrent.futures
import json
import os
import shutil
import subprocess
import sys
from pathlib import Path

from tools.artifact_executor import PIN
from tools.artifact_runtime import supervise
from tools.artifact_store import Store

ROOT = Path(__file__).resolve().parents[1]


def source_tree(root):
    body = root / "sdk/lib/rust/beep-rust"
    (body / "src").mkdir(parents=True)
    (body.parent / "Cargo.toml").write_text(
        '[workspace]\nmembers=["beep-rust"]\nresolver="2"\n[profile.release]\npanic="abort"\n'
    )
    (body.parent / "Cargo.lock").write_text(
        'version=4\n[[package]]\nname="beep-rust"\nversion="0.1.0"\n'
    )
    (body / "Cargo.toml").write_text(
        '[package]\nname="beep-rust"\nversion="0.1.0"\nedition="2021"\n'
        '[lib]\ncrate-type=["staticlib"]\n'
    )
    (body / "src/lib.rs").write_text(
        '#![no_std]\n#[no_mangle] pub extern "C" fn fixture_value()->u32 {7}\n'
        "#[panic_handler] fn panic(_: &core::panic::PanicInfo)->! {loop {}}\n"
    )
    shutil.copyfile(ROOT / "rust-toolchain.toml", root / "rust-toolchain.toml")
    for name in (
        "artifact_config",
        "artifact_runtime",
        "artifact_inputs",
        "artifact_executor",
        "artifact_worker",
        "artifact_store",
    ):
        destination = root / "tools" / (name + ".py")
        destination.parent.mkdir(exist_ok=True)
        shutil.copyfile(ROOT / "tools" / destination.name, destination)
    broker = root / ".cache/host-jobs/host-jobs"
    broker.parent.mkdir(parents=True)
    shutil.copy2(ROOT / ".cache/host-jobs/host-jobs", broker)


def test_four_real_consumers_two_source_roots_compile_once_and_keep_direct_archive(tmp_path):
    first, second = tmp_path / "lane-a", tmp_path / "lane-b"
    source_tree(first)
    source_tree(second)
    store_root = tmp_path / "store"
    toolchain = Path.home() / ".rustup/toolchains" / PIN
    contexts = [tmp_path / "context-a", tmp_path / "context-b"]
    with (
        supervise(first, store_root, toolchain, contexts[0]) as a,
        supervise(second, store_root, toolchain, contexts[1]) as b,
    ):

        def consume(index):
            command = [
                sys.executable,
                "-m",
                "tools.artifact_runtime",
                "request",
                "--supervisor",
                str(a if index % 2 == 0 else b),
                "--body",
                "beep-rust",
                "--consumer",
                f"fixture:{index}",
                "--flags=",
            ]
            result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True, timeout=120)
            assert result.returncode == 0, result.stderr
            return next(
                line.split("\t", 1)[1]
                for line in result.stdout.splitlines()
                if line.startswith("archive\t")
            )

        with concurrent.futures.ThreadPoolExecutor(max_workers=4) as workers:
            archives = list(workers.map(consume, range(4)))
        assert len(set(archives)) == 1
        archive = Path(archives[0])
        assert archive.read_bytes().startswith(b"!<arch>\n")
        assert archive.stat().st_nlink == 1
        receipts = [
            json.loads(p.read_text())
            for directory in contexts
            for p in directory.glob("*/activity/*.json")
        ]
        assert len(receipts) == 4
        assert sum(r["builder_calls"] for r in receipts) == 1
        assert sum(r["hit"] for r in receipts) == 3
        assert len({r["action_key"] for r in receipts}) == 1
        assert not list((store_root / "v1/staging").iterdir())
        assert not (first / "sdk/lib/rust/target").exists()
        assert not (second / "sdk/lib/rust/target").exists()
        gc = Store(store_root).gc(apply=True, grace_seconds=0)
        assert not gc["actions"] and not gc["blobs"]
    # A fresh outer invocation may still need the archive after both supervisors
    # have exited. Durable references keep it available through a full GC.
    Store(store_root).gc(apply=True, grace_seconds=0)
    assert archive.is_file()
    assert os.stat(archive).st_mode & 0o222 == 0
