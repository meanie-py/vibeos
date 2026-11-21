"""Resource and DTO adapters for native build coordination."""

from __future__ import annotations

import dataclasses
import os
import time
from pathlib import Path

from tools import qemu_infra_compat as native
from tools.qemu_infra_runner import Bindings

from . import build as implementation


class BuildBindings(Bindings):
    def __init__(self, repo, run, args, arch, cfg, build_dir):
        super().__init__(repo, run)
        self.args, self.arch, self.cfg, self.build_dir = args, arch, cfg, build_dir
        self.payload_compile = None

    def state(self, _arguments=None):
        return {
            "classification": getattr(self.run, "classification", ""),
            "extra": getattr(self.run, "extra", {}),
            "stages": [dataclasses.asdict(s) for s in getattr(self.run, "stages", [])],
            "verdict": getattr(self.run, "verdict", "OK"),
            "dir": getattr(self.run, "dir", self.build_dir),
            "arch": self.arch,
            "command": getattr(self.run, "command", "build"),
        }

    def helper(self, request):
        m = implementation
        a = request["arguments"]
        name = request["name"]
        repo, run, arch, bd = self.repo, self.run, self.arch, self.build_dir

        if name == "failure_hook":
            run.finalize_hooks.append(lambda: m._failure_over_moved_tree(run, repo, a["tree"]))
        elif name == "wrap":
            return m.canon.wrap(repo, a["cmd"])
        elif name == "retire_build_refs":
            from tools.artifact_runtime import retire_build_refs

            retire_build_refs(Path(a["supervisor"]))
        elif name == "classify":
            return m._classify_build_failure(a["stdout"], a["stderr"], a["rc"])
        elif name == "cohab_rels":
            return m.provenance.boot_set_cohab_rels(repo, arch)
        elif name == "payload_roots":
            if not a["stamped"]:
                return []
            if self.payload_compile is None:
                raise RuntimeError("payload compile adapter missing")
            return self.payload_compile.ninja_targets(a["stamped"], arch)
        elif name == "prune":
            try:
                victims = m.native.call(
                    "ninja_prune",
                    {
                        "build": bd,
                        "targets": a["roots"],
                        "canon_root": m.canon.canon_path() + f"/{bd.name}",
                        "optional": a["optional"],
                    },
                    root=repo,
                )
                return {"victims": victims}
            except (OSError, m.native.NativeError) as error:
                return {"error": f"<{error}>"}
        elif name == "binary_lints":
            m.lints.run_binary_lints(run, repo, arch, strict=a["strict"])
        elif name == "source_lints":
            m.lints.run_source_lints(run, repo, strict=False)
        elif name == "stamp":
            m.provenance.make_stamp(repo, arch, cold=a["cold"], ts=int(time.time())).write(bd)
        elif name == "staleness":
            return dataclasses.asdict(m.provenance.check_staleness(repo, arch))
        else:
            raise RuntimeError("Unknown native build adapter " + name)
        return None


def locked(args, repo, run, arch, cfg, build_dir, cold, *, artifact_env=None):
    # Mechanisms answers the facts and external commands the native build
    # mechanisms ask for, so they run in this session rather than a fresh one.
    from .build_mechanisms import Mechanisms

    bindings = Mechanisms(repo, run, args, arch, cfg, build_dir)
    options = {
        k: getattr(args, k)
        for k in dir(args)
        if not k.startswith("_") and not callable(getattr(args, k))
    }
    with native.DedicatedSession(root=repo) as session:
        return session.call(
            "cli_build_locked",
            {
                "args": options,
                "arch": arch,
                "cfg": dataclasses.asdict(cfg)
                if dataclasses.is_dataclass(cfg) and not isinstance(cfg, type)
                else (vars(cfg) if cfg else {}),
                "build_dir": build_dir,
                "cold": cold,
                "artifact_env": artifact_env,
                "ninja_jobs": implementation._ninja_jobs(),
                "ninja_timeout": implementation._ninja_timeout(),
                "host_env": dict(os.environ),
            },
            callbacks={
                **bindings.callbacks,
                "build_helper": bindings.helper,
                "build_fact": bindings.fact,
                "build_external": bindings.external,
            },
        )
