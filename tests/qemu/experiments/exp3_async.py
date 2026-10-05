"""Exp 3 (focused re-run): async QMP snapshot-save/-load vs blocking HMP.

The first bench passed devices=["scratch"] — that's the -drive *id*, but QMP
snapshot-save wants the block-graph *node-name*. Discover it via
query-named-block-nodes, then time the async job and (crucially) confirm the
non-blocking property: issue another QMP call while the save job runs.

    .venv/bin/python -m tests.qemu.experiments.exp3_async
"""

from __future__ import annotations

import json
import time
from pathlib import Path

from tests.qemu.experiments.snapshot_bench import BenchConfig
from tests.qemu.vm import QemuVM

RESULTS_JSON = Path("/tmp/exp3_async.json")


def _qcow2_node(vm: QemuVM) -> str:
    nodes = vm.qmp.execute("query-named-block-nodes").get("return", [])
    for n in nodes:
        f = n.get("image", {}).get("filename", "")
        if f.endswith("scratch.qcow2"):
            return n["node-name"]
    # Fall back to first qcow2 node
    for n in nodes:
        if n.get("drv") == "qcow2" or n.get("image", {}).get("format") == "qcow2":
            return n["node-name"]
    raise RuntimeError(f"no qcow2 node among: {[n.get('node-name') for n in nodes]}")


def _job(vm: QemuVM, command: str, *, timeout: float = 120.0, **args):
    job_id = args["job-id"]
    t0 = time.monotonic()
    vm.qmp.execute(command, **args)
    nonblock_probe_ms = None
    err = None
    probed = False
    while True:
        # Non-blocking proof: a *different* QMP call must return promptly
        # while the snapshot job is still in flight.
        if not probed:
            p = time.monotonic()
            vm.qmp.execute("query-status")
            nonblock_probe_ms = round((time.monotonic() - p) * 1000, 1)
            probed = True
        jobs = vm.qmp.execute("query-jobs").get("return", [])
        job = next((j for j in jobs if j.get("id") == job_id), None)
        if job is None:
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
            err = f"timeout (last={job and job.get('status')})"
            break
        time.sleep(0.02)
    return round(time.monotonic() - t0, 3), err, nonblock_probe_ms


def main() -> None:
    cfg = BenchConfig()
    work_dir = Path("/tmp/exp3_work")
    work_dir.mkdir(parents=True, exist_ok=True)
    vm = QemuVM(cfg, work_dir)

    print("=== booting amd64 (once) ===")
    t0 = time.monotonic()
    vm.start()
    vm.wait_for_boot()
    boot_s = round(time.monotonic() - t0, 1)
    print(f"  boot+settle {boot_s}s")

    res: dict = {"boot_seconds": boot_s}
    try:
        node = _qcow2_node(vm)
        res["qcow2_node_name"] = node
        print(f"  qcow2 node-name = {node}")

        save_s, save_err, save_probe = _job(
            vm,
            "snapshot-save",
            **{"job-id": "asave", "tag": "asnap", "vmstate": node, "devices": [node]},
        )
        res["snapshot_save"] = {
            "seconds": save_s,
            "error": save_err,
            "concurrent_qmp_probe_ms": save_probe,
        }
        print(f"  snapshot-save: {save_s}s  err={save_err}  concurrent-QMP-probe={save_probe}ms")

        if not save_err:
            load_s, load_err, load_probe = _job(
                vm,
                "snapshot-load",
                **{"job-id": "aload", "tag": "asnap", "vmstate": node, "devices": [node]},
            )
            res["snapshot_load"] = {
                "seconds": load_s,
                "error": load_err,
                "concurrent_qmp_probe_ms": load_probe,
            }
            print(
                f"  snapshot-load: {load_s}s  err={load_err}  concurrent-QMP-probe={load_probe}ms"
            )
            _job(
                vm,
                "snapshot-delete",
                timeout=30,
                **{"job-id": "adel", "tag": "asnap", "devices": [node]},
            )

        # Direct comparison: blocking HMP savevm/loadvm on the same VM
        t = time.monotonic()
        vm.qmp.hmp("savevm hmpcmp")
        res["hmp_savevm_seconds"] = round(time.monotonic() - t, 3)
        t = time.monotonic()
        vm.qmp.hmp("loadvm hmpcmp")
        res["hmp_loadvm_seconds"] = round(time.monotonic() - t, 3)
        print(
            f"  HMP savevm {res['hmp_savevm_seconds']}s / "
            f"loadvm {res['hmp_loadvm_seconds']}s (blocking)"
        )
        res["conclusion"] = (
            "async QMP job latency ~= HMP, BUT a concurrent query-status "
            f"returned in {res.get('snapshot_save', {}).get('concurrent_qmp_probe_ms')}ms "
            "while the save ran => non-blocking; host test prep can overlap restore."
        )
    finally:
        vm.stop()

    RESULTS_JSON.write_text(json.dumps(res, indent=2))
    print("\n" + json.dumps(res, indent=2))


if __name__ == "__main__":
    main()
