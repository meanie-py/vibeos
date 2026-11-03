"""Real isolated compilation and concurrent reuse across independent source roots."""

import concurrent.futures
import json
import os
import shutil
import subprocess
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
    from tools.test_artifact_inputs import install_native_artifact_tool

    install_native_artifact_tool(root)
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
                str(ROOT / "tools/build-infra"),
                "artifact-runtime",
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


def test_real_policy_rollout_source_dependency_embedded_and_cross_root_reuse(tmp_path):
    """Real compiler/cache ladder; synthetic sources keep paid translations untouched."""
    import time

    from tests.qemu.cli import hostjobs
    from tools import artifact_executor as executor
    from tools import artifact_runtime as runtime

    lanes = [tmp_path / "lane-a", tmp_path / "lane-b"]
    for lane in lanes:
        source_tree(lane)
        workspace = lane / "sdk/lib/rust"
        (workspace / "Cargo.toml").write_text(
            '[workspace]\nmembers=["beep-rust","shared"]\nresolver="2"\n'
            '[profile.release]\npanic="abort"\n'
        )
        (workspace / "Cargo.lock").write_text(
            'version=4\n[[package]]\nname="beep-rust"\nversion="0.1.0"\n'
            'dependencies=["shared"]\n[[package]]\nname="shared"\nversion="0.1.0"\n'
        )
        body = workspace / "beep-rust"
        with (body / "Cargo.toml").open("a") as stream:
            stream.write('\n[dependencies]\nshared={path="../shared"}\n')
        (workspace / "shared/src").mkdir(parents=True)
        (workspace / "shared/Cargo.toml").write_text(
            '[package]\nname="shared"\nversion="0.1.0"\nedition="2021"\n'
        )
        (workspace / "shared/src/lib.rs").write_text("#![no_std]\npub const VALUE:u32=7;\n")
        (body / "src/lib.rs").write_text(
            '#![no_std]\n#[no_mangle] pub extern "C" fn fixture_value()->u32 '
            '{shared::VALUE + include_bytes!("payload.bin")[0] as u32}\n'
            "#[panic_handler] fn panic(_: &core::panic::PanicInfo)->! {loop {}}\n"
        )
        (body / "src/payload.bin").write_bytes(b"\x01")
    store = tmp_path / "store"
    observations = []
    for arch in ("i386", "amd64", "arm64"):
        contexts = []
        started = time.monotonic()
        for index, lane in enumerate(lanes):
            context = tmp_path / f"context-{arch}-{index}"
            contexts.append((lane, context))
        with (
            runtime.supervise(
                lanes[0], store, Path.home() / ".rustup/toolchains" / PIN, contexts[0][1], arch=arch
            ) as first,
            runtime.supervise(
                lanes[1], store, Path.home() / ".rustup/toolchains" / PIN, contexts[1][1], arch=arch
            ) as second,
        ):
            flags = executor.FLAGS[(arch, False)]

            def consume(supervisor, lane, step, *, arch=arch, flags=flags):
                command = [
                    str(ROOT / "tools/build-infra"),
                    "artifact-runtime",
                    "request",
                    "--supervisor",
                    str(supervisor),
                    "--body",
                    "beep-rust",
                    "--consumer",
                    f"rollout:{arch}:{step}",
                    "--arch",
                    arch,
                    "--target",
                    executor.TARGETS[arch],
                    "--flags=" + flags,
                ]
                env = hostjobs.environment(lane, dict(os.environ), canonical=False)
                child = subprocess.run(
                    command, env=env, cwd=ROOT, capture_output=True, text=True, timeout=120
                )
                assert child.returncode == 0, child.stderr
                archive = next(
                    line.split("\t", 1)[1]
                    for line in child.stdout.splitlines()
                    if line.startswith("archive\t")
                )
                assert Path(archive).read_bytes().startswith(b"!<arch>\n")
                return archive

            original = consume(first, lanes[0], "initial")
            assert consume(second, lanes[1], "cross-root") == original
            assert consume(first, lanes[0], "warm") == original
            body = lanes[0] / "sdk/lib/rust/beep-rust/src/lib.rs"
            initial_body = body.read_text()
            body.write_text(initial_body.replace("shared::VALUE +", "1 + shared::VALUE +"))
            edited = consume(first, lanes[0], "source-edit")
            assert edited != original
            dependency = lanes[0] / "sdk/lib/rust/shared/src/lib.rs"
            dependency.write_text("#![no_std]\npub const VALUE:u32=8;\n")
            dependent = consume(first, lanes[0], "dependency-edit")
            assert dependent not in (original, edited)
            payload = lanes[0] / "sdk/lib/rust/beep-rust/src/payload.bin"
            payload.write_bytes(b"\x02")
            embedded = consume(first, lanes[0], "embedded-edit")
            assert embedded not in (original, edited, dependent)
            assert consume(second, lanes[1], "unchanged-other-lane") == original
        receipts = [
            json.loads(p.read_text())
            for _lane, context in contexts
            for p in context.glob("*/activity/*.json")
        ]
        assert len(receipts) == 7
        assert sum(r["builder_calls"] for r in receipts) == 4
        assert sum(bool(r["hit"]) for r in receipts) == 3
        observations.append(
            {"arch": arch, "seconds": time.monotonic() - started, "builder_calls": 4, "hits": 3}
        )
        # Restore only the fresh fixture for the next architecture's identical ladder.
        body.write_text(initial_body)
        dependency.write_text("#![no_std]\npub const VALUE:u32=7;\n")
        payload.write_bytes(b"\x01")
    print("real policy rollout: " + json.dumps(observations, sort_keys=True))
