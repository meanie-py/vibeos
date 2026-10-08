"""Diagnose: why does injected GUI input produce no reaction, and is the
guest idle or spinning? Also a reusable idle-vs-spin discriminator.

Boots amd64 ONCE, then:
  A) FRESH boot (no loadvm yet): measure idle CPU%, inject Ctrl+Esc,
     measure CPU% during/after, screenshot.   -> does input work at all?
  B) after savevm + loadvm: repeat the exact same injection.
     -> does loadvm break QEMU input routing? (the leading hypothesis:
        test_post_shell_ready_input only asserts survival, which is
        trivially true if post-restore input is silently dropped.)

CPU% tier: under -accel kvm, a guest HLT-idle blocks the vCPU thread
(qemu ~0% host CPU); a spinning guest pegs ~100% of a core. Summing
/proc/<pid>/task/*/stat utime+stime over a window distinguishes
idle / transient-work / wedged with no gdb.

    .venv/bin/python -m tests.qemu.experiments.input_liveness
"""

from __future__ import annotations

import json
import os
import time
from pathlib import Path

from tests.qemu.experiments.snapshot_bench import BenchConfig
from tests.qemu.vm import PROJECT_DIR, QemuVM

_CLK = os.sysconf("SC_CLK_TCK")
RESULTS_JSON = Path("/tmp/input_liveness.json")


def _proc_jiffies(pid: int) -> int:
    """Sum utime+stime across ALL threads of the qemu process (the TGID
    leader's stat excludes sibling threads, and the vCPU runs on one)."""
    total = 0
    taskdir = Path(f"/proc/{pid}/task")
    for t in taskdir.iterdir():
        try:
            parts = (t / "stat").read_text().rsplit(")", 1)[1].split()
            # after "comm)": state=0; utime=11, stime=12 (0-based here)
            total += int(parts[11]) + int(parts[12])
        except (FileNotFoundError, ProcessLookupError, IndexError):
            pass
    return total


def cpu_pct(pid: int, window: float = 2.0) -> float:
    """Percent of ONE core the qemu process used over `window` seconds.
    ~0 => guest HLT-idle (KVM); ~100 => one core pegged (spin/livelock)."""
    j0 = _proc_jiffies(pid)
    time.sleep(window)
    j1 = _proc_jiffies(pid)
    return round((j1 - j0) / _CLK / window * 100.0, 1)


def _shot(vm: QemuVM, tag: str) -> None:
    try:
        from PIL import Image

        ppm = vm.work_dir / f"_il_{tag}.ppm"
        vm.screendump(ppm)
        Image.open(ppm).save(PROJECT_DIR / "qemu" / f"amd64_il_{tag}.png", "PNG")
    except Exception:
        pass


def _key(vm: QemuVM, qcode: str, down: bool) -> None:
    vm.qmp.execute(
        "input-send-event",
        events=[{"type": "key", "data": {"down": down, "key": {"type": "qcode", "data": qcode}}}],
    )


def _ctrl_esc(vm: QemuVM) -> None:
    # Send the chord BOTH ways: 4 separate events (as before) AND once as a
    # single batched event (more faithful to how a real chord arrives).
    _key(vm, "ctrl", True)
    _key(vm, "esc", True)
    _key(vm, "esc", False)
    _key(vm, "ctrl", False)
    time.sleep(0.5)
    vm.qmp.execute(
        "input-send-event",
        events=[
            {"type": "key", "data": {"down": True, "key": {"type": "qcode", "data": "ctrl"}}},
            {"type": "key", "data": {"down": True, "key": {"type": "qcode", "data": "esc"}}},
            {"type": "key", "data": {"down": False, "key": {"type": "qcode", "data": "esc"}}},
            {"type": "key", "data": {"down": False, "key": {"type": "qcode", "data": "ctrl"}}},
        ],
    )


def _phase(vm: QemuVM, pid: int, label: str) -> dict:
    idle = cpu_pct(pid, 2.0)
    _shot(vm, f"{label}_pre")
    _ctrl_esc(vm)
    during = cpu_pct(pid, 2.0)  # immediately after injection
    time.sleep(2.0)
    after = cpu_pct(pid, 2.0)  # settled
    _shot(vm, f"{label}_post")
    # Did the screen change? (Start menu would be a big delta.)
    changed = "n/a"
    try:
        from PIL import Image, ImageChops

        a = Image.open(vm.work_dir / f"_il_{label}_pre.ppm").convert("RGB")
        b = Image.open(vm.work_dir / f"_il_{label}_post.ppm").convert("RGB")
        d = ImageChops.difference(a, b).convert("L")
        moved = sum(d.histogram()[17:])  # pixels with a delta > 16
        changed = round(moved / (a.width * a.height) * 100, 3)
    except Exception:
        pass
    r = {
        "idle_cpu_pct": idle,
        "cpu_pct_during_inject": during,
        "cpu_pct_after_settle": after,
        "screen_changed_pct": changed,
    }
    print(f"  [{label}] idle={idle}%  during={during}%  after={after}%  screenΔ={changed}%")
    return r


def main() -> None:
    cfg = BenchConfig()
    work = Path("/tmp/input_liveness_work")
    work.mkdir(parents=True, exist_ok=True)
    vm = QemuVM(cfg, work)

    print("=== boot amd64 (once) ===")
    t0 = time.monotonic()
    vm.start()
    vm.wait_for_boot()
    assert vm._proc is not None
    pid = vm._proc.pid
    res: dict = {"boot_seconds": round(time.monotonic() - t0, 1), "qemu_pid": pid}
    print(f"  booted in {res['boot_seconds']}s; qemu pid={pid}")

    try:
        print("\n=== A: FRESH boot, no loadvm yet ===")
        res["A_fresh"] = _phase(vm, pid, "A_fresh")

        print("\n=== savevm + loadvm, then same injection ===")
        vm.save_snapshot("ready")
        vm.restore_snapshot("ready")
        res["B_post_loadvm"] = _phase(vm, pid, "B_post_loadvm")

        a, b = res["A_fresh"], res["B_post_loadvm"]
        verdict = []
        if a["screen_changed_pct"] not in ("n/a",) and a["screen_changed_pct"] > 1:
            verdict.append("FRESH input WORKS (Start opened)")
        else:
            verdict.append("FRESH input produced no UI change")
        if (
            a["screen_changed_pct"] != "n/a"
            and b["screen_changed_pct"] != "n/a"
            and a["screen_changed_pct"] > 1 >= b["screen_changed_pct"]
        ):
            verdict.append("=> loadvm BREAKS input routing")
        # idle vs spin read
        floor = max(a["idle_cpu_pct"], b["idle_cpu_pct"])
        if floor > 60:
            verdict.append(
                f"guest NOT idle at rest ({floor}% — spin/storm, "
                f"consistent with known KiUnwaitThread storm)"
            )
        elif floor < 15:
            verdict.append(f"guest cleanly HLT-idle at rest ({floor}%)")
        res["verdict"] = verdict
    finally:
        vm.stop()

    RESULTS_JSON.write_text(json.dumps(res, indent=2))
    print("\n" + json.dumps(res, indent=2))


if __name__ == "__main__":
    main()
