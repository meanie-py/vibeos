"""Owned context/lease transport for native build entry and supervision."""

from __future__ import annotations

import contextlib
import dataclasses
import os
from pathlib import Path

from tools import qemu_infra_compat as native

from . import build as implementation
from .build_bindings import BuildBindings


class EntryBindings(BuildBindings):
    def __init__(self, *args):
        super().__init__(*args)
        self.resources = contextlib.ExitStack()
        self.source_reads = contextlib.ExitStack()
        self.reservation = contextlib.ExitStack()
        self.supervisor = None
        self.cold = False

    def arguments(self, a):
        for key, value in a["args"].items():
            setattr(self.args, key, value)

    def fact(self, a):
        m = implementation
        name = a["name"]
        if name == "canon_available":
            return m.canon.available()
        if name == "fex_prerequisites":
            return m.fex_prerequisites_missing(self.repo)
        if name == "toolchain":
            return m.canon.ensure_toolchain(self.repo)
        if name == "broker":
            return str(m.hostjobs.ensure(self.repo))
        if name == "trim":
            return m._trim_own_target(self.repo)
        if name == "store_gc":
            return m._scheduled_store_gc(self.args)
        raise RuntimeError("unknown build entry fact " + name)

    def open(self, a):
        m = implementation
        try:
            admission = self.resources.enter_context(
                m.hostlock.build_lock(Path(a["build"]), a["arch"])
            )
            m.hostjobs.ensure(self.repo)
            return {"admission": admission}
        except m.hostlock.buildpolicy.PolicyError as error:
            self.close_resources({})
            return {"error": str(error)}

    def source(self, a):
        m = implementation
        kwargs = {"timeout": 0} if a["nonblocking"] else {}
        fd = m.hostlock.acquire_build_dir(
            Path(a["build"]), arch="i386", reader="amd64 SysWOW64 staging", **kwargs
        )
        if fd is None:
            return False
        self.source_reads.callback(m.hostlock.release, fd)
        return True

    def reserve(self, _):
        self.reservation.enter_context(implementation.diskspace.reserve(self.repo, "build"))

    def close_reservation(self, _):
        self.reservation.close()

    def close_resources(self, _):
        self.reservation.close()
        self.source_reads.close()
        self.resources.close()

    def locked(self, a):
        m = implementation
        try:
            if a.get("artifact_env") is not None:
                return m._build_locked(
                    self.args,
                    self.repo,
                    self.run,
                    self.arch,
                    self.cfg,
                    self.build_dir,
                    self.cold,
                    artifact_env=a["artifact_env"],
                )
            return m._build_locked(
                self.args, self.repo, self.run, self.arch, self.cfg, self.build_dir, self.cold
            )
        except (OSError, ValueError, RuntimeError) as error:
            if a.get("capture_errors"):
                return {"error": str(error)}
            raise

    def supervised(self, _):
        return implementation._build_supervised(
            self.args, self.repo, self.run, self.arch, self.cfg, self.build_dir, self.cold
        )

    def open_supervisor(self, a):
        from tools.artifact_runtime import supervise

        try:
            manager = supervise(
                self.repo,
                Path(a["store"]),
                Path(a["toolchain"]),
                Path(a["output"]),
                clean=a["clean"],
                verify=a["verify"],
                arch=a["arch"],
            )
            path = manager.__enter__()
            self.supervisor = manager
            return {"path": path}
        except (OSError, ValueError, RuntimeError) as error:
            return {"error": str(error)}

    def record(self, a):
        from tools.artifact_runtime import record_outer_result

        record_outer_result(Path(a["path"]), a["code"])

    def close_supervisor(self, _):
        if self.supervisor is None:
            return {}
        manager, self.supervisor = self.supervisor, None
        try:
            manager.__exit__(None, None, None)
            return {}
        except (OSError, ValueError, RuntimeError) as error:
            return {"error": str(error)}

    @property
    def entry_callbacks(self):
        return {
            **self.callbacks,
            "build_arguments": self.arguments,
            "build_entry_fact": self.fact,
            "build_resources_open": self.open,
            "build_source_acquire": self.source,
            "build_reservation_open": self.reserve,
            "build_reservation_close": self.close_reservation,
            "build_resources_close": self.close_resources,
            "build_locked": self.locked,
            "build_supervised": self.supervised,
            "build_supervisor_open": self.open_supervisor,
            "build_supervisor_close": self.close_supervisor,
            "build_supervisor_record": self.record,
        }


def call(args, repo, run, arch, cfg, build_dir, cold, operation, *, reads=None):
    bindings = EntryBindings(repo, run, args, arch, cfg, build_dir)
    if reads is not None:
        bindings.source_reads = reads
    bindings.cold = cold
    options = {
        k: getattr(args, k)
        for k in dir(args)
        if not k.startswith("_") and not callable(getattr(args, k))
    }
    try:
        with native.DedicatedSession(root=repo) as session:
            return session.call(
                "cli_build_entry_" + operation,
                {
                    "args": options,
                    "arch": arch,
                    "cfg": dataclasses.asdict(cfg)
                    if dataclasses.is_dataclass(cfg) and not isinstance(cfg, type)
                    else (vars(cfg) if cfg else {}),
                    "build": build_dir,
                    "cold": cold,
                    "env": dict(os.environ),
                    "home": Path.home(),
                },
                callbacks=bindings.entry_callbacks,
            )
    finally:
        bindings.close_supervisor({})
        if reads is None:
            bindings.close_resources({})
