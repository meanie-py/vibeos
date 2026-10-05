"""Bisect WHICH vm.py-only QEMU flag turns the idle vCPU from 3% -> ~92%.

Per-thread + query-cpus-fast proved the burn is the *vCPU* thread (vmexit
storm), not main-loop. So it's a guest-visible-hardware flag. On i386 the
scratch `-drive if=none` has no frontend (invisible to guest), leaving
`-debugcon` (port 0xE9 — the Rust kernel's trace markers => 1 vmexit each)
as prime suspect. Confirm by direct-spawn A/B (no QemuVM wrapper):

  V0 LEAN                 (start.sh-equivalent; expect ~3%)
  V1 LEAN + -debugcon
  V2 LEAN + scratch -drive (control; should stay ~3% on i386)

    python -m tests.qemu.experiments.flag_bisect
"""

from __future__ import annotations

import os
import subprocess
import time
from pathlib import Path

from tests.qemu.qmp_client import QmpClient

ISO = "/home/curi/Projects/os-playground/vibeos/build-i386/livecd.iso"
CLK = os.sysconf("SC_CLK_TCK")
W = Path("/tmp/flag_bisect")
W.mkdir(parents=True, exist_ok=True)


def _vcpu_cpu_pct(pid: int, vcpu_tid: int, window: float) -> float:
    def j(tid):
        p = Path(f"/proc/{pid}/task/{tid}/stat").read_text().rsplit(")", 1)[1].split()
        return int(p[11]) + int(p[12])

    a = j(vcpu_tid)
    time.sleep(window)
    b = j(vcpu_tid)
    return (b - a) / CLK / window * 100.0


def run_variant(name: str, extra: list[str]) -> str:
    ser = W / f"{name}.serial"
    qmp = W / f"{name}.qmp"
    for f in (ser, qmp):
        f.unlink(missing_ok=True)
    cmd = [
        "qemu-system-i386",
        "-M",
        "pc",
        "-accel",
        "kvm",
        "-cpu",
        "host",
        "-m",
        "512M",
        "-cdrom",
        ISO,
        "-display",
        "none",
        "-no-reboot",
        "-serial",
        f"file:{ser}",
        "-qmp",
        f"unix:{qmp},server=on,wait=off",
        *extra,
    ]
    proc = subprocess.Popen(
        cmd, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, preexec_fn=os.setpgrp
    )
    try:
        # Wait for shell ready (i386 boots ~50s here).
        deadline = time.monotonic() + 180
        while time.monotonic() < deadline:
            if ser.exists() and b"shell ready" in ser.read_bytes():
                break
            if proc.poll() is not None:
                return f"{name}: QEMU exited early rc={proc.returncode}"
            time.sleep(2)
        else:
            return f"{name}: TIMEOUT waiting for shell ready"

        time.sleep(20)  # settle
        q = QmpClient()
        q.connect(str(qmp), timeout=10)
        vcpu = q.execute("query-cpus-fast")["return"][0]["thread-id"]
        pct = _vcpu_cpu_pct(proc.pid, vcpu, 5.0)
        q.close()
        return f"{name:<26} vCPU(tid={vcpu}) = {pct:5.1f}%   [{' '.join(extra) or 'LEAN'}]"
    finally:
        proc.terminate()
        try:
            proc.wait(timeout=10)
        except subprocess.TimeoutExpired:
            proc.kill()


def main() -> None:
    scratch = W / "scratch.qcow2"
    subprocess.run(
        ["qemu-img", "create", "-f", "qcow2", str(scratch), "64M"], check=True, capture_output=True
    )
    variants = [
        ("V0_lean", []),
        ("V1_lean+debugcon", ["-debugcon", f"file:{W}/dbg.bin"]),
        ("V2_lean+scratch", ["-drive", f"file={scratch},format=qcow2,if=none,id=scratch"]),
    ]
    print("=== flag bisect: idle vCPU host-CPU per variant ===")
    print("(start.sh lean baseline ~3.4%; FAT vm.py ~92%)\n")
    for nm, ex in variants:
        print(run_variant(nm, ex), flush=True)


if __name__ == "__main__":
    main()
