"""Resource and DTO adapters for native build coordination."""

from __future__ import annotations

import dataclasses
import time
from pathlib import Path

from tools import qemu_infra_compat as native
from tools.qemu_infra_runner import Bindings

from . import build as implementation


class BuildBindings(Bindings):
    def __init__(self, repo, run, args, arch, cfg, build_dir):
        super().__init__(repo, run)
        self.args, self.arch, self.cfg, self.build_dir = args, arch, cfg, build_dir
        self.cache = None
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
        repo, run, args, arch, bd = self.repo, self.run, self.args, self.arch, self.build_dir

        def wrap(cmd):
            return m.canon.wrap(repo, cmd) if a.get("wrapped", False) else cmd

        if name == "failure_hook":
            run.finalize_hooks.append(lambda: m._failure_over_moved_tree(run, repo, a["tree"]))
        elif name == "canonize":
            return m._maybe_canonize(
                run,
                repo,
                bd,
                arch,
                a["flags"],
                sticky_on=tuple(a["sticky"]),
                instr_values=a["values"],
            )
        elif name == "wrap":
            return m.canon.wrap(repo, a["cmd"])
        elif name == "stage_payloads":
            result = m._stage_payloads(run, repo, bd, arch)
            if result is None:
                return None
            selection, self.payload_compile = result
            return selection
        elif name == "clean":
            m._clean(run, repo, arch)
        elif name == "retire_build_refs":
            from tools.artifact_runtime import retire_build_refs

            retire_build_refs(Path(a["supervisor"]))
        elif name == "build_environment":
            result = m._mechanism(
                run,
                repo,
                "environment",
                build_dir=bd,
                args=args,
                wrapped=a["wrapped"],
                artifact_env=a["artifact_env"],
                wrap=wrap,
            )[0]
            if result is not None:
                self.cache = result["cache"]
            return result
        elif name == "refresh_guests":
            return m._refresh_wow64_guests(run, repo, stage=a["stage"])
        elif name == "refresh_cpu":
            return m._refresh_wowx64_cpu(run, repo)
        elif name == "ninja_jobs":
            return m._ninja_jobs()
        elif name == "ninja_timeout":
            return m._ninja_timeout()
        elif name == "classify":
            return m._classify_build_failure(a["stdout"], a["stderr"], a["rc"])
        elif name == "cohab_rels":
            return m.provenance.boot_set_cohab_rels(repo, arch)
        elif name == "verify_flags":
            return m._verify_flag_artifacts(run, repo, bd, args)
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
        elif name == "cache_stats":
            if self.cache is None:
                raise RuntimeError("cache adapter missing")
            m._collect_cache_stats(run, self.cache, a["env"], repo=repo, wrap=wrap)
        elif name == "binary_lints":
            m.lints.run_binary_lints(run, repo, arch, strict=a["strict"])
        elif name == "source_lints":
            m.lints.run_source_lints(run, repo, strict=False)
        elif name == "inputs_moved":
            return m._inputs_moved(repo, a["tree"])
        elif name == "stamp":
            m.provenance.make_stamp(repo, arch, cold=a["cold"], ts=int(time.time())).write(bd)
        elif name == "staleness":
            return dataclasses.asdict(m.provenance.check_staleness(repo, arch))
        elif name == "dedup":
            return dataclasses.asdict(m.kick_dedup())
        else:
            raise RuntimeError("Unknown native build adapter " + name)
        return None


def locked(args, repo, run, arch, cfg, build_dir, cold, *, artifact_env=None):
    bindings = BuildBindings(repo, run, args, arch, cfg, build_dir)
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
            },
            callbacks={**bindings.callbacks, "build_helper": bindings.helper},
        )
