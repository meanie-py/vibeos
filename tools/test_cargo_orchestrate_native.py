"""Native orchestrator executes declared producers and refuses failed output."""

import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]
TOOL = REPO / "tools/build-infra"


def test_manifest_execution_and_failed_producer():
    # Provision the launcher before the fixture prepends its fake Cargo to PATH.
    subprocess.run([str(TOOL), "--help"], capture_output=True, check=True)
    with tempfile.TemporaryDirectory() as td:
        root = Path(td)
        (root / "sdk/lib/rust/cells").mkdir(parents=True)
        (root / "sdk/tools").mkdir(parents=True)
        (root / "dll/win32/dbghelp").mkdir(parents=True)
        (root / "sdk/include/host").mkdir(parents=True)
        (root / "sdk/tools/pefixup.c").write_text(
            """#include <stdio.h>
int main(int n, char **v) {
    if(n != 3) return 3;
    FILE *f=fopen(v[2], "a");
    if(!f) return 2;
    fputs("-fixed", f);
    return fclose(f);
}
"""
        )
        (root / "dll/win32/dbghelp/compat.h").write_text("")
        for prefix in ("sdk/include/reactos", "host-tools/bin/sdk/include/reactos"):
            path = root / "build-i386" / prefix
            path.mkdir(parents=True)
            for header in ("version.h", "buildno.h"):
                (path / header).write_text("fixture")
        cells = []
        for name in ("one", "two"):
            cells.append(
                {
                    "bins": [name],
                    "build_std": False,
                    "crate": name + "-cargo",
                    "env": {},
                    "outputs": [
                        {
                            "bin": name,
                            "out": "drivers/" + name + ".sys",
                            "post": ["pefixup --kernel"],
                        }
                    ],
                    "target": "i686-pc-windows-gnu",
                    "target_dir": "cell-" + name,
                }
            )
        (root / "sdk/lib/rust/cells/i386.jsonl").write_text(
            "".join(json.dumps(c) + "\n" for c in cells)
        )
        fakebin = root / "bin"
        fakebin.mkdir()
        cargo = fakebin / "cargo"
        cargo.write_text("""#!/bin/sh
[ -z "$FAIL_PRODUCER" ] || exit 7
while [ "$#" -gt 0 ]; do
  case "$1" in --target) target="$2"; shift;; --bin) bin="$2"; shift;; esac
  shift
done
mkdir -p "$CONTROL_BARRIER" "$CARGO_TARGET_DIR/$target/release"
touch "$CONTROL_BARRIER/$bin.started"
i=0
while [ ! -f "$CONTROL_BARRIER/one.started" ] || [ ! -f "$CONTROL_BARRIER/two.started" ]; do
  i=$((i+1))
  [ "$i" -lt 500 ] || exit 8
  sleep 0.01
done
printf 'built' > "$CARGO_TARGET_DIR/$target/release/$bin.exe"
""")
        cargo.chmod(0o755)
        env = {
            **os.environ,
            "PATH": str(fakebin) + os.pathsep + os.environ["PATH"],
            "VIBEOS_CANON": str(root),
            "CONTROL_BARRIER": str(root / "barrier"),
        }
        out = root / "out"
        cmd = [
            str(TOOL),
            "--root",
            str(root),
            "cargo-orchestrate",
            "--arch",
            "i386",
            "--manifest",
            "--no-artifact-store",
            "--out-dir",
            str(out),
            "--jobs",
            "2",
            "one-cargo",
            "two-cargo",
        ]
        result = subprocess.run(cmd, env=env, text=True, capture_output=True, check=False)
        assert result.returncode == 0, (result.stdout, result.stderr)
        for name in ("one", "two"):
            assert (out / "drivers" / f"{name}.sys").read_text() == "built-fixed"
        assert not (out / ".cells.json").exists()
        result = subprocess.run(
            cmd, env={**env, "FAIL_PRODUCER": "1"}, text=True, capture_output=True, check=False
        )
        assert result.returncode != 0
        assert not (out / ".cells.json").exists()


def test_worker_stops_child_after_owner_exit():
    import time

    with tempfile.TemporaryDirectory() as td:
        root = Path(td)
        child_pid_path = root / "child.pid"
        helper_pid_path = root / "worker.pid"
        launcher = """import subprocess, sys
p = subprocess.Popen([
    sys.argv[1], "--root", sys.argv[2], "cargo-orchestrate-worker", "/bin/sh", "-c",
    'echo $$ > "$1.tmp" && mv "$1.tmp" "$1"; exec sleep 30', "_", sys.argv[3],
])
open(sys.argv[4], "w").write(str(p.pid))
p.wait()
"""
        owner = subprocess.Popen(
            [
                sys.executable,
                "-c",
                launcher,
                str(TOOL),
                str(root),
                str(child_pid_path),
                str(helper_pid_path),
            ]
        )
        try:
            limit = time.monotonic() + 10
            # The child publishes with mv: `echo > file` makes it exist empty first, and the
            # int() below raced it (the same flake as test_buildbench's, fixed in da65575237e).
            while time.monotonic() < limit and not child_pid_path.exists():
                assert owner.poll() is None
                time.sleep(0.02)
            assert child_pid_path.exists(), "worker failed to start child"
            child_pid = int(child_pid_path.read_text())
            owner.terminate()
            owner.wait(timeout=5)
            while time.monotonic() < limit:
                try:
                    state = Path(f"/proc/{child_pid}/stat").read_text().rsplit(")", 1)[1].split()[0]
                except FileNotFoundError:
                    break
                if state == "Z":
                    break
                time.sleep(0.02)
            else:
                raise AssertionError("worker left its child running after the owner exited")
        finally:
            if owner.poll() is None:
                owner.terminate()
                owner.wait(timeout=5)


def test_worker_streams_output_before_the_producer_exits(tmp_path):
    import select

    subprocess.run([str(TOOL), "--help"], check=True, capture_output=True)
    process = subprocess.Popen(
        [str(TOOL), "cargo-orchestrate-worker", "/bin/sh", "-c", "printf 'live-log\\n'; sleep 30"],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )
    try:
        assert process.stdout is not None
        assert select.select([process.stdout], [], [], 5)[0], "producer log was buffered"
        assert process.stdout.readline() == "live-log\n"
        assert process.poll() is None, "control producer must still be running"
    finally:
        process.terminate()
        process.communicate(timeout=10)
