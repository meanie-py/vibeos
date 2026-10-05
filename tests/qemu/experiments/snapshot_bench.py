"""Snapshot-harness feasibility bench.

Brainstorm question: once we reach steady state, can we snapshot QEMU and
restore-per-test to run more intensive tests faster?

This boots amd64 ONCE (the expensive step), saves a 'ready' snapshot, then
runs six experiments against that single boot — dogfooding the amortize-boot
thesis it is meant to evaluate.

Run from the worktree root:
    .venv/bin/python -m tests.qemu.experiments.snapshot_bench

Results: human summary to stdout + JSON to /tmp/snapshot_bench.json
"""

from __future__ import annotations

import json
import statistics
import time
from dataclasses import dataclass, field
from pathlib import Path

from tests.qemu.vm import Amd64Config, QemuVM

# Insulated iso copy so a concurrent `ninja livecd` can't corrupt a live boot.
STABLE_ISO = Path("/tmp/smokeharness-amd64.iso")
RESULTS_JSON = Path("/tmp/snapshot_bench.json")


@dataclass
class BenchConfig(Amd64Config):
    """amd64, but boot from the stable iso copy and settle briefly.

    The snapshot blob is full guest RAM (512M) regardless of settle time, so
    loadvm latency is RAM-bound, not settle-bound — 30s is enough to be past
    explorer init and representative of a real working set.
    """

    desktop_settle_seconds: float = 30.0
    machine_args: list[str] = field(
        default_factory=lambda: ["-M", "pc", "-accel", "kvm", "-cpu", "host"]
    )

    def qemu_args(self, work_dir: Path) -> list[str]:
        return [
            *self.machine_args,
            "-m",
            self.memory,
            "-vga",
            "cirrus",
            "-cdrom",
            str(STABLE_ISO),
            "-display",
            "none",
            "-no-reboot",
        ]


def _stat_mb(p: Path) -> float:
    return p.stat().st_size / (1024 * 1024) if p.exists() else 0.0


def _qmp_job(
    vm: QemuVM, command: str, *, timeout: float = 120.0, **args
) -> tuple[float, str | None]:
    """Run an async QMP snapshot job, poll query-jobs to conclusion.

    Returns (elapsed_seconds, error_or_None).
    """
    job_id = args["job-id"]
    t0 = time.monotonic()
    vm.qmp.execute(command, **args)
    err: str | None = None
    while True:
        jobs = vm.qmp.execute("query-jobs").get("return", [])
        job = next((j for j in jobs if j.get("id") == job_id), None)
        if job is None:
            # Concluded jobs disappear from the list after dismissal; if we
            # never saw it conclude treat absence (after >0.2s) as done.
            if time.monotonic() - t0 > 0.3:
                break
        elif job.get("status") == "concluded":
            err = job.get("error")
            try:
                vm.qmp.execute("job-dismiss", id=job_id)
            except Exception:
                pass
            break
        if time.monotonic() - t0 > timeout:
            err = f"timeout after {timeout}s (last status={job and job.get('status')})"
            break
        time.sleep(0.05)
    return time.monotonic() - t0, err


