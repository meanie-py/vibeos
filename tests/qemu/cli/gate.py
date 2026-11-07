"""Compatibility DTO/resource transport for the native gate and child controllers."""

from __future__ import annotations

import os
import subprocess
import sys
import traceback
from pathlib import Path

from tools import build_infra_compat as build_native
from tools import qemu_infra_compat as native

from . import canon as canon
from . import hostdisk as hostdisk
from . import hostlock, lints, rustlints, triage
from . import ladder as ladder_mod
from .artifacts import CommandResult, Run, Stage

__all__ = ["canon", "hostdisk", "ladder_mod"]

RETRYABLE = frozenset(native.call("gate_retry_policy", {}))
BIOS_LANE = "amd64-bios"
GATE_SMP = 2
FULL_ARCH_SET = frozenset({"i386", "amd64", "arm64"})
STORTEST_TIMEOUT_S = 900


def _execute(cmd, *, cwd, env=None, timeout=None, root=None, **_):
    selected = Path(root) if root is not None else Path(cwd)
    directory = selected / ".command-output" if selected != Path(cwd) else None
    with build_native.DedicatedSession("--api-jsonl") as session:
        result = session.call(
            "cli_artifacts_command",
            {
                "dir": directory,
                "repo": cwd,
                "argv": cmd,
                "cwd": cwd,
                "env": env,
                "timeout": timeout,
            },
        )
    if result.get("launch_error"):
        raise OSError(result["launch_error"])
    value = CommandResult(cmd, result["returncode"], result["stdout"], result["stderr"])
    value.timed_out = result["timed_out"]
    return value


def _dto(proc):
    return {
        "returncode": proc.returncode,
        "stdout": getattr(proc, "stdout", "") or "",
        "stderr": getattr(proc, "stderr", "") or "",
        "timed_out": getattr(proc, "timed_out", False),
    }


def _child(operation, repo, run=None, **arguments):
    def command(a):
        try:
            return _dto(
                _execute(
                    a["argv"],
                    cwd=a["cwd"],
                    env=a.get("env"),
                    timeout=a.get("timeout"),
                    root=a.get("root"),
                )
            )
        except subprocess.TimeoutExpired as error:

            def text(value):
                return value.decode(errors="replace") if isinstance(value, bytes) else (value or "")

            return {
                "returncode": 124,
                "stdout": text(error.stdout),
                "stderr": text(error.stderr),
                "timed_out": True,
            }

    def parent_command(a):
        if run is None:
            raise RuntimeError("gate build callback requires a reporter")
        return _dto(run.run(a["argv"], cwd=repo, env=a.get("env"), stage=a.get("stage")))

    def binary(a):
        return _dto(
            rustlints.run_binary(
                repo,
                a["arch"],
                ["verify_cohab_coverage"],
                lambda cmd: _execute(cmd, cwd=repo, root=arguments["root"]),
            )["verify_cohab_coverage"]
        )

    callbacks = {
        "gate_command": command,
        "command": parent_command,
        "gate_triage": lambda a: triage.classify(
            a["text"], reached=False, fail_marker=None, debugcon=None
        ),
        "gate_starved": lambda a: hostlock.looks_starved(a["text"]),
        "gate_scanner_error": lambda a: lints._scanner_error(
            subprocess.CompletedProcess([], a["returncode"], a["stdout"], a["stderr"]), "stdout"
        ),
        "gate_source_inventory": lambda a: [
            name for name, _, _ in lints.source_partition_specs(repo, a["arch"])
        ],
        "gate_binary_cohab": binary,
    }
    with native.DedicatedSession(root=repo) as session:
        return session.call(
            "gate_child_" + operation,
            {"env": dict(os.environ), "python": sys.executable, **arguments},
            callbacks=callbacks,
        )


def _retryable(classification, summary):
    return native.call("gate_retryable", {"classification": classification, "summary": summary})


def _resolve_arches(raw):
    return native.call("gate_arches", {"raw": raw or ""})


def _last_words(console, limit=240):
    return native.call("gate_child_last_words", {"console": console, "limit": limit})


def _read_summary(root):
    return native.call("gate_child_summary", {"root": root})


def _first_salient(summary):
    return native.call("gate_child_salient", {"summary": summary})


def _child_classification(stdout):
    return native.call("gate_child_classification", {"stdout": stdout})


def _smoke_once(repo, arch, root, extra):
    return _child("smoke", repo, arch=arch, root=root, extra=extra)


def _ensure_kmtest_payload(repo, arches):
    return tuple(_child("kmtest", repo, arches=arches))


def _conformance_once(repo, arch, root):
    return _child("conformance", repo, arch=arch, root=root)


def _stortest_once(repo, arch, root):
    return _child("stortest", repo, arch=arch, root=root)


def _net_once(repo, root):
    return _child("net", repo, root=root)


def _lint_once(repo, arch, root):
    return _child("check", repo, kind="lint", arch=arch, root=root)


def _cohab_once(repo, arch, root):
    return _child("cohab", repo, arch=arch, root=root)


def _tooling_once(repo, root):
    return _child("check", repo, kind="tooling", root=root)


def _difftest_once(repo, root):
    return _child("check", repo, kind="difftest", root=root)


def _build_once(repo, run, arch, args, *, defer_wow64=False, restage=False):
    options = {
        k: getattr(args, k)
        for k in dir(args)
        if not k.startswith("_") and not callable(getattr(args, k))
    }
    return _child(
        "build", repo, run, args=options, arch=arch, defer_wow64=defer_wow64, restage=restage
    )


def _tree_state(repo):
    return tuple(native.call("gate_stamp_state", {}, root=repo))


def _prune_stale_stamps(stamps_dir, max_age_s=24 * 3600):
    native.call("gate_stamp_prune", {"dir": stamps_dir, "max_age_s": max_age_s})


def _write_gate_stamp(run, repo, arches, gated=None):
    result = native.call("gate_stamp_write", {"arches": arches, "gated": gated}, root=repo)
    if result.get("log"):
        run.log(result["log"], header="stamp")
    if result.get("stage"):
        run.add(Stage(**result["stage"]))


def gate(args, repo):
    from . import gate_bindings

    run = Run("gate", "+".join(_resolve_arches(getattr(args, "arches", None))))
    try:
        return gate_bindings.run(args, repo, run)
    except BaseException as error:
        run.classification = (
            "GATE_INTERRUPTED" if isinstance(error, KeyboardInterrupt) else "GATE_ERROR"
        )
        run.log(traceback.format_exc(), header="gate coordinator failed")
        run.add(
            Stage(
                "gate-coordinator", exit=1, status="fail", detail=str(error) or type(error).__name__
            )
        )
        run.finalize(repo)
        raise
