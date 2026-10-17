"""Early scanners must work without builds and ignore changing generated inputs."""

import ctypes
import os
import shutil
import struct
import subprocess
import sys
import threading
from pathlib import Path

import pytest

from tests.qemu.cli import rustlints
from tests.qemu.cli.lints import EARLY_SOURCE_CHECKS, source_partition_specs

REPO = Path(__file__).resolve().parents[1]


# The shared fixture plants an octal literal and a Width u32/u64 disagreement, so
# only these two have something to find in it. The rest are proven by the audit
# hook and the concurrent-churn run, not by their output.
_POSITIVE_FIXTURE = ("scan_octal_literals", "scan_type_alias_contradictions")


def _registered_flags(scanner: str) -> list[str]:
    for name, cmd, _src in source_partition_specs(REPO, "source-early"):
        if name == scanner:
            return [arg for arg in cmd[2:]]
    raise AssertionError(f"{scanner} is in EARLY_SOURCE_CHECKS but has no spec")


def _run_native_audited(root: Path, command: list[str]) -> subprocess.CompletedProcess:
    """Observe native opens through the kernel, independently of the lint binary."""
    libc = ctypes.CDLL(None, use_errno=True)
    fd = libc.inotify_init1(os.O_NONBLOCK | os.O_CLOEXEC)
    if fd < 0:
        raise OSError(ctypes.get_errno(), "inotify_init1")
    watches = {}
    try:
        paths = [(root / "build-amd64", "forbidden"), (root / "sdk/lib/rust/target", "forbidden")]
        paths.extend((path, "source") for path in (root / "sdk/lib/rust").glob("*/src"))
        for path, kind in paths:
            if path.is_dir():
                watch = libc.inotify_add_watch(fd, os.fsencode(path), 0x20)  # IN_OPEN
                if watch < 0:
                    raise OSError(ctypes.get_errno(), f"inotify_add_watch {path}")
                watches[watch] = (kind, path)
        result = subprocess.run(command, cwd=root, capture_output=True, text=True, timeout=30)
        source_reads = []
        forbidden_reads = []
        while True:
            try:
                events = os.read(fd, 65536)
            except BlockingIOError:
                break
            offset = 0
            while offset < len(events):
                watch, mask, _cookie, size = struct.unpack_from("iIII", events, offset)
                name = os.fsdecode(events[offset + 16 : offset + 16 + size].split(b"\0", 1)[0])
                offset += 16 + size
                if mask & 0x4000:  # IN_Q_OVERFLOW must never hide an open.
                    raise AssertionError("native input audit overflowed")
                kind, path = watches[watch]
                if mask & 0x20:
                    if kind == "forbidden":
                        forbidden_reads.append(str(path / name))
                    elif name.endswith(".rs"):
                        source_reads.append(str(path / name))
        assert not forbidden_reads, forbidden_reads
        assert source_reads, "scanner did not read the positive source corpus"
        return result
    finally:
        os.close(fd)


def test_native_input_audit_detects_a_forbidden_directory_open(tmp_path):
    source = tmp_path / "sdk/lib/rust/probe/src"
    source.mkdir(parents=True)
    (source / "lib.rs").write_text("fn source() {}")
    (tmp_path / "build-amd64").mkdir()
    script = (
        "from pathlib import Path; "
        "Path('sdk/lib/rust/probe/src/lib.rs').read_text(); "
        "list(Path('build-amd64').iterdir())"
    )
    with pytest.raises(AssertionError, match="build-amd64"):
        _run_native_audited(tmp_path, [sys.executable, "-c", script])


@pytest.mark.parametrize("scanner", sorted(EARLY_SOURCE_CHECKS))
def test_early_scanner_without_builds_and_during_generated_header_writes(tmp_path, scanner):
    script = tmp_path / "tools" / f"{scanner}.py"
    script.parent.mkdir()
    if scanner not in rustlints.PORTED:
        shutil.copyfile(REPO / "tools" / script.name, script)
    shutil.copyfile(REPO / "tools/source_walk.py", script.parent / "source_walk.py")
    shutil.copyfile(REPO / "tools/lint_facts.py", script.parent / "lint_facts.py")
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

    subprocess.run(["git", "init", "-q", str(tmp_path)], check=True)

    # An audit hook observes real opens, even if a scanner catches the exception.
    # This subprocess uses the shipping scanner unchanged, with its normal CLI.
    harness = r"""
import os, runpy, sys
root, script, *flags = sys.argv[1:]
forbidden = (root + '/build-amd64/', root + '/sdk/lib/rust/target/')
reads = []
source_reads = []
def audit(event, args):
    if event in ('open', 'os.scandir') and isinstance(args[0], (str, bytes)):
        path = os.path.abspath(os.fsdecode(args[0]))
        if path.startswith(forbidden) or path in [p.rstrip('/') for p in forbidden]:
            reads.append(path)
            raise RuntimeError('early scanner entered/opened generated output: ' + path)
        if event == 'open' and path.endswith('.rs') and path.startswith(root + '/sdk/'):
            source_reads.append(path)
sys.addaudithook(audit)
sys.argv = [script, *flags]
code = 0
try:
    runpy.run_path(script, run_name='__main__')
except SystemExit as e:
    code = e.code
assert not reads, reads
assert source_reads, 'scanner did not read the positive source corpus'
sys.exit(code)
"""
    # Take the flags from the registration, not a list kept here: the gate owns how
    # each scanner is invoked, and a second copy drifts silently. It already had --
    # this exempted scan_type_alias_contradictions, which the gate DOES pass --lint,
    # and passed --lint to scan_octal_literals, which it does not -- so the contract
    # under test was an invocation that never runs.
    flags = _registered_flags(scanner)

    def run():
        if scanner in rustlints.PORTED:
            return _run_native_audited(
                tmp_path, [str(rustlints.ensure(REPO)), "--root", str(tmp_path), "--rules", scanner]
            )
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
    if scanner in _POSITIVE_FIXTURE:
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
            # Exercise directory removal too: mutating file contents alone
            # cannot reproduce pathlib's scandir/entry race.
            churn_dir = target.parent / "vanishing"
            churn_dir.mkdir()
            (churn_dir / "generated.rs").write_text("fn generated() {}")
            shutil.rmtree(churn_dir)
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
