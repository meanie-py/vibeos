"""Compatibility DTOs and QMP bindings for the native PC sampler."""

from __future__ import annotations

import os
import threading
from pathlib import Path

from tools import qemu_infra_compat as native

SAMPLE_ENV_VAR = "VIBEOS_QEMU_PC_SAMPLE"
SAMPLES_FILE = "pcsamples.jsonl"
CALLER_DEPTH = 3
STACK_WORDS = 48
KERNEL_HALF = 0xFFFF800000000000


def _call(op, *, repo=None, callbacks=None, **kwargs):
    try:
        if callbacks is not None:
            with native.DedicatedSession(root=repo or native.REPO) as session:
                return session.call("pcsample_" + op, kwargs, callbacks=callbacks)
        return native.call("pcsample_" + op, kwargs, root=repo or native.REPO)
    except native.NativeError as e:
        if e.kind == "FileNotFoundError":
            raise FileNotFoundError(str(e)) from e
        raise ValueError(str(e)) from e


def _bindings(qmp):
    lock = getattr(qmp, "lock", None) or threading.RLock()
    return {
        "diagnostic_hmp": lambda a: qmp.hmp(a["command"]),
        "diagnostic_begin_sample": lambda _: lock.acquire() and None,
        "diagnostic_end_sample": lambda _: lock.release(),
    }


def env_period():
    return _call("env_period", raw=os.environ.get(SAMPLE_ENV_VAR, ""))


def parse_sample(state):
    return _call("parse_sample", state=state)


def _callers(qmp, cpu, fp_hex, depth):
    return _call("callers", cpu=cpu, fp=fp_hex, depth=depth, callbacks=_bindings(qmp))


def _stack_words(qmp, cpu, sp_hex):
    return _call("stack_words", cpu=cpu, sp=sp_hex, callbacks=_bindings(qmp))


def sample_once(qmp):
    return _call("sample_once", callbacks=_bindings(qmp))


def _resolve_batch(repo, arch, pcs, *, mp, reloc):
    return _call("resolve_batch", repo=repo, arch=arch, pcs=pcs, mp=mp, reloc=reloc)


def _where(pc, sym):
    return _call("where", pc=pc, sym=sym)


def _likely_caller(row, sym):
    return _call("likely_caller", row=row, sym=sym)


def report(repo, arch, samples, serial, top=15):
    return _call("report", repo=repo, arch=arch, samples=samples, serial=serial, top=top)


class _SamplerThread(threading.Thread):
    error: BaseException | None = None


def start(qmp, out: Path, period: float):
    def run():
        try:
            _call("run", out=out, period=period, callbacks=_bindings(qmp))
        except BaseException as e:
            thread.error = e
            raise

    thread = _SamplerThread(target=run, name="pc-sample", daemon=True)
    thread.error = None
    thread.start()
    return thread
