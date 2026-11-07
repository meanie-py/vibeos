"""External fact and resource adapters for native build preparation."""

from __future__ import annotations

import contextlib
import dataclasses
import importlib.util
import io
import os
from collections.abc import Callable
from pathlib import Path
from types import SimpleNamespace

from tools import qemu_infra_compat as native

from . import build as implementation
from .build_bindings import BuildBindings


class Mechanisms(BuildBindings):
    trim_plan = None
    wrapper: Callable[[list[str]], list[str]] | None = None

    def module(self):
        if self.payload_compile is None:
            spec = importlib.util.spec_from_file_location(
                "payload_compile", self.repo / "tools/payloads/compile.py"
            )
            if spec is None or spec.loader is None:
                raise RuntimeError("payload adapter not found")
            module = importlib.util.module_from_spec(spec)
            spec.loader.exec_module(module)
            self.payload_compile = module
        return self.payload_compile

    def fact(self, request):
        m = implementation
        k = request["kind"]
        a = request["arguments"]
        repo, bd, arch = self.repo, self.build_dir, self.arch
        if k == "canonical":
            return m.canon.is_canonical(bd)
        if k == "cache_value":
            return m._value_in_cache(bd, a["key"])
        if k == "canon_available":
            return m.canon.available()
        if k == "toolchain":
            return m.canon.ensure_toolchain(repo)
        if k == "carried":
            return m.canon.carried_options(bd)
        if k == "configure_cmd":
            return m.canon.configure_cmd(arch, a["extra"])
        if k == "canon_path":
            return m.canon.canon_path()
        if k == "wrap":
            return self.wrapper(a["cmd"]) if self.wrapper else m.canon.wrap(repo, a["cmd"])
        if k == "guests_to_refresh":
            return m.ladder.guests_to_refresh(repo)
        if k == "refresh_blocker":
            return m.ladder.refresh_blocker(repo)
        if k == "guests_deferred":
            return m.ladder.x64_refresh_deferred(repo, a["todo"])
        if k == "guest_builder":
            return m.ladder.GUEST_BUILDER
        if k == "source_epoch":
            return m.source_date_epoch(repo)
        if k == "cache_make":
            return m.cachecfg.make(repo)
        if k == "cache_enabled":
            return m.cachecfg.cache_enabled(self.args)
        if k == "policy":
            return m.hostlock.buildpolicy.read(m.hostlock.POLICY_DIR)
        if k == "policy_environment":
            return m._policy_build_env(a["env"])
        if k == "host_environment":
            return m.hostjobs.environment(
                repo, a["env"], canonical=m.canon.is_canonical(bd), run_dir=self.run.dir
            )
        if k == "rust_targets":
            return m.provenance.discover_rust_targets(bd)
        if k == "workspace_targets":
            return m.provenance.workspace_cargo_targets(repo, arch)
        if k == "rust_archives":
            return m.provenance.discover_rust_archives(bd)
        if k == "module_outputs":
            return m.provenance.discover_rust_module_outputs(repo, arch)
        if k == "payload_resolve":
            return self.module().resolve_env_payloads(os.environ)
        if k == "payload_ensure":
            error = io.StringIO()
            try:
                with contextlib.redirect_stderr(error):
                    rebuilt = self.module().ensure_built(a["wanted"], arch)
                return {"ok": True, "rebuilt": rebuilt}
            except SystemExit as caught:
                why = [line for line in error.getvalue().strip().splitlines() if line.strip()]
                if isinstance(caught.code, str) and caught.code.strip():
                    why.append(caught.code.strip())
                return {"ok": False, "why": why}
        if k == "payload_stamp":
            return self.module().write_stamp(bd)
        if k == "snapshot":
            try:
                return m.warmth.worktree_tree(repo)
            except (OSError, RuntimeError, m.subprocess.SubprocessError):
                return None
        if k == "trim_plan":
            try:
                self.trim_plan = m.trim.plan_worktree(repo)
                return dataclasses.asdict(self.trim_plan)
            except OSError as error:
                return {"error": str(error)}
        if k == "trim_apply":
            if self.trim_plan is None:
                raise RuntimeError("trim plan is not owned")
            try:
                freed, _ = m.trim.apply(repo, self.trim_plan)
                return {"freed": freed}
            except OSError as error:
                return {"error": str(error)}
        if k == "store_gc":
            from tools.artifact_runtime import scheduled_gc

            try:
                return scheduled_gc(Path(a["store"]))
            except Exception as error:
                return {"error": str(error)}
        raise RuntimeError("unknown build fact " + k)

    def external(self, a):
        m = implementation
        try:
            p = m._execute(
                a["argv"],
                cwd=Path(a["cwd"]) if a.get("cwd") else self.repo,
                timeout=a.get("timeout"),
                check=a.get("check", False),
            )
            return {
                "returncode": p.returncode,
                "stdout": getattr(p, "stdout", "") or "",
                "stderr": getattr(p, "stderr", "") or "",
            }
        except (OSError, m.subprocess.SubprocessError) as error:
            return {"error": str(error)}


def call(run, repo, operation, *, arch="", build_dir=None, args=None, wrap=None, **arguments):
    args = args or SimpleNamespace()
    bindings = Mechanisms(repo, run, args, arch, None, build_dir or repo / f"build-{arch}")
    bindings.wrapper = wrap
    options = {
        k: getattr(args, k)
        for k in dir(args)
        if not k.startswith("_") and not callable(getattr(args, k))
    }
    with native.DedicatedSession(root=repo) as session:
        result = session.call(
            "cli_build_helper_" + operation,
            {
                "arch": arch,
                "build": bindings.build_dir,
                "args": options,
                "env": dict(os.environ),
                **arguments,
            },
            callbacks={
                **bindings.callbacks,
                "build_fact": bindings.fact,
                "build_external": bindings.external,
            },
        )
    return result, bindings
