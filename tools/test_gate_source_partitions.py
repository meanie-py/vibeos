"""Early scanners must work without builds and ignore changing generated inputs."""

import shutil
import subprocess
import sys
import threading
from pathlib import Path

import pytest

from tests.qemu.cli.lints import EARLY_SOURCE_CHECKS

REPO = Path(__file__).resolve().parents[1]


@pytest.mark.parametrize("scanner", sorted(EARLY_SOURCE_CHECKS))
def test_early_scanner_without_builds_and_during_generated_header_writes(tmp_path, scanner):
    script = tmp_path / "tools" / f"{scanner}.py"
    script.parent.mkdir()
    shutil.copyfile(REPO / "tools" / script.name, script)
    for crate, source in (
        ("probe-rust", "pub type Width = u32;\npub fn probe() { let _value = 0444; }\n"),
        ("probe-sys", "pub type Width = u64;\n"),
    ):
        directory = tmp_path / "sdk/lib/rust" / crate / "src"
        directory.mkdir(parents=True)
        (directory / "lib.rs").write_text(source)
    if scanner == "scan_invented_constants":
        (tmp_path / "sdk/lib/rust/probe-rust/src/lib.rs").write_text(
            "pub const PROBE_FLAG: u32 = 8; // guessed value\n"
        )
        header = tmp_path / "sdk/include/probe.h"
        header.parent.mkdir(parents=True)
        header.write_text("#define PROBE_FLAG 1\n")

    # An audit hook observes real opens, even if a scanner catches the exception.
    # This subprocess uses the shipping scanner unchanged, with its normal CLI.
    harness = r"""
import os, runpy, sys
root, script, *flags = sys.argv[1:]
forbidden = (root + '/build-amd64/', root + '/sdk/lib/rust/target/')
reads = []
def audit(event, args):
    if event == 'open' and isinstance(args[0], (str, bytes)):
        path = os.path.abspath(os.fsdecode(args[0]))
        if path.startswith(forbidden):
            reads.append(path)
            raise RuntimeError('early scanner opened generated input: ' + path)
sys.addaudithook(audit)
sys.argv = [script, *flags]
code = 0
try:
    runpy.run_path(script, run_name='__main__')
except SystemExit as e:
    code = e.code
assert not reads, reads
sys.exit(code)
"""
    flags = (
        [] if scanner in ("scan_octal_literals", "scan_type_alias_contradictions") else ["--lint"]
    )

    def run():
        return subprocess.run(
            [sys.executable, "-c", harness, str(tmp_path), str(script), *flags],
            cwd=tmp_path,
            capture_output=True,
            text=True,
            timeout=30,
        )

    fresh = run()
    assert fresh.returncode in (0, 1), fresh.stderr
    assert "Traceback" not in fresh.stderr
    if scanner in ("scan_octal_literals", "scan_type_alias_contradictions"):
        assert fresh.stdout.strip(), "positive fixture must still be analyzed"
    if scanner == "scan_invented_constants":
        assert "MISMATCH" in fresh.stdout and "PROBE_FLAG" in fresh.stdout
    generated = tmp_path / "build-amd64/sdk/include/ddk/wdm.h"
    target = tmp_path / "sdk/lib/rust/target/debug/build/probe/out/generated.rs"
    generated.parent.mkdir(parents=True)
    target.parent.mkdir(parents=True)
    if scanner == "scan_invented_constants":
        # Same module name under a configured build must not join the C oracle.
        foreign_oracle = tmp_path / "build-amd64/drivers/probe/generated.h"
        foreign_oracle.parent.mkdir(parents=True)
        foreign_oracle.write_text("#define PROBE_FLAG 8\n")
    stopped = threading.Event()
    started = threading.Event()

    def churn():
        counter = 0
        while not stopped.is_set():
            generated.write_text(f"#define VALUE {counter}\n")
            target.write_text(
                f"pub type Width = u8;\nfn generated() {{ let _ = 0777; }} // {counter}\n"
            )
            counter += 1
            started.set()
            stopped.wait(0.002)

    writer = threading.Thread(target=churn)
    writer.start()
    try:
        assert started.wait(5)
        concurrent = run()
    finally:
        stopped.set()
        writer.join(timeout=5)
    assert not writer.is_alive()
    assert (concurrent.returncode, concurrent.stdout, concurrent.stderr) == (
        fresh.returncode,
        fresh.stdout,
        fresh.stderr,
    )
