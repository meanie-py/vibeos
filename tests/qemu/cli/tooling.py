"""Compatibility front door for native tooling planning, scheduling and receipts."""

from __future__ import annotations

import os
import sys
import tempfile
import traceback
from pathlib import Path

from tools import qemu_infra_compat as native

from .artifacts import Run, Stage
from .lints import lint_jobs
from .tooling_receipts import runtime_identity

_PYTEST_SUITES = native.call("tooling_policy", {})["_PYTEST_SUITES"]


def _options(args):
    return {
        name: bool(getattr(args, name, False))
        for name in ("no_tests", "no_types", "skip_generated")
    }


def _checks(args, repo):
    return native.call(
        "tooling_plan", {"python": sys.executable, "jobs": lint_jobs(), **_options(args)}, root=repo
    )


def _run(label, cmd, repo, *, cwd=None, env_pp=None, run=None):
    if label == "pytest:tools/test_lint_facts.py":
        try:
            _provision({"repo": repo})
        except (OSError, RuntimeError, ValueError) as error:
            return label, False, f"native lint provisioning failed: {error}"
    with tempfile.TemporaryDirectory(prefix="tooling-check-") as temp:
        with native.DedicatedSession(root=repo) as session:
            result = session.call(
                "tooling_check",
                {
                    "label": label,
                    "cmd": cmd,
                    "cwd": cwd or repo,
                    "env_pp": env_pp,
                    "environment": dict(os.environ),
                    "dir": run.dir if run else temp,
                    "run_start": run.start if run else 0,
                },
            )
        if run is not None:
            run.command_timings.append(result["timing"])
        return tuple(result["result"])


def _provision(arguments):
    from .rustlints import ensure

    return str(ensure(Path(arguments["repo"])))


def _check_tooling(args, repo, run):
    plan = _checks(args, repo)
    with native.DedicatedSession(root=repo) as session:
        result = session.call(
            "tooling_run",
            {
                "dir": run.dir,
                "run_start": run.start,
                "python": sys.executable,
                "jobs": lint_jobs(),
                "environment": dict(os.environ),
                "cache": os.environ.get("VIBEOS_TOOLING_CACHE") != "0",
                **_options(args),
                **plan,
            },
            callbacks={
                "tooling_runtime": lambda a: runtime_identity(Path(a["repo"])),
                "tooling_provision_lint": _provision,
            },
        )
    run.command_timings.extend(result["command_timings"])
    for stage in result["stages"]:
        run.add(Stage(**stage))
    run.extra["checks"] = result["checks"]
    print(result["output"], end="")
    return result["code"]


def check_tooling(args, repo):
    run = Run("check-tooling", "all")
    run.extra["options"] = _options(args)
    try:
        code = _check_tooling(args, repo, run)
    except BaseException as error:
        run.classification = (
            "TOOLING_INTERRUPTED" if isinstance(error, KeyboardInterrupt) else "TOOLING_ERROR"
        )
        run.log(traceback.format_exc(), header="tooling runner failed")
        run.add(
            Stage(
                "tooling-runner", exit=1, status="fail", detail=str(error) or type(error).__name__
            )
        )
        run.finalize(repo)
        raise
    if code:
        run.classification = "TOOLING_FAIL"
    return run.finalize(repo)
