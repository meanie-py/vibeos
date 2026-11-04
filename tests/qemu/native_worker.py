"""Owned DTO/reporting worker; all VM and test orchestration executes in Rust."""

from __future__ import annotations

import argparse
import contextlib
import json
import os
import sys
import tempfile
from pathlib import Path

from tools import build_infra_compat as transport
from tools import test_infra_compat as native
from tools.test_infra_runner import Bindings, request_options


def _publish(path: Path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.NamedTemporaryFile(mode="w", dir=path.parent, delete=False) as output:
        temporary = Path(output.name)
        try:
            json.dump(transport._json(value), output, ensure_ascii=False)
            output.flush()
            os.fsync(output.fileno())
        except BaseException:
            temporary.unlink(missing_ok=True)
            raise
    temporary.replace(path)


def _diagnostic(operation):
    from tests.qemu import native_diagnostics

    payload = json.load(sys.stdin)
    saved = os.dup(sys.stdout.fileno())
    try:
        os.dup2(sys.stderr.fileno(), sys.stdout.fileno())
        with contextlib.redirect_stdout(sys.stderr):
            value = (
                native_diagnostics.prepare(payload)
                if operation == "vm_diagnostic_prepare"
                else native_diagnostics.sidecar(payload)
            )
        result = {"ok": transport._json(value)}
        code = 0
    except Exception as error:
        result = {"error": {"kind": type(error).__name__, "message": str(error)}}
        code = 2
    finally:
        os.dup2(saved, sys.stdout.fileno())
        os.close(saved)
    print(json.dumps(result, ensure_ascii=False), flush=True)
    return code


def main():
    if len(sys.argv) > 1 and sys.argv[1] in ("vm_diagnostic_prepare", "vm_diagnostic_sidecar"):
        return _diagnostic(sys.argv[1])
    parser = argparse.ArgumentParser()
    parser.add_argument("--request", type=Path, required=True)
    parser.add_argument("--result", type=Path, required=True)
    args = parser.parse_args()
    request = json.loads(args.request.read_text())
    repo = Path(request["repo"])
    from tests.qemu.cli.artifacts import Run, Stage

    run = Run(request.get("command", "farm"), request["arch"])
    bindings = Bindings(repo, run)
    options = request_options(argparse.Namespace(), repo, run)
    options.update(request.get("kwargs", {}))
    options["arch"] = request["arch"]
    options["env"] = {**os.environ, **request.get("env", {})}
    options["env"]["VIBEOS_TEST_PYTHON"] = sys.executable
    try:
        with native.DedicatedSession(root=repo) as connection:
            result = connection.call(request["op"], options, bindings.callbacks)
        run.classification = result.get("classification", "") or ""
        run.extra["boots"] = [result]
        run.add(
            Stage(
                "boot",
                status="ok" if result["ok"] else "fail",
                detail=result.get("classification") or result.get("outcome", ""),
                salient=result.get("salient", []),
            )
        )
    except Exception as error:
        result = {
            "ok": False,
            "outcome": "farm_error",
            "classification": "FARM_ERROR",
            "salient": [str(error)[:200]],
        }
        run.classification = "FARM_ERROR"
        run.add(Stage("native", status="fail", detail=str(error)))
    code = run.finalize(repo)
    _publish(args.result, {"result": result, "run_dir": run.dir, "exit_code": code})
    return code


if __name__ == "__main__":
    raise SystemExit(main())
