"""Compatibility entry points for native replay fingerprints and divergence checks."""

from __future__ import annotations

import os
import threading
from pathlib import Path

from tools import qemu_infra_compat as native

SAMPLE_ENV_VAR = "VIBEOS_QEMU_RR_SAMPLE"
SAMPLES_FILE = "samples.jsonl"
VERDICT_FILE = "diverge.json"
RAM_STORE = "ram"
CHUNK = 1 << 20
EVENT_INTERRUPT = 1


def _call(op, **kwargs):
    return native.call("rrsample_" + op, kwargs)


def env_sample():
    return os.environ.get(SAMPLE_ENV_VAR, "").strip() or None


def _ram_bytes(argv):
    return _call("ram_bytes", argv=argv)


def ram_ranges(argv):
    return [tuple(v) for v in _call("ram_ranges", argv=argv)]


def compare(rec, rep, store=None):
    return _call("compare", rec=rec, rep=rep, store=store)


def artifact(d, io_at_pause, irq_at_pause, rec_regs, rep_regs):
    return _call(
        "artifact", diff=d, io=io_at_pause, irq=irq_at_pause, rec_regs=rec_regs, rep_regs=rep_regs
    )


def byte_diff(store, base, rec_hash, rep_hash, runs=24):
    return _call(
        "byte_diff", store=store, base=base, rec_hash=rec_hash, rep_hash=rep_hash, runs=runs
    )


def qmp_args(sock_path):
    return _call("qmp_args", sock_path=sock_path)


class _SamplerThread(threading.Thread):
    error: BaseException | None = None


def start(mode, value, trace: Path, argv, sock_path: Path, work: Path):
    def run():
        try:
            with native.DedicatedSession() as session:
                session.call(
                    "rrsample_run",
                    dict(mode=mode, value=value, trace=trace, argv=argv, sock=sock_path, work=work),
                )
        except BaseException as e:
            thread.error = e
            raise

    thread = _SamplerThread(target=run, name=f"rr-sample-{mode}", daemon=True)
    thread.start()
    return thread
