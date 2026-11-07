"""Task-handle and reporting transport for the native gate coordinator."""

from __future__ import annotations

import concurrent.futures as futures
from functools import partial
from pathlib import Path
from types import SimpleNamespace

from tools import qemu_infra_compat as native
from tools.qemu_infra_runner import Bindings

from . import gate as implementation


class GateBindings(Bindings):
    def __init__(self, repo, run):
        super().__init__(repo, run)
        self.tasks = {}
        self.next = 0
        self.pool = futures.ThreadPoolExecutor(max_workers=20)

    def submit(self, a):
        m = implementation
        kind = a["kind"]
        lane = a["lane"]
        root = Path(a["root"])
        args = SimpleNamespace(**a["args"])
        extra = a.get("extra") or {}
        # These handles are external-call leaves. Dependencies and retry policy
        # come exclusively from the coordinator's submitted task DTOs.
        if kind in ("build", "restage"):
            call = partial(m._build_once, self.repo, self.run, lane, args, **extra)
        elif kind in ("lane", "retry"):
            if lane == "net":
                call = partial(m._net_once, self.repo, root)
            elif lane.startswith("conf:"):
                call = partial(m._conformance_once, self.repo, lane[5:], root)
            elif lane.startswith("stor:"):
                call = partial(m._stortest_once, self.repo, lane[5:], root)
            else:
                call = partial(m._smoke_once, self.repo, lane, root, list(extra))
        elif kind == "lint":
            call = partial(m._lint_once, self.repo, lane, root)
        elif kind == "cohab":
            call = partial(m._cohab_once, self.repo, lane, root)
        elif kind == "tooling":
            call = partial(m._tooling_once, self.repo, root)
        elif kind == "difftest":
            call = partial(m._difftest_once, self.repo, root)
        else:
            raise RuntimeError("unknown gate task " + kind)
        self.next += 1
        self.tasks[self.next] = self.pool.submit(call)
        return self.next

    def poll(self, a):
        selected = {token: self.tasks[token] for token in a["tokens"]}
        done, _ = futures.wait(list(selected.values()), return_when=futures.FIRST_COMPLETED)
        out = []
        for token, job in selected.items():
            if job not in done:
                continue
            try:
                out.append({"token": token, "result": job.result()})
            except BaseException as error:
                out.append({"token": token, "error": f"{type(error).__name__}: {error}"})
            del self.tasks[token]
        return out

    @property
    def gate_callbacks(self):
        m = implementation
        return {
            **self.callbacks,
            "gate_task_submit": self.submit,
            "gate_task_poll": self.poll,
            "gate_preflight": lambda a: m.hostdisk.preflight(self.run, self.repo, a["arches"]),
            "gate_tree_state": lambda _: m._tree_state(self.repo),
            "gate_kmtest": lambda a: m._ensure_kmtest_payload(self.repo, a["arches"]),
            "gate_ladder": lambda a: m.ladder_mod.gate_decision(self.repo, a["lane"]),
            "gate_tooling_waits": lambda a: any(
                not m.canon.is_canonical(self.repo / f"build-{arch}") for arch in a["arches"]
            ),
            "gate_stamp": lambda a: m._write_gate_stamp(
                self.run, self.repo, a["arches"], tuple(a["gated"])
            ),
        }


def run(args, repo, reporter):
    bindings = GateBindings(repo, reporter)
    try:
        options = {
            k: getattr(args, k)
            for k in dir(args)
            if not k.startswith("_") and not callable(getattr(args, k))
        }
        with native.DedicatedSession(root=repo) as session:
            return session.call(
                "gate_run",
                {"args": options, "run_dir": reporter.dir},
                callbacks=bindings.gate_callbacks,
            )
    finally:
        bindings.pool.shutdown(wait=True)