def main() -> None:
    results: dict = {"arch": "amd64", "guest_ram": "512M"}

    cfg = BenchConfig()
    work_dir = Path("/tmp/snapshot_bench_work")
    work_dir.mkdir(parents=True, exist_ok=True)

    vm = QemuVM(cfg, work_dir)

    print("=== booting amd64 (the expensive step, done once) ===")
    t0 = time.monotonic()
    vm.start()
    vm.wait_for_boot()
    boot_secs = time.monotonic() - t0
    results["boot_seconds"] = round(boot_secs, 1)
    print(f"  boot+settle: {boot_secs:.1f}s")

    try:
        # ---- Exp 1: loadvm latency ceiling (HMP savevm/loadvm) ----
        print("\n=== Exp 1: HMP loadvm latency (the speedup ceiling) ===")
        t = time.monotonic()
        vm.save_snapshot("ready")
        save_secs = time.monotonic() - t
        loadvm_times = []
        for i in range(15):
            t = time.monotonic()
            vm.restore_snapshot("ready")
            dt = time.monotonic() - t
            loadvm_times.append(dt)
            print(f"  loadvm[{i:2d}] {dt * 1000:7.1f} ms")
        results["exp1_hmp"] = {
            "savevm_seconds": round(save_secs, 3),
            "loadvm_ms_min": round(min(loadvm_times) * 1000, 1),
            "loadvm_ms_median": round(statistics.median(loadvm_times) * 1000, 1),
            "loadvm_ms_p95": round(sorted(loadvm_times)[int(len(loadvm_times) * 0.95)] * 1000, 1),
            "loadvm_ms_max": round(max(loadvm_times) * 1000, 1),
            "speedup_vs_boot": round(boot_secs / statistics.median(loadvm_times), 1),
        }
        print(
            f"  -> median loadvm {results['exp1_hmp']['loadvm_ms_median']} ms; "
            f"{results['exp1_hmp']['speedup_vs_boot']}x faster than reboot"
        )

        # ---- Exp 2: snapshot blob size vs guest RAM ----
        print("\n=== Exp 2: snapshot blob size vs guest RAM ===")
        scratch_mb = _stat_mb(vm.scratch_path)
        results["exp2_blob"] = {
            "scratch_qcow2_mb": round(scratch_mb, 1),
            "guest_ram_mb": 512,
            "nominal_virtual_disk_mb": 64,
            "note": "qcow2 internal snapshot stores RAM+device blob as overflow "
            "clusters; host file grows past the 64M nominal size — the "
            "real loadvm read floor.",
        }
        print(f"  scratch.qcow2 = {scratch_mb:.1f} MB (RAM=512MB, nominal vdisk=64MB)")

        # ---- Exp 3: async QMP snapshot-save/-load vs HMP ----
        print("\n=== Exp 3: async QMP snapshot jobs vs blocking HMP ===")
        exp3: dict = {}
        save_dt, save_err = _qmp_job(
            vm,
            "snapshot-save",
            **{"job-id": "asave", "tag": "asnap", "vmstate": "scratch", "devices": ["scratch"]},
        )
        exp3["snapshot_save_seconds"] = round(save_dt, 3)
        exp3["snapshot_save_error"] = save_err
        if save_err:
            print(f"  snapshot-save FAILED: {save_err}")
        else:
            print(f"  async snapshot-save: {save_dt * 1000:.1f} ms")
            load_dt, load_err = _qmp_job(
                vm,
                "snapshot-load",
                **{"job-id": "aload", "tag": "asnap", "vmstate": "scratch", "devices": ["scratch"]},
            )
            exp3["snapshot_load_seconds"] = round(load_dt, 3)
            exp3["snapshot_load_error"] = load_err
            print(
                f"  async snapshot-load: {load_dt * 1000:.1f} ms"
                + (f" (ERROR {load_err})" if load_err else "")
            )
            _qmp_job(
                vm,
                "snapshot-delete",
                timeout=30,
                **{"job-id": "adel", "tag": "asnap", "devices": ["scratch"]},
            )
        exp3["note"] = (
            "async jobs are non-blocking — host-side test prep can "
            "overlap the restore; HMP loadvm blocks the QMP socket."
        )
        results["exp3_async"] = exp3

        # ---- Exp 4: snapshot tree (multi-named fixture states) ----
        print("\n=== Exp 4: snapshot tree (>1 named fixture state) ===")
        vm.save_snapshot("ready")  # state A
        time.sleep(15)  # advance guest
        vm.save_snapshot("ready2")  # state B (deeper)
        snaps = vm.qmp.hmp("info snapshots")
        tree_ok = "ready" in snaps and "ready2" in snaps
        alt_times = []
        for _ in range(3):
            for tag in ("ready", "ready2"):
                t = time.monotonic()
                try:
                    vm.restore_snapshot(tag)
                    e = None
                except Exception as exc:
                    e = str(exc)
                alt_times.append((tag, round((time.monotonic() - t) * 1000, 1), e))
        results["exp4_tree"] = {
            "both_snapshots_present": tree_ok,
            "alternating_restores": alt_times,
            "note": "two named snapshots coexist on one scratch qcow2; tests can "
            "branch from boot OR a deeper post-setup state.",
        }
        print(
            f"  both snapshots present: {tree_ok}; "
            f"alternating restores all-clean: {all(e is None for _, _, e in alt_times)}"
        )

        # ---- Exp 5: restore torture loop (OS robustness under repeat loadvm) ----
        print("\n=== Exp 5: 50x restore torture loop ===")
        vm.restore_snapshot("ready")
        torture_err = None
        for i in range(50):
            try:
                vm.restore_snapshot("ready")
            except Exception as exc:
                torture_err = f"loadvm #{i} failed: {exc}"
                break
        status = vm.qmp.execute("query-status").get("return", {})
        shot = work_dir / "post_torture.ppm"
        shot_ok = False
        try:
            vm.screendump(shot)
            shot_ok = shot.exists() and shot.stat().st_size > 1000
        except Exception:
            pass
        results["exp5_torture"] = {
            "iterations": 50,
            "error": torture_err,
            "qmp_status_after": status.get("status"),
            "screendump_ok_after": shot_ok,
            "verdict": "PASS"
            if (torture_err is None and status.get("status") == "running" and shot_ok)
            else "FAIL",
        }
        print(
            f"  verdict: {results['exp5_torture']['verdict']} "
            f"(status={status.get('status')}, screendump_ok={shot_ok}, "
            f"err={torture_err})"
        )

        # ---- Exp 6: post-restore serial desync characterization ----
        print("\n=== Exp 6: post-restore serial desync (informs handshake) ===")
        byte_counts = []
        for _ in range(10):
            vm.restore_snapshot("ready")
            t = time.monotonic()
            junk = vm.serial.read_available(timeout=2.0)
            byte_counts.append(len(junk))
        results["exp6_serial"] = {
            "post_restore_serial_bytes": byte_counts,
            "min": min(byte_counts),
            "max": max(byte_counts),
            "mean": round(statistics.mean(byte_counts), 1),
            "stdev": round(statistics.pstdev(byte_counts), 1),
            "note": "current restore hack is read_available(0.2s); high/variable "
            "byte counts here => a guest->host nonce handshake is needed "
            "for reliable per-test resync at scale.",
        }
        print(
            f"  bytes/restore: min={min(byte_counts)} max={max(byte_counts)} "
            f"mean={results['exp6_serial']['mean']} "
            f"stdev={results['exp6_serial']['stdev']}"
        )

    finally:
        vm.stop()

    RESULTS_JSON.write_text(json.dumps(results, indent=2))
    print(f"\n=== JSON written to {RESULTS_JSON} ===")
    print(json.dumps(results, indent=2))


if __name__ == "__main__":
    main()
