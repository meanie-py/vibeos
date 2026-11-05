"""CLI presentation/configuration bindings for native testing controllers."""

from __future__ import annotations

import dataclasses
import os
import sys
from pathlib import Path

from tests.qemu.cli import archcfg, config, hostlock
from tests.qemu.cli.artifacts import Run, Stage, create_run_dir, ledger_append


def request_options(args, repo: Path, run: Run | None = None) -> dict:
    values = {key: value for key, value in vars(args).items() if not callable(value)}
    result = {
        "args": values,
        "env": dict(os.environ),
        "cli": config.load(repo).data,
        "python": sys.executable,
        "cfgs": {arch: dataclasses.asdict(archcfg.arch_build(arch)) for arch in archcfg.ARCHES},
    }
    if run is not None:
        result["run_dir"] = run.dir
    return result


class Bindings:
    def __init__(self, repo: Path, run: Run | None):
        self.repo = repo
        self._run = run
        self.finalized = False

    @property
    def run(self) -> Run:
        if self._run is None:
            raise RuntimeError("native operation requires an owned Run reporter")
        return self._run

    def state(self, _arguments=None):
        return {
            "classification": self.run.classification,
            "extra": self.run.extra,
            "stages": [dataclasses.asdict(stage) for stage in self.run.stages],
            "verdict": self.run.verdict,
            "dir": self.run.dir,
            "arch": self.run.arch,
            "command": self.run.command,
        }

    def update(self, arguments):
        if "classification" in arguments:
            self.run.classification = arguments["classification"]
        if "verdict" in arguments:
            self.run.verdict = arguments["verdict"]
        self.run.extra.update(arguments.get("extra", {}))
        return self.state()

    def finalize(self, arguments):
        self.update(arguments)
        self.finalized = True
        return self.run.finalize(self.repo)

    def command(self, arguments):
        result = self.run.run(
            arguments["argv"],
            cwd=Path(arguments["cwd"]) if arguments.get("cwd") else self.repo,
            env=arguments.get("env"),
            timeout=arguments.get("timeout"),
            stage=arguments.get("stage"),
        )
        return {
            "returncode": result.returncode,
            "stdout": result.stdout,
            "stderr": result.stderr,
            "timed_out": getattr(result, "timed_out", False),
        }

    def stage(self, arguments):
        self.run.add(Stage(**arguments))

    def log(self, arguments):
        self.run.log(arguments.get("text", ""), header=arguments.get("header"))

    def display(self, arguments):
        print(
            arguments.get("text", ""),
            file=sys.stderr if arguments.get("stderr") else sys.stdout,
            flush=arguments.get("flush", True),
        )

    def environment(self, arguments):
        os.environ.update(arguments["overlay"])

    def payload_stamp(self, arguments):
        from tools.payloads import compile as payload_compile

        return payload_compile.read_stamp(self.repo / f"build-{arguments['arch']}")

    def quiet(self, _arguments):
        return {
            "land_holder": hostlock.land_holder(),
            "foreign_boot_holders": hostlock.foreign_boot_holders(0),
        }

    def debug_owner(self, arguments):
        from tests.qemu.cli import debugcmd, gdbengine

        declared = os.environ.get("VIBEOS_QEMU_OWNER_PID", "")
        return {
            "owner_pid": int(declared)
            if declared.isdigit()
            else debugcmd._session_owner(os.getppid()),
            "kernel_image": gdbengine._kernel_image(arguments["arch"]),
        }

    def ledger(self, arguments):
        ledger_append(self.repo, **arguments)

    def conformance_signatures(self, arguments):
        from tests.qemu.cli.flakes import conformance_signatures

        return conformance_signatures(arguments["record"])

    @property
    def callbacks(self):
        return {
            "command": self.command,
            "conformance_signatures": self.conformance_signatures,
            "log": self.log,
            "stage": self.stage,
            "set_run": self.update,
            "get_run": self.state,
            "finalize": self.finalize,
            "print": self.display,
            "env": self.environment,
            "payload_stamp": self.payload_stamp,
            "host_quiet": self.quiet,
            "debug_owner": self.debug_owner,
            "ledger_append": self.ledger,
            "create_run_dir": lambda a: create_run_dir(a["command"], a["arch"]),
        }


def native_run_call(run, repo, op, kwargs):
    from tools import qemu_infra_compat as native

    bindings = Bindings(repo, run)
    with native.DedicatedSession(root=repo) as session:
        return session.call(op, kwargs, callbacks=bindings.callbacks)


def run_controller(op, args, repo, *, command, arch, extra=None):
    from tools import qemu_infra_compat as native

    run = Run(command=command, arch=arch)
    bindings = Bindings(repo, run)
    options = request_options(args, repo, run)
    options.update(extra or {})
    with native.DedicatedSession(root=repo) as session:
        result = session.call(op, options, callbacks=bindings.callbacks)
    if isinstance(result, dict):
        code = result["code"]
        if "rounds" in result:
            rounds = result["rounds"]
            if extra is not None and isinstance(extra.get("rounds"), list):
                extra["rounds"][:] = rounds
            run.extra["rounds"] = rounds
    else:
        code = result
    if not bindings.finalized:
        if code:
            run.add(
                Stage(command, status="fail", exit=code, detail=f"native controller exited {code}")
            )
        run.finalize(repo)
    return code
