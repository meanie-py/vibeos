"""Compatibility entry points for the native RR command coordinator.

Command policy, replay guards, finalisation, comparison and presentation live in
vibeos-test::rr_commands. This module adapts argparse and external process I/O.
"""

from __future__ import annotations

import os
import sys
import threading
from pathlib import Path

from tools import qemu_infra_compat as native

from . import recordreplay as rr

DEFAULT_ROOT = Path(os.environ.get("VIBEOS_RR_ROOT", Path.home() / "vibeos-rr"))


def _native_call(operation, *, callbacks=None, **kwargs):
    try:
        if callbacks is not None:
            with native.DedicatedSession() as session:
                return session.call("rr_cli_" + operation, kwargs, callbacks=callbacks)
        return native.call("rr_cli_" + operation, kwargs)
    except native.NativeError as error:
        raise rr.RecordReplayError(str(error)) from error


def _root(args):
    return Path(getattr(args, "root", None) or DEFAULT_ROOT)


def _execute(arguments):
    from .artifacts import Run, Stage

    run = Run("rr-command", "all")
    result = run.run(
        arguments["argv"], cwd=Path(arguments["cwd"]), env=arguments["env"], stage="rr-child"
    )
    if result.stdout:
        print(result.stdout, end="")
    if result.stderr:
        print(result.stderr, end="", file=sys.stderr)
    run.add(
        Stage("rr-child", exit=result.returncode, status="ok" if result.returncode == 0 else "fail")
    )
    run.finalize(Path(arguments["cwd"]))
    return result.returncode


def _display(arguments):
    print(arguments["text"], file=sys.stderr if arguments.get("stderr") else sys.stdout, flush=True)


def _command(args, repo, verb):
    names = (
        "trace",
        "arch",
        "payload",
        "name",
        "smp",
        "build",
        "timeout",
        "shift",
        "sample",
        "gdb",
        "break_at",
        "keep",
        "apply",
        "near",
        "count",
        "at",
        "icount",
    )
    options = {name: getattr(args, name) for name in names if hasattr(args, name)}
    return _native_call(
        "command",
        repo=repo,
        trace_root=_root(args),
        args=options,
        env=dict(os.environ),
        verb=verb,
        callbacks={"rr_execute": _execute, "print": _display},
    )


def _boot_cmd(repo, arch, payload, smp, build, timeout):
    return _native_call(
        "command",
        action="boot_command",
        repo=repo,
        arch=arch,
        payload=payload,
        smp=smp,
        build=build,
        timeout=timeout,
    )


def await_trace_quiescence(trace: Path, *, settle: float = 6.0, limit: float = 420.0) -> int:
    """Refuse finalization unless the native monotonic wait observes a stable trace."""
    return _native_call("await_quiescence", trace_dir=trace, settle=settle, limit=limit)


def _keep_serial(trace: Path, name: str, *, expect_mode: str) -> Path | None:
    result = _native_call("keep_serial", trace_dir=trace, name=name, expect_mode=expect_mode)
    if result.get("warning"):
        print(result["warning"], file=sys.stderr)
    return Path(result["path"]) if result["path"] else None


def _guest_lines(p: Path) -> list[str]:
    return _native_call("guest_lines", path=p)


def _serial_divergence(a: list[str], b: list[str]) -> tuple[int, str] | None:
    result = _native_call("serial_divergence", record=a, replay=b)
    return tuple(result) if result is not None else None


class _TraceProgress:
    """Thread/DTO adapter for the native trace progress watcher."""

    def __init__(self, trace: Path, period: float = 20.0) -> None:
        self.trace, self.period = trace, period
        self.total = (trace / "replay.bin").stat().st_size
        self.max_pos, self.stalled_for = 0, 0.0
        self._stop = threading.Event()
        self._thread: threading.Thread | None = None
        self._error: BaseException | None = None

    def _run(self) -> None:
        try:
            result = _native_call(
                "watch_progress",
                trace_dir=self.trace,
                period=self.period,
                callbacks={"rr_cli_alive": lambda _: not self._stop.is_set()},
            )
            self.total, self.max_pos, self.stalled_for = (
                result["total"],
                result["max_pos"],
                result["stalled_for"],
            )
        except BaseException as error:
            self._error = error

    def __enter__(self) -> _TraceProgress:
        self._thread = threading.Thread(target=self._run, daemon=True)
        self._thread.start()
        return self

    def __exit__(self, *exc) -> None:
        self._stop.set()
        if self._thread:
            self._thread.join(timeout=10)
            if self._thread.is_alive():
                raise rr.RecordReplayError("native trace progress watcher did not stop")
        if self._error and exc[0] is None:
            raise self._error

    def report(self) -> str | None:
        return _native_call(
            "progress_report",
            state=dict(total=self.total, max_pos=self.max_pos, stalled_for=self.stalled_for),
        )


def cmd_record(args, repo):
    return _command(args, repo, "record")


def cmd_replay(args, repo):
    return _command(args, repo, "replay")


def cmd_verify(args, repo):
    return _command(args, repo, "verify")


def cmd_diverge(args, repo):
    return _command(args, repo, "diverge")


def cmd_list(args, repo):
    return _command(args, repo, "list")


def cmd_prune(args, repo):
    return _command(args, repo, "prune")


def cmd_info(args, repo):
    return _command(args, repo, "info")


def cmd_breaks(args, repo):
    return _command(args, repo, "breaks")


def cmd_wedge(args, repo):
    return _command(args, repo, "wedge")


def rr_cmd(args, repo):
    try:
        return _command(args, repo, args.rr_verb)
    except rr.RecordReplayError as error:
        print(f"[rr] {error}", file=sys.stderr)
        return 2
